//! Error types for the OAuth proxy.

/// Result type alias for this crate.
pub type Result<T> = std::result::Result<T, OAuthError>;

/// Errors that can occur in the OAuth proxy.
#[derive(Debug, thiserror::Error)]
pub enum OAuthError {
    /// Network/HTTP error.
    #[error("Network error: {0}")]
    Network(String),

    /// Backend API returned an error.
    #[error("Backend error: {0}")]
    Backend(String),

    /// Invalid request.
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Configuration error.
    #[error("Config error: {0}")]
    Config(String),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl From<reqwest::Error> for OAuthError {
    fn from(e: reqwest::Error) -> Self {
        OAuthError::Network(e.to_string())
    }
}
