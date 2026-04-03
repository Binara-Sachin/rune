use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// The central error type for the Rune OIDC provider.

#[derive(Debug, thiserror::Error)]
pub enum RuneError {
    // ── Infrastructure Errors ────────────────────────────────────────────────

    #[error("Configuration error: {0}")]
    Config(#[from] crate::config::ConfigError),

    #[error("Internal server error: {0}")]
    Internal(String),


    // ── OIDC Protocol Errors (RFC 6749 §5.2) ────────────────────────────────

    /// The request is missing a required parameter or contains an invalid value.
    #[error("invalid_request: {description}")]
    InvalidRequest { description: String },

    /// Client authentication failed.
    /// Per RFC 6749 §5.2: respond with HTTP 401 and WWW-Authenticate header.
    #[error("invalid_client: {description}")]
    InvalidClient { description: String },

    /// The provided authorization grant is invalid, expired, or revoked.
    #[error("invalid_grant: {description}")]
    InvalidGrant { description: String },

    /// The client is not authorized to use this grant type.
    #[error("unauthorized_client: {description}")]
    UnauthorizedClient { description: String },

    /// The requested scope is invalid or unknown.
    #[error("invalid_scope: {description}")]
    InvalidScope { description: String },


    // ── Storage Errors ────────────────────────────────────────────────────────

    #[error("Storage error: {0}")]
    Storage(String), // TODO: Replace with a typed StorageError enum

    // ── Crypto Errors ─────────────────────────────────────────────────────────

    #[error("Crypto error: {0}")]
    Crypto(String), // TODO: Replace with a typed CryptoError enum
}

impl RuneError {
    /// Returns the HTTP status code appropriate for this error.
    fn status_code(&self) -> StatusCode {
        match self {
            // RFC 6749 §5.2: invalid_client → 401
            RuneError::InvalidClient { .. } => StatusCode::UNAUTHORIZED,

            // RFC 6749 §5.2: all other token endpoint errors → 400
            RuneError::InvalidRequest { .. }
            | RuneError::InvalidGrant { .. }
            | RuneError::UnauthorizedClient { .. }
            | RuneError::InvalidScope { .. } => StatusCode::BAD_REQUEST,

            // Config is always our fault
            RuneError::Config(_) | RuneError::Internal(_) | RuneError::Storage(_) | RuneError::Crypto(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }

    /// Returns the machine-readable error code string.
    /// Per RFC 6749 §5.2, error responses MUST include an `error` field containing one of the defined error codes.
    fn error_code(&self) -> &'static str {
        match self {
            RuneError::InvalidRequest { .. } => "invalid_request",
            RuneError::InvalidClient { .. } => "invalid_client",
            RuneError::InvalidGrant { .. } => "invalid_grant",
            RuneError::UnauthorizedClient { .. } => "unauthorized_client",
            RuneError::InvalidScope { .. } => "invalid_scope",
            RuneError::Config(_) | RuneError::Internal(_) | RuneError::Storage(_) | RuneError::Crypto(_) => {
                "server_error"
            }
        }
    }
}

/// Implement axum's `IntoResponse` so any handler can return `Result<T, RuneError>`
impl IntoResponse for RuneError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let code = self.error_code();

        // For internal errors, don't leak implementation details to clients.
        let (log_message, client_description) = match &self {
            RuneError::Internal(msg) => {
                tracing::error!(error = %self, "Internal server error");
                (msg.clone(), "An internal server error occurred.".to_string())
            }
            RuneError::Config(e) => {
                tracing::error!(error = %e, "Configuration error");
                (e.to_string(), "Server misconfiguration.".to_string())
            }
            RuneError::Storage(msg) => {
                tracing::error!(error = %msg, "Storage error");
                (msg.clone(), "An internal server error occurred.".to_string())
            }
            RuneError::Crypto(msg) => {
                tracing::error!(error = %msg, "Crypto error");
                (msg.clone(), "An internal server error occurred.".to_string())
            }
            // OIDC protocol errors: the description is intentionally client-facing
            RuneError::InvalidRequest { description }
            | RuneError::InvalidClient { description }
            | RuneError::InvalidGrant { description }
            | RuneError::UnauthorizedClient { description }
            | RuneError::InvalidScope { description } => {
                tracing::warn!(error_code = code, description = %description, "OIDC protocol error");
                (description.clone(), description.clone())
            }
        };

        // RFC 6749 §5.2 mandates this exact JSON shape for error responses.
        let body = json!({
            "error": code,
            "error_description": client_description,
        });

        tracing::debug!(
            status = status.as_u16(),
            error_code = code,
            message = %log_message,
            "Sending error response"
        );

        (status, Json(body)).into_response()
    }
}