//! Session management endpoints.

use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use arawn_agent::{Session, SessionId};

use crate::auth::Identity;
use crate::error::ServerError;
use crate::state::AppState;

// ─────────────────────────────────────────────────────────────────────────────
// Types
// ─────────────────────────────────────────────────────────────────────────────

/// Request to create a new session.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateSessionRequest {
    /// Optional title for the session.
    #[serde(default)]
    pub title: Option<String>,
    /// Optional metadata to attach to the session.
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Request to update a session.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateSessionRequest {
    /// New title for the session.
    #[serde(default)]
    pub title: Option<String>,
    /// Metadata to merge into the session (existing keys will be overwritten).
    #[serde(default)]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    /// Move session to a different workstream.
    #[serde(default)]
    pub workstream_id: Option<String>,
}

/// Message info for conversation history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageInfo {
    /// Role of the message sender.
    pub role: String,
    /// Content of the message.
    pub content: String,
    /// Timestamp of the message.
    pub timestamp: String,
}

/// Response containing session messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMessagesResponse {
    /// Session ID.
    pub session_id: String,
    /// List of messages in the session.
    pub messages: Vec<MessageInfo>,
    /// Total message count.
    pub count: usize,
}

/// Summary info for a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    /// Session ID.
    pub id: String,
    /// Session title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Number of turns in the session.
    pub turn_count: usize,
    /// Creation time (ISO 8601).
    pub created_at: String,
    /// Last update time (ISO 8601).
    pub updated_at: String,
}

/// Full session details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionDetail {
    /// Session ID.
    pub id: String,
    /// All turns in the session.
    pub turns: Vec<TurnInfo>,
    /// Creation time.
    pub created_at: String,
    /// Last update time.
    pub updated_at: String,
    /// Session metadata.
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// Turn info for API responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnInfo {
    /// Turn ID.
    pub id: String,
    /// User message.
    pub user_message: String,
    /// Assistant response (if complete).
    pub assistant_response: Option<String>,
    /// Number of tool calls.
    pub tool_call_count: usize,
    /// When the turn started.
    pub started_at: String,
    /// When the turn completed.
    pub completed_at: Option<String>,
}

/// Response for list sessions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListSessionsResponse {
    /// List of sessions.
    pub sessions: Vec<SessionSummary>,
    /// Total count.
    pub total: usize,
}

// ─────────────────────────────────────────────────────────────────────────────
// Handlers
// ─────────────────────────────────────────────────────────────────────────────

/// POST /api/v1/sessions - Create a new session.
pub async fn create_session_handler(
    State(state): State<AppState>,
    Extension(_identity): Extension<Identity>,
    Json(request): Json<CreateSessionRequest>,
) -> Result<(StatusCode, Json<SessionDetail>), ServerError> {
    // Create a new session with optional metadata
    let session_id = state.get_or_create_session(None).await;

    // Update metadata if provided
    if !request.metadata.is_empty() || request.title.is_some() {
        state
            .session_cache
            .with_session_mut(&session_id, |session| {
                // Set title as metadata if provided
                if let Some(title) = &request.title {
                    session
                        .metadata
                        .insert("title".to_string(), serde_json::Value::String(title.clone()));
                }
                // Merge additional metadata
                for (key, value) in &request.metadata {
                    session.metadata.insert(key.clone(), value.clone());
                }
            })
            .await;
    }

    // Return the created session
    let session = state
        .session_cache
        .get(&session_id)
        .await
        .ok_or_else(|| ServerError::Internal("Failed to retrieve created session".to_string()))?;

    Ok((StatusCode::CREATED, Json(session_to_detail(&session))))
}

/// GET /api/v1/sessions - List all sessions.
pub async fn list_sessions_handler(
    State(state): State<AppState>,
    Extension(_identity): Extension<Identity>,
) -> Result<Json<ListSessionsResponse>, ServerError> {
    let mut summaries: Vec<SessionSummary> = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();

    // Get sessions from the cache (active sessions)
    let cached_sessions = state.session_cache.all_sessions().await;
    for (_, session) in cached_sessions {
        let title = session
            .metadata
            .get("title")
            .and_then(|v| v.as_str())
            .map(String::from);
        seen_ids.insert(session.id.to_string());
        summaries.push(SessionSummary {
            id: session.id.to_string(),
            title,
            turn_count: session.turn_count(),
            created_at: session.created_at.to_rfc3339(),
            updated_at: session.updated_at.to_rfc3339(),
        });
    }

    // Also include sessions from workstream storage (for historical sessions)
    if let Some(ref workstreams) = state.workstreams {
        if let Ok(ws_list) = workstreams.list_workstreams() {
            for ws in ws_list {
                if let Ok(ws_sessions) = workstreams.list_sessions(&ws.id) {
                    for ws_session in ws_sessions {
                        // Skip if we already have this session from cache
                        if seen_ids.contains(&ws_session.id) {
                            continue;
                        }
                        seen_ids.insert(ws_session.id.clone());

                        summaries.push(SessionSummary {
                            id: ws_session.id.clone(),
                            title: ws_session.summary.clone(),
                            turn_count: ws_session.turn_count.unwrap_or(0) as usize,
                            created_at: ws_session.started_at.to_rfc3339(),
                            updated_at: ws_session
                                .ended_at
                                .unwrap_or(ws_session.started_at)
                                .to_rfc3339(),
                        });
                    }
                }
            }
        }
    }

    // Sort by updated_at descending (most recent first)
    summaries.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

    let total = summaries.len();

    Ok(Json(ListSessionsResponse {
        sessions: summaries,
        total,
    }))
}

/// GET /api/v1/sessions/:id - Get session details.
pub async fn get_session_handler(
    State(state): State<AppState>,
    Extension(_identity): Extension<Identity>,
    Path(session_id): Path<String>,
) -> Result<Json<SessionDetail>, ServerError> {
    let id = parse_session_id(&session_id)?;

    // Try session cache first
    if let Some(session) = state.session_cache.get(&id).await {
        return Ok(Json(session_to_detail(&session)));
    }

    // Try to load from workstream if workstreams are configured
    if let Some(ref workstreams) = state.workstreams {
        // First, find which workstream this session belongs to
        if let Ok(ws_list) = workstreams.list_workstreams() {
            for ws in ws_list {
                if let Ok(ws_sessions) = workstreams.list_sessions(&ws.id) {
                    if ws_sessions.iter().any(|s| s.id == session_id) {
                        // Found the workstream, try to load the session
                        if let Ok((session, _)) = state.session_cache.get_or_load(id, &ws.id).await {
                            return Ok(Json(session_to_detail(&session)));
                        }
                    }
                }
            }
        }
    }

    Err(ServerError::NotFound(format!("Session {} not found", session_id)))
}

/// DELETE /api/v1/sessions/:id - Delete a session.
///
/// Removes the session and triggers background indexing (if enabled).
pub async fn delete_session_handler(
    State(state): State<AppState>,
    Extension(_identity): Extension<Identity>,
    Path(session_id): Path<String>,
) -> Result<StatusCode, ServerError> {
    let id = parse_session_id(&session_id)?;

    if state.close_session(id).await {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ServerError::NotFound(format!(
            "Session {} not found",
            session_id
        )))
    }
}

/// PATCH /api/v1/sessions/:id - Update session metadata.
pub async fn update_session_handler(
    State(state): State<AppState>,
    Extension(_identity): Extension<Identity>,
    Path(session_id): Path<String>,
    Json(request): Json<UpdateSessionRequest>,
) -> Result<Json<SessionDetail>, ServerError> {
    tracing::info!(
        session_id = %session_id,
        workstream_id = ?request.workstream_id,
        title = ?request.title,
        "PATCH /sessions/:id - update_session_handler called"
    );

    let id = parse_session_id(&session_id)?;

    // Track the workstream ID for potential reload after reassignment
    let mut target_workstream_id: Option<String> = None;

    // Handle workstream reassignment if requested
    if let Some(ref new_workstream_id) = request.workstream_id {
        tracing::info!(
            session_id = %session_id,
            new_workstream_id = %new_workstream_id,
            "Attempting to reassign session to new workstream"
        );
        if let Some(ref workstreams) = state.workstreams {
            match workstreams.reassign_session(&session_id, new_workstream_id) {
                Ok(session) => {
                    tracing::info!(
                        session_id = %session_id,
                        new_workstream_id = %new_workstream_id,
                        result_workstream_id = %session.workstream_id,
                        "Session reassignment successful"
                    );
                    // Invalidate the session cache so it reloads from the new workstream
                    state.invalidate_session(id).await;
                    target_workstream_id = Some(new_workstream_id.clone());
                }
                Err(e) => {
                    tracing::error!(
                        session_id = %session_id,
                        new_workstream_id = %new_workstream_id,
                        error = %e,
                        "Session reassignment failed"
                    );
                    return Err(ServerError::BadRequest(format!(
                        "Failed to reassign session: {}",
                        e
                    )));
                }
            }
        } else {
            tracing::error!("Workstreams not configured");
            return Err(ServerError::BadRequest(
                "Workstreams not configured".to_string(),
            ));
        }
    }

    // If we reassigned, we need to reload the session from the new workstream
    // before applying any metadata updates
    if let Some(ref workstream_id) = target_workstream_id {
        // Reload session from the new workstream
        let (mut session, _) = state
            .session_cache
            .get_or_load(id, workstream_id)
            .await
            .map_err(|e| {
                tracing::error!(
                    session_id = %session_id,
                    workstream_id = %workstream_id,
                    error = %e,
                    "Failed to reload session after reassignment"
                );
                ServerError::NotFound(format!("Session {} not found after reassignment", session_id))
            })?;

        // Apply title/metadata updates if provided
        if request.title.is_some() || request.metadata.is_some() {
            if let Some(ref title) = request.title {
                session
                    .metadata
                    .insert("title".to_string(), serde_json::Value::String(title.clone()));
            }
            if let Some(ref metadata) = request.metadata {
                for (key, value) in metadata {
                    session.metadata.insert(key.clone(), value.clone());
                }
            }
            session.updated_at = chrono::Utc::now();

            // Update the cache with the modified session
            let _ = state.session_cache.update(id, session.clone()).await;
        }

        return Ok(Json(session_to_detail(&session)));
    }

    // No reassignment - update session via cache directly
    let updated = state
        .session_cache
        .with_session_mut(&id, |session| {
            // Update title if provided
            if let Some(ref title) = request.title {
                session
                    .metadata
                    .insert("title".to_string(), serde_json::Value::String(title.clone()));
            }

            // Merge metadata if provided
            if let Some(ref metadata) = request.metadata {
                for (key, value) in metadata {
                    session.metadata.insert(key.clone(), value.clone());
                }
            }

            // Update timestamp
            session.updated_at = chrono::Utc::now();
            session_to_detail(session)
        })
        .await;

    match updated {
        Some(detail) => Ok(Json(detail)),
        None => Err(ServerError::NotFound(format!(
            "Session {} not found",
            session_id
        ))),
    }
}

/// GET /api/v1/sessions/:id/messages - Get session conversation history.
pub async fn get_session_messages_handler(
    State(state): State<AppState>,
    Extension(_identity): Extension<Identity>,
    Path(session_id): Path<String>,
) -> Result<Json<SessionMessagesResponse>, ServerError> {
    let id = parse_session_id(&session_id)?;

    // Try session cache first
    let session = if let Some(session) = state.session_cache.get(&id).await {
        session
    } else if let Some(ref workstreams) = state.workstreams {
        // Try to load from workstream
        let mut found_session = None;
        if let Ok(ws_list) = workstreams.list_workstreams() {
            for ws in ws_list {
                if let Ok(ws_sessions) = workstreams.list_sessions(&ws.id) {
                    if ws_sessions.iter().any(|s| s.id == session_id) {
                        // Found the workstream, try to load the session
                        if let Ok((session, _)) = state.session_cache.get_or_load(id, &ws.id).await
                        {
                            found_session = Some(session);
                            break;
                        }
                    }
                }
            }
        }
        found_session.ok_or_else(|| ServerError::NotFound(format!("Session {} not found", session_id)))?
    } else {
        return Err(ServerError::NotFound(format!("Session {} not found", session_id)));
    };

    let mut messages = Vec::new();
    for turn in session.all_turns() {
        messages.push(MessageInfo {
            role: "user".to_string(),
            content: turn.user_message.clone(),
            timestamp: turn.started_at.to_rfc3339(),
        });
        if let Some(ref response) = turn.assistant_response {
            messages.push(MessageInfo {
                role: "assistant".to_string(),
                content: response.clone(),
                timestamp: turn
                    .completed_at
                    .map(|dt| dt.to_rfc3339())
                    .unwrap_or_else(|| turn.started_at.to_rfc3339()),
            });
        }
    }

    let count = messages.len();

    Ok(Json(SessionMessagesResponse {
        session_id: session_id.clone(),
        messages,
        count,
    }))
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

fn parse_session_id(s: &str) -> Result<SessionId, ServerError> {
    uuid::Uuid::parse_str(s)
        .map(SessionId::from_uuid)
        .map_err(|_| ServerError::BadRequest(format!("Invalid session ID: {}", s)))
}

fn session_to_detail(session: &Session) -> SessionDetail {
    SessionDetail {
        id: session.id.to_string(),
        turns: session
            .all_turns()
            .iter()
            .map(|t| TurnInfo {
                id: t.id.to_string(),
                user_message: t.user_message.clone(),
                assistant_response: t.assistant_response.clone(),
                tool_call_count: t.tool_calls.len(),
                started_at: t.started_at.to_rfc3339(),
                completed_at: t.completed_at.map(|dt| dt.to_rfc3339()),
            })
            .collect(),
        created_at: session.created_at.to_rfc3339(),
        updated_at: session.updated_at.to_rfc3339(),
        metadata: session.metadata.clone(),
    }
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
        routing::{get, post},
    };
    use tower::ServiceExt;

    fn create_test_state() -> AppState {
        let backend = MockBackend::with_text("Test");
        let agent = Agent::builder()
            .with_backend(backend)
            .with_tools(ToolRegistry::new())
            .build()
            .unwrap();

        AppState::new(agent, ServerConfig::new(Some("test-token".to_string())))
    }

    fn create_test_router(state: AppState) -> Router {
        Router::new()
            .route(
                "/sessions",
                post(create_session_handler).get(list_sessions_handler),
            )
            .route(
                "/sessions/{id}",
                get(get_session_handler)
                    .patch(update_session_handler)
                    .delete(delete_session_handler),
            )
            .route("/sessions/{id}/messages", get(get_session_messages_handler))
            .layer(middleware::from_fn_with_state(
                state.clone(),
                auth_middleware,
            ))
            .with_state(state)
    }

    #[tokio::test]
    async fn test_list_sessions_empty() {
        let state = create_test_state();
        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/sessions")
                    .header("Authorization", "Bearer test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let result: ListSessionsResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(result.total, 0);
        assert!(result.sessions.is_empty());
    }

    #[tokio::test]
    async fn test_list_sessions_with_data() {
        let state = create_test_state();

        // Create some sessions
        state.get_or_create_session(None).await;
        state.get_or_create_session(None).await;

        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/sessions")
                    .header("Authorization", "Bearer test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let result: ListSessionsResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(result.total, 2);
    }

    #[tokio::test]
    async fn test_get_session() {
        let state = create_test_state();
        let session_id = state.get_or_create_session(None).await;

        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri(&format!("/sessions/{}", session_id))
                    .header("Authorization", "Bearer test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let result: SessionDetail = serde_json::from_slice(&body).unwrap();
        assert_eq!(result.id, session_id.to_string());
        assert!(result.turns.is_empty());
    }

    #[tokio::test]
    async fn test_get_session_not_found() {
        let state = create_test_state();
        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/sessions/00000000-0000-0000-0000-000000000000")
                    .header("Authorization", "Bearer test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_session_invalid_id() {
        let state = create_test_state();
        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/sessions/not-a-uuid")
                    .header("Authorization", "Bearer test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_delete_session() {
        let state = create_test_state();
        let session_id = state.get_or_create_session(None).await;

        let app = create_test_router(state.clone());

        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(&format!("/sessions/{}", session_id))
                    .header("Authorization", "Bearer test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);

        // Verify deleted
        assert!(!state.session_cache.contains(&session_id).await);
    }

    #[tokio::test]
    async fn test_delete_session_not_found() {
        let state = create_test_state();
        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri("/sessions/00000000-0000-0000-0000-000000000000")
                    .header("Authorization", "Bearer test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_create_session() {
        let state = create_test_state();
        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/sessions")
                    .header("Authorization", "Bearer test-token")
                    .header("Content-Type", "application/json")
                    .body(Body::from(r#"{"title": "Test Session"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let result: SessionDetail = serde_json::from_slice(&body).unwrap();
        assert!(!result.id.is_empty());
        assert!(result.metadata.contains_key("title"));
        assert_eq!(result.metadata["title"], "Test Session");
    }

    #[tokio::test]
    async fn test_create_session_with_metadata() {
        let state = create_test_state();
        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/sessions")
                    .header("Authorization", "Bearer test-token")
                    .header("Content-Type", "application/json")
                    .body(Body::from(
                        r#"{"title": "My Session", "metadata": {"project": "test"}}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let result: SessionDetail = serde_json::from_slice(&body).unwrap();
        assert_eq!(result.metadata["project"], "test");
    }

    #[tokio::test]
    async fn test_update_session() {
        let state = create_test_state();
        let session_id = state.get_or_create_session(None).await;

        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(&format!("/sessions/{}", session_id))
                    .header("Authorization", "Bearer test-token")
                    .header("Content-Type", "application/json")
                    .body(Body::from(r#"{"title": "Updated Title"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let result: SessionDetail = serde_json::from_slice(&body).unwrap();
        assert_eq!(result.metadata["title"], "Updated Title");
    }

    #[tokio::test]
    async fn test_update_session_not_found() {
        let state = create_test_state();
        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri("/sessions/00000000-0000-0000-0000-000000000000")
                    .header("Authorization", "Bearer test-token")
                    .header("Content-Type", "application/json")
                    .body(Body::from(r#"{"title": "Test"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_session_messages_empty() {
        let state = create_test_state();
        let session_id = state.get_or_create_session(None).await;

        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri(&format!("/sessions/{}/messages", session_id))
                    .header("Authorization", "Bearer test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let result: SessionMessagesResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(result.session_id, session_id.to_string());
        assert!(result.messages.is_empty());
        assert_eq!(result.count, 0);
    }

    #[tokio::test]
    async fn test_get_session_messages_with_data() {
        let state = create_test_state();
        let session_id = state.get_or_create_session(None).await;

        // Add a turn with messages
        state
            .session_cache
            .with_session_mut(&session_id, |session| {
                let turn = session.start_turn("Hello");
                turn.complete("Hi there!");
            })
            .await;

        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri(&format!("/sessions/{}/messages", session_id))
                    .header("Authorization", "Bearer test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let result: SessionMessagesResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(result.count, 2);
        assert_eq!(result.messages[0].role, "user");
        assert_eq!(result.messages[0].content, "Hello");
        assert_eq!(result.messages[1].role, "assistant");
        assert_eq!(result.messages[1].content, "Hi there!");
    }

    #[tokio::test]
    async fn test_get_session_messages_not_found() {
        let state = create_test_state();
        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/sessions/00000000-0000-0000-0000-000000000000/messages")
                    .header("Authorization", "Bearer test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
