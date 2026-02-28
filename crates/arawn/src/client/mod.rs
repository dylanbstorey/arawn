//! HTTP/WebSocket client for communicating with the Arawn server.
//!
//! This module provides a client for interacting with the Arawn server's
//! REST API and WebSocket endpoints.

// Allow unused items - this is a client API where not all methods are used yet
#![allow(dead_code)]

use anyhow::Result;
use futures::{SinkExt, Stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;

// ─────────────────────────────────────────────────────────────────────────────
// Types
// ─────────────────────────────────────────────────────────────────────────────

/// Health check response from the server.
#[derive(Debug, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

/// Memory search result.
#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryResult {
    pub id: String,
    #[serde(default)]
    pub content_type: String,
    pub content: String,
    #[serde(default)]
    pub score: f32,
    #[serde(default)]
    pub source: String,
}

/// Memory search response.
#[derive(Debug, Deserialize)]
pub struct MemorySearchResponse {
    pub results: Vec<MemoryResult>,
    #[serde(default)]
    pub query: String,
    #[serde(default)]
    pub count: usize,
}

/// Note from the server.
#[derive(Debug, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub content: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Create note request.
#[derive(Debug, Serialize)]
struct CreateNoteRequest {
    content: String,
}

/// Chat request.
#[derive(Debug, Serialize)]
pub struct ChatRequest {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

/// Session info.
#[derive(Debug, Deserialize)]
pub struct SessionInfo {
    pub id: String,
    pub created_at: String,
    pub message_count: usize,
}

/// Session list response.
#[derive(Debug, Deserialize)]
pub struct SessionListResponse {
    pub sessions: Vec<SessionInfo>,
}

/// Notes list response.
#[derive(Debug, Deserialize)]
pub struct NotesResponse {
    pub notes: Vec<Note>,
    #[serde(default)]
    pub total: usize,
    #[serde(default)]
    pub limit: usize,
    #[serde(default)]
    pub offset: usize,
}

// ─────────────────────────────────────────────────────────────────────────────
// WebSocket Protocol Types (matching arawn-server)
// ─────────────────────────────────────────────────────────────────────────────

/// Messages sent to the server via WebSocket.
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum WsClientMessage {
    Auth {
        token: String,
    },
    Chat {
        session_id: Option<String>,
        message: String,
    },
    Ping,
}

/// Messages received from the server via WebSocket.
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsServerMessage {
    AuthResult {
        success: bool,
        error: Option<String>,
    },
    SessionCreated {
        session_id: String,
    },
    ChatChunk {
        session_id: String,
        chunk: String,
        done: bool,
    },
    ToolStart {
        session_id: String,
        tool_id: String,
        tool_name: String,
    },
    ToolEnd {
        session_id: String,
        tool_id: String,
        success: bool,
    },
    Error {
        code: String,
        message: String,
    },
    Pong,
}

// ─────────────────────────────────────────────────────────────────────────────
// Chat Event Stream
// ─────────────────────────────────────────────────────────────────────────────

/// Events from streaming chat responses.
#[derive(Debug)]
pub enum ChatEvent {
    /// Text chunk from the response.
    Text(String),
    /// A tool started executing.
    ToolStart { id: String, name: String },
    /// A tool finished executing.
    ToolEnd { id: String, success: bool },
    /// The response is complete.
    Done,
    /// An error occurred.
    Error(String),
}

/// Streaming chat response.
pub struct ChatStream {
    receiver: Pin<Box<dyn Stream<Item = Result<ChatEvent>> + Send>>,
}

impl ChatStream {
    /// Get the next event from the stream.
    pub async fn next(&mut self) -> Option<Result<String>> {
        match self.receiver.next().await {
            Some(Ok(ChatEvent::Text(text))) => Some(Ok(text)),
            Some(Ok(ChatEvent::Done)) => None,
            Some(Ok(ChatEvent::Error(e))) => Some(Err(anyhow::anyhow!(e))),
            Some(Ok(ChatEvent::ToolStart { name, .. })) => {
                Some(Ok(format!("\n[Running: {}]\n", name)))
            }
            Some(Ok(ChatEvent::ToolEnd { success, .. })) => {
                let status = if success { "done" } else { "failed" };
                Some(Ok(format!("[{}]\n", status)))
            }
            Some(Err(e)) => Some(Err(e)),
            None => None,
        }
    }

    /// Get the next raw event from the stream.
    pub async fn next_event(&mut self) -> Option<Result<ChatEvent>> {
        self.receiver.next().await
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Client
// ─────────────────────────────────────────────────────────────────────────────

/// HTTP/WebSocket client for the Arawn server.
pub struct Client {
    base_url: Url,
    http: reqwest::Client,
    token: Option<String>,
}

impl Client {
    /// Create a new client for the given server URL.
    pub fn new(base_url: &str) -> Result<Self> {
        let base_url = Url::parse(base_url)?;
        let token = std::env::var("ARAWN_API_TOKEN").ok();

        Ok(Self {
            base_url,
            http: reqwest::Client::new(),
            token,
        })
    }

    /// Create a client with a specific token.
    pub fn with_token(base_url: &str, token: String) -> Result<Self> {
        let base_url = Url::parse(base_url)?;

        Ok(Self {
            base_url,
            http: reqwest::Client::new(),
            token: Some(token),
        })
    }

    /// Check server health.
    pub async fn health(&self) -> Result<HealthResponse> {
        let url = self.base_url.join("/health")?;

        let response = self.http.get(url).send().await?;

        if !response.status().is_success() {
            anyhow::bail!("Server returned error: {}", response.status());
        }

        let health: HealthResponse = response.json().await?;
        Ok(health)
    }

    /// Send a chat message and get a streaming response via WebSocket.
    pub async fn chat_stream(&self, message: &str, session_id: Option<&str>) -> Result<ChatStream> {
        // Convert HTTP URL to WebSocket URL
        let mut ws_url = self.base_url.clone();
        ws_url
            .set_scheme(if self.base_url.scheme() == "https" {
                "wss"
            } else {
                "ws"
            })
            .map_err(|_| anyhow::anyhow!("Failed to set WebSocket scheme"))?;
        ws_url.set_path("/ws");

        // Connect to WebSocket
        let (ws_stream, _) = connect_async(ws_url.as_str()).await?;
        let (mut write, mut read) = ws_stream.split();

        // Authenticate if we have a token
        if let Some(ref token) = self.token {
            let auth_msg = WsClientMessage::Auth {
                token: token.clone(),
            };
            let json = serde_json::to_string(&auth_msg)?;
            write.send(Message::Text(json.into())).await?;

            // Wait for auth response
            if let Some(Ok(msg)) = read.next().await {
                if let Message::Text(text) = msg {
                    let response: WsServerMessage = serde_json::from_str(&text)?;
                    if let WsServerMessage::AuthResult { success, error } = response {
                        if !success {
                            anyhow::bail!("Authentication failed: {}", error.unwrap_or_default());
                        }
                    }
                }
            }
        }

        // Send chat message
        let chat_msg = WsClientMessage::Chat {
            session_id: session_id.map(String::from),
            message: message.to_string(),
        };
        let json = serde_json::to_string(&chat_msg)?;
        write.send(Message::Text(json.into())).await?;

        // Create stream from WebSocket messages
        let stream = async_stream::stream! {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        match serde_json::from_str::<WsServerMessage>(&text) {
                            Ok(ws_msg) => {
                                match ws_msg {
                                    WsServerMessage::ChatChunk { chunk, done, .. } => {
                                        if !chunk.is_empty() {
                                            yield Ok(ChatEvent::Text(chunk));
                                        }
                                        if done {
                                            yield Ok(ChatEvent::Done);
                                            break;
                                        }
                                    }
                                    WsServerMessage::ToolStart { tool_id, tool_name, .. } => {
                                        yield Ok(ChatEvent::ToolStart {
                                            id: tool_id,
                                            name: tool_name,
                                        });
                                    }
                                    WsServerMessage::ToolEnd { tool_id, success, .. } => {
                                        yield Ok(ChatEvent::ToolEnd {
                                            id: tool_id,
                                            success,
                                        });
                                    }
                                    WsServerMessage::Error { message, .. } => {
                                        yield Ok(ChatEvent::Error(message));
                                        break;
                                    }
                                    WsServerMessage::SessionCreated { .. } => {
                                        // Session created, continue waiting for chunks
                                    }
                                    _ => {}
                                }
                            }
                            Err(e) => {
                                yield Err(anyhow::anyhow!("Failed to parse message: {}", e));
                                break;
                            }
                        }
                    }
                    Ok(Message::Close(_)) => break,
                    Err(e) => {
                        yield Err(anyhow::anyhow!("WebSocket error: {}", e));
                        break;
                    }
                    _ => {}
                }
            }
        };

        Ok(ChatStream {
            receiver: Box::pin(stream),
        })
    }

    /// Send a chat message via HTTP (non-streaming).
    pub async fn chat(&self, message: &str, session_id: Option<&str>) -> Result<String> {
        let url = self.base_url.join("/api/v1/chat")?;

        let mut request = self.http.post(url).json(&ChatRequest {
            message: message.to_string(),
            session_id: session_id.map(String::from),
        });

        if let Some(ref token) = self.token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Server returned error {}: {}", status, body);
        }

        #[derive(Deserialize)]
        struct ChatResponse {
            response: String,
        }

        let chat_response: ChatResponse = response.json().await?;
        Ok(chat_response.response)
    }

    /// Search memories.
    pub async fn memory_search(&self, query: &str, limit: usize) -> Result<Vec<MemoryResult>> {
        let url = self.base_url.join("/api/v1/memory/search")?;

        let mut request = self
            .http
            .get(url)
            .query(&[("q", query), ("limit", &limit.to_string())]);

        if let Some(ref token) = self.token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            anyhow::bail!("Server returned error: {}", response.status());
        }

        let search_response: MemorySearchResponse = response.json().await?;
        Ok(search_response.results)
    }

    /// Create a note.
    pub async fn create_note(&self, content: &str) -> Result<Note> {
        let url = self.base_url.join("/api/v1/notes")?;

        let mut request = self.http.post(url).json(&CreateNoteRequest {
            content: content.to_string(),
        });

        if let Some(ref token) = self.token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            anyhow::bail!("Server returned error: {}", response.status());
        }

        let note: Note = response.json().await?;
        Ok(note)
    }

    /// List all notes.
    pub async fn list_notes(&self) -> Result<Vec<Note>> {
        let url = self.base_url.join("/api/v1/notes")?;

        let mut request = self.http.get(url);

        if let Some(ref token) = self.token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            anyhow::bail!("Server returned error: {}", response.status());
        }

        let response: NotesResponse = response.json().await?;
        Ok(response.notes)
    }

    /// Get a single note by ID.
    pub async fn get_note(&self, id: &str) -> Result<Note> {
        let url = self.base_url.join(&format!("/api/v1/notes/{}", id))?;

        let mut request = self.http.get(url);

        if let Some(ref token) = self.token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            anyhow::bail!("Note not found: {}", id);
        }

        if !response.status().is_success() {
            anyhow::bail!("Server returned error: {}", response.status());
        }

        let note: Note = response.json().await?;
        Ok(note)
    }

    /// Delete a note by ID.
    pub async fn delete_note(&self, id: &str) -> Result<()> {
        let url = self.base_url.join(&format!("/api/v1/notes/{}", id))?;

        let mut request = self.http.delete(url);

        if let Some(ref token) = self.token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            anyhow::bail!("Note not found: {}", id);
        }

        if !response.status().is_success() {
            anyhow::bail!("Server returned error: {}", response.status());
        }

        Ok(())
    }

    /// Search notes via memory search endpoint, filtering for note results.
    pub async fn search_notes(&self, query: &str, limit: usize) -> Result<Vec<MemoryResult>> {
        let url = self.base_url.join("/api/v1/memory/search")?;

        // Request more results since we'll filter to notes only
        let search_limit = (limit * 3).max(20);
        let mut request = self
            .http
            .get(url)
            .query(&[("q", query), ("limit", &search_limit.to_string())]);

        if let Some(ref token) = self.token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            anyhow::bail!("Server returned error: {}", response.status());
        }

        let search_response: MemorySearchResponse = response.json().await?;

        // Filter to note-sourced results and respect the requested limit
        let notes: Vec<MemoryResult> = search_response
            .results
            .into_iter()
            .filter(|r| r.source == "notes")
            .take(limit)
            .collect();

        Ok(notes)
    }

    /// List sessions.
    pub async fn list_sessions(&self) -> Result<Vec<SessionInfo>> {
        let url = self.base_url.join("/api/v1/sessions")?;

        let mut request = self.http.get(url);

        if let Some(ref token) = self.token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            anyhow::bail!("Server returned error: {}", response.status());
        }

        let response: SessionListResponse = response.json().await?;
        Ok(response.sessions)
    }

    /// Delete a session.
    pub async fn delete_session(&self, session_id: &str) -> Result<()> {
        let url = self
            .base_url
            .join(&format!("/api/v1/sessions/{}", session_id))?;

        let mut request = self.http.delete(url);

        if let Some(ref token) = self.token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            anyhow::bail!("Server returned error: {}", response.status());
        }

        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = Client::new("http://localhost:8080").unwrap();
        assert!(client.token.is_none());
    }

    #[test]
    fn test_client_with_token() {
        let client = Client::with_token("http://localhost:8080", "test-token".to_string()).unwrap();
        assert_eq!(client.token, Some("test-token".to_string()));
    }

    #[test]
    fn test_ws_client_message_serialization() {
        let msg = WsClientMessage::Auth {
            token: "secret".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("auth"));
        assert!(json.contains("secret"));

        let msg = WsClientMessage::Chat {
            session_id: Some("123".to_string()),
            message: "hello".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("chat"));
        assert!(json.contains("hello"));
    }

    #[test]
    fn test_ws_server_message_deserialization() {
        let json =
            r#"{"type": "chat_chunk", "session_id": "123", "chunk": "hello", "done": false}"#;
        let msg: WsServerMessage = serde_json::from_str(json).unwrap();
        assert!(matches!(msg, WsServerMessage::ChatChunk { .. }));

        let json = r#"{"type": "auth_result", "success": true}"#;
        let msg: WsServerMessage = serde_json::from_str(json).unwrap();
        assert!(matches!(
            msg,
            WsServerMessage::AuthResult { success: true, .. }
        ));
    }
}
