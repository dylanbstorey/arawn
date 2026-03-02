//! Unified API: store, get_with_context, delete_cascade, update_indexed.

use tracing::{debug, info, warn};

use crate::error::Result;
use crate::graph::{GraphNode, GraphRelationship, RelationshipType};
use crate::types::{Memory, MemoryId};

use super::{MemoryStore, MemoryWithContext, RelatedEntity, StoreFactResult, StoreOptions};

impl MemoryStore {
    /// Store a memory with optional embedding and graph entities.
    ///
    /// This is the primary unified API for storing memories. It handles:
    /// - Inserting the memory into SQLite
    /// - Storing the embedding (if vectors initialized and provided)
    /// - Creating graph nodes and relationships (if graph initialized)
    ///
    /// # Example
    ///
    /// ```ignore
    /// let memory = Memory::new(ContentType::Note, "Rust has great memory safety");
    /// let options = StoreOptions {
    ///     embedding: Some(vec![0.1, 0.2, ...]),  // 384 dims
    ///     entities: vec![
    ///         EntityLink::new("rust", "Language", RelationshipType::Mentions),
    ///         EntityLink::new("memory_safety", "Concept", RelationshipType::Mentions),
    ///     ],
    /// };
    /// store.store(&memory, options)?;
    /// ```
    pub fn store(&self, memory: &Memory, options: StoreOptions) -> Result<()> {
        // 1. Insert the memory
        self.insert_memory(memory)?;

        // 2. Store embedding if provided and vectors are initialized
        if let Some(embedding) = &options.embedding {
            if self.has_vectors() {
                let conn = self.conn.lock().unwrap();
                crate::vector::store_embedding(&conn, memory.id, embedding)?;
            } else {
                warn!("Embedding provided but vectors not initialized - skipping");
            }
        }

        // 3. Create graph entities and relationships if graph is initialized
        if !options.entities.is_empty() {
            if let Some(graph) = &self.graph {
                let memory_node = GraphNode::new(memory.id.to_string(), "Memory")
                    .with_property("content_type", memory.content_type.as_str());
                graph.add_entity(&memory_node)?;

                for entity_link in &options.entities {
                    let mut entity_node =
                        GraphNode::new(&entity_link.entity_id, &entity_link.label);
                    for (key, value) in &entity_link.properties {
                        entity_node = entity_node.with_property(key, value);
                    }
                    graph.add_entity(&entity_node)?;

                    let rel = GraphRelationship::new(
                        memory.id.to_string(),
                        &entity_link.entity_id,
                        entity_link.relationship,
                    );
                    graph.add_relationship(&rel)?;
                }

                debug!(
                    "Added memory {} to graph with {} entity links",
                    memory.id,
                    options.entities.len()
                );
            } else {
                warn!("Entities provided but graph not initialized - skipping");
            }
        }

        Ok(())
    }

    /// Retrieve a memory with its graph context.
    ///
    /// Returns the memory along with related entities from the knowledge graph
    /// and whether it has an embedding stored.
    pub fn get_with_context(&self, id: MemoryId) -> Result<Option<MemoryWithContext>> {
        let memory = match self.get_memory(id)? {
            Some(m) => m,
            None => return Ok(None),
        };

        let has_embedding = if self.has_vectors() {
            let conn = self.conn.lock().unwrap();
            crate::vector::has_embedding(&conn, id)?
        } else {
            false
        };

        let related_entities = if let Some(graph) = &self.graph {
            match graph.get_neighbors(&id.to_string()) {
                Ok(neighbor_ids) => neighbor_ids
                    .into_iter()
                    .map(|entity_id| RelatedEntity {
                        entity_id,
                        relationship: RelationshipType::RelatedTo,
                    })
                    .collect(),
                Err(_) => Vec::new(),
            }
        } else {
            Vec::new()
        };

        Ok(Some(MemoryWithContext {
            memory,
            related_entities,
            has_embedding,
        }))
    }

    /// Delete a memory and all associated data (cascade delete).
    ///
    /// Removes:
    /// - The memory from SQLite
    /// - The embedding (if stored)
    /// - The graph node and its relationships (if in graph)
    ///
    /// Returns `true` if the memory was found and deleted.
    pub fn delete_cascade(&self, id: MemoryId) -> Result<bool> {
        // 1. Delete from graph if initialized
        if let Some(graph) = &self.graph
            && let Err(e) = graph.delete_entity(&id.to_string())
        {
            debug!("Graph delete for {} failed (may not exist): {}", id, e);
        }

        // 2. Delete embedding if vectors initialized
        if self.has_vectors() {
            let conn = self.conn.lock().unwrap();
            let _ = crate::vector::delete_embedding(&conn, id);
        }

        // 3. Delete the memory itself
        self.delete_memory(id)
    }

    /// Update a memory and re-index its embedding and entities.
    ///
    /// This updates the memory content and optionally:
    /// - Replaces the embedding
    /// - Updates graph entities (removes old links, adds new ones)
    ///
    /// Note: For entity updates, this removes all existing relationships
    /// from this memory and creates new ones based on the provided entities.
    pub fn update_indexed(&self, memory: &Memory, options: StoreOptions) -> Result<()> {
        // 1. Update the memory
        self.update_memory(memory)?;

        // 2. Update embedding if provided
        if let Some(embedding) = &options.embedding
            && self.has_vectors()
        {
            let conn = self.conn.lock().unwrap();
            crate::vector::store_embedding(&conn, memory.id, embedding)?;
        }

        // 3. Update graph if entities provided
        if !options.entities.is_empty()
            && let Some(graph) = &self.graph
        {
            let _ = graph.delete_entity(&memory.id.to_string());

            let memory_node = GraphNode::new(memory.id.to_string(), "Memory")
                .with_property("content_type", memory.content_type.as_str());
            graph.add_entity(&memory_node)?;

            for entity_link in &options.entities {
                let mut entity_node = GraphNode::new(&entity_link.entity_id, &entity_link.label);
                for (key, value) in &entity_link.properties {
                    entity_node = entity_node.with_property(key, value);
                }
                graph.add_entity(&entity_node)?;

                let rel = GraphRelationship::new(
                    memory.id.to_string(),
                    &entity_link.entity_id,
                    entity_link.relationship,
                );
                graph.add_relationship(&rel)?;
            }
        }

        Ok(())
    }

    /// Store a fact with automatic reinforcement and contradiction detection.
    ///
    /// If the memory's metadata contains `subject` and `predicate` fields:
    /// 1. If an existing memory has the same subject+predicate+content, reinforce it
    ///    (increment count, update last_accessed) and skip insertion.
    /// 2. Otherwise, supersede any existing memories with different content.
    ///
    /// Returns a `StoreFactResult` describing what happened.
    pub fn store_fact(&self, memory: &Memory, options: StoreOptions) -> Result<StoreFactResult> {
        // Check for reinforcement/contradiction if subject+predicate are set
        if let (Some(subject), Some(predicate)) =
            (&memory.metadata.subject, &memory.metadata.predicate)
        {
            let existing = self.find_contradictions(subject, predicate)?;

            // Check for exact match (reinforcement)
            for old_memory in &existing {
                if old_memory.id != memory.id && old_memory.content.trim() == memory.content.trim()
                {
                    self.reinforce(old_memory.id)?;
                    info!(
                        "Reinforced memory {} (subject={}, predicate={})",
                        old_memory.id, subject, predicate
                    );
                    return Ok(StoreFactResult::Reinforced {
                        existing_id: old_memory.id,
                    });
                }
            }

            // No exact match — supersede contradictions
            let mut superseded_ids = Vec::new();
            for old_memory in &existing {
                if old_memory.id == memory.id {
                    continue;
                }
                self.supersede(old_memory.id, memory.id)?;
                superseded_ids.push(old_memory.id);
                info!(
                    "Superseded memory {} (subject={}, predicate={}) with {}",
                    old_memory.id, subject, predicate, memory.id
                );
            }

            self.store(memory, options)?;

            if superseded_ids.is_empty() {
                return Ok(StoreFactResult::Inserted);
            } else {
                return Ok(StoreFactResult::Superseded { superseded_ids });
            }
        }

        // No subject/predicate — just store
        self.store(memory, options)?;
        Ok(StoreFactResult::Inserted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::EntityLink;
    use crate::types::{ContentType, MemoryId};
    use serial_test::serial;

    fn create_unified_test_store() -> MemoryStore {
        crate::vector::init_vector_extension();
        let mut store = MemoryStore::open_in_memory().unwrap();
        store.init_vectors(4, "mock").unwrap();
        store.init_graph().unwrap();
        store
    }

    #[test]
    #[serial]
    fn test_store_with_embedding() {
        let store = create_unified_test_store();

        let memory = Memory::new(ContentType::Note, "Test content");
        let options = StoreOptions {
            embedding: Some(vec![0.1, 0.2, 0.3, 0.4]),
            entities: vec![],
        };

        store.store(&memory, options).unwrap();

        let fetched = store.get_memory(memory.id).unwrap().unwrap();
        assert_eq!(fetched.content, "Test content");

        assert!(store.has_embedding(memory.id).unwrap());
    }

    #[test]
    #[serial]
    fn test_store_with_entities() {
        let store = create_unified_test_store();

        let memory = Memory::new(ContentType::Note, "Rust is great for memory safety");
        let options = StoreOptions {
            embedding: None,
            entities: vec![
                EntityLink::new("rust", "Language", RelationshipType::Mentions)
                    .with_property("name", "Rust"),
                EntityLink::new("memory_safety", "Concept", RelationshipType::Mentions)
                    .with_property("name", "Memory Safety"),
            ],
        };

        store.store(&memory, options).unwrap();

        let stats = store.graph_stats().unwrap();
        assert_eq!(stats.node_count, 3);
        assert_eq!(stats.relationship_count, 2);
    }

    #[test]
    #[serial]
    fn test_store_full_options() {
        let store = create_unified_test_store();

        let memory = Memory::new(ContentType::Note, "Learning about cats");
        let options = StoreOptions {
            embedding: Some(vec![0.5, 0.5, 0.0, 0.0]),
            entities: vec![EntityLink::new(
                "cats",
                "Animal",
                RelationshipType::Mentions,
            )],
        };

        store.store(&memory, options).unwrap();

        assert!(store.has_embedding(memory.id).unwrap());
        assert_eq!(store.graph_stats().unwrap().node_count, 2);
    }

    #[test]
    #[serial]
    fn test_get_with_context() {
        let store = create_unified_test_store();

        let memory = Memory::new(ContentType::Note, "Test with context");
        let options = StoreOptions {
            embedding: Some(vec![0.1, 0.2, 0.3, 0.4]),
            entities: vec![EntityLink::new(
                "topic1",
                "Topic",
                RelationshipType::Mentions,
            )],
        };

        store.store(&memory, options).unwrap();

        let ctx = store.get_with_context(memory.id).unwrap().unwrap();

        assert_eq!(ctx.memory.content, "Test with context");
        assert!(ctx.has_embedding);
        assert_eq!(ctx.related_entities.len(), 1);
        assert_eq!(ctx.related_entities[0].entity_id, "topic1");
    }

    #[test]
    #[serial]
    fn test_get_with_context_not_found() {
        let store = create_unified_test_store();

        let fake_id = MemoryId::new();
        let result = store.get_with_context(fake_id).unwrap();
        assert!(result.is_none());
    }

    #[test]
    #[serial]
    fn test_delete_cascade() {
        let store = create_unified_test_store();

        let memory = Memory::new(ContentType::Note, "To be deleted");
        let options = StoreOptions {
            embedding: Some(vec![0.1, 0.2, 0.3, 0.4]),
            entities: vec![EntityLink::new(
                "deleteme",
                "Test",
                RelationshipType::Mentions,
            )],
        };

        store.store(&memory, options).unwrap();

        assert!(store.get_memory(memory.id).unwrap().is_some());
        assert!(store.has_embedding(memory.id).unwrap());
        assert_eq!(store.graph_stats().unwrap().node_count, 2);

        let deleted = store.delete_cascade(memory.id).unwrap();
        assert!(deleted);

        assert!(store.get_memory(memory.id).unwrap().is_none());
        assert!(!store.has_embedding(memory.id).unwrap());
        assert_eq!(store.graph_stats().unwrap().node_count, 1);
    }

    #[test]
    #[serial]
    fn test_update_indexed() {
        let store = create_unified_test_store();

        let memory = Memory::new(ContentType::Note, "Original content");
        let options = StoreOptions {
            embedding: Some(vec![1.0, 0.0, 0.0, 0.0]),
            entities: vec![EntityLink::new(
                "old_topic",
                "Topic",
                RelationshipType::Mentions,
            )],
        };

        store.store(&memory, options).unwrap();

        let mut updated = memory.clone();
        updated.content = "Updated content".to_string();

        let new_options = StoreOptions {
            embedding: Some(vec![0.0, 1.0, 0.0, 0.0]),
            entities: vec![EntityLink::new(
                "new_topic",
                "Topic",
                RelationshipType::Mentions,
            )],
        };

        store.update_indexed(&updated, new_options).unwrap();

        let fetched = store.get_memory(memory.id).unwrap().unwrap();
        assert_eq!(fetched.content, "Updated content");

        let results = store.search_similar(&[0.0, 1.0, 0.0, 0.0], 1).unwrap();
        assert_eq!(results[0].memory_id, memory.id);
    }

    #[test]
    fn test_store_without_subsystems() {
        let store = MemoryStore::open_in_memory().unwrap();

        let memory = Memory::new(ContentType::Note, "Simple memory");
        let options = StoreOptions {
            embedding: Some(vec![0.1, 0.2, 0.3, 0.4]),
            entities: vec![EntityLink::new(
                "topic",
                "Topic",
                RelationshipType::Mentions,
            )],
        };

        store.store(&memory, options).unwrap();

        let fetched = store.get_memory(memory.id).unwrap().unwrap();
        assert_eq!(fetched.content, "Simple memory");
    }

    fn make_fact(subject: &str, predicate: &str, content: &str) -> Memory {
        let mut memory = Memory::new(ContentType::Fact, content);
        memory.metadata.subject = Some(subject.to_string());
        memory.metadata.predicate = Some(predicate.to_string());
        memory
    }

    #[test]
    fn test_store_fact_supersedes_contradiction() {
        let store = MemoryStore::open_in_memory().unwrap();

        let old = make_fact("user.model", "is", "GPT-4");
        let result = store.store_fact(&old, StoreOptions::default()).unwrap();
        assert!(matches!(result, StoreFactResult::Inserted));

        let new = make_fact("user.model", "is", "Claude");
        let result = store.store_fact(&new, StoreOptions::default()).unwrap();
        match result {
            StoreFactResult::Superseded { superseded_ids } => {
                assert_eq!(superseded_ids.len(), 1);
                assert_eq!(superseded_ids[0], old.id);
            }
            _ => panic!("Expected Superseded, got {:?}", result),
        }

        // Old memory is superseded
        let old_mem = store.get_memory(old.id).unwrap().unwrap();
        assert!(old_mem.confidence.superseded);
        assert_eq!(old_mem.confidence.superseded_by, Some(new.id));
        assert_eq!(old_mem.confidence.score, 0.0);

        // New memory is fine
        let new_mem = store.get_memory(new.id).unwrap().unwrap();
        assert!(!new_mem.confidence.superseded);
    }

    #[test]
    fn test_store_fact_no_contradiction_different_predicate() {
        let store = MemoryStore::open_in_memory().unwrap();

        let m1 = make_fact("user.model", "is", "Claude");
        store.store_fact(&m1, StoreOptions::default()).unwrap();

        let m2 = make_fact("user.model", "was", "GPT-4");
        let result = store.store_fact(&m2, StoreOptions::default()).unwrap();
        assert!(matches!(result, StoreFactResult::Inserted));
    }

    #[test]
    fn test_store_fact_no_subject_skips_contradiction_check() {
        let store = MemoryStore::open_in_memory().unwrap();

        let m1 = Memory::new(ContentType::Fact, "Some fact");
        let result = store.store_fact(&m1, StoreOptions::default()).unwrap();
        assert!(matches!(result, StoreFactResult::Inserted));
    }

    #[test]
    fn test_store_fact_reinforces_exact_match() {
        let store = MemoryStore::open_in_memory().unwrap();

        let m1 = make_fact("user.model", "is", "Claude");
        store.store_fact(&m1, StoreOptions::default()).unwrap();

        // Same subject+predicate+content → reinforce
        let m2 = make_fact("user.model", "is", "Claude");
        let result = store.store_fact(&m2, StoreOptions::default()).unwrap();
        match result {
            StoreFactResult::Reinforced { existing_id } => {
                assert_eq!(existing_id, m1.id);
            }
            _ => panic!("Expected Reinforced, got {:?}", result),
        }

        // Original memory has reinforcement_count=1
        let mem = store.get_memory(m1.id).unwrap().unwrap();
        assert_eq!(mem.confidence.reinforcement_count, 1);

        // m2 was NOT inserted (only m1 exists)
        assert!(store.get_memory(m2.id).unwrap().is_none());
    }

    #[test]
    fn test_store_fact_reinforced_score_higher() {
        use crate::types::ConfidenceParams;

        let store = MemoryStore::open_in_memory().unwrap();

        let m1 = make_fact("user.model", "is", "Claude");
        store.store_fact(&m1, StoreOptions::default()).unwrap();

        let base_score = store
            .get_memory(m1.id)
            .unwrap()
            .unwrap()
            .confidence
            .compute_score(&ConfidenceParams::default());

        // Reinforce 3 times
        for _ in 0..3 {
            let dup = make_fact("user.model", "is", "Claude");
            store.store_fact(&dup, StoreOptions::default()).unwrap();
        }

        let reinforced_score = store
            .get_memory(m1.id)
            .unwrap()
            .unwrap()
            .confidence
            .compute_score(&ConfidenceParams::default());

        assert!(
            reinforced_score > base_score,
            "reinforced={} should be > base={}",
            reinforced_score,
            base_score
        );
    }

    #[test]
    fn test_store_fact_multiple_supersessions() {
        let store = MemoryStore::open_in_memory().unwrap();

        let m1 = make_fact("user.lang", "is", "Python");
        store.store_fact(&m1, StoreOptions::default()).unwrap();

        let m2 = make_fact("user.lang", "is", "Rust");
        store.store_fact(&m2, StoreOptions::default()).unwrap();

        let m3 = make_fact("user.lang", "is", "Go");
        let result = store.store_fact(&m3, StoreOptions::default()).unwrap();

        // m2 should be superseded (m1 was already superseded by m2)
        match result {
            StoreFactResult::Superseded { superseded_ids } => {
                assert_eq!(superseded_ids.len(), 1);
                assert_eq!(superseded_ids[0], m2.id);
            }
            _ => panic!("Expected Superseded, got {:?}", result),
        }

        // Both m1 and m2 are superseded
        assert!(
            store
                .get_memory(m1.id)
                .unwrap()
                .unwrap()
                .confidence
                .superseded
        );
        assert!(
            store
                .get_memory(m2.id)
                .unwrap()
                .unwrap()
                .confidence
                .superseded
        );
        assert!(
            !store
                .get_memory(m3.id)
                .unwrap()
                .unwrap()
                .confidence
                .superseded
        );
    }
}
