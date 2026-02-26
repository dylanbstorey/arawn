//! Domain error types.

use thiserror::Error;

/// Domain-level errors.
#[derive(Debug, Error)]
pub enum DomainError {
    /// Session not found.
    #[error("Session not found: {0}")]
    SessionNotFound(String),

    /// Workstream not found.
    #[error("Workstream not found: {0}")]
    WorkstreamNotFound(String),

    /// Agent execution error.
    #[error("Agent error: {0}")]
    Agent(#[from] arawn_agent::AgentError),

    /// MCP server error.
    #[error("MCP error: {0}")]
    Mcp(String),

    /// Workstream error.
    #[error("Workstream error: {0}")]
    Workstream(#[from] arawn_workstream::WorkstreamError),

    /// Configuration error.
    #[error("Configuration error: {0}")]
    Config(String),

    /// Internal error.
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result type for domain operations.
pub type Result<T> = std::result::Result<T, DomainError>;
