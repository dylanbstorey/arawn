//! WebSocket connection lifecycle and state management.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use arawn_agent::SessionId;

use super::handlers::{MessageResponse, handle_message};
use super::protocol::{ClientMessage, ServerMessage};
use crate::state::AppState;

/// Unique identifier for a WebSocket connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConnectionId(Uuid);

impl ConnectionId {
    /// Create a new unique connection ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ConnectionId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ConnectionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Idle timeout for WebSocket connections (5 minutes).
/// Connections that receive no messages for this duration will be closed.
pub const IDLE_TIMEOUT: Duration = Duration::from_secs(5 * 60);

/// State for a WebSocket connection.
pub struct ConnectionState {
    /// Unique identifier for this connection.
    pub id: ConnectionId,
    /// Whether the connection is authenticated.
    pub authenticated: bool,
    /// Current subscribed sessions.
    pub subscriptions: std::collections::HashSet<SessionId>,
    /// Reconnect tokens for owned sessions (session_id -> token).
    /// Used to create pending reconnects on disconnect.
    pub reconnect_tokens: std::collections::HashMap<SessionId, String>,
    /// Cancellation token for cleanup.
    pub cancellation: CancellationToken,
}

impl ConnectionState {
    /// Create a new connection state.
    pub fn new() -> Self {
        Self {
            id: ConnectionId::new(),
            authenticated: false,
            subscriptions: std::collections::HashSet::new(),
            reconnect_tokens: std::collections::HashMap::new(),
            cancellation: CancellationToken::new(),
        }
    }
}

impl Default for ConnectionState {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ConnectionState {
    fn drop(&mut self) {
        self.cancellation.cancel();
    }
}

/// Handle a WebSocket connection.
pub async fn handle_socket(socket: WebSocket, state: AppState, addr: SocketAddr) {
    let (mut sender, mut receiver) = socket.split();
    let mut conn_state = ConnectionState::new();

    tracing::debug!(
        connection_id = %conn_state.id,
        remote_addr = %addr,
        "WebSocket connection established"
    );

    // Auto-authenticate if no auth token is configured (localhost mode)
    if state.config().auth_token.is_none() {
        conn_state.authenticated = true;
    }

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

    // Release all session ownerships held by this connection, creating pending reconnects
    state
        .release_all_session_ownerships(conn_state.id, &conn_state.reconnect_tokens)
        .await;

    // Index any sessions this connection was subscribed to
    for session_id in &conn_state.subscriptions {
        if let Some(indexer) = state.indexer() {
            let session_opt = state.session_cache().get(session_id).await;
            if let Some(session) = session_opt
                && !session.is_empty()
            {
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

    tracing::debug!(connection_id = %conn_state.id, "WebSocket connection closed");
}

/// Send a message over the WebSocket.
pub async fn send_message(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    msg: ServerMessage,
) -> Result<(), axum::Error> {
    let json = serde_json::to_string(&msg).map_err(axum::Error::new)?;
    sender
        .send(Message::Text(json.into()))
        .await
        .map_err(axum::Error::new)
}
