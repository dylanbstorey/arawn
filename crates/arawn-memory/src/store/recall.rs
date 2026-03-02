//! Recall and text search operations.

use chrono::{DateTime, Utc};
use rusqlite::params;

use crate::error::{MemoryError, Result};
use crate::types::{ConfidenceParams, Memory, Staleness};

use super::{MemoryStore, RecallMatch, RecallQuery, RecallResult, TimeRange};

impl MemoryStore {
    /// Combined recall query blending vector similarity and graph context.
    ///
    /// This is the primary retrieval interface for the agent. It combines:
    /// - Vector similarity search for semantic matching
    /// - Knowledge graph traversal for related entities
    /// - Time and content type filtering
    /// - Configurable blending of results
    ///
    /// # Example
    ///
    /// ```ignore
    /// let query = RecallQuery::new(embedding)
    ///     .with_limit(20)
    ///     .with_time_range(TimeRange::Week)
    ///     .with_content_type(ContentType::Note);
    ///
    /// let result = store.recall(query)?;
    /// for m in result.matches {
    ///     println!("{}: {} (score: {})", m.memory.id, m.memory.content, m.score);
    /// }
    /// ```
    pub fn recall(&self, query: RecallQuery) -> Result<RecallResult> {
        use std::time::Instant;
        let start = Instant::now();

        // Check if vectors are initialized
        if !self.has_vectors() {
            return Err(MemoryError::Query("Vectors not initialized".to_string()));
        }

        // Step 1: Vector similarity search
        let conn = self.conn.lock().unwrap();
        let vector_results = crate::vector::search_similar(
            &conn,
            &query.embedding,
            query.limit * 2, // Fetch extra to allow for filtering
        )?;
        drop(conn);

        // Step 2: Fetch memories and apply filters
        let mut matches = Vec::new();
        let mut all_entities: Vec<String> = Vec::new();
        let time_cutoff = query.time_range.cutoff();

        for vr in vector_results {
            // Get the full memory
            let memory = match self.get_memory(vr.memory_id)? {
                Some(m) => m,
                None => continue, // Memory was deleted
            };

            // Apply session filter
            if let Some(ref sid) = query.session_id
                && memory.session_id.as_deref() != Some(sid.as_str())
            {
                continue;
            }

            // Apply time filter
            if let Some(cutoff) = time_cutoff
                && memory.created_at < cutoff
            {
                continue;
            }

            // Skip superseded memories
            if memory.confidence.superseded {
                continue;
            }

            // Apply content type filter
            if !query.content_types.is_empty()
                && !query.content_types.contains(&memory.content_type)
            {
                continue;
            }

            // Get graph context if enabled
            let related_entities = if query.include_graph_context {
                if let Some(graph) = &self.graph {
                    graph
                        .get_neighbors(&memory.id.to_string())
                        .unwrap_or_default()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            };

            // Collect unique entities
            for entity in &related_entities {
                if !all_entities.contains(entity) {
                    all_entities.push(entity.clone());
                }
            }

            // Calculate component scores
            let similarity_score = 1.0 / (1.0 + vr.distance);
            let confidence_score = memory
                .confidence
                .compute_score(&ConfidenceParams::default());

            let graph_score = if related_entities.is_empty() {
                0.0
            } else {
                (related_entities.len() as f32).min(5.0) / 5.0
            };

            // Blended score incorporating confidence.
            // With graph: similarity * 0.4 + graph * 0.3 + confidence * 0.3
            // Without graph: similarity * 0.6 + confidence * 0.4
            // vector_weight scales the similarity component (default 0.7).
            let has_graph_context =
                self.has_graph() && query.include_graph_context && graph_score > 0.0;
            let score = if has_graph_context {
                similarity_score * 0.4 + graph_score * 0.3 + confidence_score * 0.3
            } else {
                similarity_score * 0.6 + confidence_score * 0.4
            };

            // Compute staleness from citation (default to Unknown if no citation)
            let staleness = Self::compute_staleness(&memory);

            matches.push(RecallMatch {
                memory,
                distance: vr.distance,
                similarity_score,
                confidence_score,
                score,
                related_entities,
                staleness,
            });

            if matches.len() >= query.limit {
                break;
            }
        }

        // Filter by minimum score threshold
        if let Some(min) = query.min_score {
            matches.retain(|m| m.score >= min);
        }

        // Sort by score (highest first)
        matches.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let query_time_ms = start.elapsed().as_millis() as u64;

        Ok(RecallResult {
            matches,
            entities: all_entities,
            searched_count: self.count_embeddings()?,
            query_time_ms,
        })
    }

    /// Simple text search across memories.
    ///
    /// This is a fallback when vector search is not available or for
    /// exact text matching.
    pub fn search_memories(&self, query: &str, limit: usize) -> Result<Vec<Memory>> {
        let conn = self.conn.lock().unwrap();

        let pattern = format!("%{}%", query);

        let mut stmt = conn.prepare(
            r#"
            SELECT id, session_id, content_type, content, metadata, created_at, accessed_at, access_count,
                   confidence_source, reinforcement_count, superseded, superseded_by,
                   last_accessed, confidence_score, citation
            FROM memories
            WHERE content LIKE ?1
            ORDER BY created_at DESC
            LIMIT ?2
            "#,
        )?;

        let mut rows = stmt.query(params![pattern, limit as i64])?;

        let mut memories = Vec::new();
        while let Some(row) = rows.next()? {
            memories.push(Self::row_to_memory(row)?);
        }

        Ok(memories)
    }

    /// Compute staleness status for a memory based on its citation.
    ///
    /// For file citations, checks if the file's mtime has changed.
    /// For web citations, applies age-based staleness (7 days default).
    /// Other citation types and missing citations return Unknown.
    fn compute_staleness(memory: &Memory) -> Staleness {
        use crate::types::Citation;

        let Some(citation) = &memory.citation else {
            return Staleness::Unknown;
        };

        match citation {
            Citation::File { path, mtime, .. } => {
                // Check if file still exists and compare mtime
                match std::fs::metadata(path) {
                    Ok(metadata) => {
                        if let Some(stored_mtime) = mtime {
                            // Convert system time to DateTime<Utc>
                            if let Ok(current_mtime) = metadata.modified() {
                                let current: DateTime<Utc> = current_mtime.into();
                                // Compare with 1-second tolerance
                                let diff = (current - *stored_mtime).num_seconds().abs();
                                if diff > 1 {
                                    return Staleness::PotentiallyStale {
                                        reason: "file_modified".to_string(),
                                        last_verified: Some(*stored_mtime),
                                    };
                                }
                            }
                        }
                        Staleness::Fresh
                    }
                    Err(_) => Staleness::PotentiallyStale {
                        reason: "file_not_found".to_string(),
                        last_verified: *mtime,
                    },
                }
            }
            Citation::Web { fetched_at, .. } => {
                // Age-based staleness for web content (7 days default)
                let age_days = (Utc::now() - *fetched_at).num_days();
                if age_days > 7 {
                    Staleness::PotentiallyStale {
                        reason: "age_exceeded".to_string(),
                        last_verified: Some(*fetched_at),
                    }
                } else {
                    Staleness::Fresh
                }
            }
            Citation::Session { .. } | Citation::User { .. } => {
                // Session and user citations don't go stale
                Staleness::Fresh
            }
            Citation::System { .. } => {
                // System-derived data could be stale if dependencies changed,
                // but we can't easily check that, so mark as Unknown
                Staleness::Unknown
            }
        }
    }

    /// Search memories with time range filter.
    pub fn search_memories_in_range(
        &self,
        query: &str,
        time_range: TimeRange,
        limit: usize,
    ) -> Result<Vec<Memory>> {
        let conn = self.conn.lock().unwrap();

        let pattern = format!("%{}%", query);

        let memories = if let Some(cutoff) = time_range.cutoff() {
            let mut stmt = conn.prepare(
                r#"
                SELECT id, session_id, content_type, content, metadata, created_at, accessed_at, access_count,
                   confidence_source, reinforcement_count, superseded, superseded_by,
                   last_accessed, confidence_score, citation
                FROM memories
                WHERE content LIKE ?1 AND created_at >= ?2
                ORDER BY created_at DESC
                LIMIT ?3
                "#,
            )?;

            let mut rows = stmt.query(params![pattern, cutoff.to_rfc3339(), limit as i64])?;

            let mut mems = Vec::new();
            while let Some(row) = rows.next()? {
                mems.push(Self::row_to_memory(row)?);
            }
            mems
        } else {
            drop(conn);
            return self.search_memories(query, limit);
        };

        Ok(memories)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::RelationshipType;
    use crate::store::{EntityLink, StoreOptions};
    use crate::types::{ConfidenceSource, ContentType, MemoryConfidence};
    use serial_test::serial;

    fn create_test_store() -> MemoryStore {
        MemoryStore::open_in_memory().unwrap()
    }

    fn create_test_store_with_vectors() -> MemoryStore {
        crate::vector::init_vector_extension();
        let store = MemoryStore::open_in_memory().unwrap();
        store.init_vectors(4, "mock").unwrap();
        store
    }

    #[test]
    fn test_recall_basic() {
        let store = create_test_store_with_vectors();

        let m1 = Memory::new(ContentType::Note, "Rust programming language");
        let m2 = Memory::new(ContentType::Note, "Python programming");
        let m3 = Memory::new(ContentType::Fact, "JavaScript is for web");

        store
            .insert_memory_with_embedding(&m1, &[1.0, 0.0, 0.0, 0.0])
            .unwrap();
        store
            .insert_memory_with_embedding(&m2, &[0.8, 0.2, 0.0, 0.0])
            .unwrap();
        store
            .insert_memory_with_embedding(&m3, &[0.0, 0.0, 1.0, 0.0])
            .unwrap();

        let query = RecallQuery::new(vec![0.9, 0.1, 0.0, 0.0]).with_limit(10);

        let result = store.recall(query).unwrap();

        assert!(!result.matches.is_empty());
        assert!(
            result.matches[0].memory.content.contains("Rust")
                || result.matches[0].memory.content.contains("Python")
        );
    }

    #[test]
    fn test_recall_with_content_type_filter() {
        let store = create_test_store_with_vectors();

        let m1 = Memory::new(ContentType::Note, "A note about cats");
        let m2 = Memory::new(ContentType::Fact, "A fact about cats");

        store
            .insert_memory_with_embedding(&m1, &[1.0, 0.0, 0.0, 0.0])
            .unwrap();
        store
            .insert_memory_with_embedding(&m2, &[0.9, 0.1, 0.0, 0.0])
            .unwrap();

        let query = RecallQuery::new(vec![1.0, 0.0, 0.0, 0.0])
            .with_content_type(ContentType::Note)
            .with_limit(10);

        let result = store.recall(query).unwrap();

        assert_eq!(result.matches.len(), 1);
        assert_eq!(result.matches[0].memory.content_type, ContentType::Note);
    }

    #[test]
    fn test_recall_with_time_filter() {
        let store = create_test_store_with_vectors();

        let m1 = Memory::new(ContentType::Note, "Recent memory");
        store
            .insert_memory_with_embedding(&m1, &[1.0, 0.0, 0.0, 0.0])
            .unwrap();

        let query = RecallQuery::new(vec![1.0, 0.0, 0.0, 0.0])
            .with_time_range(TimeRange::Today)
            .with_limit(10);

        let result = store.recall(query).unwrap();
        assert_eq!(result.matches.len(), 1);

        let query = RecallQuery::new(vec![1.0, 0.0, 0.0, 0.0])
            .with_time_range(TimeRange::All)
            .with_limit(10);

        let result = store.recall(query).unwrap();
        assert_eq!(result.matches.len(), 1);
    }

    #[test]
    #[serial]
    fn test_recall_with_graph_context() {
        let mut store = MemoryStore::open_in_memory().unwrap();
        crate::vector::init_vector_extension();
        store.init_vectors(4, "mock").unwrap();
        store.init_graph().unwrap();

        let memory = Memory::new(ContentType::Note, "Learning about Rust");
        let options = StoreOptions {
            embedding: Some(vec![1.0, 0.0, 0.0, 0.0]),
            entities: vec![EntityLink::new(
                "rust_lang",
                "Language",
                RelationshipType::Mentions,
            )],
        };
        store.store(&memory, options).unwrap();

        let query = RecallQuery::new(vec![1.0, 0.0, 0.0, 0.0])
            .with_graph_context(true)
            .with_limit(10);

        let result = store.recall(query).unwrap();
        assert_eq!(result.matches.len(), 1);
        assert!(!result.matches[0].related_entities.is_empty());
        assert!(result.entities.contains(&"rust_lang".to_string()));
    }

    #[test]
    fn test_recall_vector_weight() {
        let store = create_test_store_with_vectors();

        let m1 = Memory::new(ContentType::Note, "Memory 1");
        store
            .insert_memory_with_embedding(&m1, &[1.0, 0.0, 0.0, 0.0])
            .unwrap();

        let query = RecallQuery::new(vec![1.0, 0.0, 0.0, 0.0])
            .with_vector_weight(1.0)
            .with_limit(10);

        let result = store.recall(query).unwrap();
        assert_eq!(result.matches.len(), 1);

        let score = result.matches[0].score;
        // With default Inferred confidence (base 0.5): 0.6*sim + 0.4*0.5 ≈ 0.8
        assert!(score > 0.7, "score was {}", score);
    }

    #[test]
    fn test_recall_result_ordering() {
        let store = create_test_store_with_vectors();

        let m1 = Memory::new(ContentType::Note, "Very similar");
        let m2 = Memory::new(ContentType::Note, "Somewhat similar");
        let m3 = Memory::new(ContentType::Note, "Not similar");

        store
            .insert_memory_with_embedding(&m1, &[1.0, 0.0, 0.0, 0.0])
            .unwrap();
        store
            .insert_memory_with_embedding(&m2, &[0.5, 0.5, 0.0, 0.0])
            .unwrap();
        store
            .insert_memory_with_embedding(&m3, &[0.0, 0.0, 1.0, 0.0])
            .unwrap();

        let query = RecallQuery::new(vec![1.0, 0.0, 0.0, 0.0]).with_limit(10);

        let result = store.recall(query).unwrap();

        for i in 1..result.matches.len() {
            assert!(result.matches[i - 1].score >= result.matches[i].score);
        }
    }

    #[test]
    fn test_recall_query_builder() {
        let query = RecallQuery::new(vec![0.1, 0.2, 0.3, 0.4])
            .with_limit(20)
            .with_time_range(TimeRange::Week)
            .with_content_type(ContentType::Note)
            .with_content_type(ContentType::Fact)
            .with_vector_weight(0.8)
            .with_graph_context(false);

        assert_eq!(query.limit, 20);
        assert_eq!(query.time_range, TimeRange::Week);
        assert_eq!(query.content_types.len(), 2);
        assert!((query.vector_weight - 0.8).abs() < 0.01);
        assert!(!query.include_graph_context);
    }

    #[test]
    fn test_recall_without_vectors_fails() {
        let store = MemoryStore::open_in_memory().unwrap();

        let query = RecallQuery::new(vec![0.1, 0.2, 0.3, 0.4]);
        let result = store.recall(query);

        assert!(result.is_err());
    }

    #[test]
    fn test_search_memories_text() {
        let store = create_test_store();

        store
            .insert_memory(&Memory::new(ContentType::Note, "Rust is fast"))
            .unwrap();
        store
            .insert_memory(&Memory::new(ContentType::Note, "Python is dynamic"))
            .unwrap();
        store
            .insert_memory(&Memory::new(ContentType::Note, "Rust memory safety"))
            .unwrap();

        let results = store.search_memories("Rust", 10).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_time_range_cutoffs() {
        assert!(TimeRange::Today.cutoff().is_some());
        assert!(TimeRange::Week.cutoff().is_some());
        assert!(TimeRange::Month.cutoff().is_some());
        assert!(TimeRange::All.cutoff().is_none());
    }

    #[test]
    fn test_recall_performance_many_memories() {
        let store = create_test_store_with_vectors();

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
        }

        let start = std::time::Instant::now();
        let query = RecallQuery::new(vec![1.0, 0.0, 0.5, 0.5]).with_limit(10);

        let result = store.recall(query).unwrap();
        let elapsed = start.elapsed();

        assert_eq!(result.matches.len(), 10);
        assert_eq!(result.searched_count, 100);

        assert!(
            elapsed.as_millis() < 100,
            "Recall took too long: {}ms",
            elapsed.as_millis()
        );
    }

    #[test]
    #[serial]
    fn test_recall_mixed_content_integration() {
        let mut store = MemoryStore::open_in_memory().unwrap();
        crate::vector::init_vector_extension();
        store.init_vectors(4, "mock").unwrap();
        store.init_graph().unwrap();

        let msg1 = Memory::new(ContentType::UserMessage, "How do I use Rust async?");
        store
            .store(
                &msg1,
                StoreOptions {
                    embedding: Some(vec![0.9, 0.1, 0.0, 0.0]),
                    entities: vec![EntityLink::new(
                        "rust",
                        "Language",
                        RelationshipType::Mentions,
                    )],
                },
            )
            .unwrap();

        let msg2 = Memory::new(
            ContentType::AssistantMessage,
            "In Rust, async/await works with tokio...",
        );
        store
            .store(
                &msg2,
                StoreOptions {
                    embedding: Some(vec![0.85, 0.15, 0.0, 0.0]),
                    entities: vec![
                        EntityLink::new("rust", "Language", RelationshipType::Mentions),
                        EntityLink::new("tokio", "Library", RelationshipType::Mentions),
                    ],
                },
            )
            .unwrap();

        let note = Memory::new(
            ContentType::Note,
            "Remember: Rust async requires an executor",
        );
        store
            .store(
                &note,
                StoreOptions {
                    embedding: Some(vec![0.8, 0.2, 0.0, 0.0]),
                    entities: vec![],
                },
            )
            .unwrap();

        let fact = Memory::new(ContentType::Fact, "Python asyncio is similar to Rust async");
        store
            .store(
                &fact,
                StoreOptions {
                    embedding: Some(vec![0.5, 0.5, 0.0, 0.0]),
                    entities: vec![
                        EntityLink::new("python", "Language", RelationshipType::Mentions),
                        EntityLink::new("rust", "Language", RelationshipType::Mentions),
                    ],
                },
            )
            .unwrap();

        let query = RecallQuery::new(vec![0.9, 0.1, 0.0, 0.0]).with_limit(10);

        let result = store.recall(query).unwrap();

        assert!(!result.matches.is_empty());
        assert!(result.entities.contains(&"rust".to_string()));

        let query = RecallQuery::new(vec![0.9, 0.1, 0.0, 0.0])
            .with_content_type(ContentType::AssistantMessage)
            .with_limit(10);

        let result = store.recall(query).unwrap();
        assert_eq!(result.matches.len(), 1);
        assert_eq!(
            result.matches[0].memory.content_type,
            ContentType::AssistantMessage
        );
    }

    #[test]
    fn test_recall_high_confidence_ranks_above_low() {
        let store = create_test_store_with_vectors();

        // Both memories have identical embeddings → same similarity
        let high_conf = Memory::new(ContentType::Fact, "High confidence fact")
            .with_confidence(MemoryConfidence::with_source(ConfidenceSource::Stated));
        let low_conf = Memory::new(ContentType::Fact, "Low confidence fact")
            .with_confidence(MemoryConfidence::with_source(ConfidenceSource::Inferred));

        store
            .insert_memory_with_embedding(&high_conf, &[1.0, 0.0, 0.0, 0.0])
            .unwrap();
        store
            .insert_memory_with_embedding(&low_conf, &[1.0, 0.0, 0.0, 0.0])
            .unwrap();

        let query = RecallQuery::new(vec![1.0, 0.0, 0.0, 0.0]).with_limit(10);
        let result = store.recall(query).unwrap();

        assert_eq!(result.matches.len(), 2);
        // High confidence should rank first
        assert_eq!(result.matches[0].memory.id, high_conf.id);
        assert!(result.matches[0].confidence_score > result.matches[1].confidence_score);
    }

    #[test]
    fn test_recall_superseded_excluded_by_min_score() {
        let store = create_test_store_with_vectors();

        let m1 = Memory::new(ContentType::Fact, "Old fact");
        let m2 = Memory::new(ContentType::Fact, "New fact");

        store
            .insert_memory_with_embedding(&m1, &[1.0, 0.0, 0.0, 0.0])
            .unwrap();
        store
            .insert_memory_with_embedding(&m2, &[0.9, 0.1, 0.0, 0.0])
            .unwrap();

        // Supersede m1
        store.supersede(m1.id, m2.id).unwrap();

        let query = RecallQuery::new(vec![1.0, 0.0, 0.0, 0.0])
            .with_min_score(0.01) // Any positive min_score excludes superseded (score=0)
            .with_limit(10);
        let result = store.recall(query).unwrap();

        // Only m2 should remain
        assert_eq!(result.matches.len(), 1);
        assert_eq!(result.matches[0].memory.id, m2.id);
    }

    #[test]
    fn test_recall_match_includes_confidence_score() {
        let store = create_test_store_with_vectors();

        let m = Memory::new(ContentType::Note, "Test memory")
            .with_confidence(MemoryConfidence::with_source(ConfidenceSource::Observed));
        store
            .insert_memory_with_embedding(&m, &[1.0, 0.0, 0.0, 0.0])
            .unwrap();

        let query = RecallQuery::new(vec![1.0, 0.0, 0.0, 0.0]).with_limit(10);
        let result = store.recall(query).unwrap();

        assert_eq!(result.matches.len(), 1);
        let rm = &result.matches[0];
        assert!(rm.similarity_score > 0.0);
        assert!(rm.confidence_score > 0.0);
        // Observed base = 0.7
        assert!((rm.confidence_score - 0.7).abs() < 0.1);
    }
}
