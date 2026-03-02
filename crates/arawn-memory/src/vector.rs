//! Vector storage and similarity search using sqlite-vec.
//!
//! This module provides vector embedding storage and semantic search capabilities
//! using the sqlite-vec SQLite extension.

use rusqlite::{Connection, params};
use tracing::{debug, info};
use zerocopy::IntoBytes;

use crate::error::Result;
use crate::types::MemoryId;

// ─────────────────────────────────────────────────────────────────────────────
// Constants
// ─────────────────────────────────────────────────────────────────────────────

/// Default embedding dimensions (MiniLM-L6-v2 produces 384-dim vectors).
pub const DEFAULT_EMBEDDING_DIMS: usize = 384;

// ─────────────────────────────────────────────────────────────────────────────
// Vector Store
// ─────────────────────────────────────────────────────────────────────────────

/// Initialize sqlite-vec extension for a connection.
///
/// This must be called before using any vector operations.
/// Note: When using `sqlite3_auto_extension`, it applies globally to all connections.
pub fn init_vector_extension() {
    use rusqlite::ffi::sqlite3_auto_extension;
    use sqlite_vec::sqlite3_vec_init;

    unsafe {
        #[allow(clippy::missing_transmute_annotations)]
        sqlite3_auto_extension(Some(std::mem::transmute(sqlite3_vec_init as *const ())));
    }
}

/// Check if sqlite-vec extension is loaded.
pub fn check_vector_extension(conn: &Connection) -> Result<String> {
    let version: String = conn.query_row("SELECT vec_version()", [], |row| row.get(0))?;
    Ok(version)
}

/// Create the vector embeddings table.
///
/// Creates a vec0 virtual table for storing memory embeddings.
pub fn create_vector_table(conn: &Connection, dims: usize) -> Result<()> {
    // Create the vec0 virtual table for memory embeddings
    let sql = format!(
        r#"
        CREATE VIRTUAL TABLE IF NOT EXISTS memory_embeddings USING vec0(
            memory_id TEXT PRIMARY KEY,
            embedding float[{dims}]
        )
        "#
    );

    conn.execute_batch(&sql)?;

    info!("Created memory_embeddings table with {} dimensions", dims);
    Ok(())
}

/// Drop the vector embeddings table.
///
/// Used during reindex to recreate with new dimensions.
pub fn drop_vector_table(conn: &Connection) -> Result<()> {
    conn.execute_batch("DROP TABLE IF EXISTS memory_embeddings")?;
    info!("Dropped memory_embeddings table");
    Ok(())
}

/// Store an embedding for a memory.
///
/// If an embedding already exists for this memory, it will be replaced.
pub fn store_embedding(conn: &Connection, memory_id: MemoryId, embedding: &[f32]) -> Result<()> {
    // vec0 doesn't support INSERT OR REPLACE, so delete first if exists
    conn.execute(
        "DELETE FROM memory_embeddings WHERE memory_id = ?1",
        params![memory_id.to_string()],
    )?;

    conn.execute(
        "INSERT INTO memory_embeddings (memory_id, embedding) VALUES (?1, ?2)",
        params![memory_id.to_string(), embedding.as_bytes()],
    )?;

    debug!("Stored embedding for memory {}", memory_id);
    Ok(())
}

/// Delete an embedding for a memory.
pub fn delete_embedding(conn: &Connection, memory_id: MemoryId) -> Result<bool> {
    let rows = conn.execute(
        "DELETE FROM memory_embeddings WHERE memory_id = ?1",
        params![memory_id.to_string()],
    )?;

    Ok(rows > 0)
}

/// Result of a similarity search.
#[derive(Debug, Clone)]
pub struct SimilarityResult {
    /// The memory ID.
    pub memory_id: MemoryId,
    /// Distance from the query vector (lower = more similar).
    pub distance: f32,
}

/// Search for memories similar to a query embedding.
///
/// Returns the top-k most similar memories ordered by distance (ascending).
pub fn search_similar(
    conn: &Connection,
    query_embedding: &[f32],
    limit: usize,
) -> Result<Vec<SimilarityResult>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT memory_id, distance
        FROM memory_embeddings
        WHERE embedding MATCH ?1
        ORDER BY distance
        LIMIT ?2
        "#,
    )?;

    let mut rows = stmt.query(params![query_embedding.as_bytes(), limit as i64])?;

    let mut results = Vec::new();
    while let Some(row) = rows.next()? {
        let memory_id_str: String = row.get(0)?;
        let distance: f32 = row.get(1)?;

        let memory_id = MemoryId::parse(&memory_id_str)?;
        results.push(SimilarityResult {
            memory_id,
            distance,
        });
    }

    debug!(
        "Found {} similar memories (limit: {})",
        results.len(),
        limit
    );
    Ok(results)
}

/// Search for memories similar to a query, filtered by memory IDs.
///
/// This is useful when you want to search within a subset of memories
/// (e.g., only memories from a specific session).
pub fn search_similar_filtered(
    conn: &Connection,
    query_embedding: &[f32],
    memory_ids: &[MemoryId],
    limit: usize,
) -> Result<Vec<SimilarityResult>> {
    if memory_ids.is_empty() {
        return Ok(Vec::new());
    }

    // Build the IN clause
    let placeholders: Vec<String> = (0..memory_ids.len())
        .map(|i| format!("?{}", i + 3))
        .collect();
    let in_clause = placeholders.join(", ");

    let sql = format!(
        r#"
        SELECT memory_id, distance
        FROM memory_embeddings
        WHERE embedding MATCH ?1
          AND memory_id IN ({})
        ORDER BY distance
        LIMIT ?2
        "#,
        in_clause
    );

    let mut stmt = conn.prepare(&sql)?;

    // Build params: query_embedding, limit, then memory_ids
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![
        Box::new(query_embedding.as_bytes().to_vec()),
        Box::new(limit as i64),
    ];
    for id in memory_ids {
        params_vec.push(Box::new(id.to_string()));
    }

    let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| b.as_ref()).collect();
    let mut rows = stmt.query(params_refs.as_slice())?;

    let mut results = Vec::new();
    while let Some(row) = rows.next()? {
        let memory_id_str: String = row.get(0)?;
        let distance: f32 = row.get(1)?;

        let memory_id = MemoryId::parse(&memory_id_str)?;
        results.push(SimilarityResult {
            memory_id,
            distance,
        });
    }

    Ok(results)
}

/// Get the count of stored embeddings.
pub fn count_embeddings(conn: &Connection) -> Result<usize> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM memory_embeddings", [], |row| {
        row.get(0)
    })?;
    Ok(count as usize)
}

/// Check if an embedding exists for a memory.
pub fn has_embedding(conn: &Connection, memory_id: MemoryId) -> Result<bool> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM memory_embeddings WHERE memory_id = ?1",
        params![memory_id.to_string()],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_connection() -> Connection {
        init_vector_extension();
        let conn = Connection::open_in_memory().unwrap();
        create_vector_table(&conn, 4).unwrap(); // Small dims for testing
        conn
    }

    #[test]
    fn test_vector_extension_loads() {
        init_vector_extension();
        let conn = Connection::open_in_memory().unwrap();
        let version = check_vector_extension(&conn).unwrap();
        assert!(!version.is_empty());
        println!("sqlite-vec version: {}", version);
    }

    #[test]
    fn test_create_vector_table() {
        let conn = create_test_connection();
        // Table should exist - try to query it
        let count = count_embeddings(&conn).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_store_and_retrieve_embedding() {
        let conn = create_test_connection();

        let memory_id = MemoryId::new();
        let embedding = vec![0.1f32, 0.2, 0.3, 0.4];

        store_embedding(&conn, memory_id, &embedding).unwrap();

        assert!(has_embedding(&conn, memory_id).unwrap());
        assert_eq!(count_embeddings(&conn).unwrap(), 1);
    }

    #[test]
    fn test_delete_embedding() {
        let conn = create_test_connection();

        let memory_id = MemoryId::new();
        let embedding = vec![0.1f32, 0.2, 0.3, 0.4];

        store_embedding(&conn, memory_id, &embedding).unwrap();
        assert!(has_embedding(&conn, memory_id).unwrap());

        let deleted = delete_embedding(&conn, memory_id).unwrap();
        assert!(deleted);
        assert!(!has_embedding(&conn, memory_id).unwrap());
    }

    #[test]
    fn test_similarity_search() {
        let conn = create_test_connection();

        // Store some embeddings
        let id1 = MemoryId::new();
        let id2 = MemoryId::new();
        let id3 = MemoryId::new();

        // Embedding 1: pointing in one direction
        store_embedding(&conn, id1, &[1.0f32, 0.0, 0.0, 0.0]).unwrap();
        // Embedding 2: similar to 1
        store_embedding(&conn, id2, &[0.9f32, 0.1, 0.0, 0.0]).unwrap();
        // Embedding 3: different direction
        store_embedding(&conn, id3, &[0.0f32, 0.0, 1.0, 0.0]).unwrap();

        // Search for something similar to embedding 1
        let query = vec![1.0f32, 0.0, 0.0, 0.0];
        let results = search_similar(&conn, &query, 10).unwrap();

        assert_eq!(results.len(), 3);
        // First result should be id1 (exact match)
        assert_eq!(results[0].memory_id, id1);
        assert!(results[0].distance < 0.01); // Should be very close to 0
        // Second should be id2 (similar)
        assert_eq!(results[1].memory_id, id2);
        // Third should be id3 (different)
        assert_eq!(results[2].memory_id, id3);
    }

    #[test]
    fn test_similarity_search_with_limit() {
        let conn = create_test_connection();

        // Store 5 embeddings
        for i in 0..5 {
            let id = MemoryId::new();
            let embedding = vec![i as f32, 0.0, 0.0, 0.0];
            store_embedding(&conn, id, &embedding).unwrap();
        }

        let query = vec![2.5f32, 0.0, 0.0, 0.0];
        let results = search_similar(&conn, &query, 2).unwrap();

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_update_embedding() {
        let conn = create_test_connection();

        let memory_id = MemoryId::new();

        // Store initial embedding
        store_embedding(&conn, memory_id, &[1.0f32, 0.0, 0.0, 0.0]).unwrap();

        // Update with new embedding (INSERT OR REPLACE)
        store_embedding(&conn, memory_id, &[0.0f32, 1.0, 0.0, 0.0]).unwrap();

        // Should still have only 1 embedding
        assert_eq!(count_embeddings(&conn).unwrap(), 1);

        // Search should find the updated embedding
        let query = vec![0.0f32, 1.0, 0.0, 0.0];
        let results = search_similar(&conn, &query, 1).unwrap();
        assert_eq!(results[0].memory_id, memory_id);
        assert!(results[0].distance < 0.01);
    }
}
