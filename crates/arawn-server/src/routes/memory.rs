//! Memory and notes endpoints.
//!
//! These endpoints provide access to the memory search and notes functionality.
//! Note: Full memory integration (vector search, graph queries) requires
//! arawn-memory to be added to the server state. Currently notes are stored
//! in-memory only.

use axum::{
    Extension, Json,
    extract::{Query, State},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::auth::Identity;
use crate::error::ServerError;
use crate::state::AppState;

// ─────────────────────────────────────────────────────────────────────────────
// Types
// ─────────────────────────────────────────────────────────────────────────────

/// In-memory note storage (temporary until full memory integration).
pub type NoteStore = Arc<RwLock<HashMap<String, Note>>>;

/// A simple note.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    /// Note ID.
    pub id: String,
    /// Note content.
    pub content: String,
    /// Tags for categorization.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Creation time.
    pub created_at: String,
}

/// Request to create a note.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateNoteRequest {
    /// Note content.
    pub content: String,
    /// Optional tags.
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Response with created note.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNoteResponse {
    /// The created note.
    pub note: Note,
}

/// Query params for listing notes.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ListNotesQuery {
    /// Filter by tag.
    pub tag: Option<String>,
    /// Maximum number of notes to return.
    pub limit: Option<usize>,
}

/// Response for listing notes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListNotesResponse {
    /// List of notes.
    pub notes: Vec<Note>,
    /// Total count (may be more than returned if limit applied).
    pub total: usize,
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub citation: Option<serde_json::Value>,
}

/// Response for memory search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySearchResponse {
    /// Search results.
    pub results: Vec<MemorySearchResult>,
    /// Query that was executed.
    pub query: String,
    /// Total results returned.
    pub count: usize,
}

// ─────────────────────────────────────────────────────────────────────────────
// State Extension
// ─────────────────────────────────────────────────────────────────────────────

/// Get the note store from the app state, creating it if needed.
///
/// This uses a static to provide a simple in-memory note store.
/// In a real implementation, this would use arawn-memory.
fn get_note_store() -> NoteStore {
    use std::sync::OnceLock;
    static NOTE_STORE: OnceLock<NoteStore> = OnceLock::new();
    NOTE_STORE
        .get_or_init(|| Arc::new(RwLock::new(HashMap::new())))
        .clone()
}

// ─────────────────────────────────────────────────────────────────────────────
// Handlers
// ─────────────────────────────────────────────────────────────────────────────

/// POST /api/v1/notes - Create a new note.
pub async fn create_note_handler(
    State(_state): State<AppState>,
    Extension(_identity): Extension<Identity>,
    Json(request): Json<CreateNoteRequest>,
) -> Result<Json<CreateNoteResponse>, ServerError> {
    let note_store = get_note_store();

    let note = Note {
        id: uuid::Uuid::new_v4().to_string(),
        content: request.content,
        tags: request.tags,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    let mut store = note_store.write().await;
    store.insert(note.id.clone(), note.clone());

    Ok(Json(CreateNoteResponse { note }))
}

/// GET /api/v1/notes - List notes.
pub async fn list_notes_handler(
    State(_state): State<AppState>,
    Extension(_identity): Extension<Identity>,
    Query(query): Query<ListNotesQuery>,
) -> Result<Json<ListNotesResponse>, ServerError> {
    let note_store = get_note_store();
    let store = note_store.read().await;

    let mut notes: Vec<Note> = store.values().cloned().collect();

    // Filter by tag if specified
    if let Some(ref tag) = query.tag {
        notes.retain(|n| n.tags.contains(tag));
    }

    // Sort by created_at descending
    notes.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    let total = notes.len();

    // Apply limit
    if let Some(limit) = query.limit {
        notes.truncate(limit);
    }

    Ok(Json(ListNotesResponse { notes, total }))
}

/// GET /api/v1/memory/search - Search memories.
///
/// Searches the MemoryStore (text match on indexed facts, summaries, etc.)
/// and falls back to in-memory notes if no indexer is configured.
pub async fn memory_search_handler(
    State(state): State<AppState>,
    Extension(_identity): Extension<Identity>,
    Query(query): Query<MemorySearchQuery>,
) -> Result<Json<MemorySearchResponse>, ServerError> {
    let mut results: Vec<MemorySearchResult> = Vec::new();

    // Search the real MemoryStore if indexer is available
    if let Some(ref indexer) = state.indexer {
        let store = indexer.store();
        match store.search_memories(&query.q, query.limit) {
            Ok(memories) => {
                for memory in memories {
                    // Apply session filter if provided
                    if let Some(ref sid) = query.session_id {
                        if memory.session_id.as_deref() != Some(sid.as_str()) {
                            continue;
                        }
                    }
                    // Serialize citation to JSON Value if present
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
            }
        }
    }

    // Also search in-memory notes
    let note_store = get_note_store();
    let store = note_store.read().await;
    let query_lower = query.q.to_lowercase();

    let remaining = query.limit.saturating_sub(results.len());
    for note in store.values() {
        if remaining == 0 {
            break;
        }
        if note.content.to_lowercase().contains(&query_lower) {
            results.push(MemorySearchResult {
                id: note.id.clone(),
                content_type: "note".to_string(),
                content: note.content.clone(),
                session_id: None,
                score: 1.0,
                source: "notes".to_string(),
                citation: None,
            });
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
    }))
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
            .route("/notes", post(create_note_handler).get(list_notes_handler))
            .route("/memory/search", get(memory_search_handler))
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

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let result: CreateNoteResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(result.note.content, "Test note");
        assert_eq!(result.note.tags, vec!["test", "example"]);
    }

    #[tokio::test]
    async fn test_list_notes() {
        let state = create_test_state();

        // Create some notes directly in the store
        let note_store = get_note_store();
        {
            let mut store = note_store.write().await;
            store.insert(
                "1".to_string(),
                Note {
                    id: "1".to_string(),
                    content: "First note".to_string(),
                    tags: vec!["test".to_string()],
                    created_at: "2024-01-01T00:00:00Z".to_string(),
                },
            );
            store.insert(
                "2".to_string(),
                Note {
                    id: "2".to_string(),
                    content: "Second note".to_string(),
                    tags: vec!["other".to_string()],
                    created_at: "2024-01-02T00:00:00Z".to_string(),
                },
            );
        }

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
        assert!(result.total >= 2);
    }

    #[tokio::test]
    async fn test_list_notes_with_tag_filter() {
        let state = create_test_state();
        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/notes?tag=test")
                    .header("Authorization", "Bearer test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
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
        use arawn_memory::MemoryStore;

        let state = create_test_state();

        // Create a MemoryStore with some facts
        let store = MemoryStore::open_in_memory().unwrap();
        let memory = arawn_memory::types::Memory::new(
            arawn_memory::types::ContentType::Fact,
            "Rust is a systems programming language",
        );
        store.insert_memory(&memory).unwrap();

        // Build an indexer with the store so the handler can find it
        let indexer = arawn_agent::SessionIndexer::with_backend(
            std::sync::Arc::new(store),
            std::sync::Arc::new(arawn_llm::MockBackend::with_text("{}")),
            None,
            arawn_agent::IndexerConfig::default(),
        );
        let state = state.with_indexer(indexer);
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
        assert_eq!(result.count, 1);
        assert_eq!(result.results[0].content_type, "fact");
        assert!(result.results[0].content.contains("Rust"));
        assert_eq!(result.results[0].source, "memory_store");
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
}
