//! Error types for the LLM crate.

use thiserror::Error;

/// Result type alias using the LLM error type.
pub type Result<T> = std::result::Result<T, LlmError>;

// ─────────────────────────────────────────────────────────────────────────────
// Response Validation Errors
// ─────────────────────────────────────────────────────────────────────────────

/// Error type for LLM response validation failures.
///
/// These errors indicate that a response from an LLM provider didn't match
/// the expected structure or constraints. This helps catch malformed data
/// before it causes issues in the agent loop.
#[derive(Debug, Clone, Error)]
pub enum ResponseValidationError {
    /// A required field is missing from the response.
    #[error("missing required field '{field}' in response")]
    MissingField {
        /// The name of the missing field.
        field: &'static str,
    },

    /// A tool_use block has invalid structure.
    #[error("invalid tool_use block '{id}': {reason}")]
    InvalidToolUse {
        /// The tool use ID (if available).
        id: String,
        /// Why the tool use is invalid.
        reason: String,
    },

    /// Token count has an invalid value.
    #[error("invalid token count: {field} has value {value}, {constraint}")]
    InvalidTokenCount {
        /// The field name (e.g., "input_tokens").
        field: &'static str,
        /// The invalid value.
        value: i64,
        /// The constraint that was violated.
        constraint: &'static str,
    },

    /// Content block has malformed structure.
    #[error("malformed content block at index {index}: {reason}")]
    MalformedContent {
        /// Index of the malformed content block.
        index: usize,
        /// Why it's malformed.
        reason: String,
    },

    /// Stop reason is invalid or unexpected.
    #[error("invalid stop_reason: {reason}")]
    InvalidStopReason {
        /// Why the stop reason is invalid.
        reason: String,
    },

    /// Streaming event has invalid structure.
    #[error("invalid stream event: {reason}")]
    InvalidStreamEvent {
        /// Why the event is invalid.
        reason: String,
    },

    /// Multiple validation errors occurred.
    #[error("multiple validation errors: {}", .0.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("; "))]
    Multiple(Vec<ResponseValidationError>),
}

impl ResponseValidationError {
    /// Create a missing field error.
    pub fn missing_field(field: &'static str) -> Self {
        Self::MissingField { field }
    }

    /// Create an invalid tool use error.
    pub fn invalid_tool_use(id: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidToolUse {
            id: id.into(),
            reason: reason.into(),
        }
    }

    /// Create an invalid token count error.
    pub fn invalid_token_count(field: &'static str, value: i64, constraint: &'static str) -> Self {
        Self::InvalidTokenCount {
            field,
            value,
            constraint,
        }
    }

    /// Create a malformed content error.
    pub fn malformed_content(index: usize, reason: impl Into<String>) -> Self {
        Self::MalformedContent {
            index,
            reason: reason.into(),
        }
    }

    /// Create an invalid stop reason error.
    pub fn invalid_stop_reason(reason: impl Into<String>) -> Self {
        Self::InvalidStopReason {
            reason: reason.into(),
        }
    }

    /// Create an invalid stream event error.
    pub fn invalid_stream_event(reason: impl Into<String>) -> Self {
        Self::InvalidStreamEvent {
            reason: reason.into(),
        }
    }

    /// Create from multiple errors.
    pub fn multiple(errors: Vec<ResponseValidationError>) -> Self {
        Self::Multiple(errors)
    }

    /// Returns true if this is a critical error that should abort processing.
    pub fn is_critical(&self) -> bool {
        matches!(
            self,
            Self::MissingField { .. } | Self::InvalidToolUse { .. }
        )
    }
}

impl From<ResponseValidationError> for LlmError {
    fn from(err: ResponseValidationError) -> Self {
        LlmError::InvalidRequest(err.to_string())
    }
}

/// Error type for LLM operations.
#[derive(Debug, Error)]
pub enum LlmError {
    /// Backend/API error from the provider.
    #[error("Backend error: {0}")]
    Backend(String),

    /// Network/connectivity error (retryable).
    #[error("Network error: {0}")]
    Network(String),

    /// Configuration error (API key missing, etc.).
    #[error("Configuration error: {0}")]
    Config(String),

    /// Serialization/deserialization error.
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Invalid request parameters.
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Rate limit exceeded.
    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    /// Authentication failed.
    #[error("Authentication error: {0}")]
    Auth(String),

    /// Internal error.
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<reqwest::Error> for LlmError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            LlmError::Network(format!("Request timed out: {}", err))
        } else if err.is_connect() {
            LlmError::Network(format!("Connection failed: {}", err))
        } else {
            LlmError::Network(err.to_string())
        }
    }
}

impl From<serde_json::Error> for LlmError {
    fn from(err: serde_json::Error) -> Self {
        LlmError::Serialization(err.to_string())
    }
}

/// Check if an error is retryable.
///
/// Only network errors are considered retryable. Config, serialization,
/// and other errors should not be retried.
pub fn is_retryable(error: &LlmError) -> bool {
    matches!(error, LlmError::Network(_))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_retryable() {
        assert!(is_retryable(&LlmError::Network("timeout".to_string())));
        assert!(!is_retryable(&LlmError::Config("bad config".to_string())));
        assert!(!is_retryable(&LlmError::Auth("unauthorized".to_string())));
        assert!(!is_retryable(&LlmError::Backend(
            "server error".to_string()
        )));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Response Validation Error Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_missing_field_error() {
        let err = ResponseValidationError::missing_field("id");
        assert!(err.to_string().contains("id"));
        assert!(err.to_string().contains("missing"));
        assert!(err.is_critical());
    }

    #[test]
    fn test_invalid_tool_use_error() {
        let err = ResponseValidationError::invalid_tool_use("tool_123", "empty name");
        assert!(err.to_string().contains("tool_123"));
        assert!(err.to_string().contains("empty name"));
        assert!(err.is_critical());
    }

    #[test]
    fn test_invalid_token_count_error() {
        let err = ResponseValidationError::invalid_token_count("input_tokens", -5, "must be >= 0");
        assert!(err.to_string().contains("input_tokens"));
        assert!(err.to_string().contains("-5"));
        assert!(err.to_string().contains("must be >= 0"));
        assert!(!err.is_critical());
    }

    #[test]
    fn test_malformed_content_error() {
        let err = ResponseValidationError::malformed_content(2, "unknown block type");
        assert!(err.to_string().contains("index 2"));
        assert!(err.to_string().contains("unknown block type"));
        assert!(!err.is_critical());
    }

    #[test]
    fn test_invalid_stop_reason_error() {
        let err = ResponseValidationError::invalid_stop_reason("unknown value 'foo'");
        assert!(err.to_string().contains("unknown value 'foo'"));
        assert!(!err.is_critical());
    }

    #[test]
    fn test_invalid_stream_event_error() {
        let err = ResponseValidationError::invalid_stream_event("missing index");
        assert!(err.to_string().contains("missing index"));
        assert!(!err.is_critical());
    }

    #[test]
    fn test_multiple_errors() {
        let errors = vec![
            ResponseValidationError::missing_field("id"),
            ResponseValidationError::invalid_tool_use("x", "bad"),
        ];
        let err = ResponseValidationError::multiple(errors);
        let msg = err.to_string();
        assert!(msg.contains("id"));
        assert!(msg.contains("bad"));
    }

    #[test]
    fn test_validation_error_into_llm_error() {
        let val_err = ResponseValidationError::missing_field("id");
        let llm_err: LlmError = val_err.into();
        assert!(matches!(llm_err, LlmError::InvalidRequest(_)));
    }
}
