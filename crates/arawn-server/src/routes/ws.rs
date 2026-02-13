//! WebSocket handler for real-time bidirectional communication.

use std::sync::Arc;
use std::time::Duration;

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

use arawn_agent::{SessionId, ToolCall, ToolResultRecord, Turn, TurnId};

use crate::state::AppState;

/// Idle timeout for WebSocket connections (5 minutes).
/// Connections that receive no messages for this duration will be closed.
const IDLE_TIMEOUT: Duration = Duration::from_secs(5 * 60);

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

    loop {
        // Wait for next message with idle timeout
        let msg = match tokio::time::timeout(IDLE_TIMEOUT, receiver.next()).await {
            Ok(Some(msg)) => msg,
            Ok(None) => {
                // Stream ended normally
                break;
            }
            Err(_) => {
                // Idle timeout exceeded
                tracing::info!("WebSocket connection closed due to idle timeout");
                let _ = send_message(
                    &mut sender,
                    ServerMessage::error("idle_timeout", "Connection closed due to inactivity"),
                )
                .await;
                break;
            }
        };

        // Parse incoming message. We accept both Text and Binary frames,
        // but Binary frames must contain valid UTF-8 JSON. This provides
        // flexibility for clients that may send JSON as binary.
        let msg = match msg {
            Ok(Message::Text(text)) => text.to_string(),
            Ok(Message::Binary(data)) => {
                // Try to interpret binary as UTF-8 text (JSON payloads)
                match String::from_utf8(data.to_vec()) {
                    Ok(text) => text,
                    Err(_) => {
                        // Reject non-UTF-8 binary data with clear error
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
            let session_opt = state.session_cache.get(session_id).await;
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
            workstream_id,
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

            // Resolve workstream ID (default to "scratch")
            let ws_id = workstream_id.as_deref().unwrap_or("scratch");

            // Get or create session using the session cache
            let session_id = app_state
                .get_or_create_session_in_workstream(session_id, ws_id)
                .await;
            let session_id_str = session_id.to_string();

            // Store user message in workstream (if workstreams enabled)
            // workstream_id=None routes to "scratch" workstream
            // Pass agent session_id to link workstream messages to agent sessions
            if let Some(ref ws_manager) = app_state.workstreams {
                use arawn_workstream::MessageRole;
                if let Err(e) = ws_manager.send_message(
                    workstream_id.as_deref(),
                    Some(&session_id_str),
                    MessageRole::User,
                    &message,
                    None,
                ) {
                    tracing::warn!("Failed to store user message in workstream: {}", e);
                }
            }

            // Get the agent stream - use session from cache, fall back to legacy store
            let stream_result = {
                // First try the session cache
                if let Some(mut session) = app_state.session_cache.get(&session_id).await {
                    let cancellation = conn_state.cancellation.clone();
                    let stream = app_state.agent.turn_stream(&mut session, &message, cancellation);
                    // Update session in cache after turn_stream modifies it
                    app_state.update_session(session_id, session).await;
                    Some(stream)
                } else {
                    None
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

            // Clone references for use in async stream
            let workstream_id_for_stream = workstream_id.clone();
            let session_cache = app_state.session_cache.clone();
            let user_message = message.clone();

            // Create response stream
            let session_id_for_stream = session_id_str.clone();
            let response_stream = async_stream::stream! {
                // First, send session created
                yield ServerMessage::SessionCreated {
                    session_id: session_id_for_stream.clone(),
                };

                // Accumulate the full assistant response and tool data for workstream storage
                let mut full_response = String::new();
                let mut tool_calls: Vec<ToolCall> = Vec::new();
                let mut tool_results: Vec<ToolResultRecord> = Vec::new();
                let mut current_tool_output: std::collections::HashMap<String, String> = std::collections::HashMap::new();

                let mut stream = std::pin::pin!(stream);
                while let Some(chunk) = stream.next().await {
                    use arawn_agent::StreamChunk;

                    match chunk {
                        StreamChunk::Text { content } => {
                            full_response.push_str(&content);
                            yield ServerMessage::ChatChunk {
                                session_id: session_id_for_stream.clone(),
                                chunk: content,
                                done: false,
                            };
                        }
                        StreamChunk::ToolStart { id, name } => {
                            // Track tool call
                            tool_calls.push(ToolCall {
                                id: id.clone(),
                                name: name.clone(),
                                arguments: serde_json::Value::Null, // Arguments not available in stream
                            });
                            yield ServerMessage::ToolStart {
                                session_id: session_id_for_stream.clone(),
                                tool_id: id,
                                tool_name: name,
                            };
                        }
                        StreamChunk::ToolOutput { id, content } => {
                            // Accumulate tool output
                            current_tool_output
                                .entry(id.clone())
                                .or_default()
                                .push_str(&content);
                            yield ServerMessage::ToolOutput {
                                session_id: session_id_for_stream.clone(),
                                tool_id: id,
                                content,
                            };
                        }
                        StreamChunk::ToolEnd { id, success, .. } => {
                            // Track tool result
                            let output = current_tool_output.remove(&id).unwrap_or_default();
                            tool_results.push(ToolResultRecord {
                                tool_call_id: id.clone(),
                                success,
                                content: output,
                            });
                            yield ServerMessage::ToolEnd {
                                session_id: session_id_for_stream.clone(),
                                tool_id: id,
                                success,
                            };
                        }
                        StreamChunk::Done { .. } => {
                            // Persist the complete turn to workstream storage
                            let workstream_id_str = workstream_id_for_stream
                                .as_deref()
                                .unwrap_or("scratch")
                                .to_string();

                            // Build a Turn to save
                            let turn = Turn {
                                id: TurnId::new(),
                                user_message: user_message.clone(),
                                assistant_response: if full_response.is_empty() {
                                    None
                                } else {
                                    Some(full_response.clone())
                                },
                                tool_calls: tool_calls.clone(),
                                tool_results: tool_results.clone(),
                                started_at: chrono::Utc::now(),
                                completed_at: Some(chrono::Utc::now()),
                            };

                            // Save via session cache
                            if let Err(e) = session_cache.save_turn(session_id, &turn, &workstream_id_str).await {
                                tracing::warn!("Failed to persist turn to workstream: {}", e);
                            }

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
