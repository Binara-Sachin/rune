mod config;
mod error;
mod server;

use anyhow::Context;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Shared application state passed to every request handler.
pub struct AppState {
    pub config: config::Config,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize structured logging.
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rune=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load config
    let config = config::Config::load()
        .context("Failed to load configuration from config.toml")?;

    tracing::info!(
        issuer = %config.oidc.issuer_url,
        bind = %config.server.bind_address,
        "Rune starting"
    );

    let bind_address = config.server.bind_address.clone();
    let state = Arc::new(AppState { config });

    let app = server::router(state);

    let listener = tokio::net::TcpListener::bind(&bind_address)
        .await
        .with_context(|| format!("Failed to bind to {}", bind_address))?;

    tracing::info!("Listening on http://{}", bind_address);

    axum::serve(listener, app)
        .await
        .context("Server error")?;

    Ok(())
}