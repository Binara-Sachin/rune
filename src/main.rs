mod config;
mod crypto;
mod error;
mod oidc;
mod server;
mod store;
mod ui;

use axum::{routing::get, Json, Router};
use tracing::info;

#[tokio::main]
async fn main() {
    // Initialize the tracing subscriber with default INFO logs
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Starting Rune");

    // Build the HTTP router.
    let app = Router::new().route("/health", get(health));

    // Bind to a TCP listener on localhost port 3000.
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Failed to bind to port 3000");

    info!("Listening on http://127.0.0.1:3000");

    // Hand the listener and router to axum and start serving.
    axum::serve(listener, app)
        .await
        .expect("Server error");
}

/// Health check handler.
async fn health() -> Json<serde_json::Value> {
    info!("Health Check endpoint called.");
    Json(serde_json::json!({
        "status": "ok"
    }))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify that the health handler returns the expected JSON shape.
    #[tokio::test]
    async fn test_health_returns_ok() {
        let Json(body) = health().await;
        assert_eq!(body["status"], "ok");
    }
}