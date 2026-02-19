//! WebSocket handler for real-time bidirectional communication.
//!
//! This module provides WebSocket support for the Arawn server, enabling:
//! - Real-time chat with streaming responses
//! - Tool execution status updates
//! - Session subscription for multi-client scenarios
//!
//! ## Module Structure
//!
//! - `protocol` - Message types (ClientMessage, ServerMessage)
//! - `connection` - Connection lifecycle and state management
//! - `handlers` - Message processing logic

mod connection;
mod handlers;
mod protocol;

use axum::{
    extract::{State, ws::WebSocketUpgrade},
    response::Response,
};

use crate::state::AppState;

// Re-export public types
pub use connection::ConnectionId;
pub use protocol::{ClientMessage, ServerMessage};
pub use handlers::MessageResponse;

/// GET /ws - WebSocket upgrade handler.
///
/// Note: Authentication happens via the first message (Auth type) rather than
/// HTTP headers to support browsers that can't set custom headers on WebSocket.
pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(|socket| connection::handle_socket(socket, state))
}
