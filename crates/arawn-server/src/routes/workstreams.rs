use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use arawn_workstream::WorkstreamManager;

use crate::error::ServerError;
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

#[derive(Debug, Deserialize)]
pub struct UpdateWorkstreamRequest {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub default_model: Option<String>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct SessionResponse {
    pub id: String,
    pub workstream_id: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Serialize)]
pub struct SessionListResponse {
    pub sessions: Vec<SessionResponse>,
}

// ── Helpers ─────────────────────────────────────────────────────────

fn get_manager(state: &AppState) -> Result<&Arc<WorkstreamManager>, ServerError> {
    state
        .workstreams
        .as_ref()
        .ok_or_else(|| ServerError::ServiceUnavailable("Workstreams not configured".to_string()))
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
) -> Result<(StatusCode, Json<WorkstreamResponse>), ServerError> {
    let mgr = get_manager(&state)?;

    let ws = mgr
        .create_workstream(&req.title, req.default_model.as_deref(), &req.tags)
        ?;

    let tags = mgr.get_tags(&ws.id).ok();
    Ok((StatusCode::CREATED, Json(to_workstream_response(&ws, tags))))
}

/// GET /api/v1/workstreams
pub async fn list_workstreams_handler(
    State(state): State<AppState>,
) -> Result<Json<WorkstreamListResponse>, ServerError> {
    let mgr = get_manager(&state)?;

    let list = mgr.list_workstreams()?;
    let workstreams: Vec<_> = list
        .iter()
        .map(|ws| to_workstream_response(ws, None))
        .collect();

    Ok(Json(WorkstreamListResponse { workstreams }))
}

/// GET /api/v1/workstreams/:id
pub async fn get_workstream_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<WorkstreamResponse>, ServerError> {
    let mgr = get_manager(&state)?;

    let ws = mgr.get_workstream(&id)?;
    let tags = mgr.get_tags(&ws.id).ok();

    Ok(Json(to_workstream_response(&ws, tags)))
}

/// DELETE /api/v1/workstreams/:id
pub async fn delete_workstream_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ServerError> {
    let mgr = get_manager(&state)?;

    mgr.archive_workstream(&id)?;

    Ok(StatusCode::NO_CONTENT)
}

/// PATCH /api/v1/workstreams/:id
pub async fn update_workstream_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateWorkstreamRequest>,
) -> Result<Json<WorkstreamResponse>, ServerError> {
    let mgr = get_manager(&state)?;

    // Update workstream fields
    let ws = mgr
        .update_workstream(
            &id,
            req.title.as_deref(),
            req.summary.as_deref(),
            req.default_model.as_deref(),
        )
        ?;

    // Update tags if provided
    if let Some(ref tags) = req.tags {
        mgr.set_tags(&id, tags)?;
    }

    let tags = mgr.get_tags(&ws.id).ok();
    Ok(Json(to_workstream_response(&ws, tags)))
}

/// GET /api/v1/workstreams/:id/sessions
pub async fn list_workstream_sessions_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<SessionListResponse>, ServerError> {
    let mgr = get_manager(&state)?;

    let sessions = mgr.list_sessions(&id)?;
    let sessions: Vec<_> = sessions
        .iter()
        .map(|s| SessionResponse {
            id: s.id.clone(),
            workstream_id: s.workstream_id.clone(),
            started_at: s.started_at.to_rfc3339(),
            ended_at: s.ended_at.map(|dt| dt.to_rfc3339()),
            is_active: s.ended_at.is_none(),
        })
        .collect();

    Ok(Json(SessionListResponse { sessions }))
}

/// POST /api/v1/workstreams/:id/messages
pub async fn send_message_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<SendMessageRequest>,
) -> Result<(StatusCode, Json<MessageResponse>), ServerError> {
    let mgr = get_manager(&state)?;

    let role = match req.role.as_deref().unwrap_or("user") {
        "user" => arawn_workstream::MessageRole::User,
        "assistant" => arawn_workstream::MessageRole::Assistant,
        "system" => arawn_workstream::MessageRole::System,
        "agent_push" => arawn_workstream::MessageRole::AgentPush,
        other => {
            return Err(ServerError::BadRequest(format!("Invalid role: {other}")));
        }
    };

    let msg = mgr
        .send_message(Some(&id), None, role, &req.content, req.metadata.as_deref())
        ?;

    Ok((StatusCode::CREATED, Json(to_message_response(&msg))))
}

/// GET /api/v1/workstreams/:id/messages
pub async fn list_messages_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<MessageQuery>,
) -> Result<Json<MessageListResponse>, ServerError> {
    let mgr = get_manager(&state)?;

    let result = if let Some(since_str) = &query.since {
        let since = since_str
            .parse::<DateTime<Utc>>()
            .map_err(|_| ServerError::BadRequest("Invalid 'since' timestamp. Use RFC 3339 format.".to_string()))?;
        mgr.get_messages_since(&id, since)
    } else {
        mgr.get_messages(&id)
    };

    let msgs = result?;
    let messages: Vec<_> = msgs.iter().map(to_message_response).collect();

    Ok(Json(MessageListResponse { messages }))
}

/// POST /api/v1/workstreams/:id/promote
pub async fn promote_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<PromoteRequest>,
) -> Result<(StatusCode, Json<WorkstreamResponse>), ServerError> {
    let mgr = get_manager(&state)?;

    if id != arawn_workstream::SCRATCH_ID {
        return Err(ServerError::BadRequest(
            "Only the scratch workstream can be promoted".to_string(),
        ));
    }

    let ws = mgr
        .promote_scratch(&req.title, &req.tags, req.default_model.as_deref())
        ?;

    let tags = mgr.get_tags(&ws.id).ok();
    Ok((StatusCode::CREATED, Json(to_workstream_response(&ws, tags))))
}
