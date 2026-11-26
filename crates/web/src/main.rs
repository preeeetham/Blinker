//! The Blinker server binary: one Tokio/Axum process serving the API,
//! the Solana Actions endpoints and the server-rendered Leptos UI.

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> anyhow_free::Result {
    use axum::Router;
    use blinker_web::app::{shell, App};
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};

    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,blinker_server=info,tower_http=warn".into()),
        )
        .init();

    let conf = get_configuration(None)?;
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    let cfg = blinker_server::Config::from_env().map_err(|e| e.to_string())?;
    let (api, _state) = blinker_server::start(cfg).await.map_err(|e| format!("{e:#}"))?;

    let ui = Router::new()
        .leptos_routes(&leptos_options, routes, {
            let opts = leptos_options.clone();
            move || shell(opts.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    let app = api.merge(ui).layer(tower_http::compression::CompressionLayer::new());

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Blinker listening on http://{addr}");
    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}

#[cfg(feature = "ssr")]
mod anyhow_free {
    pub type Result = std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // The client entry point is `hydrate()` in lib.rs.
}
