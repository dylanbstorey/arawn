//! Error types for the server.

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use std::time::Duration;
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

    /// Rate limit exceeded with optional retry timing.
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(RateLimitError),

    /// Service unavailable.
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    /// Internal server error.
    #[error("Internal error: {0}")]
    Internal(String),

    /// Agent error (may wrap LLM rate limits).
    #[error("Agent error: {0}")]
    Agent(#[from] arawn_agent::AgentError),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Database/storage error.
    #[error("Storage error: {0}")]
    Storage(String),

    /// Configuration error.
    #[error("Configuration error: {0}")]
    Config(String),
}

/// Rate limit error with optional retry timing.
#[derive(Debug, Clone)]
pub struct RateLimitError {
    /// Error message.
    pub message: String,
    /// How long to wait before retrying.
    pub retry_after: Option<Duration>,
}

impl std::fmt::Display for RateLimitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(retry_after) = self.retry_after {
            write!(f, " (retry after {}s)", retry_after.as_secs())?;
        }
        Ok(())
    }
}

impl RateLimitError {
    /// Create a new rate limit error.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            retry_after: None,
        }
    }

    /// Create a rate limit error with retry timing.
    pub fn with_retry_after(message: impl Into<String>, retry_after: Duration) -> Self {
        Self {
            message: message.into(),
            retry_after: Some(retry_after),
        }
    }
}

impl ServerError {
    /// Check if this is a rate limit error and extract retry timing.
    pub fn retry_after(&self) -> Option<Duration> {
        match self {
            Self::RateLimitExceeded(info) => info.retry_after,
            Self::Agent(agent_err) => {
                // Check if the agent error wraps an LLM rate limit
                if let Some(llm_err) = agent_err.llm_error() {
                    llm_err.retry_after()
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Check if this error should be returned as HTTP 429.
    pub fn is_rate_limit(&self) -> bool {
        match self {
            Self::RateLimitExceeded(_) => true,
            Self::Agent(agent_err) => agent_err.is_rate_limit(),
            _ => false,
        }
    }
}

impl From<arawn_workstream::WorkstreamError> for ServerError {
    fn from(e: arawn_workstream::WorkstreamError) -> Self {
        match e {
            arawn_workstream::WorkstreamError::NotFound(msg) => ServerError::NotFound(msg),
            arawn_workstream::WorkstreamError::Database(e) => {
                ServerError::Storage(e.to_string())
            }
            arawn_workstream::WorkstreamError::Io(e) => {
                ServerError::Storage(format!("IO error: {}", e))
            }
            arawn_workstream::WorkstreamError::Serde(e) => {
                ServerError::Serialization(e)
            }
            arawn_workstream::WorkstreamError::Migration(msg) => {
                ServerError::Internal(format!("Migration error: {}", msg))
            }
        }
    }
}

impl From<arawn_config::ConfigError> for ServerError {
    fn from(e: arawn_config::ConfigError) -> Self {
        match e {
            arawn_config::ConfigError::ContextNotFound(ctx) => {
                ServerError::NotFound(format!("Context '{}' not found", ctx))
            }
            arawn_config::ConfigError::LlmNotFound { name, .. } => {
                ServerError::NotFound(format!("LLM config '{}' not found", name))
            }
            arawn_config::ConfigError::NoDefaultLlm => {
                ServerError::BadRequest("No default LLM configured".to_string())
            }
            arawn_config::ConfigError::MissingField { field, context } => {
                ServerError::BadRequest(format!("Missing field '{}' in {}", field, context))
            }
            arawn_config::ConfigError::ApiKeyNotFound { backend, .. } => {
                ServerError::Config(format!("API key not found for backend '{}'", backend))
            }
            _ => ServerError::Config(e.to_string()),
        }
    }
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
        // Check for rate limit (including wrapped in Agent errors)
        let retry_after = self.retry_after();
        let is_rate_limit = self.is_rate_limit();

        let (status, code) = if is_rate_limit {
            (StatusCode::TOO_MANY_REQUESTS, "rate_limit_exceeded")
        } else {
            match &self {
                ServerError::Unauthorized(_) => (StatusCode::UNAUTHORIZED, "unauthorized"),
                ServerError::NotFound(_) => (StatusCode::NOT_FOUND, "not_found"),
                ServerError::BadRequest(_) => (StatusCode::BAD_REQUEST, "bad_request"),
                ServerError::RateLimitExceeded(_) => {
                    (StatusCode::TOO_MANY_REQUESTS, "rate_limit_exceeded")
                }
                ServerError::ServiceUnavailable(_) => {
                    (StatusCode::SERVICE_UNAVAILABLE, "service_unavailable")
                }
                ServerError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
                ServerError::Agent(_) => (StatusCode::INTERNAL_SERVER_ERROR, "agent_error"),
                ServerError::Serialization(_) => {
                    (StatusCode::INTERNAL_SERVER_ERROR, "serialization_error")
                }
                ServerError::Storage(_) => (StatusCode::INTERNAL_SERVER_ERROR, "storage_error"),
                ServerError::Config(_) => (StatusCode::INTERNAL_SERVER_ERROR, "config_error"),
            }
        };

        let message = self.to_string();

        match &self {
            ServerError::Internal(_) | ServerError::Agent(_) | ServerError::Serialization(_)
                if !is_rate_limit =>
            {
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

        // Build response with optional Retry-After header
        let mut response = (status, Json(body)).into_response();

        if let Some(retry_after) = retry_after {
            let seconds = retry_after.as_secs().max(1); // At least 1 second
            if let Ok(value) = axum::http::HeaderValue::from_str(&seconds.to_string()) {
                response.headers_mut().insert("Retry-After", value);
            }
        }

        response
    }
}
