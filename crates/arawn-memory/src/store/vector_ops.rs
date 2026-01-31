//! Vector search and embedding operations.

use rusqlite::params;
use tracing::{debug, warn};

use crate::error::{MemoryError, Result};
use crate::types::{Memory, MemoryId};

use super::{MemoryStore, ReindexDryRun, ReindexReport};

impl MemoryStore {
    /// Initialize vector search capabilities.
    ///
    /// This must be called before using any vector operations.
    /// Creates the vector embeddings table if it doesn't exist.
    /// Initialize vector storage with dimension mismatch detection.
    ///
    /// Stores the embedding dimensions and provider name in the metadata table.
    /// If previously stored dimensions differ from the requested dimensions,
    /// vectors are marked as stale and search will return empty results until
    /// a reindex is performed.
    pub fn init_vectors(&self, dims: usize, provider: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        // Check for dimension mismatch with previously stored config
        let stored_dims: Option<String> = conn
            .query_row(
                "SELECT value FROM meta WHERE key = 'embedding.dimensions'",
                [],
                |row| row.get(0),
            )
            .ok();

        if let Some(ref stored) = stored_dims {
            if let Ok(old_dims) = stored.parse::<usize>() {
                if old_dims != dims {
                    warn!(
                        "Embedding dimension mismatch: stored={}, configured={}. \
                         Vector search disabled until reindex. Run `arawn memory reindex`.",
                        old_dims, dims
                    );
                    *self.vectors_stale.lock().unwrap() = true;
                    *self.vectors_initialized.lock().unwrap() = true;
                    return Ok(());
                }
            }
        }

        crate::vector::create_vector_table(&conn, dims)?;

        // Store metadata (fresh install or matching dims)
        conn.execute(
            "INSERT OR REPLACE INTO meta (key, value) VALUES ('embedding.dimensions', ?1)",
            params![dims.to_string()],
        )?;
        conn.execute(
            "INSERT OR REPLACE INTO meta (key, value) VALUES ('embedding.provider', ?1)",
            params![provider],
        )?;

        *self.vectors_initialized.lock().unwrap() = true;
        Ok(())
    }

    /// Check if vector embeddings are stale (dimension/provider mismatch).
    pub fn vectors_stale(&self) -> bool {
        *self.vectors_stale.lock().unwrap()
    }

    /// Dry-run reindex: returns counts without doing any work.
    pub fn reindex_dry_run(&self) -> Result<ReindexDryRun> {
        let conn = self.conn.lock().unwrap();
        let memory_count: i64 =
            conn.query_row("SELECT COUNT(*) FROM memories", [], |row| row.get(0))?;
        let total_chars: i64 = conn.query_row(
            "SELECT COALESCE(SUM(LENGTH(content)), 0) FROM memories",
            [],
            |row| row.get(0),
        )?;
        Ok(ReindexDryRun {
            memory_count: memory_count as usize,
            estimated_tokens: (total_chars as usize) / 4,
        })
    }

    /// Reindex all memory embeddings with a new embedder/dimensions.
    ///
    /// Drops the existing vector table, recreates it with the new dimensions,
    /// and re-embeds all memories using the provided embed function.
    ///
    /// The `embed_batch` closure receives a batch of text strings and returns
    /// their embeddings. This avoids coupling arawn-memory to arawn-llm.
    pub async fn reindex<F, Fut>(
        &self,
        embed_batch: F,
        new_dims: usize,
        new_provider: &str,
    ) -> Result<ReindexReport>
    where
        F: Fn(Vec<String>) -> Fut,
        Fut: std::future::Future<Output = std::result::Result<Vec<Vec<f32>>, String>>,
    {
        let start = std::time::Instant::now();

        // 1. Read all memories
        let memories = {
            let conn = self.conn.lock().unwrap();
            let mut stmt = conn.prepare("SELECT id, content FROM memories")?;
            let rows = stmt.query_map([], |row| {
                let id_str: String = row.get(0)?;
                let content: String = row.get(1)?;
                Ok((id_str, content))
            })?;
            rows.collect::<std::result::Result<Vec<_>, _>>()?
        };

        let total = memories.len();
        let mut embedded = 0usize;
        let mut skipped = 0usize;

        // 2. Drop and recreate vector table
        {
            let conn = self.conn.lock().unwrap();
            crate::vector::drop_vector_table(&conn)?;
            crate::vector::create_vector_table(&conn, new_dims)?;
        }

        // 3. Batch embed in chunks
        let batch_size = 32;
        for chunk in memories.chunks(batch_size) {
            let items: Vec<(&str, &str)> = chunk
                .iter()
                .map(|(id, content)| (id.as_str(), content.as_str()))
                .collect();

            let non_empty: Vec<(&str, &str)> = items
                .iter()
                .filter(|(_, content)| !content.trim().is_empty())
                .copied()
                .collect();

            skipped += items.len() - non_empty.len();

            if non_empty.is_empty() {
                continue;
            }

            let texts: Vec<String> = non_empty.iter().map(|(_, c)| c.to_string()).collect();
            let embeddings = embed_batch(texts)
                .await
                .map_err(|e| MemoryError::InvalidData(format!("Embedding failed: {e}")))?;

            let conn = self.conn.lock().unwrap();
            for ((id_str, _), embedding) in non_empty.iter().zip(embeddings.iter()) {
                let memory_id = crate::types::MemoryId::parse(id_str)?;
                crate::vector::store_embedding(&conn, memory_id, embedding)?;
                embedded += 1;
            }
        }

        // 4. Update metadata
        {
            let conn = self.conn.lock().unwrap();
            conn.execute(
                "INSERT OR REPLACE INTO meta (key, value) VALUES ('embedding.dimensions', ?1)",
                params![new_dims.to_string()],
            )?;
            conn.execute(
                "INSERT OR REPLACE INTO meta (key, value) VALUES ('embedding.provider', ?1)",
                params![new_provider],
            )?;
        }

        // 5. Clear stale flag
        *self.vectors_stale.lock().unwrap() = false;

        Ok(ReindexReport {
            total,
            embedded,
            skipped,
            elapsed: start.elapsed(),
        })
    }

    /// Store a memory with its embedding.
    ///
    /// This is a convenience method that inserts both the memory and its embedding
    /// in a single operation.
    pub fn insert_memory_with_embedding(&self, memory: &Memory, embedding: &[f32]) -> Result<()> {
        self.insert_memory(memory)?;

        let conn = self.conn.lock().unwrap();
        crate::vector::store_embedding(&conn, memory.id, embedding)?;

        Ok(())
    }

    /// Store an embedding for an existing memory.
    pub fn store_embedding(&self, memory_id: MemoryId, embedding: &[f32]) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        crate::vector::store_embedding(&conn, memory_id, embedding)
    }

    /// Delete an embedding for a memory.
    pub fn delete_embedding(&self, memory_id: MemoryId) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        crate::vector::delete_embedding(&conn, memory_id)
    }

    /// Search for similar memories using vector similarity.
    ///
    /// Returns memory IDs ordered by similarity (most similar first).
    pub fn search_similar(
        &self,
        query_embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<crate::vector::SimilarityResult>> {
        let conn = self.conn.lock().unwrap();
        crate::vector::search_similar(&conn, query_embedding, limit)
    }

    /// Search for similar memories and return the full Memory objects.
    ///
    /// This performs a vector search and then fetches the full memory details.
    pub fn search_similar_memories(
        &self,
        query_embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<(Memory, f32)>> {
        if *self.vectors_stale.lock().unwrap() {
            debug!("Vector search skipped: embeddings are stale (dimension mismatch)");
            return Ok(Vec::new());
        }
        let results = self.search_similar(query_embedding, limit)?;

        let mut memories = Vec::with_capacity(results.len());
        for result in results {
            if let Some(memory) = self.get_memory(result.memory_id)? {
                memories.push((memory, result.distance));
            }
        }

        Ok(memories)
    }

    /// Check if a memory has an embedding.
    pub fn has_embedding(&self, memory_id: MemoryId) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        crate::vector::has_embedding(&conn, memory_id)
    }

    /// Get the count of stored embeddings.
    pub fn count_embeddings(&self) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        crate::vector::count_embeddings(&conn)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ContentType;

    fn create_test_store_with_vectors() -> MemoryStore {
        crate::vector::init_vector_extension();
        let store = MemoryStore::open_in_memory().unwrap();
        store.init_vectors(4, "mock").unwrap();
        store
    }

    #[test]
    fn test_memory_with_embedding() {
        let store = create_test_store_with_vectors();

        let memory = Memory::new(ContentType::Note, "Test with embedding");
        let embedding = vec![0.1f32, 0.2, 0.3, 0.4];

        store
            .insert_memory_with_embedding(&memory, &embedding)
            .unwrap();

        let fetched = store.get_memory(memory.id).unwrap().unwrap();
        assert_eq!(fetched.content, "Test with embedding");

        assert!(store.has_embedding(memory.id).unwrap());
        assert_eq!(store.count_embeddings().unwrap(), 1);
    }

    #[test]
    fn test_vector_search_via_store() {
        let store = create_test_store_with_vectors();

        let m1 = Memory::new(ContentType::Note, "About cats");
        let m2 = Memory::new(ContentType::Note, "About dogs");
        let m3 = Memory::new(ContentType::Note, "About birds");

        store
            .insert_memory_with_embedding(&m1, &[1.0, 0.0, 0.0, 0.0])
            .unwrap();
        store
            .insert_memory_with_embedding(&m2, &[0.0, 1.0, 0.0, 0.0])
            .unwrap();
        store
            .insert_memory_with_embedding(&m3, &[0.0, 0.0, 1.0, 0.0])
            .unwrap();

        let results = store
            .search_similar_memories(&[0.9, 0.1, 0.0, 0.0], 10)
            .unwrap();

        assert_eq!(results.len(), 3);
        assert_eq!(results[0].0.id, m1.id);
        assert_eq!(results[0].0.content, "About cats");
    }

    #[test]
    fn test_vector_search_100_memories() {
        let store = create_test_store_with_vectors();

        let mut memory_ids = Vec::new();
        for i in 0..100 {
            let memory = Memory::new(ContentType::Note, format!("Memory number {}", i));

            let angle = (i as f32) * std::f32::consts::PI / 50.0;
            let embedding = vec![
                angle.cos(),
                angle.sin(),
                (i as f32 / 100.0),
                1.0 - (i as f32 / 100.0),
            ];

            store
                .insert_memory_with_embedding(&memory, &embedding)
                .unwrap();
            memory_ids.push(memory.id);
        }

        assert_eq!(store.count_memories(None).unwrap(), 100);
        assert_eq!(store.count_embeddings().unwrap(), 100);

        let query = vec![1.0f32, 0.0, 0.0, 1.0];
        let results = store.search_similar(&query, 10).unwrap();

        assert_eq!(results.len(), 10);

        for i in 1..results.len() {
            assert!(results[i - 1].distance <= results[i].distance);
        }

        let query = vec![-1.0f32, 0.0, 1.0, 0.0];
        let results = store.search_similar(&query, 5).unwrap();
        assert_eq!(results.len(), 5);
    }

    #[test]
    fn test_stats_with_embeddings() {
        let store = create_test_store_with_vectors();

        let m1 = Memory::new(ContentType::Note, "With embedding");
        let m2 = Memory::new(ContentType::Note, "Without embedding");

        store
            .insert_memory_with_embedding(&m1, &[0.1, 0.2, 0.3, 0.4])
            .unwrap();
        store.insert_memory(&m2).unwrap();

        let stats = store.stats().unwrap();
        assert_eq!(stats.memory_count, 2);
        assert_eq!(stats.embedding_count, 1);
    }

    #[test]
    fn test_init_vectors_stores_metadata() {
        let store = MemoryStore::open_in_memory().unwrap();
        store.init_vectors(384, "local").unwrap();

        assert_eq!(
            store.get_meta("embedding.dimensions").unwrap(),
            Some("384".to_string())
        );
        assert_eq!(
            store.get_meta("embedding.provider").unwrap(),
            Some("local".to_string())
        );
        assert!(!store.vectors_stale());
    }

    #[test]
    fn test_init_vectors_same_dims_ok() {
        let store = MemoryStore::open_in_memory().unwrap();
        store.init_vectors(384, "local").unwrap();
        store.init_vectors(384, "local").unwrap();
        assert!(!store.vectors_stale());
    }

    #[test]
    fn test_init_vectors_dimension_mismatch_marks_stale() {
        let store = MemoryStore::open_in_memory().unwrap();
        store.init_vectors(384, "local").unwrap();
        assert!(!store.vectors_stale());

        store.init_vectors(1536, "openai").unwrap();
        assert!(store.vectors_stale());
    }

    #[test]
    fn test_stale_vectors_search_returns_empty() {
        let store = MemoryStore::open_in_memory().unwrap();
        store.init_vectors(4, "mock").unwrap();

        let memory = Memory::new(ContentType::Note, "test memory");
        store
            .insert_memory_with_embedding(&memory, &[0.1, 0.2, 0.3, 0.4])
            .unwrap();

        store.init_vectors(8, "openai").unwrap();
        assert!(store.vectors_stale());

        let results = store
            .search_similar_memories(&[0.1, 0.2, 0.3, 0.4], 10)
            .unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_stats_includes_embedding_metadata() {
        let store = MemoryStore::open_in_memory().unwrap();
        store.init_vectors(384, "local").unwrap();

        let stats = store.stats().unwrap();
        assert_eq!(stats.embedding_provider.as_deref(), Some("local"));
        assert_eq!(stats.embedding_dimensions, Some(384));
        assert!(!stats.vectors_stale);
    }

    #[test]
    fn test_reindex_dry_run() {
        let store = MemoryStore::open_in_memory().unwrap();
        store
            .insert_memory(&Memory::new(ContentType::Note, "hello world"))
            .unwrap();
        store
            .insert_memory(&Memory::new(ContentType::Fact, "rust is great"))
            .unwrap();

        let dry = store.reindex_dry_run().unwrap();
        assert_eq!(dry.memory_count, 2);
        assert!(dry.estimated_tokens > 0);
    }

    #[tokio::test]
    async fn test_reindex_reembeds_all_memories() {
        let store = MemoryStore::open_in_memory().unwrap();
        store.init_vectors(4, "mock").unwrap();

        let m1 = Memory::new(ContentType::Note, "first");
        let m2 = Memory::new(ContentType::Note, "second");
        store.insert_memory(&m1).unwrap();
        store.insert_memory(&m2).unwrap();

        store.init_vectors(2, "new_provider").unwrap();
        assert!(store.vectors_stale());

        let report = store
            .reindex(
                |texts| async move { Ok(texts.iter().map(|_| vec![0.5, 0.5]).collect()) },
                2,
                "new_provider",
            )
            .await
            .unwrap();

        assert_eq!(report.total, 2);
        assert_eq!(report.embedded, 2);
        assert_eq!(report.skipped, 0);
        assert!(!store.vectors_stale());

        assert_eq!(
            store.get_meta("embedding.dimensions").unwrap(),
            Some("2".to_string())
        );
        assert_eq!(
            store.get_meta("embedding.provider").unwrap(),
            Some("new_provider".to_string())
        );

        assert_eq!(store.count_embeddings().unwrap(), 2);
    }

    #[tokio::test]
    async fn test_reindex_skips_empty_content() {
        let store = MemoryStore::open_in_memory().unwrap();
        store.init_vectors(2, "mock").unwrap();

        store
            .insert_memory(&Memory::new(ContentType::Note, "has content"))
            .unwrap();
        store
            .insert_memory(&Memory::new(ContentType::Note, "   "))
            .unwrap();

        let report = store
            .reindex(
                |texts| async move { Ok(texts.iter().map(|_| vec![0.5, 0.5]).collect()) },
                2,
                "mock",
            )
            .await
            .unwrap();

        assert_eq!(report.total, 2);
        assert_eq!(report.embedded, 1);
        assert_eq!(report.skipped, 1);
    }
}
