//! Headless API server (no UI). Useful for load tests and profiling the
//! backend in isolation: `cargo run --release -p blinker-server --bin blinker-api`.
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();
    let cfg = blinker_server::Config::from_env()?;
    let addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:3000".into());
    let (router, _state) = blinker_server::start(cfg).await?;
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("blinker-api listening on http://{addr}");
    axum::serve(listener, router).with_graceful_shutdown(async { let _ = tokio::signal::ctrl_c().await; }).await?;
    Ok(())
}
