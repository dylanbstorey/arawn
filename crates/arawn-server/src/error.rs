//! Error types for the server.

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;

/// Server error type.
#[derive(Debug, Error)]
pub enum ServerError {
    /// Authentication failed.
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    /// Resource not found.
    #[error("Not found: {0}")]
    NotFound(String),

    /// Bad request.
    #[error("Bad request: {0}")]
    BadRequest(String),

    /// Rate limit exceeded.
    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    /// Internal server error.
    #[error("Internal error: {0}")]
    Internal(String),

    /// Agent error.
    #[error("Agent error: {0}")]
    Agent(#[from] arawn_agent::AgentError),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Result type for server operations.
pub type Result<T> = std::result::Result<T, ServerError>;

/// Error response body.
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    /// Error code for programmatic handling.
    pub code: String,
    /// Human-readable error message.
    pub message: String,
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let (status, code) = match &self {
            ServerError::Unauthorized(_) => (StatusCode::UNAUTHORIZED, "unauthorized"),
            ServerError::NotFound(_) => (StatusCode::NOT_FOUND, "not_found"),
            ServerError::BadRequest(_) => (StatusCode::BAD_REQUEST, "bad_request"),
            ServerError::RateLimitExceeded => {
                (StatusCode::TOO_MANY_REQUESTS, "rate_limit_exceeded")
            }
            ServerError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
            ServerError::Agent(_) => (StatusCode::INTERNAL_SERVER_ERROR, "agent_error"),
            ServerError::Serialization(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "serialization_error")
            }
        };

        let message = self.to_string();

        match &self {
            ServerError::Internal(_) | ServerError::Agent(_) | ServerError::Serialization(_) => {
                tracing::error!(status = %status, code, error = %message, "Server error");
            }
            _ => {
                tracing::warn!(status = %status, code, error = %message, "Client error");
            }
        }

        let body = ErrorResponse {
            code: code.to_string(),
            message,
        };

        (status, Json(body)).into_response()
    }
}
