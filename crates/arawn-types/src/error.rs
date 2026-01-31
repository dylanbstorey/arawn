//! Error types for the Arawn system.

use thiserror::Error;

/// Result type alias using the Arawn error type.
pub type Result<T> = std::result::Result<T, Error>;

/// Top-level error type for the Arawn system.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Database error: {0}")]
    Database(String),

    #[error("LLM provider error: {0}")]
    Provider(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Task error: {0}")]
    Task(String),

    #[error("Memory error: {0}")]
    Memory(String),

    #[error("Tool execution error: {0}")]
    Tool(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
