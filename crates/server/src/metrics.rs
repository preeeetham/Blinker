//! Lock-free metrics: plain atomics plus a log2-bucketed latency histogram.
//! Rendered as Prometheus text on `/metrics` and as JSON on `/api/system`.

use std::fmt::Write as _;
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
use std::time::Duration;

use serde::Serialize;

/// Histogram over microseconds with power-of-two buckets: bucket `i` holds
/// values in `[2^i, 2^(i+1))`. 32 buckets cover 1µs..~71min. Recording is
/// two relaxed atomic adds — cheap enough for every request.
pub struct Histogram {
    buckets: [AtomicU64; 32],
    count: AtomicU64,
    sum_us: AtomicU64,
}

impl Default for Histogram {
    fn default() -> Self {
        Self { buckets: std::array::from_fn(|_| AtomicU64::new(0)), count: AtomicU64::new(0), sum_us: AtomicU64::new(0) }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Default)]
pub struct Quantiles {
    pub count: u64,
    pub mean_us: f64,
    pub p50_us: u64,
    pub p90_us: u64,
    pub p99_us: u64,
    pub max_bucket_us: u64,
}

impl Histogram {
    #[inline]
    pub fn record(&self, d: Duration) {
        let us = d.as_micros().min(u64::MAX as u128) as u64;
        let i = (63 - us.max(1).leading_zeros()) as usize;
        self.buckets[i.min(31)].fetch_add(1, Relaxed);
        self.count.fetch_add(1, Relaxed);
        self.sum_us.fetch_add(us, Relaxed);
    }

    /// Quantiles reported as each bucket's upper bound (conservative).
    pub fn quantiles(&self) -> Quantiles {
        let counts: Vec<u64> = self.buckets.iter().map(|b| b.load(Relaxed)).collect();
        let total: u64 = counts.iter().sum();
        if total == 0 {
            return Quantiles::default();
        }
        let q = |p: f64| {
            let target = (total as f64 * p).ceil() as u64;
            let mut acc = 0;
            for (i, c) in counts.iter().enumerate() {
                acc += c;
                if acc >= target {
                    return 1u64 << (i + 1);
                }
            }
            1u64 << 32
        };
        let max_i = counts.iter().rposition(|&c| c > 0).unwrap_or(0);
        Quantiles {
            count: total,
            mean_us: self.sum_us.load(Relaxed) as f64 / total as f64,
            p50_us: q(0.5),
            p90_us: q(0.9),
            p99_us: q(0.99),
            max_bucket_us: 1 << (max_i + 1),
        }
    }

    fn prometheus(&self, name: &str, out: &mut String) {
        let _ = writeln!(out, "# TYPE {name}_seconds histogram");
        let mut acc = 0;
        for (i, b) in self.buckets.iter().enumerate() {
            let c = b.load(Relaxed);
            acc += c;
            if c > 0 || i < 20 {
                let le = (1u64 << (i + 1)) as f64 / 1e6;
                let _ = writeln!(out, "{name}_seconds_bucket{{le=\"{le}\"}} {acc}");
            }
        }
        let _ = writeln!(out, "{name}_seconds_bucket{{le=\"+Inf\"}} {acc}");
        let _ = writeln!(out, "{name}_seconds_sum {}", self.sum_us.load(Relaxed) as f64 / 1e6);
        let _ = writeln!(out, "{name}_seconds_count {}", self.count.load(Relaxed));
    }
}

macro_rules! counters {
    ($($name:ident: $help:literal),* $(,)?) => {
        #[derive(Default)]
        pub struct Counters { $(pub $name: AtomicU64,)* }

        impl Counters {
            pub fn snapshot(&self) -> serde_json::Map<String, serde_json::Value> {
                let mut m = serde_json::Map::new();
                $(m.insert(stringify!($name).into(), self.$name.load(Relaxed).into());)*
                m
            }

            fn prometheus(&self, out: &mut String) {
                $(
                    let _ = writeln!(out, "# HELP blinker_{} {}", stringify!($name), $help);
                    let _ = writeln!(out, "# TYPE blinker_{} counter", stringify!($name));
                    let _ = writeln!(out, "blinker_{} {}", stringify!($name), self.$name.load(Relaxed));
                )*
            }
        }
    };
}

counters! {
    http_requests: "HTTP requests served",
    action_gets: "Solana Action GETs (blink views)",
    action_posts: "Solana Action POSTs (transactions built)",
    blinks_created: "Blink drafts created",
    events_emitted: "Events accepted into the pipeline queue",
    events_dropped: "Low-priority events shed because the queue was full",
    events_committed: "Events durably committed to the event log",
    log_batches: "Group commits performed by the log writer",
    rpc_calls: "Solana JSON-RPC calls",
    rpc_errors: "Solana JSON-RPC calls that failed after retries",
    rpc_retries: "Solana JSON-RPC retry attempts",
    blockhash_refreshes: "Background blockhash cache refreshes",
    ws_connects: "Successful websocket (logsSubscribe) connections",
    ws_disconnects: "Websocket disconnects (each triggers a backoff reconnect)",
    ws_notifications: "logsSubscribe notifications received",
    verify_jobs: "Payment verification jobs started",
    verify_ok: "Transactions verified and applied",
    verify_retries: "Verification jobs re-queued (tx not yet visible)",
    verify_rejected: "Transactions that did not match any blink rule",
    verify_dedup: "Verification skipped: signature already processed",
}

#[derive(Default)]
pub struct Metrics {
    pub c: Counters,
    pub http_latency: Histogram,
    pub rpc_latency: Histogram,
    pub action_post_latency: Histogram,
    pub log_commit_latency: Histogram,
    pub verify_latency: Histogram,
    /// Last group-commit batch size, and the running max.
    pub last_batch: AtomicU64,
    pub max_batch: AtomicU64,
    pub queue_depth: AtomicU64,
    pub queue_capacity: AtomicU64,
    pub consumer_offset: AtomicU64,
    pub committed_offset: AtomicU64,
    pub ws_connected: AtomicU64,
}

impl Metrics {
    pub fn inc(c: &AtomicU64) {
        c.fetch_add(1, Relaxed);
    }

    pub fn prometheus(&self, extra: &[(&str, f64)]) -> String {
        let mut out = String::with_capacity(8 << 10);
        self.c.prometheus(&mut out);
        self.http_latency.prometheus("blinker_http_request", &mut out);
        self.rpc_latency.prometheus("blinker_rpc_call", &mut out);
        self.action_post_latency.prometheus("blinker_action_post", &mut out);
        self.log_commit_latency.prometheus("blinker_log_commit", &mut out);
        self.verify_latency.prometheus("blinker_verify", &mut out);
        let gauges = [
            ("blinker_queue_depth", self.queue_depth.load(Relaxed) as f64),
            ("blinker_queue_capacity", self.queue_capacity.load(Relaxed) as f64),
            ("blinker_log_last_batch", self.last_batch.load(Relaxed) as f64),
            ("blinker_consumer_lag", self.committed_offset.load(Relaxed).saturating_sub(self.consumer_offset.load(Relaxed)) as f64),
            ("blinker_ws_connected", self.ws_connected.load(Relaxed) as f64),
        ];
        for (k, v) in gauges.iter().chain(extra.iter()) {
            let _ = writeln!(out, "# TYPE {k} gauge\n{k} {v}");
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn histogram_quantiles() {
        let h = Histogram::default();
        for _ in 0..90 {
            h.record(Duration::from_micros(100)); // bucket [64,128)
        }
        for _ in 0..10 {
            h.record(Duration::from_millis(10)); // bucket [8192,16384)
        }
        let q = h.quantiles();
        assert_eq!(q.count, 100);
        assert_eq!(q.p50_us, 128);
        assert_eq!(q.p99_us, 16384);
    }
}
