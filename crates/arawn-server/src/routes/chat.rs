//! Chat endpoints for interacting with the agent.
//!
//! Provides both synchronous and streaming (SSE) endpoints for chat.

use std::convert::Infallible;

use axum::{
    Extension, Json,
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
};
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use arawn_agent::{SessionId, StreamChunk};

use crate::auth::Identity;
use crate::error::ServerError;
use crate::state::AppState;

// ─────────────────────────────────────────────────────────────────────────────
// Request/Response Types
// ─────────────────────────────────────────────────────────────────────────────

/// Request body for chat endpoints.
#[derive(Debug, Clone, Deserialize)]
pub struct ChatRequest {
    /// Optional session ID. If not provided, a new session is created.
    #[serde(default)]
    pub session_id: Option<String>,

    /// The user's message.
    pub message: String,
}

/// Response from the synchronous chat endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    /// The session ID (new or existing).
    pub session_id: String,

    /// The agent's response text.
    pub response: String,

    /// Tool calls made during the response (if any).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_calls: Vec<ToolCallInfo>,

    /// Whether the response was truncated.
    pub truncated: bool,

    /// Token usage.
    pub usage: UsageInfo,
}

/// Simplified tool call info for API response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallInfo {
    /// Tool call ID.
    pub id: String,
    /// Tool name.
    pub name: String,
    /// Whether it succeeded.
    pub success: bool,
}

/// Token usage info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageInfo {
    /// Input tokens used.
    pub input_tokens: u32,
    /// Output tokens generated.
    pub output_tokens: u32,
}

// ─────────────────────────────────────────────────────────────────────────────
// Handlers
// ─────────────────────────────────────────────────────────────────────────────

/// POST /api/v1/chat - Synchronous chat endpoint.
///
/// Sends a message to the agent and waits for the complete response.
pub async fn chat_handler(
    State(state): State<AppState>,
    Extension(_identity): Extension<Identity>,
    Json(request): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, ServerError> {
    // Parse session ID if provided
    let session_id = request
        .session_id
        .as_ref()
        .and_then(|s| Uuid::parse_str(s).ok())
        .map(SessionId::from_uuid);

    // Get or create session
    let session_id = state.get_or_create_session(session_id).await;

    // Get session for turn
    let response = {
        let mut sessions = state.sessions.write().await;
        let session = sessions.get_mut(&session_id).ok_or_else(|| {
            ServerError::Internal("Session disappeared during processing".to_string())
        })?;

        // Execute turn
        state
            .agent
            .turn(session, &request.message)
            .await
            .map_err(ServerError::Agent)?
    };

    // Build response
    let tool_calls: Vec<ToolCallInfo> = response
        .tool_calls
        .iter()
        .zip(response.tool_results.iter())
        .map(|(call, result)| ToolCallInfo {
            id: call.id.clone(),
            name: call.name.clone(),
            success: result.success,
        })
        .collect();

    Ok(Json(ChatResponse {
        session_id: session_id.to_string(),
        response: response.text,
        tool_calls,
        truncated: response.truncated,
        usage: UsageInfo {
            input_tokens: response.usage.input_tokens,
            output_tokens: response.usage.output_tokens,
        },
    }))
}

/// POST /api/v1/chat/stream - SSE streaming chat endpoint.
///
/// Sends a message to the agent and streams the response via Server-Sent Events.
pub async fn chat_stream_handler(
    State(state): State<AppState>,
    Extension(_identity): Extension<Identity>,
    Json(request): Json<ChatRequest>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, ServerError> {
    // Parse session ID if provided
    let session_id = request
        .session_id
        .as_ref()
        .and_then(|s| Uuid::parse_str(s).ok())
        .map(SessionId::from_uuid);

    // Get or create session
    let session_id = state.get_or_create_session(session_id).await;

    // Get the agent stream
    let stream = {
        let mut sessions = state.sessions.write().await;
        let session = sessions.get_mut(&session_id).ok_or_else(|| {
            ServerError::Internal("Session disappeared during processing".to_string())
        })?;

        let cancellation = CancellationToken::new();
        state
            .agent
            .turn_stream(session, &request.message, cancellation)
    };

    // Convert agent stream to SSE events
    let sse_stream = async_stream::stream! {
        use futures::StreamExt;

        // Send session ID first
        let session_event = SseSessionEvent { session_id: session_id.to_string() };
        yield Ok(Event::default()
            .event("session")
            .json_data(session_event)
            .unwrap_or_else(|_| Event::default()));

        let mut stream = std::pin::pin!(stream);
        while let Some(chunk) = stream.next().await {
            let event = match &chunk {
                StreamChunk::Text { content } => {
                    Event::default()
                        .event("text")
                        .json_data(SseTextEvent { content: content.clone() })
                        .unwrap_or_else(|_| Event::default())
                }
                StreamChunk::ToolStart { id, name } => {
                    Event::default()
                        .event("tool_start")
                        .json_data(SseToolStartEvent {
                            id: id.clone(),
                            name: name.clone(),
                        })
                        .unwrap_or_else(|_| Event::default())
                }
                StreamChunk::ToolOutput { id, content } => {
                    Event::default()
                        .event("tool_output")
                        .json_data(SseToolOutputEvent {
                            id: id.clone(),
                            content: content.clone(),
                        })
                        .unwrap_or_else(|_| Event::default())
                }
                StreamChunk::ToolEnd { id, success, content } => {
                    Event::default()
                        .event("tool_end")
                        .json_data(SseToolEndEvent {
                            id: id.clone(),
                            success: *success,
                            content: content.clone(),
                        })
                        .unwrap_or_else(|_| Event::default())
                }
                StreamChunk::Done { iterations } => {
                    Event::default()
                        .event("done")
                        .json_data(SseDoneEvent { iterations: *iterations })
                        .unwrap_or_else(|_| Event::default())
                }
                StreamChunk::Error { message } => {
                    Event::default()
                        .event("error")
                        .json_data(SseErrorEvent { message: message.clone() })
                        .unwrap_or_else(|_| Event::default())
                }
            };
            yield Ok(event);
        }
    };

    Ok(Sse::new(sse_stream).keep_alive(KeepAlive::default()))
}

// ─────────────────────────────────────────────────────────────────────────────
// SSE Event Types
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
struct SseSessionEvent {
    session_id: String,
}

#[derive(Debug, Serialize)]
struct SseTextEvent {
    content: String,
}

#[derive(Debug, Serialize)]
struct SseToolStartEvent {
    id: String,
    name: String,
}

#[derive(Debug, Serialize)]
struct SseToolOutputEvent {
    id: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct SseToolEndEvent {
    id: String,
    success: bool,
    content: String,
}

#[derive(Debug, Serialize)]
struct SseDoneEvent {
    iterations: u32,
}

#[derive(Debug, Serialize)]
struct SseErrorEvent {
    message: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::auth_middleware;
    use crate::config::ServerConfig;
    use arawn_agent::{Agent, ToolRegistry};
    use arawn_llm::MockBackend;
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode},
        middleware,
        routing::post,
    };
    use tower::ServiceExt;

    fn create_test_state() -> AppState {
        let backend = MockBackend::with_text("Hello from the agent!");
        let agent = Agent::builder()
            .with_backend(backend)
            .with_tools(ToolRegistry::new())
            .build()
            .unwrap();

        AppState::new(agent, ServerConfig::new(Some("test-token".to_string())))
    }

    fn create_test_router(state: AppState) -> Router {
        Router::new()
            .route("/chat", post(chat_handler))
            .route("/chat/stream", post(chat_stream_handler))
            .layer(middleware::from_fn_with_state(
                state.clone(),
                auth_middleware,
            ))
            .with_state(state)
    }

    #[tokio::test]
    async fn test_chat_new_session() {
        let state = create_test_state();
        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/chat")
                    .header("Authorization", "Bearer test-token")
                    .header("Content-Type", "application/json")
                    .body(Body::from(r#"{"message": "Hello"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let chat_response: ChatResponse = serde_json::from_slice(&body).unwrap();

        assert!(!chat_response.session_id.is_empty());
        assert_eq!(chat_response.response, "Hello from the agent!");
        assert!(!chat_response.truncated);
    }

    #[tokio::test]
    async fn test_chat_existing_session() {
        // Use multi-response backend for two calls
        let backend = MockBackend::new(vec![
            arawn_llm::CompletionResponse::new(
                "msg_1",
                "model",
                vec![arawn_llm::ContentBlock::Text {
                    text: "First response".to_string(),
                    cache_control: None,
                }],
                arawn_llm::StopReason::EndTurn,
                arawn_llm::Usage::new(10, 20),
            ),
            arawn_llm::CompletionResponse::new(
                "msg_2",
                "model",
                vec![arawn_llm::ContentBlock::Text {
                    text: "Second response".to_string(),
                    cache_control: None,
                }],
                arawn_llm::StopReason::EndTurn,
                arawn_llm::Usage::new(10, 20),
            ),
        ]);
        let agent = Agent::builder()
            .with_backend(backend)
            .with_tools(ToolRegistry::new())
            .build()
            .unwrap();
        let state = AppState::new(agent, ServerConfig::new(Some("test-token".to_string())));

        // Pre-create the session
        let session_id = state.get_or_create_session(None).await;
        let app = create_test_router(state.clone());

        // First request
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/chat")
                    .header("Authorization", "Bearer test-token")
                    .header("Content-Type", "application/json")
                    .body(Body::from(format!(
                        r#"{{"session_id": "{}", "message": "Hello"}}"#,
                        session_id
                    )))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let chat_response: ChatResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(chat_response.session_id, session_id.to_string());
        assert_eq!(chat_response.response, "First response");

        // Verify session has turn
        let sessions = state.sessions.read().await;
        let session = sessions.get(&session_id).unwrap();
        assert_eq!(session.turn_count(), 1);
    }

    #[tokio::test]
    async fn test_chat_requires_auth() {
        let state = create_test_state();
        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/chat")
                    .header("Content-Type", "application/json")
                    .body(Body::from(r#"{"message": "Hello"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_chat_stream_returns_sse() {
        let state = create_test_state();
        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/chat/stream")
                    .header("Authorization", "Bearer test-token")
                    .header("Content-Type", "application/json")
                    .body(Body::from(r#"{"message": "Hello"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "text/event-stream"
        );
    }

    #[test]
    fn test_chat_request_parsing() {
        let json = r#"{"message": "Hello"}"#;
        let request: ChatRequest = serde_json::from_str(json).unwrap();
        assert!(request.session_id.is_none());
        assert_eq!(request.message, "Hello");

        let json = r#"{"session_id": "123e4567-e89b-12d3-a456-426614174000", "message": "Hello"}"#;
        let request: ChatRequest = serde_json::from_str(json).unwrap();
        assert!(request.session_id.is_some());
    }

    #[test]
    fn test_chat_response_serialization() {
        let response = ChatResponse {
            session_id: "test-id".to_string(),
            response: "Hello!".to_string(),
            tool_calls: vec![],
            truncated: false,
            usage: UsageInfo {
                input_tokens: 10,
                output_tokens: 20,
            },
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("session_id"));
        assert!(json.contains("Hello!"));
        assert!(!json.contains("tool_calls")); // Empty vec should be skipped
    }
}
