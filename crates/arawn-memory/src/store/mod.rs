//! Memory store implementation using SQLite.
//!
//! Provides persistent storage for memories, sessions, and notes using rusqlite.
//! Integrates sqlite-vec for vector search and graphqlite for knowledge graphs.
//!
//! # Unified API
//!
//! The store provides both low-level operations (insert_memory, get_memory, etc.)
//! and a unified API for operations that span multiple subsystems:
//!
//! - `store()`: Store a memory with optional embedding and graph entities
//! - `get_with_context()`: Retrieve a memory with its graph relationships
//! - `delete_cascade()`: Remove a memory and all associated data
//! - `update_indexed()`: Update a memory and re-index its embedding/entities

mod graph_ops;
mod memory_ops;
mod note_ops;
pub mod query;
mod recall;
mod session_ops;
mod unified_ops;
mod vector_ops;

use std::path::Path;
use std::sync::Mutex;

use rusqlite::{Connection, OpenFlags, params};
use tracing::{debug, info};

use crate::error::{MemoryError, Result};
use crate::graph::{GraphStore, RelationshipType};

pub use query::{
    MemoryWithContext, RecallMatch, RecallQuery, RecallResult, ReindexDryRun, ReindexReport,
    RelatedEntity, StoreFactResult, StoreStats, TimeRange,
};

// ─────────────────────────────────────────────────────────────────────────────
// Schema Version
// ─────────────────────────────────────────────────────────────────────────────

/// Current schema version for migrations.
const SCHEMA_VERSION: i32 = 4;

// ─────────────────────────────────────────────────────────────────────────────
// Memory Store
// ─────────────────────────────────────────────────────────────────────────────

/// Memory store backed by SQLite.
///
/// Provides persistent storage for memories, sessions, and notes.
/// Uses WAL mode for better concurrent read performance.
///
/// Optionally integrates:
/// - **Vector search**: sqlite-vec for semantic similarity search
/// - **Knowledge graph**: graphqlite for entity relationships
pub struct MemoryStore {
    /// The SQLite connection (wrapped in Mutex for thread safety).
    pub(crate) conn: Mutex<Connection>,
    /// Optional graph store for knowledge graph operations.
    pub(crate) graph: Option<GraphStore>,
    /// Whether vectors have been initialized.
    pub(crate) vectors_initialized: Mutex<bool>,
    /// Whether stored embeddings are stale (dimension/provider mismatch).
    pub(crate) vectors_stale: Mutex<bool>,
}

// SAFETY: All access to the inner Connection is through Mutex<Connection>,
// and GraphStore's internal connection is only accessed through &self methods
// that are serialized by the single-threaded access pattern.
unsafe impl Send for MemoryStore {}
unsafe impl Sync for MemoryStore {}

impl std::fmt::Debug for MemoryStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemoryStore")
            .field("has_graph", &self.graph.is_some())
            .field("vectors_initialized", &self.vectors_initialized)
            .field("vectors_stale", &self.vectors_stale)
            .finish_non_exhaustive()
    }
}

/// Options for storing a memory with the unified API.
#[derive(Debug, Clone, Default)]
pub struct StoreOptions {
    /// Optional embedding vector for semantic search.
    pub embedding: Option<Vec<f32>>,
    /// Entities to extract/link in the knowledge graph.
    pub entities: Vec<EntityLink>,
}

/// An entity link to create in the knowledge graph.
#[derive(Debug, Clone)]
pub struct EntityLink {
    /// Entity ID (will be created if it doesn't exist).
    pub entity_id: String,
    /// Entity label (e.g., "Person", "Concept", "Topic").
    pub label: String,
    /// Relationship type from memory to entity.
    pub relationship: RelationshipType,
    /// Optional properties for the entity.
    pub properties: Vec<(String, String)>,
}

impl EntityLink {
    /// Create a new entity link.
    pub fn new(
        entity_id: impl Into<String>,
        label: impl Into<String>,
        relationship: RelationshipType,
    ) -> Self {
        Self {
            entity_id: entity_id.into(),
            label: label.into(),
            relationship,
            properties: Vec::new(),
        }
    }

    /// Add a property to the entity.
    pub fn with_property(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.properties.push((key.into(), value.into()));
        self
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Initialization
// ─────────────────────────────────────────────────────────────────────────────

impl MemoryStore {
    /// Open or create a memory store at the given path.
    ///
    /// Creates the database file and initializes the schema if it doesn't exist.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|_| {
                    MemoryError::Database(rusqlite::Error::InvalidPath(path.to_path_buf()))
                })?;
            }
        }

        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_FULL_MUTEX,
        )?;

        let store = Self {
            conn: Mutex::new(conn),
            graph: None,
            vectors_initialized: Mutex::new(false),
            vectors_stale: Mutex::new(false),
        };
        store.initialize()?;

        info!("Memory store opened at {:?}", path);
        Ok(store)
    }

    /// Create an in-memory store (useful for testing).
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let store = Self {
            conn: Mutex::new(conn),
            graph: None,
            vectors_initialized: Mutex::new(false),
            vectors_stale: Mutex::new(false),
        };
        store.initialize()?;

        info!("In-memory store created");
        Ok(store)
    }

    /// Initialize knowledge graph capabilities.
    ///
    /// Creates an in-memory graph store. For persistent graphs,
    /// use `init_graph_at_path`.
    pub fn init_graph(&mut self) -> Result<()> {
        let graph = GraphStore::open_in_memory()?;
        self.graph = Some(graph);
        info!("Knowledge graph initialized (in-memory)");
        Ok(())
    }

    /// Initialize knowledge graph at a specific path.
    pub fn init_graph_at_path(&mut self, path: impl AsRef<Path>) -> Result<()> {
        let graph = GraphStore::open(path)?;
        self.graph = Some(graph);
        info!("Knowledge graph initialized");
        Ok(())
    }

    /// Check if the knowledge graph is initialized.
    pub fn has_graph(&self) -> bool {
        self.graph.is_some()
    }

    /// Check if vectors are initialized.
    pub fn has_vectors(&self) -> bool {
        *self.vectors_initialized.lock().unwrap()
    }

    /// Get a reference to the graph store (if initialized).
    pub fn graph(&self) -> Option<&GraphStore> {
        self.graph.as_ref()
    }

    /// Initialize the database with schema and pragmas.
    fn initialize(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        // Enable WAL mode for better concurrent reads
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;

        // Create schema
        self.create_schema(&conn)?;

        Ok(())
    }

    /// Create the database schema.
    fn create_schema(&self, conn: &Connection) -> Result<()> {
        // Check current schema version
        let current_version: i32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap_or(0);

        if current_version >= SCHEMA_VERSION {
            debug!("Schema up to date (version {})", current_version);
            return Ok(());
        }

        info!(
            "Migrating schema from version {} to {}",
            current_version, SCHEMA_VERSION
        );

        // Create tables
        conn.execute_batch(
            r#"
            -- Memories table: stores all types of memory content
            CREATE TABLE IF NOT EXISTS memories (
                id TEXT PRIMARY KEY,
                session_id TEXT,
                content_type TEXT NOT NULL,
                content TEXT NOT NULL,
                metadata TEXT NOT NULL DEFAULT '{}',
                created_at TEXT NOT NULL,
                accessed_at TEXT NOT NULL,
                access_count INTEGER NOT NULL DEFAULT 0
            );

            -- Index for content type queries
            CREATE INDEX IF NOT EXISTS idx_memories_content_type
                ON memories(content_type);

            -- Index for time-based queries
            CREATE INDEX IF NOT EXISTS idx_memories_created_at
                ON memories(created_at);

            -- NOTE: session_id index is created in migrate_v3 to handle
            -- databases that don't have the column yet

            -- Sessions table: conversation sessions
            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                title TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            -- Index for session ordering
            CREATE INDEX IF NOT EXISTS idx_sessions_updated_at
                ON sessions(updated_at);

            -- Notes table: user and agent notes
            CREATE TABLE IF NOT EXISTS notes (
                id TEXT PRIMARY KEY,
                title TEXT,
                content TEXT NOT NULL,
                tags TEXT NOT NULL DEFAULT '[]',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            -- Index for note search
            CREATE INDEX IF NOT EXISTS idx_notes_updated_at
                ON notes(updated_at);

            -- Schema metadata
            CREATE TABLE IF NOT EXISTS meta (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            "#,
        )?;

        // Run migrations
        if current_version < 2 {
            self.migrate_v2(conn)?;
        }
        if current_version < 3 {
            self.migrate_v3(conn)?;
        }
        if current_version < 4 {
            self.migrate_v4(conn)?;
        }

        // Update schema version
        conn.pragma_update(None, "user_version", SCHEMA_VERSION)?;

        info!("Schema created (version {})", SCHEMA_VERSION);
        Ok(())
    }

    /// Migration v2: Add confidence columns to memories table.
    fn migrate_v2(&self, conn: &Connection) -> Result<()> {
        info!("Running migration v2: adding confidence columns");
        conn.execute_batch(
            r#"
            ALTER TABLE memories ADD COLUMN confidence_source TEXT NOT NULL DEFAULT 'inferred';
            ALTER TABLE memories ADD COLUMN reinforcement_count INTEGER NOT NULL DEFAULT 0;
            ALTER TABLE memories ADD COLUMN superseded INTEGER NOT NULL DEFAULT 0;
            ALTER TABLE memories ADD COLUMN superseded_by TEXT;
            ALTER TABLE memories ADD COLUMN last_accessed TEXT;
            ALTER TABLE memories ADD COLUMN confidence_score REAL NOT NULL DEFAULT 1.0;
            "#,
        )?;

        // Backfill last_accessed from accessed_at for existing rows
        conn.execute(
            "UPDATE memories SET last_accessed = accessed_at WHERE last_accessed IS NULL",
            [],
        )?;

        info!("Migration v2 complete");
        Ok(())
    }

    /// Migration v3: Add session_id column to memories table and backfill from metadata JSON.
    fn migrate_v3(&self, conn: &Connection) -> Result<()> {
        info!("Running migration v3: adding session_id column to memories");

        // Check if session_id column already exists (fresh DBs have it in CREATE TABLE)
        let has_column: bool = conn
            .prepare("SELECT session_id FROM memories LIMIT 0")
            .is_ok();

        if !has_column {
            conn.execute_batch(
                r#"
                ALTER TABLE memories ADD COLUMN session_id TEXT;
                "#,
            )?;
        }

        conn.execute_batch(
            "CREATE INDEX IF NOT EXISTS idx_memories_session_id ON memories(session_id);",
        )?;

        // Backfill session_id from metadata JSON for existing rows
        conn.execute(
            r#"
            UPDATE memories
            SET session_id = json_extract(metadata, '$.session_id')
            WHERE json_extract(metadata, '$.session_id') IS NOT NULL
              AND session_id IS NULL
            "#,
            [],
        )?;

        info!("Migration v3 complete");
        Ok(())
    }

    /// Migration v4: Add citation column to memories table.
    fn migrate_v4(&self, conn: &Connection) -> Result<()> {
        info!("Running migration v4: adding citation column");

        // Check if citation column already exists
        let has_column: bool = conn
            .prepare("SELECT citation FROM memories LIMIT 0")
            .is_ok();

        if !has_column {
            conn.execute_batch(
                r#"
                ALTER TABLE memories ADD COLUMN citation TEXT;
                "#,
            )?;
        }

        // Create index on citation type for filtering
        conn.execute_batch(
            r#"
            CREATE INDEX IF NOT EXISTS idx_memories_citation_type
                ON memories(json_extract(citation, '$.type'));
            "#,
        )?;

        info!("Migration v4 complete");
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Transactions
// ─────────────────────────────────────────────────────────────────────────────

impl MemoryStore {
    /// Execute a function within a transaction.
    ///
    /// All operations within the closure are executed atomically.
    /// If the closure returns an error, all changes are rolled back.
    ///
    /// Note: This only provides transactions for SQLite operations.
    /// Graph operations are handled by graphqlite's internal transactions.
    ///
    /// # Example
    ///
    /// ```ignore
    /// store.with_transaction(|conn| {
    ///     // Multiple operations here are atomic
    ///     Ok(())
    /// })?;
    /// ```
    pub fn with_transaction<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Connection) -> Result<T>,
    {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;

        match f(&tx) {
            Ok(result) => {
                tx.commit()?;
                Ok(result)
            }
            Err(e) => {
                // Transaction is automatically rolled back when dropped
                Err(e)
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Utility Operations
// ─────────────────────────────────────────────────────────────────────────────

impl MemoryStore {
    /// Get or set a metadata value.
    pub fn get_meta(&self, key: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare("SELECT value FROM meta WHERE key = ?1")?;
        let mut rows = stmt.query(params![key])?;

        if let Some(row) = rows.next()? {
            let value: String = row.get(0)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    /// Set a metadata value.
    pub fn set_meta(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "INSERT OR REPLACE INTO meta (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;

        Ok(())
    }

    /// Get database statistics.
    pub fn stats(&self) -> Result<StoreStats> {
        let conn = self.conn.lock().unwrap();

        let memory_count: i64 =
            conn.query_row("SELECT COUNT(*) FROM memories", [], |row| row.get(0))?;
        let session_count: i64 =
            conn.query_row("SELECT COUNT(*) FROM sessions", [], |row| row.get(0))?;
        let note_count: i64 = conn.query_row("SELECT COUNT(*) FROM notes", [], |row| row.get(0))?;

        let embedding_count: usize = crate::vector::count_embeddings(&conn).unwrap_or(0);

        let embedding_provider: Option<String> = conn
            .query_row(
                "SELECT value FROM meta WHERE key = 'embedding.provider'",
                [],
                |row| row.get(0),
            )
            .ok();
        let embedding_dimensions: Option<usize> = conn
            .query_row(
                "SELECT value FROM meta WHERE key = 'embedding.dimensions'",
                [],
                |row| row.get::<_, String>(0),
            )
            .ok()
            .and_then(|s| s.parse().ok());

        Ok(StoreStats {
            memory_count: memory_count as usize,
            session_count: session_count as usize,
            note_count: note_count as usize,
            embedding_count,
            schema_version: SCHEMA_VERSION,
            embedding_provider,
            embedding_dimensions,
            vectors_stale: *self.vectors_stale.lock().unwrap(),
        })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// MemoryBackend Trait Implementation
// ─────────────────────────────────────────────────────────────────────────────

impl crate::backend::MemoryBackend for MemoryStore {
    fn insert(&self, memory: &crate::types::Memory) -> Result<()> {
        self.insert_memory(memory)
    }

    fn get(&self, id: crate::types::MemoryId) -> Result<Option<crate::types::Memory>> {
        self.get_memory(id)
    }

    fn update(&self, memory: &crate::types::Memory) -> Result<()> {
        self.update_memory(memory)
    }

    fn delete(&self, id: crate::types::MemoryId) -> Result<bool> {
        self.delete_memory(id)
    }

    fn list(
        &self,
        content_type: Option<crate::types::ContentType>,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<crate::types::Memory>> {
        self.list_memories(content_type, limit, offset)
    }

    fn count(&self, content_type: Option<crate::types::ContentType>) -> Result<usize> {
        self.count_memories(content_type)
    }

    fn touch(&self, id: crate::types::MemoryId) -> Result<()> {
        self.touch_memory(id)
    }
}

impl crate::backend::MemoryBackendExt for MemoryStore {
    fn find_contradictions(
        &self,
        subject: &str,
        predicate: &str,
    ) -> Result<Vec<crate::types::Memory>> {
        MemoryStore::find_contradictions(self, subject, predicate)
    }

    fn supersede(
        &self,
        old_id: crate::types::MemoryId,
        new_id: crate::types::MemoryId,
    ) -> Result<()> {
        MemoryStore::supersede(self, old_id, new_id)
    }

    fn reinforce(&self, id: crate::types::MemoryId) -> Result<()> {
        MemoryStore::reinforce(self, id)
    }

    fn update_last_accessed(&self, id: crate::types::MemoryId) -> Result<()> {
        MemoryStore::update_last_accessed(self, id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ContentType, Memory, Note, Session};

    fn create_test_store() -> MemoryStore {
        MemoryStore::open_in_memory().unwrap()
    }

    #[test]
    fn test_open_in_memory() {
        let store = create_test_store();
        let stats = store.stats().unwrap();
        assert_eq!(stats.memory_count, 0);
        assert_eq!(stats.schema_version, SCHEMA_VERSION);
    }

    #[test]
    fn test_meta_operations() {
        let store = create_test_store();

        assert!(store.get_meta("test_key").unwrap().is_none());

        store.set_meta("test_key", "test_value").unwrap();
        assert_eq!(
            store.get_meta("test_key").unwrap(),
            Some("test_value".to_string())
        );

        store.set_meta("test_key", "new_value").unwrap();
        assert_eq!(
            store.get_meta("test_key").unwrap(),
            Some("new_value".to_string())
        );
    }

    #[test]
    fn test_store_stats() {
        let store = create_test_store();

        store
            .insert_memory(&Memory::new(ContentType::Note, "Note"))
            .unwrap();
        store
            .insert_memory(&Memory::new(ContentType::Fact, "Fact"))
            .unwrap();
        store.insert_session(&Session::new()).unwrap();
        store.insert_note(&Note::new("Note")).unwrap();

        let stats = store.stats().unwrap();
        assert_eq!(stats.memory_count, 2);
        assert_eq!(stats.session_count, 1);
        assert_eq!(stats.note_count, 1);
    }

    #[test]
    fn test_with_transaction() {
        let store = create_test_store();

        let result = store.with_transaction(|conn| {
            conn.execute(
                "INSERT INTO meta (key, value) VALUES (?1, ?2)",
                params!["tx_key", "tx_value"],
            )?;
            Ok("success")
        });

        assert_eq!(result.unwrap(), "success");
        assert_eq!(
            store.get_meta("tx_key").unwrap(),
            Some("tx_value".to_string())
        );
    }
}
