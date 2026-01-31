//! Query types for memory recall and search.

use chrono::{DateTime, Utc};

use crate::graph::RelationshipType;
use crate::types::{ContentType, Memory, MemoryId, Staleness};

// ─────────────────────────────────────────────────────────────────────────────
// Time Range
// ─────────────────────────────────────────────────────────────────────────────

/// Time range filter for recall queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TimeRange {
    /// Only today's memories.
    Today,
    /// Last 7 days.
    Week,
    /// Last 30 days.
    Month,
    /// All time (no filter).
    #[default]
    All,
}

impl TimeRange {
    /// Get the cutoff datetime for this time range.
    pub fn cutoff(&self) -> Option<DateTime<Utc>> {
        use chrono::Duration;
        match self {
            Self::Today => Some(Utc::now() - Duration::days(1)),
            Self::Week => Some(Utc::now() - Duration::days(7)),
            Self::Month => Some(Utc::now() - Duration::days(30)),
            Self::All => None,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Recall Query
// ─────────────────────────────────────────────────────────────────────────────

/// Query parameters for combined recall.
#[derive(Debug, Clone)]
pub struct RecallQuery {
    /// Query embedding for vector similarity search.
    pub embedding: Vec<f32>,
    /// Maximum number of results to return.
    pub limit: usize,
    /// Time range filter.
    pub time_range: TimeRange,
    /// Content type filters (empty = all types).
    pub content_types: Vec<ContentType>,
    /// Weight for vector similarity (0.0-1.0, rest goes to graph).
    pub vector_weight: f32,
    /// Whether to include graph context in results.
    pub include_graph_context: bool,
    /// Minimum score threshold (0.0-1.0). Results below this are excluded.
    pub min_score: Option<f32>,
    /// Optional session ID filter. When set, only memories from this session are returned.
    pub session_id: Option<String>,
}

impl RecallQuery {
    /// Create a new recall query with an embedding.
    pub fn new(embedding: Vec<f32>) -> Self {
        Self {
            embedding,
            limit: 10,
            time_range: TimeRange::All,
            content_types: Vec::new(),
            vector_weight: 0.7,
            include_graph_context: true,
            min_score: None,
            session_id: None,
        }
    }

    /// Set the maximum number of results.
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    /// Set the time range filter.
    pub fn with_time_range(mut self, range: TimeRange) -> Self {
        self.time_range = range;
        self
    }

    /// Add a content type filter.
    pub fn with_content_type(mut self, ct: ContentType) -> Self {
        self.content_types.push(ct);
        self
    }

    /// Set the vector weight for blending (0.0-1.0).
    pub fn with_vector_weight(mut self, weight: f32) -> Self {
        self.vector_weight = weight.clamp(0.0, 1.0);
        self
    }

    /// Set whether to include graph context.
    pub fn with_graph_context(mut self, include: bool) -> Self {
        self.include_graph_context = include;
        self
    }

    /// Set the minimum score threshold (0.0-1.0).
    pub fn with_min_score(mut self, score: f32) -> Self {
        self.min_score = Some(score.clamp(0.0, 1.0));
        self
    }

    /// Filter results to a specific session.
    pub fn with_session(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Recall Results
// ─────────────────────────────────────────────────────────────────────────────

/// A single match in recall results.
#[derive(Debug, Clone)]
pub struct RecallMatch {
    /// The matched memory.
    pub memory: Memory,
    /// Vector similarity distance (lower = more similar).
    pub distance: f32,
    /// Vector similarity score (0.0-1.0, higher = more similar).
    pub similarity_score: f32,
    /// Confidence score from the memory's confidence metadata.
    pub confidence_score: f32,
    /// Combined score (higher = better match).
    pub score: f32,
    /// Related entities from the knowledge graph.
    pub related_entities: Vec<String>,
    /// Staleness status of the citation source (if citation present).
    pub staleness: Staleness,
}

/// Result of a recall query.
#[derive(Debug, Clone)]
pub struct RecallResult {
    /// Matched memories ordered by score.
    pub matches: Vec<RecallMatch>,
    /// All unique entities found across matches.
    pub entities: Vec<String>,
    /// Number of memories searched.
    pub searched_count: usize,
    /// Query time in milliseconds.
    pub query_time_ms: u64,
}

// ─────────────────────────────────────────────────────────────────────────────
// Context Types
// ─────────────────────────────────────────────────────────────────────────────

/// A memory with its graph context.
#[derive(Debug, Clone)]
pub struct MemoryWithContext {
    /// The memory itself.
    pub memory: Memory,
    /// Related entities from the knowledge graph.
    pub related_entities: Vec<RelatedEntity>,
    /// Whether the memory has an embedding stored.
    pub has_embedding: bool,
}

/// An entity related to a memory.
#[derive(Debug, Clone)]
pub struct RelatedEntity {
    /// Entity ID.
    pub entity_id: String,
    /// Relationship type.
    pub relationship: RelationshipType,
}

// ─────────────────────────────────────────────────────────────────────────────
// Stats Types
// ─────────────────────────────────────────────────────────────────────────────

/// Statistics about the memory store.
#[derive(Debug, Clone)]
pub struct StoreStats {
    /// Number of memories stored.
    pub memory_count: usize,
    /// Number of sessions stored.
    pub session_count: usize,
    /// Number of notes stored.
    pub note_count: usize,
    /// Number of embeddings stored (may differ from memory_count).
    pub embedding_count: usize,
    /// Current schema version.
    pub schema_version: i32,
    /// Configured embedding provider name (if vectors initialized).
    pub embedding_provider: Option<String>,
    /// Configured embedding dimensions (if vectors initialized).
    pub embedding_dimensions: Option<usize>,
    /// Whether embeddings are stale (dimension mismatch).
    pub vectors_stale: bool,
}

/// Report from a reindex operation.
#[derive(Debug, Clone)]
pub struct ReindexReport {
    /// Total memories found.
    pub total: usize,
    /// Memories successfully embedded.
    pub embedded: usize,
    /// Memories skipped (e.g., empty content).
    pub skipped: usize,
    /// Time elapsed.
    pub elapsed: std::time::Duration,
}

/// Dry-run result for a reindex operation.
#[derive(Debug, Clone)]
pub struct ReindexDryRun {
    /// Number of memories that would be re-embedded.
    pub memory_count: usize,
    /// Estimated token count (content chars / 4).
    pub estimated_tokens: usize,
}

/// Result of a `store_fact()` operation.
#[derive(Debug, Clone)]
pub enum StoreFactResult {
    /// A new memory was inserted (no existing match).
    Inserted,
    /// An existing memory with the same content was reinforced.
    Reinforced {
        /// The ID of the reinforced memory.
        existing_id: MemoryId,
    },
    /// Existing memories with different content were superseded.
    Superseded {
        /// IDs of the superseded memories.
        superseded_ids: Vec<MemoryId>,
    },
}
