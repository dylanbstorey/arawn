---
id: arawn-memory-persistent-knowledge
level: initiative
title: "arawn-memory: Persistent Knowledge Store"
short_code: "ARAWN-I-0002"
created_at: 2026-01-28T01:37:24.044542+00:00
updated_at: 2026-01-28T05:06:00.408471+00:00
parent: ARAWN-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
strategy_id: NULL
initiative_id: arawn-memory-persistent-knowledge
---

# arawn-memory: Persistent Knowledge Store

## Context

The memory layer is the foundation for persistent context across sessions. Combines vector similarity search (for semantic recall) with a knowledge graph (for fact relationships and provenance tracking) in a single SQLite database file.

Leverages colliery-io crates:
- **sqlite-vec**: Vector similarity search extension
- **graphqlite**: Cypher query language over SQLite

## Goals & Non-Goals

**Goals:**
- Single SQLite file containing all persistent state
- Semantic search over conversations, notes, and research findings
- Knowledge graph for entity relationships and source tracking
- Unified query interface combining vector + graph lookups
- Efficient on edge hardware (memory-mapped, lazy loading)

**Non-Goals:**
- Multi-user/tenant support
- Real-time sync to cloud
- Full-text search (vector search covers this use case)

## Detailed Design

### Schema

```sql
-- Core content storage
CREATE TABLE memories (
    id INTEGER PRIMARY KEY,
    content TEXT NOT NULL,
    content_type TEXT NOT NULL,  -- 'conversation', 'note', 'finding', 'fact'
    source TEXT,                  -- where this came from
    created_at INTEGER NOT NULL,
    session_id TEXT,
    task_id TEXT
);

-- Vector embeddings (sqlite-vec)
CREATE VIRTUAL TABLE memory_vectors USING vec0(
    memory_id INTEGER PRIMARY KEY,
    embedding FLOAT[384]          -- all-MiniLM-L6-v2 dimension
);

-- Knowledge graph (graphqlite)
-- Nodes: entities extracted from memories (people, concepts, claims, sources)
-- Edges: relationships (supports, contradicts, caused_by, related_to, cited_in)

-- Sessions table
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    started_at INTEGER NOT NULL,
    last_active INTEGER NOT NULL,
    metadata TEXT                 -- JSON blob
);

-- Notes (structured capture)
CREATE TABLE notes (
    id INTEGER PRIMARY KEY,
    title TEXT,
    content TEXT NOT NULL,
    tags TEXT,                    -- JSON array
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
```

### API Surface

```rust
pub struct MemoryStore {
    db: Connection,
}

impl MemoryStore {
    // Lifecycle
    pub fn open(path: &Path) -> Result<Self>;
    pub fn open_in_memory() -> Result<Self>;
    
    // Storage
    pub fn store(&self, content: &str, content_type: ContentType, metadata: Metadata) -> Result<MemoryId>;
    pub fn store_with_embedding(&self, content: &str, embedding: &[f32], ...) -> Result<MemoryId>;
    
    // Vector search
    pub fn search_similar(&self, query_embedding: &[f32], limit: usize) -> Result<Vec<Memory>>;
    
    // Graph operations
    pub fn add_entity(&self, entity: Entity) -> Result<EntityId>;
    pub fn add_relationship(&self, from: EntityId, rel: Relationship, to: EntityId) -> Result<()>;
    pub fn query_graph(&self, cypher: &str) -> Result<GraphResult>;
    
    // Combined queries
    pub fn recall(&self, query: &str, query_embedding: &[f32], limit: usize) -> Result<RecallResult>;
    
    // Sessions
    pub fn get_or_create_session(&self, id: &str) -> Result<Session>;
    pub fn append_to_session(&self, session_id: &str, entry: SessionEntry) -> Result<()>;
    
    // Notes
    pub fn create_note(&self, note: NewNote) -> Result<NoteId>;
    pub fn search_notes(&self, query: &str) -> Result<Vec<Note>>;
}
```

### Query Patterns

```rust
// "What do I know about distributed consensus?"
let embedding = llm.embed("distributed consensus").await?;
let similar = memory.search_similar(&embedding, 10)?;

// "How does Raft relate to Paxos?"
let graph_result = memory.query_graph(r#"
    MATCH (a:Concept {name: 'Raft'})-[r]-(b:Concept {name: 'Paxos'})
    RETURN a, r, b
"#)?;

// "What sources support the claim that X?"
let sources = memory.query_graph(r#"
    MATCH (c:Claim {id: $claim_id})<-[:SUPPORTS]-(s:Source)
    RETURN s
"#)?;

// Combined: semantic search + graph context
let recall = memory.recall("consensus algorithms", &embedding, 10)?;
// Returns: similar memories + related graph entities
```

### Dependencies

```toml
[dependencies]
rusqlite = { version = "0.32", features = ["bundled"] }
sqlite-vec = "0.1"
graphqlite = "0.1"
serde = { version = "1", features = ["derive"] }
```

## Alternatives Considered

- **Separate databases (SQLite + LanceDB)**: Rejected - single file is simpler to backup/restore, fits edge constraints
- **Postgres with pgvector**: Rejected - requires running a server, not edge-friendly
- **In-memory only**: Rejected - persistence is core requirement

## Implementation Plan

1. Basic SQLite setup with rusqlite
2. Integrate sqlite-vec for vector storage and search
3. Integrate graphqlite for knowledge graph
4. Unified MemoryStore API
5. Session management
6. Notes storage
7. Combined recall queries