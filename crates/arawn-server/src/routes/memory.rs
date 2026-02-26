//! Memory and notes endpoints.
//!
//! These endpoints provide access to the memory search and notes functionality,
//! backed by `arawn-memory::MemoryStore` for persistent storage.

use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use utoipa::ToSchema;

use arawn_memory::MemoryStore;

use super::pagination::PaginationParams;
use crate::auth::Identity;
use crate::error::ServerError;
use crate::state::AppState;

// ─────────────────────────────────────────────────────────────────────────────
// Types
// ─────────────────────────────────────────────────────────────────────────────

/// A note (API representation).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Note {
    /// Note ID.
    pub id: String,
    /// Optional title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Note content.
    pub content: String,
    /// Tags for categorization.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Creation time (RFC 3339).
    pub created_at: String,
    /// Last update time (RFC 3339).
    pub updated_at: String,
}

/// Request to create a note.
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreateNoteRequest {
    /// Note content.
    pub content: String,
    /// Optional title.
    #[serde(default)]
    pub title: Option<String>,
    /// Optional tags.
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Query params for listing notes.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ListNotesQuery {
    /// Filter by tag.
    pub tag: Option<String>,
}

/// Request to update a note.
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct UpdateNoteRequest {
    /// New title for the note.
    #[serde(default)]
    pub title: Option<String>,
    /// New content for the note.
    #[serde(default)]
    pub content: Option<String>,
    /// New tags for the note (replaces existing tags).
    #[serde(default)]
    pub tags: Option<Vec<String>>,
}

/// Response for listing notes.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ListNotesResponse {
    /// List of notes.
    pub notes: Vec<Note>,
    /// Total number of notes across all pages.
    pub total: usize,
    /// Maximum items per page (as requested).
    pub limit: usize,
    /// Offset from the start of the collection.
    pub offset: usize,
}

/// Query params for memory search.
#[derive(Debug, Clone, Deserialize)]
pub struct MemorySearchQuery {
    /// Search query text.
    pub q: String,
    /// Maximum results.
    #[serde(default = "default_limit")]
    pub limit: usize,
    /// Optional session ID to scope the search.
    pub session_id: Option<String>,
}

fn default_limit() -> usize {
    10
}

/// Memory search result item.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MemorySearchResult {
    /// Result ID.
    pub id: String,
    /// Content type.
    pub content_type: String,
    /// Content text.
    pub content: String,
    /// Session the memory belongs to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Relevance score (0.0 - 1.0).
    pub score: f32,
    /// Where the result came from (e.g., "text" or "note").
    pub source: String,
    /// Citation metadata for provenance tracking.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Object)]
    pub citation: Option<serde_json::Value>,
}

/// Response for memory search.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MemorySearchResponse {
    /// Search results ordered by relevance (highest score first).
    pub results: Vec<MemorySearchResult>,
    /// The query that was executed.
    pub query: String,
    /// Number of results returned (equal to `results.len()`).
    pub count: usize,
    /// When `true`, the search fell back to text-only matching because the
    /// vector/embedding search failed. Results may be less relevant.
    /// Only present when `true`.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub degraded: bool,
}

/// Request to store a memory directly.
///
/// Requires the memory/indexing feature to be enabled on the server
/// (returns 503 otherwise). Memories are persisted in the vector store
/// and become searchable via `GET /api/v1/memory/search`.
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct StoreMemoryRequest {
    /// The memory content (plain text).
    pub content: String,
    /// Content type. Defaults to `"fact"`.
    ///
    /// Valid values: `"fact"`, `"summary"`, `"insight"`, `"preference"`,
    /// `"procedure"`, `"entity"`.
    #[serde(default = "default_content_type")]
    #[schema(example = "fact")]
    pub content_type: String,
    /// Optional session ID to associate with this memory.
    /// When set, the memory can be filtered by session in search results.
    #[serde(default)]
    pub session_id: Option<String>,
    /// Optional metadata as key-value pairs. Stored alongside the memory.
    #[serde(default)]
    #[schema(value_type = Object)]
    pub metadata: HashMap<String, serde_json::Value>,
    /// Confidence score (0.0 to 1.0). Defaults to 0.8.
    /// Higher scores rank higher in search results.
    #[serde(default = "default_confidence")]
    pub confidence: f32,
}

fn default_content_type() -> String {
    "fact".to_string()
}

fn default_confidence() -> f32 {
    0.8
}

/// Response after storing a memory.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StoreMemoryResponse {
    /// The stored memory ID.
    pub id: String,
    /// Content type.
    pub content_type: String,
    /// Confirmation message.
    pub message: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Get the memory store from app state, returning 503 if not configured.
fn require_memory_store(state: &AppState) -> Result<&Arc<MemoryStore>, ServerError> {
    state
        .memory_store()
        .ok_or_else(|| ServerError::ServiceUnavailable("Memory storage not configured".to_string()))
}

/// Convert an `arawn_memory::Note` to the API `Note` type.
fn to_api_note(note: arawn_memory::types::Note) -> Note {
    Note {
        id: note.id.to_string(),
        title: note.title,
        content: note.content,
        tags: note.tags,
        created_at: note.created_at.to_rfc3339(),
        updated_at: note.updated_at.to_rfc3339(),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Handlers
// ─────────────────────────────────────────────────────────────────────────────

/// POST /api/v1/notes - Create a new note.
#[utoipa::path(
    post,
    path = "/api/v1/notes",
    request_body = CreateNoteRequest,
    responses(
        (status = 201, description = "Note created", body = Note),
        (status = 401, description = "Unauthorized"),
        (status = 503, description = "Memory storage not configured"),
    ),
    security(("bearer_auth" = [])),
    tag = "memory"
)]
pub async fn create_note_handler(
    State(state): State<AppState>,
    Extension(_identity): Extension<Identity>,
    Json(request): Json<CreateNoteRequest>,
) -> Result<(StatusCode, Json<Note>), ServerError> {
    let store = require_memory_store(&state)?;

    let mut note = arawn_memory::types::Note::new(request.content);
    if let Some(title) = request.title {
        note = note.with_title(title);
    }
    for tag in request.tags {
        note = note.with_tag(tag);
    }

    store
        .insert_note(&note)
        .map_err(|e| ServerError::Internal(format!("Failed to create note: {}", e)))?;

    Ok((StatusCode::CREATED, Json(to_api_note(note))))
}

/// GET /api/v1/notes - List notes.
#[utoipa::path(
    get,
    path = "/api/v1/notes",
    params(
        ("tag" = Option<String>, Query, description = "Filter by tag"),
        PaginationParams,
    ),
    responses(
        (status = 200, description = "List of notes", body = ListNotesResponse),
        (status = 401, description = "Unauthorized"),
        (status = 503, description = "Memory storage not configured"),
    ),
    security(("bearer_auth" = [])),
    tag = "memory"
)]
pub async fn list_notes_handler(
    State(state): State<AppState>,
    Extension(_identity): Extension<Identity>,
    Query(query): Query<ListNotesQuery>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Json<ListNotesResponse>, ServerError> {
    let store = require_memory_store(&state)?;
    let limit = pagination.effective_limit();

    let (notes, total) = if let Some(ref tag) = query.tag {
        // Tag-filtered: fetch all matching, paginate in memory
        let all = store
            .list_notes_by_tag(tag, 10_000)
            .map_err(|e| ServerError::Internal(format!("Failed to list notes: {}", e)))?;
        let total = all.len();
        let offset = pagination.offset.min(total);
        let end = (offset + limit).min(total);
        (all[offset..end].to_vec(), total)
    } else {
        // Unfiltered: use store pagination directly
        let notes = store
            .list_notes(limit, pagination.offset)
            .map_err(|e| ServerError::Internal(format!("Failed to list notes: {}", e)))?;
        let total = store.stats().map(|s| s.note_count).unwrap_or(notes.len());
        (notes, total)
    };

    let api_notes: Vec<Note> = notes.into_iter().map(to_api_note).collect();

    Ok(Json(ListNotesResponse {
        notes: api_notes,
        total,
        limit,
        offset: pagination.offset,
    }))
}

/// GET /api/v1/notes/:id - Get a single note.
#[utoipa::path(
    get,
    path = "/api/v1/notes/{id}",
    params(
        ("id" = String, Path, description = "Note ID"),
    ),
    responses(
        (status = 200, description = "Note found", body = Note),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Note not found"),
        (status = 503, description = "Memory storage not configured"),
    ),
    security(("bearer_auth" = [])),
    tag = "memory"
)]
pub async fn get_note_handler(
    State(state): State<AppState>,
    Extension(_identity): Extension<Identity>,
    Path(note_id): Path<String>,
) -> Result<Json<Note>, ServerError> {
    let store = require_memory_store(&state)?;

    let id = arawn_memory::types::NoteId::parse(&note_id)
        .map_err(|_| ServerError::BadRequest(format!("Invalid note ID: {}", note_id)))?;

    let note = store
        .get_note(id)
        .map_err(|e| ServerError::Internal(format!("Failed to get note: {}", e)))?
        .ok_or_else(|| ServerError::NotFound(format!("Note {} not found", note_id)))?;

    Ok(Json(to_api_note(note)))
}

/// PUT /api/v1/notes/:id - Update a note.
#[utoipa::path(
    put,
    path = "/api/v1/notes/{id}",
    params(
        ("id" = String, Path, description = "Note ID"),
    ),
    request_body = UpdateNoteRequest,
    responses(
        (status = 200, description = "Note updated", body = Note),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Note not found"),
        (status = 503, description = "Memory storage not configured"),
    ),
    security(("bearer_auth" = [])),
    tag = "memory"
)]
pub async fn update_note_handler(
    State(state): State<AppState>,
    Extension(_identity): Extension<Identity>,
    Path(note_id): Path<String>,
    Json(request): Json<UpdateNoteRequest>,
) -> Result<Json<Note>, ServerError> {
    let store = require_memory_store(&state)?;

    let id = arawn_memory::types::NoteId::parse(&note_id)
        .map_err(|_| ServerError::BadRequest(format!("Invalid note ID: {}", note_id)))?;

    let mut note = store
        .get_note(id)
        .map_err(|e| ServerError::Internal(format!("Failed to get note: {}", e)))?
        .ok_or_else(|| ServerError::NotFound(format!("Note {} not found", note_id)))?;

    if let Some(title) = request.title {
        note.title = Some(title);
    }
    if let Some(content) = request.content {
        note.content = content;
    }
    if let Some(tags) = request.tags {
        note.tags = tags;
    }
    note.updated_at = chrono::Utc::now();

    store
        .update_note(&note)
        .map_err(|e| ServerError::Internal(format!("Failed to update note: {}", e)))?;

    Ok(Json(to_api_note(note)))
}

/// DELETE /api/v1/notes/:id - Delete a note.
#[utoipa::path(
    delete,
    path = "/api/v1/notes/{id}",
    params(
        ("id" = String, Path, description = "Note ID"),
    ),
    responses(
        (status = 204, description = "Note deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Note not found"),
        (status = 503, description = "Memory storage not configured"),
    ),
    security(("bearer_auth" = [])),
    tag = "memory"
)]
pub async fn delete_note_handler(
    State(state): State<AppState>,
    Extension(_identity): Extension<Identity>,
    Path(note_id): Path<String>,
) -> Result<StatusCode, ServerError> {
    let store = require_memory_store(&state)?;

    let id = arawn_memory::types::NoteId::parse(&note_id)
        .map_err(|_| ServerError::BadRequest(format!("Invalid note ID: {}", note_id)))?;

    let deleted = store
        .delete_note(id)
        .map_err(|e| ServerError::Internal(format!("Failed to delete note: {}", e)))?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ServerError::NotFound(format!("Note {} not found", note_id)))
    }
}

/// GET /api/v1/memory/search - Search memories.
///
/// Searches the MemoryStore (text match on indexed facts, summaries, etc.)
/// and supplements with matching notes. Results are sorted by relevance
/// score (highest first).
///
/// When the memory store search fails, sets `degraded: true` and returns
/// note-only results.
#[utoipa::path(
    get,
    path = "/api/v1/memory/search",
    params(
        ("q" = String, Query, description = "Search query text (case-insensitive for note matching)"),
        ("limit" = Option<usize>, Query, description = "Maximum results to return (default: 10)"),
        ("session_id" = Option<String>, Query, description = "Filter results to a specific session. Only applies to memory results, not notes."),
    ),
    responses(
        (status = 200, description = "Search results ordered by relevance. Check `degraded` to detect fallback mode.", body = MemorySearchResponse),
        (status = 401, description = "Unauthorized — missing or invalid bearer token"),
        (status = 503, description = "Memory storage not configured"),
    ),
    security(("bearer_auth" = [])),
    tag = "memory"
)]
pub async fn memory_search_handler(
    State(state): State<AppState>,
    Extension(_identity): Extension<Identity>,
    Query(query): Query<MemorySearchQuery>,
) -> Result<Json<MemorySearchResponse>, ServerError> {
    let store = require_memory_store(&state)?;

    let mut results: Vec<MemorySearchResult> = Vec::new();
    let mut degraded = false;

    // Search memories (facts, summaries, etc.)
    match store.search_memories(&query.q, query.limit) {
        Ok(memories) => {
            for memory in memories {
                // Apply session filter if provided
                if let Some(ref sid) = query.session_id {
                    if memory.session_id.as_deref() != Some(sid.as_str()) {
                        continue;
                    }
                }
                let citation = memory
                    .citation
                    .as_ref()
                    .and_then(|c| serde_json::to_value(c).ok());
                results.push(MemorySearchResult {
                    id: memory.id.to_string(),
                    content_type: memory.content_type.as_str().to_string(),
                    content: memory.content,
                    session_id: memory.session_id,
                    score: memory.confidence.score,
                    source: "memory_store".to_string(),
                    citation,
                });
            }
        }
        Err(e) => {
            tracing::warn!(error = %e, "MemoryStore search failed, falling back to notes");
            degraded = true;
        }
    }

    // Supplement with matching notes
    let remaining = query.limit.saturating_sub(results.len());
    if remaining > 0 {
        if let Ok(notes) = store.search_notes(&query.q, remaining) {
            for note in notes {
                results.push(MemorySearchResult {
                    id: note.id.to_string(),
                    content_type: "note".to_string(),
                    content: note.content,
                    session_id: None,
                    score: 1.0,
                    source: "notes".to_string(),
                    citation: None,
                });
            }
        }
    }

    // Sort by score descending
    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    results.truncate(query.limit);

    let count = results.len();
    Ok(Json(MemorySearchResponse {
        results,
        query: query.q,
        count,
        degraded,
    }))
}

/// POST /api/v1/memory - Store a memory directly.
#[utoipa::path(
    post,
    path = "/api/v1/memory",
    request_body = StoreMemoryRequest,
    responses(
        (status = 201, description = "Memory stored", body = StoreMemoryResponse),
        (status = 401, description = "Unauthorized"),
        (status = 503, description = "Memory storage not configured"),
    ),
    security(("bearer_auth" = [])),
    tag = "memory"
)]
pub async fn store_memory_handler(
    State(state): State<AppState>,
    Extension(_identity): Extension<Identity>,
    Json(request): Json<StoreMemoryRequest>,
) -> Result<(StatusCode, Json<StoreMemoryResponse>), ServerError> {
    let store = require_memory_store(&state)?;

    // Parse content type (default to Fact if invalid)
    let content_type = arawn_memory::types::ContentType::from_str(&request.content_type)
        .unwrap_or(arawn_memory::types::ContentType::Fact);
    let mut memory = arawn_memory::types::Memory::new(content_type, &request.content);

    // Set session ID if provided
    if let Some(ref session_id) = request.session_id {
        memory = memory.with_session(session_id);
    }

    // Set confidence
    memory.confidence.score = request.confidence;

    // Store the memory
    store
        .insert_memory(&memory)
        .map_err(|e| ServerError::Internal(format!("Failed to store memory: {}", e)))?;

    Ok((
        StatusCode::CREATED,
        Json(StoreMemoryResponse {
            id: memory.id.to_string(),
            content_type: request.content_type,
            message: "Memory stored successfully".to_string(),
        }),
    ))
}

/// DELETE /api/v1/memory/:id - Delete a memory.
#[utoipa::path(
    delete,
    path = "/api/v1/memory/{id}",
    params(
        ("id" = String, Path, description = "Memory ID (UUID)"),
    ),
    responses(
        (status = 204, description = "Memory deleted"),
        (status = 400, description = "Invalid memory ID"),
        (status = 401, description = "Unauthorized"),
        (status = 503, description = "Memory storage not configured"),
    ),
    security(("bearer_auth" = [])),
    tag = "memory"
)]
pub async fn delete_memory_handler(
    State(state): State<AppState>,
    Extension(_identity): Extension<Identity>,
    Path(memory_id): Path<String>,
) -> Result<StatusCode, ServerError> {
    let store = require_memory_store(&state)?;

    // Parse UUID and wrap in MemoryId
    let uuid = uuid::Uuid::parse_str(&memory_id)
        .map_err(|_| ServerError::BadRequest(format!("Invalid memory ID: {}", memory_id)))?;
    let id = arawn_memory::MemoryId(uuid);

    // Delete the memory
    store
        .delete_memory(id)
        .map_err(|e| ServerError::Internal(format!("Failed to delete memory: {}", e)))?;

    Ok(StatusCode::NO_CONTENT)
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
        routing::{delete, get, post, put},
    };
    use tower::ServiceExt;

    fn create_test_state() -> AppState {
        let backend = MockBackend::with_text("Test");
        let agent = Agent::builder()
            .with_backend(backend)
            .with_tools(ToolRegistry::new())
            .build()
            .unwrap();

        let store = Arc::new(MemoryStore::open_in_memory().unwrap());
        let mut state = AppState::new(agent, ServerConfig::new(Some("test-token".to_string())));
        state.services.memory_store = Some(store);
        state
    }

    fn create_test_router(state: AppState) -> Router {
        Router::new()
            .route("/notes", post(create_note_handler).get(list_notes_handler))
            .route(
                "/notes/{id}",
                get(get_note_handler)
                    .put(update_note_handler)
                    .delete(delete_note_handler),
            )
            .route("/memory/search", get(memory_search_handler))
            .route("/memory", post(store_memory_handler))
            .route("/memory/{id}", delete(delete_memory_handler))
            .layer(middleware::from_fn_with_state(
                state.clone(),
                auth_middleware,
            ))
            .with_state(state)
    }

    #[tokio::test]
    async fn test_create_note() {
        let state = create_test_state();
        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/notes")
                    .header("Authorization", "Bearer test-token")
                    .header("Content-Type", "application/json")
                    .body(Body::from(
                        r#"{"content": "Test note", "tags": ["test", "example"]}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let result: Note = serde_json::from_slice(&body).unwrap();
        assert_eq!(result.content, "Test note");
        assert_eq!(result.tags, vec!["test", "example"]);
        assert!(result.title.is_none());
    }

    #[tokio::test]
    async fn test_create_note_with_title() {
        let state = create_test_state();
        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/notes")
                    .header("Authorization", "Bearer test-token")
                    .header("Content-Type", "application/json")
                    .body(Body::from(
                        r#"{"content": "Test note", "title": "My Title", "tags": []}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let result: Note = serde_json::from_slice(&body).unwrap();
        assert_eq!(result.title, Some("My Title".to_string()));
    }

    #[tokio::test]
    async fn test_get_note() {
        let state = create_test_state();
        let store = state.memory_store().unwrap().clone();

        // Insert a note directly into the store
        let note = arawn_memory::types::Note::new("Direct note").with_title("Direct");
        store.insert_note(&note).unwrap();
        let note_id = note.id.to_string();

        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/notes/{}", note_id))
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
        let result: Note = serde_json::from_slice(&body).unwrap();
        assert_eq!(result.content, "Direct note");
        assert_eq!(result.title, Some("Direct".to_string()));
    }

    #[tokio::test]
    async fn test_get_note_not_found() {
        let state = create_test_state();
        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/notes/{}", uuid::Uuid::new_v4()))
                    .header("Authorization", "Bearer test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_update_note() {
        let state = create_test_state();
        let store = state.memory_store().unwrap().clone();

        let note = arawn_memory::types::Note::new("Original content");
        store.insert_note(&note).unwrap();
        let note_id = note.id.to_string();

        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(format!("/notes/{}", note_id))
                    .header("Authorization", "Bearer test-token")
                    .header("Content-Type", "application/json")
                    .body(Body::from(
                        r#"{"content": "Updated content", "title": "New Title"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let result: Note = serde_json::from_slice(&body).unwrap();
        assert_eq!(result.content, "Updated content");
        assert_eq!(result.title, Some("New Title".to_string()));
    }

    #[tokio::test]
    async fn test_delete_note() {
        let state = create_test_state();
        let store = state.memory_store().unwrap().clone();

        let note = arawn_memory::types::Note::new("To delete");
        store.insert_note(&note).unwrap();
        let note_id = note.id.to_string();

        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/notes/{}", note_id))
                    .header("Authorization", "Bearer test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);

        // Verify it's gone
        assert!(store.get_note(note.id).unwrap().is_none());
    }

    #[tokio::test]
    async fn test_list_notes() {
        let state = create_test_state();
        let store = state.memory_store().unwrap().clone();

        store
            .insert_note(&arawn_memory::types::Note::new("First note").with_tag("test"))
            .unwrap();
        store
            .insert_note(&arawn_memory::types::Note::new("Second note").with_tag("other"))
            .unwrap();

        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/notes")
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
        let result: ListNotesResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(result.total, 2);
        assert_eq!(result.notes.len(), 2);
    }

    #[tokio::test]
    async fn test_list_notes_with_tag_filter() {
        let state = create_test_state();
        let store = state.memory_store().unwrap().clone();

        store
            .insert_note(&arawn_memory::types::Note::new("Tagged").with_tag("rust"))
            .unwrap();
        store
            .insert_note(&arawn_memory::types::Note::new("Untagged"))
            .unwrap();

        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/notes?tag=rust")
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
        let result: ListNotesResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(result.total, 1);
        assert_eq!(result.notes[0].content, "Tagged");
    }

    #[tokio::test]
    async fn test_memory_search() {
        let state = create_test_state();
        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/memory/search?q=test")
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
        let result: MemorySearchResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(result.query, "test");
        assert_eq!(result.count, result.results.len());
    }

    #[tokio::test]
    async fn test_memory_search_with_store() {
        let state = create_test_state();
        let store = state.memory_store().unwrap().clone();

        let memory = arawn_memory::types::Memory::new(
            arawn_memory::types::ContentType::Fact,
            "Rust is a systems programming language",
        );
        store.insert_memory(&memory).unwrap();

        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/memory/search?q=Rust")
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
        let result: MemorySearchResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(result.query, "Rust");
        assert!(result.count >= 1);
        assert_eq!(result.results[0].content_type, "fact");
        assert!(result.results[0].content.contains("Rust"));
        assert_eq!(result.results[0].source, "memory_store");
    }

    #[tokio::test]
    async fn test_memory_search_includes_notes() {
        let state = create_test_state();
        let store = state.memory_store().unwrap().clone();

        // Insert a note that matches
        store
            .insert_note(&arawn_memory::types::Note::new(
                "Tokio is an async runtime for Rust",
            ))
            .unwrap();

        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/memory/search?q=Tokio")
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
        let result: MemorySearchResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(result.count, 1);
        assert_eq!(result.results[0].source, "notes");
    }

    #[tokio::test]
    async fn test_memory_search_requires_auth() {
        let state = create_test_state();
        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/memory/search?q=test")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_notes_require_memory_store() {
        // State WITHOUT memory store
        let backend = MockBackend::with_text("Test");
        let agent = Agent::builder()
            .with_backend(backend)
            .with_tools(ToolRegistry::new())
            .build()
            .unwrap();
        let state = AppState::new(agent, ServerConfig::new(Some("test-token".to_string())));
        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/notes")
                    .header("Authorization", "Bearer test-token")
                    .header("Content-Type", "application/json")
                    .body(Body::from(r#"{"content": "test"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_store_memory() {
        let state = create_test_state();
        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/memory")
                    .header("Authorization", "Bearer test-token")
                    .header("Content-Type", "application/json")
                    .body(Body::from(
                        r#"{"content": "Test memory", "content_type": "fact"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let result: StoreMemoryResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(result.content_type, "fact");
    }

    #[tokio::test]
    async fn test_delete_memory() {
        let state = create_test_state();
        let store = state.memory_store().unwrap().clone();

        let memory = arawn_memory::types::Memory::new(
            arawn_memory::types::ContentType::Fact,
            "To be deleted",
        );
        store.insert_memory(&memory).unwrap();
        let memory_id = memory.id.to_string();

        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/memory/{}", memory_id))
                    .header("Authorization", "Bearer test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }
}
