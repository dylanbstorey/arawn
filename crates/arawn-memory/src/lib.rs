//! Memory and knowledge storage for Arawn.
//!
//! This crate provides persistent storage for the agent's memories, conversation
//! sessions, and notes. It uses SQLite for durability with planned support for:
//! - **sqlite-vec**: Vector similarity search for semantic recall
//! - **graphqlite**: Knowledge graph with Cypher queries for entity relationships
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────┐
//! │  MemoryStore                                                            │
//! │  - Single SQLite file with WAL mode                                     │
//! │  - Memories, sessions, notes tables                                     │
//! │  - Future: embeddings table + vector search                             │
//! │  - Future: entities/relationships + graph queries                       │
//! └─────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Usage
//!
//! ```no_run
//! use arawn_memory::{MemoryStore, Memory, ContentType, Note, Session};
//!
//! // Open or create a memory store
//! let store = MemoryStore::open("~/.arawn/memory.db")?;
//!
//! // Store a memory
//! let memory = Memory::new(ContentType::Note, "Important finding about X");
//! store.insert_memory(&memory)?;
//!
//! // Create a session
//! let session = Session::new().with_title("Research on topic Y");
//! store.insert_session(&session)?;
//!
//! // Create a note
//! let note = Note::new("Remember to follow up on Z")
//!     .with_title("Follow-up")
//!     .with_tag("todo");
//! store.insert_note(&note)?;
//!
//! // Search notes
//! let results = store.search_notes("follow up", 10)?;
//! # Ok::<(), arawn_memory::MemoryError>(())
//! ```
//!
//! # Memory Types
//!
//! The store supports different types of memories:
//! - `UserMessage`: User input from conversations
//! - `AssistantMessage`: Agent responses
//! - `ToolUse`: Tool invocations and results
//! - `FileContent`: Indexed file contents
//! - `Note`: User or agent notes
//! - `Fact`: Extracted facts or knowledge
//! - `WebContent`: Fetched web page content

pub mod backend;
pub mod error;
pub mod graph;
pub mod store;
pub mod types;
pub mod validation;
pub mod vector;

// Re-export backend traits
pub use backend::{MemoryBackend, MemoryBackendExt, MockMemoryBackend};

// Re-export error types
pub use error::{MemoryError, Result};

// Re-export store
pub use store::{
    EntityLink,
    MemoryStore,
    MemoryWithContext,
    RecallMatch,
    RecallQuery,
    RecallResult,
    ReindexDryRun,
    ReindexReport,
    RelatedEntity,
    // Unified API types
    StoreFactResult,
    StoreOptions,
    StoreStats,
    // Recall types
    TimeRange,
};

// Re-export types
pub use types::{
    // Citation types
    Citation,
    // Memory types
    ConfidenceParams,
    ConfidenceSource,
    ContentType,
    // Entity types (for future knowledge graph)
    Entity,
    EntityId,
    Memory,
    MemoryConfidence,
    MemoryId,
    Metadata,
    // Note types
    Note,
    NoteId,
    // Session types
    Session,
    SessionId,
    Staleness,
};

// Re-export vector search
pub use vector::{
    DEFAULT_EMBEDDING_DIMS, SimilarityResult, check_vector_extension, count_embeddings,
    create_vector_table, delete_embedding, drop_vector_table, has_embedding, init_vector_extension,
    search_similar, search_similar_filtered, store_embedding,
};

// Re-export graph/knowledge store
pub use graph::{
    GraphNode, GraphRelationship, GraphStats, GraphStore, QueryResult, RelationshipType,
};

// Re-export validation
pub use validation::{
    ValidationError, validate_confidence_score, validate_embedding, validate_embedding_result,
    validate_memory, validate_memory_content, validate_memory_result, validate_session_id,
    validate_session_id_result,
};
