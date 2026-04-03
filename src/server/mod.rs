use crate::AppState;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde_json::json;
use std::sync::Arc;

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        // TODO: .route("/.well-known/openid-configuration", get(discovery_handler))
        // TODO: .route("/authorize", get(authorize_handler))
        // TODO: .route("/token", post(token_handler))
        .with_state(state)
}

/// Basic health check endpoint.
async fn health_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "status": "ok",
            "issuer": state.config.oidc.issuer_url,
        })),
    )
}