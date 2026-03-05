//! Parameter validation error types and helper traits.

use crate::error::AgentError;

/// Error type for tool parameter validation failures.
///
/// Provides detailed error messages that help the LLM understand what went wrong
/// and how to fix it.
///
/// # Examples
///
/// ```rust,ignore
/// use arawn_agent::tool::ParameterValidationError;
///
/// let err = ParameterValidationError::missing("path", "provide a file path");
/// assert!(err.to_string().contains("path"));
/// assert_eq!(err.parameter_name(), Some("path"));
/// ```
#[derive(Debug, Clone, thiserror::Error)]
pub enum ParameterValidationError {
    /// A required parameter is missing.
    #[error("missing required parameter '{name}': {hint}")]
    MissingRequired {
        /// The parameter name.
        name: &'static str,
        /// Hint for the LLM on how to fix.
        hint: &'static str,
    },

    /// A parameter has an invalid type.
    #[error("invalid type for '{name}': expected {expected}, got {actual}")]
    InvalidType {
        /// The parameter name.
        name: &'static str,
        /// The expected type.
        expected: &'static str,
        /// The actual type found.
        actual: String,
    },

    /// A parameter value is out of range.
    #[error("'{name}' value {value} is out of range: {constraint}")]
    OutOfRange {
        /// The parameter name.
        name: &'static str,
        /// The actual value as string.
        value: String,
        /// Description of the valid range.
        constraint: String,
    },

    /// A parameter value doesn't match expected pattern/enum.
    #[error("'{name}' has invalid value '{value}': {message}")]
    InvalidValue {
        /// The parameter name.
        name: &'static str,
        /// The invalid value.
        value: String,
        /// Why it's invalid.
        message: String,
    },

    /// Multiple validation errors.
    #[error("parameter validation failed: {}", .0.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("; "))]
    Multiple(Vec<ParameterValidationError>),
}

impl ParameterValidationError {
    /// Create a missing required parameter error.
    pub fn missing(name: &'static str, hint: &'static str) -> Self {
        Self::MissingRequired { name, hint }
    }

    /// Create an invalid type error.
    pub fn invalid_type(
        name: &'static str,
        expected: &'static str,
        actual: impl Into<String>,
    ) -> Self {
        Self::InvalidType {
            name,
            expected,
            actual: actual.into(),
        }
    }

    /// Create an out of range error.
    pub fn out_of_range(
        name: &'static str,
        value: impl ToString,
        constraint: impl Into<String>,
    ) -> Self {
        Self::OutOfRange {
            name,
            value: value.to_string(),
            constraint: constraint.into(),
        }
    }

    /// Create an invalid value error.
    pub fn invalid_value(
        name: &'static str,
        value: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::InvalidValue {
            name,
            value: value.into(),
            message: message.into(),
        }
    }

    /// Create from multiple errors.
    pub fn multiple(errors: Vec<ParameterValidationError>) -> Self {
        Self::Multiple(errors)
    }

    /// Get the parameter name associated with this error (if single error).
    pub fn parameter_name(&self) -> Option<&str> {
        match self {
            Self::MissingRequired { name, .. } => Some(name),
            Self::InvalidType { name, .. } => Some(name),
            Self::OutOfRange { name, .. } => Some(name),
            Self::InvalidValue { name, .. } => Some(name),
            Self::Multiple(_) => None,
        }
    }
}

impl From<ParameterValidationError> for AgentError {
    fn from(err: ParameterValidationError) -> Self {
        AgentError::Tool(err.to_string())
    }
}

/// Result type for parameter validation.
pub type ParamResult<T> = std::result::Result<T, ParameterValidationError>;

/// Helper trait for extracting and validating parameters from JSON.
///
/// # Examples
///
/// ```rust,ignore
/// use arawn_agent::tool::ParamExt;
///
/// let params = serde_json::json!({"command": "ls -la", "timeout": 30});
/// let cmd = params.required_str("command", "provide a shell command")?;
/// let timeout = params.optional_u64("timeout", 60);
/// ```
pub trait ParamExt {
    /// Get a required string parameter.
    fn required_str(&self, name: &'static str, hint: &'static str) -> ParamResult<&str>;

    /// Get an optional string parameter.
    fn optional_str(&self, name: &str) -> Option<&str>;

    /// Get a required integer parameter.
    fn required_i64(&self, name: &'static str, hint: &'static str) -> ParamResult<i64>;

    /// Get an optional integer parameter with default.
    fn optional_i64(&self, name: &str, default: i64) -> i64;

    /// Get an optional u64 parameter with default.
    fn optional_u64(&self, name: &str, default: u64) -> u64;

    /// Get a required boolean parameter.
    fn required_bool(&self, name: &'static str, hint: &'static str) -> ParamResult<bool>;

    /// Get an optional boolean parameter with default.
    fn optional_bool(&self, name: &str, default: bool) -> bool;

    /// Get an optional array parameter.
    fn optional_array(&self, name: &str) -> Option<&Vec<serde_json::Value>>;
}

impl ParamExt for serde_json::Value {
    fn required_str(&self, name: &'static str, hint: &'static str) -> ParamResult<&str> {
        self.get(name)
            .and_then(|v| v.as_str())
            .ok_or_else(|| ParameterValidationError::missing(name, hint))
    }

    fn optional_str(&self, name: &str) -> Option<&str> {
        self.get(name).and_then(|v| v.as_str())
    }

    fn required_i64(&self, name: &'static str, hint: &'static str) -> ParamResult<i64> {
        self.get(name)
            .and_then(|v| v.as_i64())
            .ok_or_else(|| ParameterValidationError::missing(name, hint))
    }

    fn optional_i64(&self, name: &str, default: i64) -> i64 {
        self.get(name).and_then(|v| v.as_i64()).unwrap_or(default)
    }

    fn optional_u64(&self, name: &str, default: u64) -> u64 {
        self.get(name).and_then(|v| v.as_u64()).unwrap_or(default)
    }

    fn required_bool(&self, name: &'static str, hint: &'static str) -> ParamResult<bool> {
        self.get(name)
            .and_then(|v| v.as_bool())
            .ok_or_else(|| ParameterValidationError::missing(name, hint))
    }

    fn optional_bool(&self, name: &str, default: bool) -> bool {
        self.get(name).and_then(|v| v.as_bool()).unwrap_or(default)
    }

    fn optional_array(&self, name: &str) -> Option<&Vec<serde_json::Value>> {
        self.get(name).and_then(|v| v.as_array())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_param_validation_error_missing() {
        let err = ParameterValidationError::missing("command", "provide a shell command");
        assert!(err.to_string().contains("command"));
        assert!(err.to_string().contains("missing"));
        assert_eq!(err.parameter_name(), Some("command"));
    }

    #[test]
    fn test_param_validation_error_invalid_type() {
        let err = ParameterValidationError::invalid_type("count", "integer", "string");
        assert!(err.to_string().contains("count"));
        assert!(err.to_string().contains("integer"));
        assert!(err.to_string().contains("string"));
    }

    #[test]
    fn test_param_validation_error_out_of_range() {
        let err = ParameterValidationError::out_of_range("timeout", 0, "must be > 0 and < 3600");
        assert!(err.to_string().contains("timeout"));
        assert!(err.to_string().contains("0"));
        assert!(err.to_string().contains("3600"));
    }

    #[test]
    fn test_param_validation_error_invalid_value() {
        let err = ParameterValidationError::invalid_value(
            "action",
            "delete",
            "must be 'read' or 'write'",
        );
        assert!(err.to_string().contains("action"));
        assert!(err.to_string().contains("delete"));
    }

    #[test]
    fn test_param_validation_error_multiple() {
        let errors = vec![
            ParameterValidationError::missing("a", "hint"),
            ParameterValidationError::missing("b", "hint"),
        ];
        let err = ParameterValidationError::multiple(errors);
        let msg = err.to_string();
        assert!(msg.contains("a"));
        assert!(msg.contains("b"));
        assert_eq!(err.parameter_name(), None);
    }

    #[test]
    fn test_param_ext_required_str() {
        let params = serde_json::json!({"command": "ls -la"});
        assert_eq!(params.required_str("command", "hint").unwrap(), "ls -la");

        let err = params.required_str("missing", "provide it").unwrap_err();
        assert!(matches!(
            err,
            ParameterValidationError::MissingRequired {
                name: "missing",
                ..
            }
        ));
    }

    #[test]
    fn test_param_ext_optional_str() {
        let params = serde_json::json!({"path": "/tmp"});
        assert_eq!(params.optional_str("path"), Some("/tmp"));
        assert_eq!(params.optional_str("missing"), None);
    }

    #[test]
    fn test_param_ext_required_i64() {
        let params = serde_json::json!({"count": 42});
        assert_eq!(params.required_i64("count", "hint").unwrap(), 42);

        let err = params.required_i64("missing", "hint").unwrap_err();
        assert!(matches!(
            err,
            ParameterValidationError::MissingRequired {
                name: "missing",
                ..
            }
        ));
    }

    #[test]
    fn test_param_ext_optional_i64() {
        let params = serde_json::json!({"limit": 100});
        assert_eq!(params.optional_i64("limit", 10), 100);
        assert_eq!(params.optional_i64("missing", 10), 10);
    }

    #[test]
    fn test_param_ext_optional_u64() {
        let params = serde_json::json!({"timeout": 30});
        assert_eq!(params.optional_u64("timeout", 60), 30);
        assert_eq!(params.optional_u64("missing", 60), 60);
    }

    #[test]
    fn test_param_ext_required_bool() {
        let params = serde_json::json!({"enabled": true});
        assert!(params.required_bool("enabled", "hint").unwrap());

        let err = params.required_bool("missing", "hint").unwrap_err();
        assert!(matches!(
            err,
            ParameterValidationError::MissingRequired {
                name: "missing",
                ..
            }
        ));
    }

    #[test]
    fn test_param_ext_optional_bool() {
        let params = serde_json::json!({"verbose": true});
        assert!(params.optional_bool("verbose", false));
        assert!(!params.optional_bool("missing", false));
    }

    #[test]
    fn test_param_ext_optional_array() {
        let params = serde_json::json!({"items": [1, 2, 3]});
        assert!(params.optional_array("items").is_some());
        assert!(params.optional_array("missing").is_none());
    }

    #[test]
    fn test_param_validation_error_into_agent_error() {
        let param_err = ParameterValidationError::missing("test", "hint");
        let agent_err: AgentError = param_err.into();
        assert!(matches!(agent_err, AgentError::Tool(_)));
    }
}
