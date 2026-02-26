//! Error types for the LLM crate.

use std::time::Duration;
use thiserror::Error;

/// Result type alias using the LLM error type.
pub type Result<T> = std::result::Result<T, LlmError>;

// ─────────────────────────────────────────────────────────────────────────────
// Rate Limit Info
// ─────────────────────────────────────────────────────────────────────────────

/// Information about a rate limit error.
#[derive(Debug, Clone)]
pub struct RateLimitInfo {
    /// The error message from the provider.
    pub message: String,
    /// How long to wait before retrying (if the provider specified).
    pub retry_after: Option<Duration>,
    /// The type of rate limit hit (if known).
    pub limit_type: Option<RateLimitType>,
}

/// Type of rate limit encountered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateLimitType {
    /// Tokens per minute limit.
    TokensPerMinute,
    /// Requests per minute limit.
    RequestsPerMinute,
    /// Requests per day limit.
    RequestsPerDay,
    /// Other/unknown limit type.
    Other,
}

impl RateLimitInfo {
    /// Create a new rate limit info with just a message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            retry_after: None,
            limit_type: None,
        }
    }

    /// Create a rate limit info with a retry duration.
    pub fn with_retry_after(message: impl Into<String>, retry_after: Duration) -> Self {
        Self {
            message: message.into(),
            retry_after: Some(retry_after),
            limit_type: None,
        }
    }

    /// Parse rate limit info from a Groq error message.
    ///
    /// Groq returns messages like:
    /// "Rate limit reached... Please try again in 6.57792s."
    pub fn parse_groq(message: &str) -> Self {
        let retry_after = parse_groq_retry_after(message);
        let limit_type = if message.contains("TPM") || message.contains("tokens per minute") {
            Some(RateLimitType::TokensPerMinute)
        } else if message.contains("RPM") || message.contains("requests per minute") {
            Some(RateLimitType::RequestsPerMinute)
        } else if message.contains("RPD") || message.contains("requests per day") {
            Some(RateLimitType::RequestsPerDay)
        } else {
            None
        };

        Self {
            message: message.to_string(),
            retry_after,
            limit_type,
        }
    }

    /// Parse rate limit info from OpenAI-style headers and body.
    pub fn parse_openai(message: &str, retry_after_header: Option<&str>) -> Self {
        let retry_after = retry_after_header.and_then(parse_retry_after_header);

        Self {
            message: message.to_string(),
            retry_after,
            limit_type: None,
        }
    }
}

impl std::fmt::Display for RateLimitInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(retry_after) = self.retry_after {
            write!(f, " (retry after {:.2}s)", retry_after.as_secs_f64())?;
        }
        Ok(())
    }
}

/// Parse Groq's "Please try again in Xs" format.
fn parse_groq_retry_after(message: &str) -> Option<Duration> {
    // Look for "try again in X" pattern
    let patterns = ["try again in ", "Try again in ", "retry in "];

    for pattern in patterns {
        if let Some(idx) = message.find(pattern) {
            let start = idx + pattern.len();
            let rest = &message[start..];

            // Extract the number (may include decimal point)
            let num_str: String = rest
                .chars()
                .take_while(|c| c.is_ascii_digit() || *c == '.')
                .collect();

            if let Ok(seconds) = num_str.parse::<f64>() {
                return Some(Duration::from_secs_f64(seconds));
            }
        }
    }

    None
}

/// Parse a Retry-After header value.
///
/// Supports both seconds (integer) and HTTP-date formats.
fn parse_retry_after_header(value: &str) -> Option<Duration> {
    // Try parsing as seconds first (most common)
    if let Ok(seconds) = value.trim().parse::<u64>() {
        return Some(Duration::from_secs(seconds));
    }

    // Could add HTTP-date parsing here if needed
    None
}

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

    /// Rate limit exceeded (retryable with backoff).
    #[error("Rate limit exceeded: {0}")]
    RateLimit(RateLimitInfo),

    /// Authentication failed.
    #[error("Authentication error: {0}")]
    Auth(String),

    /// Internal error.
    #[error("Internal error: {0}")]
    Internal(String),
}

impl LlmError {
    /// Create a rate limit error from a message string.
    ///
    /// This is a convenience method for cases where the provider doesn't
    /// give structured rate limit information.
    pub fn rate_limit(message: impl Into<String>) -> Self {
        Self::RateLimit(RateLimitInfo::new(message))
    }

    /// Create a rate limit error with retry timing.
    pub fn rate_limit_with_retry(message: impl Into<String>, retry_after: Duration) -> Self {
        Self::RateLimit(RateLimitInfo::with_retry_after(message, retry_after))
    }

    /// Get the retry-after duration if this is a rate limit error.
    pub fn retry_after(&self) -> Option<Duration> {
        match self {
            Self::RateLimit(info) => info.retry_after,
            _ => None,
        }
    }

    /// Returns true if this error is retryable.
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::Network(_) | Self::RateLimit(_))
    }

    /// Returns true if this is a tool validation error (LLM hallucinated a tool name).
    ///
    /// These errors are recoverable by providing feedback to the LLM about available tools.
    pub fn is_tool_validation_error(&self) -> bool {
        match self {
            Self::Backend(msg) => {
                msg.contains("tool call validation failed")
                    || msg.contains("was not in request.tools")
                    || msg.contains("unknown tool")
            }
            _ => false,
        }
    }

    /// Extract the invalid tool name from a tool validation error, if present.
    pub fn invalid_tool_name(&self) -> Option<&str> {
        match self {
            Self::Backend(msg) => {
                // Pattern: "attempted to call tool 'read_file' which was not"
                if let Some(start) = msg.find("call tool '") {
                    let rest = &msg[start + 11..];
                    if let Some(end) = rest.find('\'') {
                        return Some(&rest[..end]);
                    }
                }
                None
            }
            _ => None,
        }
    }
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
/// Network errors and rate limit errors are retryable.
/// Config, serialization, and other errors should not be retried.
pub fn is_retryable(error: &LlmError) -> bool {
    error.is_retryable()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_retryable() {
        assert!(is_retryable(&LlmError::Network("timeout".to_string())));
        assert!(is_retryable(&LlmError::rate_limit("rate limited")));
        assert!(!is_retryable(&LlmError::Config("bad config".to_string())));
        assert!(!is_retryable(&LlmError::Auth("unauthorized".to_string())));
        assert!(!is_retryable(&LlmError::Backend(
            "server error".to_string()
        )));
    }

    #[test]
    fn test_rate_limit_info_new() {
        let info = RateLimitInfo::new("Rate limited");
        assert_eq!(info.message, "Rate limited");
        assert!(info.retry_after.is_none());
        assert!(info.limit_type.is_none());
    }

    #[test]
    fn test_rate_limit_info_with_retry() {
        let info = RateLimitInfo::with_retry_after("Rate limited", Duration::from_secs(5));
        assert_eq!(info.message, "Rate limited");
        assert_eq!(info.retry_after, Some(Duration::from_secs(5)));
    }

    #[test]
    fn test_parse_groq_retry_after() {
        // Standard Groq format
        let info = RateLimitInfo::parse_groq(
            "Rate limit reached. Please try again in 6.57792s. Need more tokens?",
        );
        let retry = info.retry_after.unwrap();
        assert!((retry.as_secs_f64() - 6.57792).abs() < 0.001);

        // With TPM indicator
        let info = RateLimitInfo::parse_groq(
            "Rate limit reached for model on tokens per minute (TPM). try again in 10s",
        );
        assert_eq!(info.retry_after, Some(Duration::from_secs(10)));
        assert_eq!(info.limit_type, Some(RateLimitType::TokensPerMinute));

        // No retry timing
        let info = RateLimitInfo::parse_groq("Rate limit exceeded");
        assert!(info.retry_after.is_none());
    }

    #[test]
    fn test_parse_retry_after_header() {
        assert_eq!(parse_retry_after_header("5"), Some(Duration::from_secs(5)));
        assert_eq!(
            parse_retry_after_header(" 10 "),
            Some(Duration::from_secs(10))
        );
        assert_eq!(parse_retry_after_header("invalid"), None);
    }

    #[test]
    fn test_llm_error_retry_after() {
        let err = LlmError::rate_limit_with_retry("limited", Duration::from_secs(5));
        assert_eq!(err.retry_after(), Some(Duration::from_secs(5)));

        let err = LlmError::rate_limit("limited");
        assert_eq!(err.retry_after(), None);

        let err = LlmError::Network("timeout".to_string());
        assert_eq!(err.retry_after(), None);
    }

    #[test]
    fn test_rate_limit_info_display() {
        let info = RateLimitInfo::new("Rate limited");
        assert_eq!(info.to_string(), "Rate limited");

        let info = RateLimitInfo::with_retry_after("Rate limited", Duration::from_secs_f64(6.5));
        assert!(info.to_string().contains("retry after 6.50s"));
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

    #[test]
    fn test_is_tool_validation_error() {
        // Should match tool validation errors
        let err = LlmError::Backend(
            "tool call validation failed: attempted to call tool 'read_file' which was not in request.tools".to_string()
        );
        assert!(err.is_tool_validation_error());

        let err = LlmError::Backend("unknown tool: read_file".to_string());
        assert!(err.is_tool_validation_error());

        // Should not match other backend errors
        let err = LlmError::Backend("server error".to_string());
        assert!(!err.is_tool_validation_error());

        // Should not match other error types
        let err = LlmError::Network("timeout".to_string());
        assert!(!err.is_tool_validation_error());
    }

    #[test]
    fn test_invalid_tool_name_extraction() {
        let err = LlmError::Backend(
            "tool call validation failed: attempted to call tool 'read_file' which was not in request.tools".to_string()
        );
        assert_eq!(err.invalid_tool_name(), Some("read_file"));

        let err =
            LlmError::Backend("attempted to call tool 'file_reader' which was not".to_string());
        assert_eq!(err.invalid_tool_name(), Some("file_reader"));

        // No tool name extractable
        let err = LlmError::Backend("unknown tool error".to_string());
        assert_eq!(err.invalid_tool_name(), None);

        // Not a backend error
        let err = LlmError::Network("timeout".to_string());
        assert_eq!(err.invalid_tool_name(), None);
    }
}
