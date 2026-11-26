//! The event pipeline.
//!
//! ```text
//!  HTTP handlers / verifier          writer thread                 consumer thread
//!  ───────────────────────   mpsc   ───────────────────   watch   ─────────────────────
//!  bus.emit(Event) ───────▶ [bounded] ─▶ drain ≤ 1024      ──────▶ tail log up to the
//!   • View: try_send, shed             append all to log            committed offset,
//!     when full (counted)              commit() = 1 fsync           fold into Engine
//!   • others: await (backpressure)     publish committed offset     (RwLock, short hold)
//! ```
//!
//! The log is the source of truth for analytics: on startup the consumer
//! replays it from the oldest retained record, so the in-memory engine is
//! rebuilt exactly — no analytics tables in Postgres.
//!
//! Disk I/O runs on dedicated OS threads, not the async runtime, so a slow
//! fsync never stalls request handling.

use std::sync::atomic::Ordering::Relaxed;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use blinker_analytics::{BlinkId, Engine, Event, EventKind};
use blinker_eventlog::{Config as LogConfig, Log, LogStats};
use tokio::sync::{mpsc, watch};

use crate::metrics::Metrics;

pub const MAX_BATCH: usize = 1024;

pub fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

#[derive(Clone)]
pub struct EventBus {
    tx: mpsc::Sender<Event>,
    metrics: Arc<Metrics>,
}

impl EventBus {
    /// Records an event. Views are best-effort (shed under overload, never
    /// slow a request); everything else waits for queue space so payments
    /// and activations are never lost.
    pub async fn emit(&self, blink: BlinkId, kind: EventKind) {
        let ev = Event { ts_ms: now_ms(), blink, kind };
        let m = &self.metrics;
        let res = if matches!(kind, EventKind::View) {
            match self.tx.try_send(ev) {
                Ok(()) => Ok(()),
                Err(mpsc::error::TrySendError::Full(_)) => {
                    Metrics::inc(&m.c.events_dropped);
                    return;
                }
                Err(mpsc::error::TrySendError::Closed(_)) => Err(()),
            }
        } else {
            self.tx.send(ev).await.map_err(|_| ())
        };
        if res.is_ok() {
            Metrics::inc(&m.c.events_emitted);
            m.queue_depth.store((self.tx.max_capacity() - self.tx.capacity()) as u64, Relaxed);
        } else {
            tracing::error!("event pipeline closed; dropping {}", kind.name());
        }
    }
}

pub struct Pipeline {
    pub bus: EventBus,
    pub engine: Arc<RwLock<Engine>>,
    pub log_stats: Arc<Mutex<LogStats>>,
    pub committed: watch::Receiver<u64>,
}

pub fn start(dir: &std::path::Path, cfg: LogConfig, capacity: usize, metrics: Arc<Metrics>) -> Result<Pipeline> {
    let log = Log::open(dir, cfg).with_context(|| format!("opening event log at {}", dir.display()))?;
    let stats = log.stats();
    if stats.recovered_truncated_bytes > 0 {
        tracing::warn!(bytes = stats.recovered_truncated_bytes, "event log: truncated torn tail during recovery");
    }
    tracing::info!(segments = stats.segments, records = stats.next_offset - stats.first_offset, "event log opened");

    let (tx, rx) = mpsc::channel::<Event>(capacity);
    let (committed_tx, committed_rx) = watch::channel(log.next_offset());
    metrics.queue_capacity.store(capacity as u64, Relaxed);
    metrics.committed_offset.store(log.next_offset(), Relaxed);
    let log_stats = Arc::new(Mutex::new(stats));
    let engine = Arc::new(RwLock::new(Engine::new()));

    // Consumer first: it replays history from the oldest retained offset.
    let reader = log.reader(log.first_offset())?;
    spawn_consumer(reader, engine.clone(), committed_rx.clone(), metrics.clone());
    spawn_writer(log, rx, committed_tx, log_stats.clone(), metrics.clone());

    Ok(Pipeline { bus: EventBus { tx, metrics }, engine, log_stats, committed: committed_rx })
}

fn spawn_writer(
    mut log: Log,
    mut rx: mpsc::Receiver<Event>,
    committed: watch::Sender<u64>,
    stats: Arc<Mutex<LogStats>>,
    m: Arc<Metrics>,
) {
    std::thread::Builder::new()
        .name("eventlog-writer".into())
        .spawn(move || {
            let mut batch: Vec<Event> = Vec::with_capacity(MAX_BATCH);
            let mut buf = [0u8; blinker_analytics::event::MAX_ENCODED];
            // blocking_recv parks this OS thread until an event arrives.
            while let Some(first) = rx.blocking_recv() {
                batch.push(first);
                // Group commit: take whatever else is already queued.
                while batch.len() < MAX_BATCH {
                    match rx.try_recv() {
                        Ok(e) => batch.push(e),
                        Err(_) => break,
                    }
                }
                let t = Instant::now();
                let mut ok = true;
                for e in &batch {
                    let n = e.encode(&mut buf);
                    if let Err(err) = log.append(&buf[..n]) {
                        tracing::error!("event log append failed: {err}");
                        ok = false;
                        break;
                    }
                }
                match log.commit() {
                    Ok(end) if ok => {
                        m.log_commit_latency.record(t.elapsed());
                        m.c.events_committed.fetch_add(batch.len() as u64, Relaxed);
                        Metrics::inc(&m.c.log_batches);
                        m.last_batch.store(batch.len() as u64, Relaxed);
                        m.max_batch.fetch_max(batch.len() as u64, Relaxed);
                        m.committed_offset.store(end, Relaxed);
                        m.queue_depth.store(rx.len() as u64, Relaxed);
                        *stats.lock().unwrap() = log.stats();
                        let _ = committed.send(end);
                    }
                    Ok(_) => {}
                    Err(err) => {
                        // Keep running: the next commit retries the flush.
                        tracing::error!("event log commit failed: {err}");
                        std::thread::sleep(Duration::from_millis(100));
                    }
                }
                batch.clear();
            }
            tracing::info!("event log writer stopped");
        })
        .expect("spawn writer thread");
}

fn spawn_consumer(
    mut reader: blinker_eventlog::LogReader,
    engine: Arc<RwLock<Engine>>,
    mut committed: watch::Receiver<u64>,
    m: Arc<Metrics>,
) {
    let rt = tokio::runtime::Handle::current();
    std::thread::Builder::new()
        .name("analytics-consumer".into())
        .spawn(move || {
            let replay_start = Instant::now();
            let mut replayed = false;
            loop {
                let end = *committed.borrow_and_update();
                // Apply in chunks so readers never wait long on the lock.
                loop {
                    let mut applied = 0;
                    {
                        let mut eng = engine.write().unwrap();
                        while applied < 4096 {
                            match reader.next_before(end) {
                                Ok(Some((_, rec))) => {
                                    let _ = eng.apply_bytes(rec);
                                    applied += 1;
                                }
                                Ok(None) => break,
                                Err(e) => {
                                    tracing::error!("event log read failed: {e}");
                                    break;
                                }
                            }
                        }
                    }
                    m.consumer_offset.store(reader.offset(), Relaxed);
                    if applied < 4096 {
                        break;
                    }
                }
                if !replayed {
                    replayed = true;
                    tracing::info!(
                        events = reader.offset(),
                        ms = replay_start.elapsed().as_millis() as u64,
                        "analytics rebuilt from event log"
                    );
                }
                if rt.block_on(committed.changed()).is_err() {
                    break;
                }
            }
        })
        .expect("spawn consumer thread");
}

#[cfg(test)]
mod tests {
    use super::*;
    use blinker_analytics::Asset;
    use blinker_eventlog::SyncPolicy;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn events_flow_through_log_into_engine_and_survive_restart() {
        let dir = tempfile::tempdir().unwrap();
        let cfg = LogConfig { segment_bytes: 4096, sync: SyncPolicy::Never, retention_bytes: None };
        let id = BlinkId::parse("pipelinetest").unwrap();
        {
            let p = start(dir.path(), cfg.clone(), 64, Arc::new(Metrics::default())).unwrap();
            p.bus.emit(id, EventKind::Created { asset: Asset::Sol }).await;
            for _ in 0..500 {
                p.bus.emit(id, EventKind::ActionPost { amount: 1 }).await;
            }
            let mut c = p.committed.clone();
            while *c.borrow() < 501 {
                c.changed().await.unwrap();
            }
            // Consumer catches up shortly after the commit.
            for _ in 0..100 {
                if p.engine.read().unwrap().events_applied() == 501 {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            assert_eq!(p.engine.read().unwrap().summary(&id, now_ms()).unwrap().posts, 500);
        }
        // Restart: analytics rebuilt from the log alone.
        let p = start(dir.path(), cfg, 64, Arc::new(Metrics::default())).unwrap();
        for _ in 0..100 {
            if p.engine.read().unwrap().events_applied() == 501 {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        assert_eq!(p.engine.read().unwrap().summary(&id, now_ms()).unwrap().posts, 500);
    }
}
