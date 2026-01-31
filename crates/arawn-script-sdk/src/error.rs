//! Error types for script execution.

use std::fmt;

/// Result type for script functions.
pub type ScriptResult<T> = Result<T, ScriptError>;

/// Error type that scripts return. Serialized as JSON on the harness boundary.
#[derive(Debug)]
pub enum ScriptError {
    /// A user-facing error message.
    Message(String),
    /// JSON serialization/deserialization error.
    Json(serde_json::Error),
    /// I/O error.
    Io(std::io::Error),
    /// Regex error.
    Regex(regex::Error),
}

impl fmt::Display for ScriptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScriptError::Message(msg) => write!(f, "{msg}"),
            ScriptError::Json(e) => write!(f, "JSON error: {e}"),
            ScriptError::Io(e) => write!(f, "I/O error: {e}"),
            ScriptError::Regex(e) => write!(f, "Regex error: {e}"),
        }
    }
}

impl From<String> for ScriptError {
    fn from(msg: String) -> Self {
        ScriptError::Message(msg)
    }
}

impl From<&str> for ScriptError {
    fn from(msg: &str) -> Self {
        ScriptError::Message(msg.to_string())
    }
}

impl From<serde_json::Error> for ScriptError {
    fn from(e: serde_json::Error) -> Self {
        ScriptError::Json(e)
    }
}

impl From<std::io::Error> for ScriptError {
    fn from(e: std::io::Error) -> Self {
        ScriptError::Io(e)
    }
}

impl From<regex::Error> for ScriptError {
    fn from(e: regex::Error) -> Self {
        ScriptError::Regex(e)
    }
}
