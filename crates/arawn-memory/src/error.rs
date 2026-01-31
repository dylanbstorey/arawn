//! Error types for the memory crate.

use thiserror::Error;

/// Errors that can occur in the memory crate.
#[derive(Debug, Error)]
pub enum MemoryError {
    /// Database connection or operation failed.
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    /// Serialization/deserialization failed.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Invalid query or parameters.
    #[error("Query error: {0}")]
    Query(String),

    /// Requested resource not found.
    #[error("Not found: {0}")]
    NotFound(String),

    /// Schema migration failed.
    #[error("Migration error: {0}")]
    Migration(String),

    /// Invalid UUID format.
    #[error("Invalid UUID: {0}")]
    InvalidUuid(#[from] uuid::Error),

    /// Invalid data or state.
    #[error("Invalid data: {0}")]
    InvalidData(String),
}

/// Result type alias for memory operations.
pub type Result<T> = std::result::Result<T, MemoryError>;
