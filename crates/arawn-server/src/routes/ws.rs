//! WebSocket handler for real-time bidirectional communication.

use axum::{
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::Response,
};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use std::sync::Arc;

use arawn_agent::SessionId;

use crate::state::AppState;

// ─────────────────────────────────────────────────────────────────────────────
// Protocol Types
// ─────────────────────────────────────────────────────────────────────────────

/// Messages from client to server.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    /// Send a chat message.
    Chat {
        /// Optional session ID. If not provided, a new session is created.
        session_id: Option<String>,
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
}

// ─────────────────────────────────────────────────────────────────────────────
// Connection State
// ─────────────────────────────────────────────────────────────────────────────

/// State for a WebSocket connection.
struct ConnectionState {
    /// Whether the connection is authenticated.
    authenticated: bool,
    /// Current subscribed sessions.
    subscriptions: std::collections::HashSet<SessionId>,
    /// Cancellation token for cleanup.
    cancellation: CancellationToken,
}

impl ConnectionState {
    fn new() -> Self {
        Self {
            authenticated: false,
            subscriptions: std::collections::HashSet::new(),
            cancellation: CancellationToken::new(),
        }
    }
}

impl Drop for ConnectionState {
    fn drop(&mut self) {
        self.cancellation.cancel();
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Handler
// ─────────────────────────────────────────────────────────────────────────────

/// GET /ws - WebSocket upgrade handler.
///
/// Note: Authentication happens via the first message (Auth type) rather than
/// HTTP headers to support browsers that can't set custom headers on WebSocket.
pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let mut conn_state = ConnectionState::new();

    // Auto-authenticate if no auth token is configured (localhost mode)
    if state.config.auth_token.is_none() {
        conn_state.authenticated = true;
    }

    tracing::debug!("WebSocket connection established");

    while let Some(msg) = receiver.next().await {
        let msg = match msg {
            Ok(Message::Text(text)) => text.to_string(),
            Ok(Message::Binary(data)) => {
                // Try to interpret binary as UTF-8 text
                match String::from_utf8(data.to_vec()) {
                    Ok(text) => text,
                    Err(_) => {
                        let _ = send_message(
                            &mut sender,
                            ServerMessage::error("invalid_message", "Binary data must be UTF-8"),
                        )
                        .await;
                        continue;
                    }
                }
            }
            Ok(Message::Ping(data)) => {
                let _ = sender.send(Message::Pong(data)).await;
                continue;
            }
            Ok(Message::Pong(_)) => continue,
            Ok(Message::Close(_)) => break,
            Err(e) => {
                tracing::warn!("WebSocket error: {}", e);
                break;
            }
        };

        // Parse message
        let client_msg: ClientMessage = match serde_json::from_str(&msg) {
            Ok(m) => m,
            Err(e) => {
                let _ = send_message(
                    &mut sender,
                    ServerMessage::error("parse_error", format!("Invalid message: {}", e)),
                )
                .await;
                continue;
            }
        };

        // Handle message
        let response = handle_message(client_msg, &mut conn_state, &state).await;

        match response {
            MessageResponse::Single(msg) => {
                if send_message(&mut sender, msg).await.is_err() {
                    break;
                }
            }
            MessageResponse::Stream(stream) => {
                let mut stream = std::pin::pin!(stream);
                while let Some(msg) = stream.next().await {
                    if send_message(&mut sender, msg).await.is_err() {
                        break;
                    }
                }
            }
            MessageResponse::None => {}
        }
    }

    // Index any sessions this connection was subscribed to
    for session_id in &conn_state.subscriptions {
        if let Some(indexer) = &state.indexer {
            let session_opt = {
                let sessions = state.sessions.read().await;
                sessions.get(session_id).cloned()
            };
            if let Some(session) = session_opt {
                if !session.is_empty() {
                    let indexer = Arc::clone(indexer);
                    let messages = crate::state::session_to_messages(&session);
                    let sid = session_id.to_string();
                    tokio::spawn(async move {
                        let report = indexer
                            .index_session(&sid, &crate::state::messages_as_refs(&messages))
                            .await;
                        tracing::info!(
                            session_id = %sid,
                            report = %report,
                            "WebSocket close: background session indexing complete"
                        );
                    });
                }
            }
        }
    }

    tracing::debug!("WebSocket connection closed");
}

enum MessageResponse {
    Single(ServerMessage),
    Stream(futures::stream::BoxStream<'static, ServerMessage>),
    None,
}

async fn handle_message(
    msg: ClientMessage,
    conn_state: &mut ConnectionState,
    app_state: &AppState,
) -> MessageResponse {
    match msg {
        ClientMessage::Ping => MessageResponse::Single(ServerMessage::Pong),

        ClientMessage::Auth { token } => {
            let authed = match &app_state.config.auth_token {
                None => true,
                Some(expected) => token == *expected,
            };
            if authed {
                conn_state.authenticated = true;
                MessageResponse::Single(ServerMessage::auth_success())
            } else {
                MessageResponse::Single(ServerMessage::auth_failure("Invalid token"))
            }
        }

        ClientMessage::Subscribe { session_id } => {
            if !conn_state.authenticated {
                return MessageResponse::Single(ServerMessage::error(
                    "unauthorized",
                    "Authentication required",
                ));
            }

            match Uuid::parse_str(&session_id) {
                Ok(uuid) => {
                    conn_state.subscriptions.insert(SessionId::from_uuid(uuid));
                    MessageResponse::None
                }
                Err(_) => MessageResponse::Single(ServerMessage::error(
                    "invalid_session",
                    "Invalid session ID",
                )),
            }
        }

        ClientMessage::Unsubscribe { session_id } => {
            if let Ok(uuid) = Uuid::parse_str(&session_id) {
                conn_state.subscriptions.remove(&SessionId::from_uuid(uuid));
            }
            MessageResponse::None
        }

        ClientMessage::Chat {
            session_id,
            message,
        } => {
            if !conn_state.authenticated {
                return MessageResponse::Single(ServerMessage::error(
                    "unauthorized",
                    "Authentication required",
                ));
            }

            // Parse session ID if provided
            let session_id = session_id
                .as_ref()
                .and_then(|s| Uuid::parse_str(s).ok())
                .map(SessionId::from_uuid);

            // Get or create session
            let session_id = app_state.get_or_create_session(session_id).await;
            let session_id_str = session_id.to_string();

            // Get the agent stream
            let stream_result = {
                let mut sessions = app_state.sessions.write().await;
                match sessions.get_mut(&session_id) {
                    Some(session) => {
                        let cancellation = conn_state.cancellation.clone();
                        let stream = app_state.agent.turn_stream(session, &message, cancellation);
                        Some(stream)
                    }
                    None => None,
                }
            };

            let stream = match stream_result {
                Some(s) => s,
                None => {
                    return MessageResponse::Single(ServerMessage::error(
                        "internal",
                        "Session disappeared",
                    ));
                }
            };

            // Create response stream
            let session_id_for_stream = session_id_str.clone();
            let response_stream = async_stream::stream! {
                // First, send session created
                yield ServerMessage::SessionCreated {
                    session_id: session_id_for_stream.clone(),
                };

                let mut stream = std::pin::pin!(stream);
                while let Some(chunk) = stream.next().await {
                    use arawn_agent::StreamChunk;

                    match chunk {
                        StreamChunk::Text { content } => {
                            yield ServerMessage::ChatChunk {
                                session_id: session_id_for_stream.clone(),
                                chunk: content,
                                done: false,
                            };
                        }
                        StreamChunk::ToolStart { id, name } => {
                            yield ServerMessage::ToolStart {
                                session_id: session_id_for_stream.clone(),
                                tool_id: id,
                                tool_name: name,
                            };
                        }
                        StreamChunk::ToolOutput { id, content } => {
                            yield ServerMessage::ToolOutput {
                                session_id: session_id_for_stream.clone(),
                                tool_id: id,
                                content,
                            };
                        }
                        StreamChunk::ToolEnd { id, success, .. } => {
                            yield ServerMessage::ToolEnd {
                                session_id: session_id_for_stream.clone(),
                                tool_id: id,
                                success,
                            };
                        }
                        StreamChunk::Done { .. } => {
                            yield ServerMessage::ChatChunk {
                                session_id: session_id_for_stream.clone(),
                                chunk: String::new(),
                                done: true,
                            };
                        }
                        StreamChunk::Error { message } => {
                            yield ServerMessage::Error {
                                code: "agent_error".to_string(),
                                message,
                            };
                        }
                    }
                }
            };

            MessageResponse::Stream(Box::pin(response_stream))
        }
    }
}

async fn send_message(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    msg: ServerMessage,
) -> Result<(), axum::Error> {
    let json = serde_json::to_string(&msg).map_err(axum::Error::new)?;
    sender
        .send(Message::Text(json.into()))
        .await
        .map_err(axum::Error::new)
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

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
            matches!(msg, ClientMessage::Chat { session_id: None, message } if message == "hello")
        );

        let json = r#"{"type": "chat", "session_id": "123", "message": "hello"}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        assert!(matches!(msg, ClientMessage::Chat { session_id: Some(id), .. } if id == "123"));

        let json = r#"{"type": "subscribe", "session_id": "123"}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        assert!(matches!(msg, ClientMessage::Subscribe { session_id } if session_id == "123"));
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
}
