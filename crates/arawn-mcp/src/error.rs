//! Error types for MCP operations.

use thiserror::Error;

/// Result type for MCP operations.
pub type Result<T> = std::result::Result<T, McpError>;

/// Error type for MCP operations.
#[derive(Debug, Error)]
pub enum McpError {
    /// Failed to spawn the MCP server process.
    #[error("failed to spawn MCP server: {0}")]
    SpawnFailed(String),

    /// Failed to communicate with the MCP server.
    #[error("transport error: {0}")]
    Transport(String),

    /// JSON-RPC protocol error.
    #[error("protocol error: {0}")]
    Protocol(String),

    /// JSON serialization/deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Server returned an error response.
    #[error("server error {code}: {message}")]
    ServerError {
        /// Error code from the server.
        code: i64,
        /// Error message from the server.
        message: String,
        /// Optional additional data.
        data: Option<serde_json::Value>,
    },

    /// Tool execution failed.
    #[error("tool error: {0}")]
    ToolError(String),

    /// Server not initialized.
    #[error("server not initialized - call initialize() first")]
    NotInitialized,

    /// Connection closed.
    #[error("connection closed")]
    ConnectionClosed,

    /// Timeout waiting for response.
    #[error("timeout waiting for response")]
    Timeout,
}

impl McpError {
    /// Create a spawn failed error.
    pub fn spawn_failed(msg: impl Into<String>) -> Self {
        Self::SpawnFailed(msg.into())
    }

    /// Create a transport error.
    pub fn transport(msg: impl Into<String>) -> Self {
        Self::Transport(msg.into())
    }

    /// Create a protocol error.
    pub fn protocol(msg: impl Into<String>) -> Self {
        Self::Protocol(msg.into())
    }

    /// Create a server error from an error response.
    pub fn server_error(
        code: i64,
        message: impl Into<String>,
        data: Option<serde_json::Value>,
    ) -> Self {
        Self::ServerError {
            code,
            message: message.into(),
            data,
        }
    }

    /// Create a tool error.
    pub fn tool_error(msg: impl Into<String>) -> Self {
        Self::ToolError(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = McpError::spawn_failed("command not found");
        assert!(err.to_string().contains("spawn"));
        assert!(err.to_string().contains("command not found"));

        let err = McpError::server_error(-32600, "Invalid Request", None);
        assert!(err.to_string().contains("-32600"));
        assert!(err.to_string().contains("Invalid Request"));
    }

    #[test]
    fn test_json_error_conversion() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let mcp_err: McpError = json_err.into();
        assert!(matches!(mcp_err, McpError::Json(_)));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let mcp_err: McpError = io_err.into();
        assert!(matches!(mcp_err, McpError::Io(_)));
    }
}
