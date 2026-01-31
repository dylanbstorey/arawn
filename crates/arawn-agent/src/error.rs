//! Error types for the agent crate.

use thiserror::Error;

/// Result type alias using the agent error type.
pub type Result<T> = std::result::Result<T, AgentError>;

/// Error type for agent operations.
#[derive(Debug, Error)]
pub enum AgentError {
    /// LLM backend error.
    #[error("LLM error: {0}")]
    Llm(#[from] arawn_llm::LlmError),

    /// Tool execution error.
    #[error("Tool error: {0}")]
    Tool(String),

    /// Tool not found in registry.
    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    /// Invalid tool parameters.
    #[error("Invalid tool parameters: {0}")]
    InvalidToolParams(String),

    /// Session error.
    #[error("Session error: {0}")]
    Session(String),

    /// Context building error.
    #[error("Context error: {0}")]
    Context(String),

    /// Configuration error.
    #[error("Configuration error: {0}")]
    Config(String),

    /// Serialization/deserialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Internal error.
    #[error("Internal error: {0}")]
    Internal(String),

    /// Task was cancelled.
    #[error("Task cancelled")]
    Cancelled,

    /// Maximum iterations exceeded.
    #[error("Maximum iterations exceeded: {0}")]
    MaxIterations(u32),
}

impl AgentError {
    /// Create a tool error.
    pub fn tool(msg: impl Into<String>) -> Self {
        Self::Tool(msg.into())
    }

    /// Create a session error.
    pub fn session(msg: impl Into<String>) -> Self {
        Self::Session(msg.into())
    }

    /// Create a context error.
    pub fn context(msg: impl Into<String>) -> Self {
        Self::Context(msg.into())
    }

    /// Create an internal error.
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = AgentError::tool("failed to read file");
        assert!(err.to_string().contains("Tool error"));
        assert!(err.to_string().contains("failed to read file"));
    }

    #[test]
    fn test_tool_not_found() {
        let err = AgentError::ToolNotFound("unknown_tool".to_string());
        assert!(err.to_string().contains("Tool not found"));
    }
}
