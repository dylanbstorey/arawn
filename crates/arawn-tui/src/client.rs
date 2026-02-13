//! WebSocket client for connecting to the Arawn server.

use crate::protocol::{ClientMessage, ServerMessage};
use anyhow::{Context, Result};
use futures::{SinkExt, StreamExt};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;

/// Connection status for display in the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionStatus {
    /// Not connected to server.
    Disconnected,
    /// Attempting to connect.
    Connecting,
    /// Connected and ready.
    Connected,
    /// Connection failed, will retry.
    Reconnecting { attempt: u32 },
}

impl std::fmt::Display for ConnectionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Disconnected => write!(f, "disconnected"),
            Self::Connecting => write!(f, "connecting..."),
            Self::Connected => write!(f, "connected"),
            Self::Reconnecting { attempt } => write!(f, "reconnecting ({})", attempt),
        }
    }
}

/// WebSocket client for real-time communication with the Arawn server.
pub struct WsClient {
    /// Server URL (HTTP, will be converted to WS).
    server_url: String,
    /// Channel for sending messages to the server.
    tx: mpsc::UnboundedSender<ClientMessage>,
    /// Channel for receiving messages from the server.
    rx: mpsc::UnboundedReceiver<ServerMessage>,
    /// Channel for receiving status updates.
    status_rx: mpsc::UnboundedReceiver<ConnectionStatus>,
    /// Current connection status.
    current_status: ConnectionStatus,
    /// Task handle for the connection loop.
    #[allow(dead_code)]
    task: tokio::task::JoinHandle<()>,
}

impl WsClient {
    /// Create a new client and start connecting to the server.
    pub fn new(server_url: &str) -> Self {
        let (client_tx, client_rx) = mpsc::unbounded_channel::<ClientMessage>();
        let (server_tx, server_rx) = mpsc::unbounded_channel::<ServerMessage>();
        let (status_tx, status_rx) = mpsc::unbounded_channel::<ConnectionStatus>();

        let url = server_url.to_string();
        let task = tokio::spawn(connection_loop(url, client_rx, server_tx, status_tx));

        Self {
            server_url: server_url.to_string(),
            tx: client_tx,
            rx: server_rx,
            status_rx,
            current_status: ConnectionStatus::Connecting,
            task,
        }
    }

    /// Get the server URL.
    pub fn server_url(&self) -> &str {
        &self.server_url
    }

    /// Get the current connection status.
    pub fn status(&self) -> ConnectionStatus {
        self.current_status
    }

    /// Poll for status updates (non-blocking).
    pub fn poll_status(&mut self) -> Option<ConnectionStatus> {
        match self.status_rx.try_recv() {
            Ok(status) => {
                self.current_status = status;
                Some(status)
            }
            Err(_) => None,
        }
    }

    /// Receive the next server message (async).
    pub async fn recv(&mut self) -> Option<ServerMessage> {
        self.rx.recv().await
    }

    /// Try to receive a server message (non-blocking).
    pub fn try_recv(&mut self) -> Option<ServerMessage> {
        self.rx.try_recv().ok()
    }

    /// Send a chat message.
    pub fn send_chat(
        &self,
        message: String,
        session_id: Option<String>,
        workstream_id: Option<String>,
    ) -> Result<()> {
        self.tx
            .send(ClientMessage::Chat {
                session_id,
                workstream_id,
                message,
            })
            .context("Failed to send chat message")
    }

    /// Send a ping.
    pub fn send_ping(&self) -> Result<()> {
        self.tx
            .send(ClientMessage::Ping)
            .context("Failed to send ping")
    }

    /// Subscribe to a session.
    pub fn subscribe(&self, session_id: String) -> Result<()> {
        self.tx
            .send(ClientMessage::Subscribe { session_id })
            .context("Failed to subscribe to session")
    }

    /// Authenticate with a token.
    pub fn authenticate(&self, token: String) -> Result<()> {
        self.tx
            .send(ClientMessage::Auth { token })
            .context("Failed to send auth")
    }

    /// Cancel the current operation for a session.
    pub fn cancel(&self, session_id: String) -> Result<()> {
        self.tx
            .send(ClientMessage::Cancel { session_id })
            .context("Failed to send cancel")
    }
}

/// Connection loop that handles reconnection with exponential backoff.
async fn connection_loop(
    server_url: String,
    mut client_rx: mpsc::UnboundedReceiver<ClientMessage>,
    server_tx: mpsc::UnboundedSender<ServerMessage>,
    status_tx: mpsc::UnboundedSender<ConnectionStatus>,
) {
    let mut attempt = 0u32;
    let max_backoff = Duration::from_secs(30);

    loop {
        // Convert HTTP URL to WebSocket URL
        let ws_url = match http_to_ws_url(&server_url) {
            Ok(url) => url,
            Err(e) => {
                tracing::error!("Invalid server URL: {}", e);
                let _ = status_tx.send(ConnectionStatus::Disconnected);
                return;
            }
        };

        let _ = status_tx.send(if attempt == 0 {
            ConnectionStatus::Connecting
        } else {
            ConnectionStatus::Reconnecting { attempt }
        });

        // Try to connect
        tracing::info!("Connecting to {}", ws_url);
        match connect_async(&ws_url).await {
            Ok((ws_stream, _)) => {
                attempt = 0;
                let _ = status_tx.send(ConnectionStatus::Connected);
                tracing::info!("Connected to server");

                // Handle the connection
                let disconnected =
                    handle_connection(ws_stream, &mut client_rx, &server_tx).await;

                if !disconnected {
                    // Clean shutdown requested
                    let _ = status_tx.send(ConnectionStatus::Disconnected);
                    return;
                }

                // Connection lost, will reconnect
                tracing::warn!("Connection lost, will reconnect");
            }
            Err(e) => {
                tracing::warn!("Connection failed: {}", e);
            }
        }

        // Exponential backoff
        attempt += 1;
        let backoff = std::cmp::min(
            Duration::from_millis(100 * 2u64.pow(attempt.min(10))),
            max_backoff,
        );
        tracing::debug!("Reconnecting in {:?}", backoff);
        tokio::time::sleep(backoff).await;
    }
}

/// Handle an active WebSocket connection.
/// Returns true if we should reconnect, false for clean shutdown.
async fn handle_connection(
    ws_stream: tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    client_rx: &mut mpsc::UnboundedReceiver<ClientMessage>,
    server_tx: &mpsc::UnboundedSender<ServerMessage>,
) -> bool {
    let (mut ws_sink, mut ws_stream) = ws_stream.split();

    loop {
        tokio::select! {
            // Message from UI to send to server
            Some(msg) = client_rx.recv() => {
                let json = match serde_json::to_string(&msg) {
                    Ok(j) => j,
                    Err(e) => {
                        tracing::error!("Failed to serialize message: {}", e);
                        continue;
                    }
                };

                if let Err(e) = ws_sink.send(Message::Text(json.into())).await {
                    tracing::error!("Failed to send message: {}", e);
                    return true; // Reconnect
                }
            }

            // Message from server
            Some(msg) = ws_stream.next() => {
                match msg {
                    Ok(Message::Text(text)) => {
                        match serde_json::from_str::<ServerMessage>(&text) {
                            Ok(server_msg) => {
                                if server_tx.send(server_msg).is_err() {
                                    // Receiver dropped, clean shutdown
                                    return false;
                                }
                            }
                            Err(e) => {
                                tracing::warn!("Failed to parse server message: {} - {}", e, text);
                            }
                        }
                    }
                    Ok(Message::Ping(data)) => {
                        let _ = ws_sink.send(Message::Pong(data)).await;
                    }
                    Ok(Message::Pong(_)) => {}
                    Ok(Message::Close(_)) => {
                        tracing::info!("Server closed connection");
                        return true; // Reconnect
                    }
                    Ok(Message::Binary(_)) => {
                        tracing::warn!("Unexpected binary message");
                    }
                    Ok(Message::Frame(_)) => {}
                    Err(e) => {
                        tracing::error!("WebSocket error: {}", e);
                        return true; // Reconnect
                    }
                }
            }

            // Both channels closed
            else => {
                return false; // Clean shutdown
            }
        }
    }
}

/// Convert an HTTP URL to a WebSocket URL with /ws path.
fn http_to_ws_url(http_url: &str) -> Result<String> {
    let mut url = Url::parse(http_url).context("Invalid URL")?;

    // Change scheme
    let new_scheme = match url.scheme() {
        "http" | "ws" => "ws",
        "https" | "wss" => "wss",
        other => anyhow::bail!("Unsupported URL scheme: {}", other),
    };

    url.set_scheme(new_scheme)
        .map_err(|_| anyhow::anyhow!("Failed to set scheme"))?;

    // Add /ws path
    url.set_path("/ws");

    Ok(url.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_to_ws_url() {
        assert_eq!(
            http_to_ws_url("http://localhost:8080").unwrap(),
            "ws://localhost:8080/ws"
        );
        assert_eq!(
            http_to_ws_url("https://example.com").unwrap(),
            "wss://example.com/ws"
        );
        assert_eq!(
            http_to_ws_url("http://localhost:8080/api").unwrap(),
            "ws://localhost:8080/ws"
        );
        assert_eq!(
            http_to_ws_url("ws://localhost:8080").unwrap(),
            "ws://localhost:8080/ws"
        );
    }

    #[test]
    fn test_connection_status_display() {
        assert_eq!(ConnectionStatus::Disconnected.to_string(), "disconnected");
        assert_eq!(ConnectionStatus::Connecting.to_string(), "connecting...");
        assert_eq!(ConnectionStatus::Connected.to_string(), "connected");
        assert_eq!(
            ConnectionStatus::Reconnecting { attempt: 3 }.to_string(),
            "reconnecting (3)"
        );
    }
}
