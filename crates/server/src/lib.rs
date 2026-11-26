//! Blinker's backend, as a library so the Leptos server binary and the
//! integration tests can embed it.

pub mod api;
pub mod config;
pub mod db;
pub mod metrics;
pub mod pipeline;
pub mod procstat;
pub mod reconcile;
pub mod rpc;
pub mod txbuild;
pub mod verify;
pub mod watcher;

use std::sync::{Arc, Mutex, RwLock};

use anyhow::Result;
use blinker_analytics::Engine;
use blinker_eventlog::LogStats;

pub use config::{Cluster, Config};

pub struct AppState {
    pub cfg: Config,
    pub db: sqlx::PgPool,
    pub rpc: rpc::Rpc,
    pub blockhash: rpc::BlockhashCache,
    pub bus: pipeline::EventBus,
    pub engine: Arc<RwLock<Engine>>,
    pub log_stats: Arc<Mutex<LogStats>>,
    pub verifier: verify::Verifier,
    pub metrics: Arc<metrics::Metrics>,
    pub proc: Arc<procstat::ProcSampler>,
    pub ata_cache: Arc<txbuild::AtaCache>,
}

/// Connects to Postgres, opens the event log, starts every background task
/// (log writer, analytics consumer, blockhash cache, verifier pool,
/// websocket watcher, process sampler) and returns the API router.
pub async fn start(cfg: Config) -> Result<(axum::Router, Arc<AppState>)> {
    let metrics = Arc::new(metrics::Metrics::default());
    let db = db::connect(&cfg.database_url).await?;
    tracing::info!("database connected, migrations applied");

    let pipeline = pipeline::start(
        &cfg.data_dir.join("events"),
        blinker_eventlog::Config {
            segment_bytes: cfg.log_segment_bytes,
            sync: cfg.log_sync,
            retention_bytes: Some(1 << 30),
        },
        cfg.event_queue_capacity,
        metrics.clone(),
    )?;

    let rpc = rpc::Rpc::new(&cfg.rpc_url, metrics.clone());
    let blockhash = rpc::BlockhashCache::new(rpc.clone());
    blockhash.spawn_refresher(metrics.clone());

    let ata_cache = Arc::new(txbuild::AtaCache::default());
    let verifier = verify::Verifier::spawn(
        rpc.clone(),
        db.clone(),
        pipeline.bus.clone(),
        cfg.treasury,
        cfg.verify_workers,
        ata_cache.clone(),
        metrics.clone(),
    );
    watcher::spawn(cfg.ws_url.clone(), cfg.treasury, rpc.clone(), verifier.clone(), metrics.clone());

    let proc = Arc::new(procstat::ProcSampler::new());
    proc.spawn();

    tracing::info!(cluster = cfg.cluster.name(), rpc = %cfg.rpc_url, treasury = %cfg.treasury, "blinker backend ready");
    let state = Arc::new(AppState {
        cfg,
        db,
        rpc,
        blockhash,
        bus: pipeline.bus,
        engine: pipeline.engine,
        log_stats: pipeline.log_stats,
        verifier,
        metrics,
        proc,
        ata_cache,
    });
    {
        let st = state.clone();
        tokio::spawn(async move {
            if let Err(e) = reconcile::run(&st).await {
                tracing::warn!("analytics reconciliation failed: {e:#}");
            }
        });
    }
    Ok((api::router(state.clone()), state))
}
