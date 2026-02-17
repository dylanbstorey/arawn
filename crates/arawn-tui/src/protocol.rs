//! WebSocket protocol types matching the server.
//!
//! These types mirror the protocol defined in `arawn-server/src/routes/ws.rs`.

use serde::{Deserialize, Serialize};

/// Messages from client to server.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    /// Send a chat message.
    Chat {
        /// Optional session ID. If not provided, a new session is created.
        #[serde(skip_serializing_if = "Option::is_none")]
        session_id: Option<String>,
        /// Optional workstream ID. If not provided, uses "scratch" workstream.
        #[serde(skip_serializing_if = "Option::is_none")]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_message_serialization() {
        let msg = ClientMessage::Ping;
        let json = serde_json::to_string(&msg).unwrap();
        assert_eq!(json, r#"{"type":"ping"}"#);

        let msg = ClientMessage::Chat {
            session_id: None,
            workstream_id: None,
            message: "hello".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"chat""#));
        assert!(json.contains(r#""message":"hello""#));
        assert!(!json.contains("session_id")); // Should be skipped when None
        assert!(!json.contains("workstream_id")); // Should be skipped when None

        let msg = ClientMessage::Chat {
            session_id: Some("123".to_string()),
            workstream_id: None,
            message: "hello".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""session_id":"123""#));

        let msg = ClientMessage::Chat {
            session_id: None,
            workstream_id: Some("ws-456".to_string()),
            message: "hello".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""workstream_id":"ws-456""#));
    }

    #[test]
    fn test_server_message_deserialization() {
        let json = r#"{"type":"pong"}"#;
        let msg: ServerMessage = serde_json::from_str(json).unwrap();
        assert!(matches!(msg, ServerMessage::Pong));

        let json = r#"{"type":"chat_chunk","session_id":"123","chunk":"hello","done":false}"#;
        let msg: ServerMessage = serde_json::from_str(json).unwrap();
        assert!(matches!(
            msg,
            ServerMessage::ChatChunk {
                chunk,
                done: false,
                ..
            } if chunk == "hello"
        ));

        let json = r#"{"type":"error","code":"test","message":"Test error"}"#;
        let msg: ServerMessage = serde_json::from_str(json).unwrap();
        assert!(matches!(
            msg,
            ServerMessage::Error { code, message }
            if code == "test" && message == "Test error"
        ));
    }

    #[test]
    fn test_command_message_serialization() {
        // Command without args
        let msg = ClientMessage::Command {
            command: "compact".to_string(),
            args: serde_json::Value::Null,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"command""#));
        assert!(json.contains(r#""command":"compact""#));

        // Command with args
        let msg = ClientMessage::Command {
            command: "compact".to_string(),
            args: serde_json::json!({"session_id": "123", "force": true}),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""session_id":"123""#));
        assert!(json.contains(r#""force":true"#));
    }

    #[test]
    fn test_command_response_deserialization() {
        // Command progress
        let json = r#"{"type":"command_progress","command":"compact","message":"Starting...","percent":50}"#;
        let msg: ServerMessage = serde_json::from_str(json).unwrap();
        match msg {
            ServerMessage::CommandProgress { command, message, percent } => {
                assert_eq!(command, "compact");
                assert_eq!(message, "Starting...");
                assert_eq!(percent, Some(50));
            }
            _ => panic!("Expected CommandProgress"),
        }

        // Command progress without percent
        let json = r#"{"type":"command_progress","command":"compact","message":"Working..."}"#;
        let msg: ServerMessage = serde_json::from_str(json).unwrap();
        match msg {
            ServerMessage::CommandProgress { percent, .. } => {
                assert!(percent.is_none());
            }
            _ => panic!("Expected CommandProgress"),
        }

        // Command result success
        let json = r#"{"type":"command_result","command":"compact","success":true,"result":{"compacted":true}}"#;
        let msg: ServerMessage = serde_json::from_str(json).unwrap();
        match msg {
            ServerMessage::CommandResult { command, success, result } => {
                assert_eq!(command, "compact");
                assert!(success);
                assert_eq!(result["compacted"], true);
            }
            _ => panic!("Expected CommandResult"),
        }

        // Command result failure
        let json = r#"{"type":"command_result","command":"compact","success":false,"result":{"error":"Session not found"}}"#;
        let msg: ServerMessage = serde_json::from_str(json).unwrap();
        match msg {
            ServerMessage::CommandResult { success, result, .. } => {
                assert!(!success);
                assert_eq!(result["error"], "Session not found");
            }
            _ => panic!("Expected CommandResult"),
        }
    }
}
