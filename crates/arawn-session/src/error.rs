//! Error types for session cache operations.

/// Error type for session cache operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Session was not found in cache or storage.
    #[error("Session not found: {0}")]
    NotFound(String),

    /// The associated context (e.g., workstream) was not found.
    #[error("Context not found: {0}")]
    ContextNotFound(String),

    /// No persistence backend is configured.
    #[error("No persistence backend configured")]
    NoPersistence,

    /// Error from persistence backend.
    #[error("Persistence error: {0}")]
    Persistence(String),

    /// Session has expired due to TTL.
    #[error("Session expired: {0}")]
    Expired(String),
}

/// Result type for session cache operations.
pub type Result<T> = std::result::Result<T, Error>;
