//! Client error types.

use thiserror::Error;

/// Client error type.
#[derive(Debug, Error)]
pub enum Error {
    /// HTTP request failed.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// URL parsing failed.
    #[error("Invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),

    /// JSON serialization/deserialization failed.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Server returned an error response.
    #[error("API error ({status}): {message}")]
    Api {
        /// HTTP status code.
        status: u16,
        /// Error code from server.
        code: String,
        /// Error message from server.
        message: String,
    },

    /// Authentication failed.
    #[error("Authentication failed: {0}")]
    Auth(String),

    /// Resource not found.
    #[error("Not found: {0}")]
    NotFound(String),

    /// Invalid configuration.
    #[error("Configuration error: {0}")]
    Config(String),

    /// Stream error.
    #[error("Stream error: {0}")]
    Stream(String),
}

impl Error {
    /// Check if this is a not-found error.
    pub fn is_not_found(&self) -> bool {
        matches!(self, Error::NotFound(_)) || matches!(self, Error::Api { status: 404, .. })
    }

    /// Check if this is an authentication error.
    pub fn is_auth_error(&self) -> bool {
        matches!(self, Error::Auth(_)) || matches!(self, Error::Api { status: 401, .. })
    }

    /// Check if this is a rate limit error.
    pub fn is_rate_limited(&self) -> bool {
        matches!(self, Error::Api { status: 429, .. })
    }

    /// Check if this is a server error.
    pub fn is_server_error(&self) -> bool {
        matches!(self, Error::Api { status, .. } if *status >= 500)
    }
}

/// Result type for client operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Error response from the server.
#[derive(Debug, serde::Deserialize)]
pub(crate) struct ErrorResponse {
    pub code: String,
    pub message: String,
}
