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

    /// Resource conflict (e.g., already exists).
    #[error("Conflict: {0}")]
    Conflict(String),

    /// Service unavailable.
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    /// Internal server error.
    #[error("Internal error: {0}")]
    Internal(String),

    /// Agent error (may wrap LLM rate limits).
    #[error("Agent error: {0}")]
    Agent(#[from] arawn_domain::AgentError),

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

impl From<arawn_domain::WorkstreamError> for ServerError {
    fn from(e: arawn_domain::WorkstreamError) -> Self {
        use arawn_domain::WorkstreamError;
        match e {
            WorkstreamError::NotFound(msg) => ServerError::NotFound(msg),
            WorkstreamError::Database(e) => ServerError::Storage(e.to_string()),
            WorkstreamError::Io(e) => ServerError::Storage(format!("IO error: {}", e)),
            WorkstreamError::Serde(e) => ServerError::Serialization(e),
            WorkstreamError::Migration(msg) => {
                ServerError::Internal(format!("Migration error: {}", msg))
            }
        }
    }
}

impl From<arawn_domain::ConfigError> for ServerError {
    fn from(e: arawn_domain::ConfigError) -> Self {
        use arawn_domain::ConfigError;
        match e {
            ConfigError::ContextNotFound(ctx) => {
                ServerError::NotFound(format!("Context '{}' not found", ctx))
            }
            ConfigError::LlmNotFound { name, .. } => {
                ServerError::NotFound(format!("LLM config '{}' not found", name))
            }
            ConfigError::NoDefaultLlm => {
                ServerError::BadRequest("No default LLM configured".to_string())
            }
            ConfigError::MissingField { field, context } => {
                ServerError::BadRequest(format!("Missing field '{}' in {}", field, context))
            }
            ConfigError::ApiKeyNotFound { backend, .. } => {
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

impl ServerError {
    /// Get the HTTP status code for this error.
    pub fn status_code(&self) -> StatusCode {
        if self.is_rate_limit() {
            return StatusCode::TOO_MANY_REQUESTS;
        }
        match self {
            ServerError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            ServerError::NotFound(_) => StatusCode::NOT_FOUND,
            ServerError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ServerError::Conflict(_) => StatusCode::CONFLICT,
            ServerError::RateLimitExceeded(_) => StatusCode::TOO_MANY_REQUESTS,
            ServerError::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Get the error code string for this error.
    pub fn error_code(&self) -> &'static str {
        if self.is_rate_limit() {
            return "rate_limit_exceeded";
        }
        match self {
            ServerError::Unauthorized(_) => "unauthorized",
            ServerError::NotFound(_) => "not_found",
            ServerError::BadRequest(_) => "bad_request",
            ServerError::Conflict(_) => "conflict",
            ServerError::RateLimitExceeded(_) => "rate_limit_exceeded",
            ServerError::ServiceUnavailable(_) => "service_unavailable",
            ServerError::Internal(_) => "internal_error",
            ServerError::Agent(_) => "agent_error",
            ServerError::Serialization(_) => "serialization_error",
            ServerError::Storage(_) => "storage_error",
            ServerError::Config(_) => "config_error",
        }
    }
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let retry_after = self.retry_after();
        let is_rate_limit = self.is_rate_limit();
        let status = self.status_code();
        let code = self.error_code();

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_code_unauthorized() {
        let err = ServerError::Unauthorized("bad token".into());
        assert_eq!(err.status_code(), StatusCode::UNAUTHORIZED);
        assert_eq!(err.error_code(), "unauthorized");
    }

    #[test]
    fn test_status_code_not_found() {
        let err = ServerError::NotFound("no such thing".into());
        assert_eq!(err.status_code(), StatusCode::NOT_FOUND);
        assert_eq!(err.error_code(), "not_found");
    }

    #[test]
    fn test_status_code_bad_request() {
        let err = ServerError::BadRequest("invalid input".into());
        assert_eq!(err.status_code(), StatusCode::BAD_REQUEST);
        assert_eq!(err.error_code(), "bad_request");
    }

    #[test]
    fn test_status_code_conflict() {
        let err = ServerError::Conflict("already exists".into());
        assert_eq!(err.status_code(), StatusCode::CONFLICT);
        assert_eq!(err.error_code(), "conflict");
    }

    #[test]
    fn test_status_code_service_unavailable() {
        let err = ServerError::ServiceUnavailable("not ready".into());
        assert_eq!(err.status_code(), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(err.error_code(), "service_unavailable");
    }

    #[test]
    fn test_status_code_internal() {
        let err = ServerError::Internal("boom".into());
        assert_eq!(err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(err.error_code(), "internal_error");
    }

    #[test]
    fn test_status_code_storage() {
        let err = ServerError::Storage("db down".into());
        assert_eq!(err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(err.error_code(), "storage_error");
    }

    #[test]
    fn test_status_code_config() {
        let err = ServerError::Config("bad config".into());
        assert_eq!(err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(err.error_code(), "config_error");
    }

    #[test]
    fn test_status_code_rate_limit() {
        let err = ServerError::RateLimitExceeded(RateLimitError::new("slow down"));
        assert_eq!(err.status_code(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(err.error_code(), "rate_limit_exceeded");
    }

    #[test]
    fn test_is_rate_limit_direct() {
        let err = ServerError::RateLimitExceeded(RateLimitError::new("slow down"));
        assert!(err.is_rate_limit());
    }

    #[test]
    fn test_is_rate_limit_false_for_non_rate_limit() {
        assert!(!ServerError::Internal("x".into()).is_rate_limit());
        assert!(!ServerError::NotFound("x".into()).is_rate_limit());
    }

    #[test]
    fn test_is_rate_limit_via_agent_error() {
        let llm_err = arawn_llm::LlmError::rate_limit("limited");
        let agent_err = arawn_agent::AgentError::Llm(llm_err);
        let err = ServerError::Agent(agent_err);
        assert!(err.is_rate_limit());
        assert_eq!(err.status_code(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(err.error_code(), "rate_limit_exceeded");
    }

    #[test]
    fn test_retry_after_direct_rate_limit() {
        let err = ServerError::RateLimitExceeded(RateLimitError::with_retry_after(
            "slow down",
            Duration::from_secs(10),
        ));
        assert_eq!(err.retry_after(), Some(Duration::from_secs(10)));
    }

    #[test]
    fn test_retry_after_none_when_not_set() {
        let err = ServerError::RateLimitExceeded(RateLimitError::new("slow down"));
        assert_eq!(err.retry_after(), None);
    }

    #[test]
    fn test_retry_after_via_agent_error() {
        let llm_err =
            arawn_llm::LlmError::rate_limit_with_retry("limited", Duration::from_secs(5));
        let agent_err = arawn_agent::AgentError::Llm(llm_err);
        let err = ServerError::Agent(agent_err);
        assert_eq!(err.retry_after(), Some(Duration::from_secs(5)));
    }

    #[test]
    fn test_retry_after_non_rate_limit() {
        assert_eq!(ServerError::Internal("x".into()).retry_after(), None);
    }

    #[test]
    fn test_from_workstream_not_found() {
        let ws_err = arawn_domain::WorkstreamError::NotFound("ws-123".into());
        let err: ServerError = ws_err.into();
        assert_eq!(err.status_code(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_from_workstream_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file gone");
        let ws_err = arawn_domain::WorkstreamError::Io(io_err);
        let err: ServerError = ws_err.into();
        assert_eq!(err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(err.error_code(), "storage_error");
    }

    #[test]
    fn test_from_workstream_migration() {
        let ws_err = arawn_domain::WorkstreamError::Migration("v2 failed".into());
        let err: ServerError = ws_err.into();
        assert_eq!(err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        assert!(err.to_string().contains("Migration"));
    }

    #[test]
    fn test_from_config_context_not_found() {
        let cfg_err = arawn_domain::ConfigError::ContextNotFound("prod".into());
        let err: ServerError = cfg_err.into();
        assert_eq!(err.status_code(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_from_config_no_default_llm() {
        let cfg_err = arawn_domain::ConfigError::NoDefaultLlm;
        let err: ServerError = cfg_err.into();
        assert_eq!(err.status_code(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_from_config_missing_field() {
        let cfg_err = arawn_domain::ConfigError::MissingField {
            field: "api_key".into(),
            context: "llm".into(),
        };
        let err: ServerError = cfg_err.into();
        assert_eq!(err.status_code(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_from_config_api_key_not_found() {
        let cfg_err = arawn_domain::ConfigError::ApiKeyNotFound {
            backend: "openai".into(),
            env_var: "OPENAI_API_KEY".into(),
        };
        let err: ServerError = cfg_err.into();
        assert_eq!(err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(err.error_code(), "config_error");
    }

    #[test]
    fn test_from_config_llm_not_found() {
        let cfg_err = arawn_domain::ConfigError::LlmNotFound {
            name: "gpt5".into(),
            context: "agent".into(),
        };
        let err: ServerError = cfg_err.into();
        assert_eq!(err.status_code(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_rate_limit_error_display_without_retry() {
        let err = RateLimitError::new("too fast");
        assert_eq!(err.to_string(), "too fast");
    }

    #[test]
    fn test_rate_limit_error_display_with_retry() {
        let err = RateLimitError::with_retry_after("too fast", Duration::from_secs(30));
        assert!(err.to_string().contains("retry after 30s"));
    }

    #[test]
    fn test_into_response_rate_limit_has_retry_after_header() {
        let err = ServerError::RateLimitExceeded(RateLimitError::with_retry_after(
            "slow down",
            Duration::from_secs(10),
        ));
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(response.headers().get("Retry-After").unwrap(), "10");
    }

    #[test]
    fn test_into_response_no_retry_after_header_for_non_rate_limit() {
        let err = ServerError::NotFound("gone".into());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert!(response.headers().get("Retry-After").is_none());
    }
}
