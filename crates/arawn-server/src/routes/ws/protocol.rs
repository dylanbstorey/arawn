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
        /// Reconnect token from previous session ownership (for reclaiming after disconnect).
        #[serde(skip_serializing_if = "Option::is_none")]
        reconnect_token: Option<String>,
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
    /// Subscription acknowledgment.
    SubscribeAck {
        /// Session ID subscribed to.
        session_id: String,
        /// Whether this connection is the session owner (can send Chat).
        owner: bool,
        /// Reconnect token for reclaiming ownership after disconnect.
        /// Only present if this connection is the owner.
        #[serde(skip_serializing_if = "Option::is_none")]
        reconnect_token: Option<String>,
    },
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
    /// Context usage information.
    ContextInfo {
        /// Session ID.
        session_id: String,
        /// Current token count estimate.
        current_tokens: usize,
        /// Maximum tokens allowed.
        max_tokens: usize,
        /// Usage as percentage (0-100).
        percent: u8,
        /// Status: "ok", "warning", or "critical".
        status: String,
    },
    /// Filesystem change notification.
    FsChange {
        /// Workstream where the change occurred.
        workstream: String,
        /// Relative path within the workstream.
        path: String,
        /// Action: "created", "modified", or "deleted".
        action: String,
        /// ISO 8601 timestamp.
        timestamp: String,
    },
    /// Disk pressure alert.
    DiskPressure {
        /// Alert level: "ok", "warning", or "critical".
        level: String,
        /// Scope of the alert (e.g., "total" or workstream ID).
        scope: String,
        /// Current usage in megabytes.
        usage_mb: f64,
        /// Limit in megabytes.
        limit_mb: f64,
        /// ISO 8601 timestamp.
        timestamp: String,
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

    /// Create a context info message.
    pub fn context_info(
        session_id: impl Into<String>,
        current_tokens: usize,
        max_tokens: usize,
    ) -> Self {
        let percent = if max_tokens == 0 {
            0
        } else {
            ((current_tokens as f64 / max_tokens as f64) * 100.0).min(100.0) as u8
        };
        let status = if percent < 70 {
            "ok"
        } else if percent < 90 {
            "warning"
        } else {
            "critical"
        };
        Self::ContextInfo {
            session_id: session_id.into(),
            current_tokens,
            max_tokens,
            percent,
            status: status.to_string(),
        }
    }

    /// Create a filesystem change notification from an FsChangeEvent.
    pub fn fs_change(event: &arawn_workstream::FsChangeEvent) -> Self {
        Self::FsChange {
            workstream: event.workstream.clone(),
            path: event.path.clone(),
            action: event.action.to_string(),
            timestamp: event.timestamp.to_rfc3339(),
        }
    }

    /// Create a subscription acknowledgment message.
    pub fn subscribe_ack(
        session_id: impl Into<String>,
        owner: bool,
        reconnect_token: Option<String>,
    ) -> Self {
        Self::SubscribeAck {
            session_id: session_id.into(),
            owner,
            reconnect_token,
        }
    }

    /// Create a disk pressure alert from a DiskPressureEvent.
    pub fn disk_pressure(event: &arawn_workstream::cleanup::DiskPressureEvent) -> Self {
        Self::DiskPressure {
            level: event.level.to_string(),
            scope: event.scope.clone(),
            usage_mb: event.usage_mb,
            limit_mb: event.limit_mb,
            timestamp: event.timestamp.to_rfc3339(),
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
        assert!(matches!(msg, ClientMessage::Subscribe { session_id, reconnect_token: None } if session_id == "123"));

        // Subscribe with reconnect token
        let json = r#"{"type": "subscribe", "session_id": "456", "reconnect_token": "tok-xyz"}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        assert!(matches!(msg, ClientMessage::Subscribe { session_id, reconnect_token: Some(tok) } if session_id == "456" && tok == "tok-xyz"));
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
    fn test_subscribe_ack_serialization() {
        // Owner with token
        let msg = ServerMessage::subscribe_ack("session-123", true, Some("token-abc".to_string()));
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("subscribe_ack"));
        assert!(json.contains("session-123"));
        assert!(json.contains(r#""owner":true"#));
        assert!(json.contains("token-abc"));

        // Reader (non-owner, no token)
        let msg = ServerMessage::subscribe_ack("session-456", false, None);
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("subscribe_ack"));
        assert!(json.contains("session-456"));
        assert!(json.contains(r#""owner":false"#));
        assert!(!json.contains("reconnect_token")); // should be omitted
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

    #[test]
    fn test_context_info_serialization() {
        // OK status (< 70%)
        let msg = ServerMessage::context_info("session-123", 50000, 100000);
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("context_info"));
        assert!(json.contains("session-123"));
        assert!(json.contains("50000"));
        assert!(json.contains("100000"));
        assert!(json.contains("50")); // percent
        assert!(json.contains(r#""status":"ok""#));

        // Warning status (70-90%)
        let msg = ServerMessage::context_info("session-456", 80000, 100000);
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""status":"warning""#));
        assert!(json.contains("80")); // percent

        // Critical status (> 90%)
        let msg = ServerMessage::context_info("session-789", 95000, 100000);
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""status":"critical""#));
        assert!(json.contains("95")); // percent
    }

    #[test]
    fn test_context_info_boundary_conditions() {
        // Exactly 70% - should be warning
        let msg = ServerMessage::context_info("s", 70000, 100000);
        match msg {
            ServerMessage::ContextInfo { status, percent, .. } => {
                assert_eq!(percent, 70);
                assert_eq!(status, "warning");
            }
            _ => panic!("Expected ContextInfo"),
        }

        // Exactly 90% - should be critical
        let msg = ServerMessage::context_info("s", 90000, 100000);
        match msg {
            ServerMessage::ContextInfo { status, percent, .. } => {
                assert_eq!(percent, 90);
                assert_eq!(status, "critical");
            }
            _ => panic!("Expected ContextInfo"),
        }

        // Zero max tokens - should handle gracefully
        let msg = ServerMessage::context_info("s", 1000, 0);
        match msg {
            ServerMessage::ContextInfo { percent, status, .. } => {
                assert_eq!(percent, 0);
                assert_eq!(status, "ok");
            }
            _ => panic!("Expected ContextInfo"),
        }
    }

    #[test]
    fn test_fs_change_serialization() {
        use arawn_workstream::{FsAction, FsChangeEvent};

        let event = FsChangeEvent::new("my-blog", "production/post.md", FsAction::Modified);
        let msg = ServerMessage::fs_change(&event);
        let json = serde_json::to_string(&msg).unwrap();

        assert!(json.contains("fs_change"));
        assert!(json.contains("my-blog"));
        assert!(json.contains("production/post.md"));
        assert!(json.contains("modified"));
        assert!(json.contains("timestamp"));

        // Test created action
        let event = FsChangeEvent::new("scratch", "sessions/abc/work/file.txt", FsAction::Created);
        let msg = ServerMessage::fs_change(&event);
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("created"));

        // Test deleted action
        let event = FsChangeEvent::new("project", "work/temp.txt", FsAction::Deleted);
        let msg = ServerMessage::fs_change(&event);
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("deleted"));
    }

    #[test]
    fn test_disk_pressure_serialization() {
        use arawn_workstream::cleanup::{DiskPressureEvent, PressureLevel};

        // Warning level
        let event = DiskPressureEvent::new(PressureLevel::Warning, "my-workstream", 1800.0, 2048.0);
        let msg = ServerMessage::disk_pressure(&event);
        let json = serde_json::to_string(&msg).unwrap();

        assert!(json.contains("disk_pressure"));
        assert!(json.contains("warning"));
        assert!(json.contains("my-workstream"));
        assert!(json.contains("1800"));
        assert!(json.contains("2048"));
        assert!(json.contains("timestamp"));

        // Critical level for total
        let event = DiskPressureEvent::new(PressureLevel::Critical, "total", 12000.0, 10000.0);
        let msg = ServerMessage::disk_pressure(&event);
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("critical"));
        assert!(json.contains("total"));

        // Ok level
        let event = DiskPressureEvent::new(PressureLevel::Ok, "scratch", 500.0, 2048.0);
        let msg = ServerMessage::disk_pressure(&event);
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""level":"ok""#));
    }
}
