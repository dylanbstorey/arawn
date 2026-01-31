//! Session management endpoints.

use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};

use arawn_agent::{Session, SessionId};

use crate::auth::Identity;
use crate::error::ServerError;
use crate::state::AppState;

// ─────────────────────────────────────────────────────────────────────────────
// Types
// ─────────────────────────────────────────────────────────────────────────────

/// Summary info for a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    /// Session ID.
    pub id: String,
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

/// GET /api/v1/sessions - List all sessions.
pub async fn list_sessions_handler(
    State(state): State<AppState>,
    Extension(_identity): Extension<Identity>,
) -> Result<Json<ListSessionsResponse>, ServerError> {
    let sessions = state.sessions.read().await;

    let mut summaries: Vec<SessionSummary> = sessions
        .values()
        .map(|s| SessionSummary {
            id: s.id.to_string(),
            turn_count: s.turn_count(),
            created_at: s.created_at.to_rfc3339(),
            updated_at: s.updated_at.to_rfc3339(),
        })
        .collect();

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

    let sessions = state.sessions.read().await;
    let session = sessions
        .get(&id)
        .ok_or_else(|| ServerError::NotFound(format!("Session {} not found", session_id)))?;

    Ok(Json(session_to_detail(session)))
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
        routing::get,
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
            .route("/sessions", get(list_sessions_handler))
            .route(
                "/sessions/{id}",
                get(get_session_handler).delete(delete_session_handler),
            )
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
        let sessions = state.sessions.read().await;
        assert!(!sessions.contains_key(&session_id));
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
}
