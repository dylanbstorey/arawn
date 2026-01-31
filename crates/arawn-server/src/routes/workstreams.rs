use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use arawn_workstream::{WorkstreamError, WorkstreamManager};

use crate::state::AppState;

// ── Request/Response types ──────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateWorkstreamRequest {
    pub title: String,
    #[serde(default)]
    pub default_model: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct WorkstreamResponse {
    pub id: String,
    pub title: String,
    pub summary: Option<String>,
    pub state: String,
    pub default_model: Option<String>,
    pub is_scratch: bool,
    pub created_at: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct WorkstreamListResponse {
    pub workstreams: Vec<WorkstreamResponse>,
}

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub role: Option<String>,
    pub content: String,
    #[serde(default)]
    pub metadata: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub id: String,
    pub workstream_id: String,
    pub session_id: Option<String>,
    pub role: String,
    pub content: String,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MessageListResponse {
    pub messages: Vec<MessageResponse>,
}

#[derive(Debug, Deserialize)]
pub struct MessageQuery {
    pub since: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PromoteRequest {
    pub title: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub default_model: Option<String>,
}

// ── Helpers ─────────────────────────────────────────────────────────

fn get_manager(
    state: &AppState,
) -> Result<&Arc<WorkstreamManager>, (StatusCode, Json<serde_json::Value>)> {
    state.workstreams.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "Workstreams not configured" })),
        )
    })
}

fn workstream_error_response(e: WorkstreamError) -> (StatusCode, Json<serde_json::Value>) {
    match &e {
        WorkstreamError::NotFound(_) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

fn to_workstream_response(
    ws: &arawn_workstream::store::Workstream,
    tags: Option<Vec<String>>,
) -> WorkstreamResponse {
    WorkstreamResponse {
        id: ws.id.clone(),
        title: ws.title.clone(),
        summary: ws.summary.clone(),
        state: ws.state.clone(),
        default_model: ws.default_model.clone(),
        is_scratch: ws.is_scratch,
        created_at: ws.created_at.to_rfc3339(),
        updated_at: ws.updated_at.to_rfc3339(),
        tags,
    }
}

fn to_message_response(msg: &arawn_workstream::WorkstreamMessage) -> MessageResponse {
    MessageResponse {
        id: msg.id.clone(),
        workstream_id: msg.workstream_id.clone(),
        session_id: msg.session_id.clone(),
        role: msg.role.to_string(),
        content: msg.content.clone(),
        timestamp: msg.timestamp.to_rfc3339(),
        metadata: msg.metadata.clone(),
    }
}

// ── Handlers ────────────────────────────────────────────────────────

/// POST /api/v1/workstreams
pub async fn create_workstream_handler(
    State(state): State<AppState>,
    Json(req): Json<CreateWorkstreamRequest>,
) -> impl IntoResponse {
    let mgr = match get_manager(&state) {
        Ok(m) => m,
        Err(e) => return e.into_response(),
    };

    match mgr.create_workstream(&req.title, req.default_model.as_deref(), &req.tags) {
        Ok(ws) => {
            let tags = mgr.get_tags(&ws.id).ok();
            (StatusCode::CREATED, Json(to_workstream_response(&ws, tags))).into_response()
        }
        Err(e) => workstream_error_response(e).into_response(),
    }
}

/// GET /api/v1/workstreams
pub async fn list_workstreams_handler(State(state): State<AppState>) -> impl IntoResponse {
    let mgr = match get_manager(&state) {
        Ok(m) => m,
        Err(e) => return e.into_response(),
    };

    match mgr.list_workstreams() {
        Ok(list) => {
            let workstreams: Vec<_> = list
                .iter()
                .map(|ws| to_workstream_response(ws, None))
                .collect();
            Json(WorkstreamListResponse { workstreams }).into_response()
        }
        Err(e) => workstream_error_response(e).into_response(),
    }
}

/// GET /api/v1/workstreams/:id
pub async fn get_workstream_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let mgr = match get_manager(&state) {
        Ok(m) => m,
        Err(e) => return e.into_response(),
    };

    match mgr.get_workstream(&id) {
        Ok(ws) => {
            let tags = mgr.get_tags(&ws.id).ok();
            Json(to_workstream_response(&ws, tags)).into_response()
        }
        Err(e) => workstream_error_response(e).into_response(),
    }
}

/// DELETE /api/v1/workstreams/:id
pub async fn delete_workstream_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let mgr = match get_manager(&state) {
        Ok(m) => m,
        Err(e) => return e.into_response(),
    };

    match mgr.archive_workstream(&id) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => workstream_error_response(e).into_response(),
    }
}

/// POST /api/v1/workstreams/:id/messages
pub async fn send_message_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<SendMessageRequest>,
) -> impl IntoResponse {
    let mgr = match get_manager(&state) {
        Ok(m) => m,
        Err(e) => return e.into_response(),
    };

    let role = match req.role.as_deref().unwrap_or("user") {
        "user" => arawn_workstream::MessageRole::User,
        "assistant" => arawn_workstream::MessageRole::Assistant,
        "system" => arawn_workstream::MessageRole::System,
        "agent_push" => arawn_workstream::MessageRole::AgentPush,
        other => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": format!("Invalid role: {other}") })),
            )
                .into_response();
        }
    };

    match mgr.send_message(Some(&id), role, &req.content, req.metadata.as_deref()) {
        Ok(msg) => (StatusCode::CREATED, Json(to_message_response(&msg))).into_response(),
        Err(e) => workstream_error_response(e).into_response(),
    }
}

/// GET /api/v1/workstreams/:id/messages
pub async fn list_messages_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<MessageQuery>,
) -> impl IntoResponse {
    let mgr = match get_manager(&state) {
        Ok(m) => m,
        Err(e) => return e.into_response(),
    };

    let result = if let Some(since_str) = &query.since {
        match since_str.parse::<DateTime<Utc>>() {
            Ok(since) => mgr.get_messages_since(&id, since),
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": "Invalid 'since' timestamp. Use RFC 3339 format." })),
                )
                    .into_response();
            }
        }
    } else {
        mgr.get_messages(&id)
    };

    match result {
        Ok(msgs) => {
            let messages: Vec<_> = msgs.iter().map(to_message_response).collect();
            Json(MessageListResponse { messages }).into_response()
        }
        Err(e) => workstream_error_response(e).into_response(),
    }
}

/// POST /api/v1/workstreams/:id/promote
pub async fn promote_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<PromoteRequest>,
) -> impl IntoResponse {
    let mgr = match get_manager(&state) {
        Ok(m) => m,
        Err(e) => return e.into_response(),
    };

    if id != arawn_workstream::SCRATCH_ID {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Only the scratch workstream can be promoted" })),
        )
            .into_response();
    }

    match mgr.promote_scratch(&req.title, &req.tags, req.default_model.as_deref()) {
        Ok(ws) => {
            let tags = mgr.get_tags(&ws.id).ok();
            (StatusCode::CREATED, Json(to_workstream_response(&ws, tags))).into_response()
        }
        Err(e) => workstream_error_response(e).into_response(),
    }
}
