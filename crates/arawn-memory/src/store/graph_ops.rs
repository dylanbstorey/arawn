//! Graph passthrough operations.

use crate::error::{MemoryError, Result};
use crate::graph::{GraphNode, GraphRelationship, GraphStats};

use super::MemoryStore;

impl MemoryStore {
    /// Add an entity to the knowledge graph.
    pub fn add_graph_entity(&self, node: &GraphNode) -> Result<()> {
        match &self.graph {
            Some(graph) => graph.add_entity(node),
            None => Err(MemoryError::Query("Graph not initialized".to_string())),
        }
    }

    /// Add a relationship to the knowledge graph.
    pub fn add_graph_relationship(&self, rel: &GraphRelationship) -> Result<()> {
        match &self.graph {
            Some(graph) => graph.add_relationship(rel),
            None => Err(MemoryError::Query("Graph not initialized".to_string())),
        }
    }

    /// Delete an entity from the knowledge graph.
    pub fn delete_graph_entity(&self, id: &str) -> Result<bool> {
        match &self.graph {
            Some(graph) => graph.delete_entity(id),
            None => Err(MemoryError::Query("Graph not initialized".to_string())),
        }
    }

    /// Get neighbors of an entity in the knowledge graph.
    pub fn get_graph_neighbors(&self, id: &str) -> Result<Vec<String>> {
        match &self.graph {
            Some(graph) => graph.get_neighbors(id),
            None => Err(MemoryError::Query("Graph not initialized".to_string())),
        }
    }

    /// Get knowledge graph statistics.
    pub fn graph_stats(&self) -> Result<GraphStats> {
        match &self.graph {
            Some(graph) => graph.stats(),
            None => Err(MemoryError::Query("Graph not initialized".to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::RelationshipType;
    use serial_test::serial;

    #[test]
    fn test_graph_operations_without_init() {
        let store = MemoryStore::open_in_memory().unwrap();

        let node = GraphNode::new("test", "Test");
        let result = store.add_graph_entity(&node);
        assert!(result.is_err());

        let result = store.graph_stats();
        assert!(result.is_err());
    }

    #[test]
    #[serial]
    fn test_graph_passthrough_operations() {
        let mut store = MemoryStore::open_in_memory().unwrap();
        store.init_graph().unwrap();

        let node = GraphNode::new("alice", "Person").with_property("name", "Alice");
        store.add_graph_entity(&node).unwrap();

        let node2 = GraphNode::new("bob", "Person").with_property("name", "Bob");
        store.add_graph_entity(&node2).unwrap();

        let rel = GraphRelationship::new("alice", "bob", RelationshipType::RelatedTo);
        store.add_graph_relationship(&rel).unwrap();

        let stats = store.graph_stats().unwrap();
        assert_eq!(stats.node_count, 2);
        assert_eq!(stats.relationship_count, 1);

        let neighbors = store.get_graph_neighbors("alice").unwrap();
        assert!(!neighbors.is_empty());

        store.delete_graph_entity("bob").unwrap();
        assert_eq!(store.graph_stats().unwrap().node_count, 1);
    }

    #[test]
    #[serial]
    fn test_has_vectors_and_has_graph() {
        crate::vector::init_vector_extension();
        let mut store = MemoryStore::open_in_memory().unwrap();

        assert!(!store.has_vectors());
        assert!(!store.has_graph());

        crate::vector::init_vector_extension();
        store.init_vectors(4, "mock").unwrap();
        assert!(store.has_vectors());
        assert!(!store.has_graph());

        store.init_graph().unwrap();
        assert!(store.has_vectors());
        assert!(store.has_graph());
    }
}
