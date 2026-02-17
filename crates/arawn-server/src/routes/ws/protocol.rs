//! WebSocket protocol types for client-server communication.

use serde::{Deserialize, Serialize};

/// Messages from client to server.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    /// Send a chat message.
    Chat {
        /// Optional session ID. If not provided, a new session is created.
        session_id: Option<String>,
        /// Optional workstream ID. If not provided, uses "scratch" workstream.
        workstream_id: Option<String>,
        /// The message content.
        message: String,
    },
    /// Subscribe to updates for a session.
    Subscribe {
        /// Session ID to subscribe to.
        session_id: String,
    },
    /// Unsubscribe from session updates.
    Unsubscribe {
        /// Session ID to unsubscribe from.
        session_id: String,
    },
    /// Ping to keep connection alive.
    Ping,
    /// Authenticate the connection.
    Auth {
        /// Bearer token for authentication.
        token: String,
    },
    /// Cancel the current operation.
    Cancel {
        /// Session ID to cancel.
        session_id: String,
    },
    /// Execute a server command.
    Command {
        /// Command name (e.g., "compact").
        command: String,
        /// Command arguments as JSON.
        #[serde(default)]
        args: serde_json::Value,
    },
}

/// Messages from server to client.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    /// Authentication result.
    AuthResult {
        /// Whether authentication succeeded.
        success: bool,
        /// Error message if failed.
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
    },
    /// Session created/confirmed.
    SessionCreated {
        /// The session ID.
        session_id: String,
    },
    /// Text chunk from agent response.
    ChatChunk {
        /// Session ID.
        session_id: String,
        /// Text content.
        chunk: String,
        /// Whether this is the final chunk.
        done: bool,
    },
    /// Tool execution started.
    ToolStart {
        /// Session ID.
        session_id: String,
        /// Tool call ID.
        tool_id: String,
        /// Tool name.
        tool_name: String,
    },
    /// Tool execution output (streaming).
    ToolOutput {
        /// Session ID.
        session_id: String,
        /// Tool call ID.
        tool_id: String,
        /// Output content chunk.
        content: String,
    },
    /// Tool execution completed.
    ToolEnd {
        /// Session ID.
        session_id: String,
        /// Tool call ID.
        tool_id: String,
        /// Whether tool succeeded.
        success: bool,
    },
    /// Error occurred.
    Error {
        /// Error code.
        code: String,
        /// Error message.
        message: String,
    },
    /// Pong response to ping.
    Pong,
    /// Command execution progress.
    CommandProgress {
        /// Command name.
        command: String,
        /// Progress message.
        message: String,
        /// Progress percentage (0-100).
        #[serde(skip_serializing_if = "Option::is_none")]
        percent: Option<u8>,
    },
    /// Command execution result.
    CommandResult {
        /// Command name.
        command: String,
        /// Whether the command succeeded.
        success: bool,
        /// Result data (on success) or error details (on failure).
        result: serde_json::Value,
    },
}

impl ServerMessage {
    /// Create an error message.
    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Error {
            code: code.into(),
            message: message.into(),
        }
    }

    /// Create an auth success message.
    pub fn auth_success() -> Self {
        Self::AuthResult {
            success: true,
            error: None,
        }
    }

    /// Create an auth failure message.
    pub fn auth_failure(error: impl Into<String>) -> Self {
        Self::AuthResult {
            success: false,
            error: Some(error.into()),
        }
    }

    /// Create a command progress message.
    pub fn command_progress(
        command: impl Into<String>,
        message: impl Into<String>,
        percent: Option<u8>,
    ) -> Self {
        Self::CommandProgress {
            command: command.into(),
            message: message.into(),
            percent,
        }
    }

    /// Create a successful command result message.
    pub fn command_success(command: impl Into<String>, result: serde_json::Value) -> Self {
        Self::CommandResult {
            command: command.into(),
            success: true,
            result,
        }
    }

    /// Create a failed command result message.
    pub fn command_failure(command: impl Into<String>, error: impl Into<String>) -> Self {
        Self::CommandResult {
            command: command.into(),
            success: false,
            result: serde_json::json!({ "error": error.into() }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_message_parsing() {
        let json = r#"{"type": "ping"}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        assert!(matches!(msg, ClientMessage::Ping));

        let json = r#"{"type": "auth", "token": "secret"}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        assert!(matches!(msg, ClientMessage::Auth { token } if token == "secret"));

        let json = r#"{"type": "chat", "message": "hello"}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        assert!(
            matches!(msg, ClientMessage::Chat { session_id: None, workstream_id: None, message } if message == "hello")
        );

        let json = r#"{"type": "chat", "session_id": "123", "message": "hello"}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        assert!(matches!(msg, ClientMessage::Chat { session_id: Some(id), workstream_id: None, .. } if id == "123"));

        let json = r#"{"type": "chat", "workstream_id": "ws-456", "message": "hello"}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        assert!(matches!(msg, ClientMessage::Chat { session_id: None, workstream_id: Some(ws_id), .. } if ws_id == "ws-456"));

        let json = r#"{"type": "subscribe", "session_id": "123"}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        assert!(matches!(msg, ClientMessage::Subscribe { session_id } if session_id == "123"));
    }

    #[test]
    fn test_command_message_parsing() {
        // Minimal command message
        let json = r#"{"type": "command", "command": "compact"}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        match msg {
            ClientMessage::Command { command, args } => {
                assert_eq!(command, "compact");
                assert!(args.is_null());
            }
            _ => panic!("Expected Command"),
        }

        // Command with args
        let json = r#"{"type": "command", "command": "compact", "args": {"session_id": "123", "force": true}}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        match msg {
            ClientMessage::Command { command, args } => {
                assert_eq!(command, "compact");
                assert_eq!(args["session_id"], "123");
                assert_eq!(args["force"], true);
            }
            _ => panic!("Expected Command"),
        }
    }

    #[test]
    fn test_server_message_serialization() {
        let msg = ServerMessage::Pong;
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("pong"));

        let msg = ServerMessage::error("test_error", "Test message");
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("error"));
        assert!(json.contains("test_error"));
        assert!(json.contains("Test message"));

        let msg = ServerMessage::ChatChunk {
            session_id: "123".to_string(),
            chunk: "hello".to_string(),
            done: false,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("chat_chunk"));
        assert!(json.contains("hello"));

        let msg = ServerMessage::auth_success();
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("auth_result"));
        assert!(json.contains("true"));
    }

    #[test]
    fn test_auth_messages() {
        let success = ServerMessage::auth_success();
        let json = serde_json::to_string(&success).unwrap();
        assert!(!json.contains("error"));

        let failure = ServerMessage::auth_failure("bad token");
        let json = serde_json::to_string(&failure).unwrap();
        assert!(json.contains("error"));
        assert!(json.contains("bad token"));
    }

    #[test]
    fn test_command_progress_serialization() {
        let msg = ServerMessage::command_progress("compact", "Summarizing turns...", Some(50));
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("command_progress"));
        assert!(json.contains("compact"));
        assert!(json.contains("Summarizing"));
        assert!(json.contains("50"));

        // Without percent
        let msg = ServerMessage::command_progress("compact", "Starting...", None);
        let json = serde_json::to_string(&msg).unwrap();
        assert!(!json.contains("percent"));
    }

    #[test]
    fn test_command_result_serialization() {
        // Success
        let msg = ServerMessage::command_success(
            "compact",
            serde_json::json!({"compacted": true, "turns_compacted": 5}),
        );
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("command_result"));
        assert!(json.contains("compact"));
        assert!(json.contains("true")); // success
        assert!(json.contains("compacted"));
        assert!(json.contains("turns_compacted"));

        // Failure
        let msg = ServerMessage::command_failure("compact", "Session not found");
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("command_result"));
        assert!(json.contains("compact"));
        assert!(json.contains("false")); // success: false
        assert!(json.contains("Session not found"));
    }
}
