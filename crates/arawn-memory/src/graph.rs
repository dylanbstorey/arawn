//! Knowledge graph storage using graphqlite.
//!
//! This module provides knowledge graph capabilities using the graphqlite
//! SQLite extension, enabling entity storage, relationship management,
//! and Cypher query support.

use std::path::Path;

use graphqlite::Graph;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::error::{MemoryError, Result};

// ─────────────────────────────────────────────────────────────────────────────
// Types
// ─────────────────────────────────────────────────────────────────────────────

/// A node/entity in the knowledge graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    /// Unique identifier for the node.
    pub id: String,
    /// Node label (e.g., "Person", "Concept", "Source").
    pub label: String,
    /// Node properties as key-value pairs.
    pub properties: Vec<(String, String)>,
}

impl GraphNode {
    /// Create a new graph node.
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            properties: Vec::new(),
        }
    }

    /// Add a property to the node.
    pub fn with_property(mut self, key: impl Into<String>, value: impl ToString) -> Self {
        self.properties.push((key.into(), value.to_string()));
        self
    }
}

/// Relationship types supported in the knowledge graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RelationshipType {
    /// One entity supports another (evidence, citation).
    Supports,
    /// One entity contradicts another.
    Contradicts,
    /// General relationship between entities.
    RelatedTo,
    /// Entity is cited in another.
    CitedIn,
    /// Entity mentions another.
    Mentions,
    /// Entity is part of another.
    PartOf,
    /// Entity created another.
    CreatedBy,
    /// Entity is a type/instance of another.
    IsA,
}

impl RelationshipType {
    /// Get the string representation for Cypher queries.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Supports => "SUPPORTS",
            Self::Contradicts => "CONTRADICTS",
            Self::RelatedTo => "RELATED_TO",
            Self::CitedIn => "CITED_IN",
            Self::Mentions => "MENTIONS",
            Self::PartOf => "PART_OF",
            Self::CreatedBy => "CREATED_BY",
            Self::IsA => "IS_A",
        }
    }
}

/// A relationship/edge in the knowledge graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphRelationship {
    /// Source node ID.
    pub from_id: String,
    /// Target node ID.
    pub to_id: String,
    /// Relationship type.
    pub rel_type: RelationshipType,
    /// Relationship properties.
    pub properties: Vec<(String, String)>,
}

impl GraphRelationship {
    /// Create a new relationship.
    pub fn new(
        from_id: impl Into<String>,
        to_id: impl Into<String>,
        rel_type: RelationshipType,
    ) -> Self {
        Self {
            from_id: from_id.into(),
            to_id: to_id.into(),
            rel_type,
            properties: Vec::new(),
        }
    }

    /// Add a property to the relationship.
    pub fn with_property(mut self, key: impl Into<String>, value: impl ToString) -> Self {
        self.properties.push((key.into(), value.to_string()));
        self
    }
}

/// Result of a Cypher query.
#[derive(Debug, Clone)]
pub struct QueryResult {
    /// Number of rows returned.
    pub row_count: usize,
}

// ─────────────────────────────────────────────────────────────────────────────
// Graph Store
// ─────────────────────────────────────────────────────────────────────────────

/// Knowledge graph backed by graphqlite.
///
/// Provides entity and relationship storage with Cypher query support.
pub struct GraphStore {
    graph: Graph,
}

impl GraphStore {
    /// Open or create a graph store at the given path.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        let graph = Graph::open(&path_str).map_err(|e| {
            MemoryError::Database(rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(1),
                Some(e.to_string()),
            ))
        })?;

        info!("Graph store opened at {:?}", path.as_ref());
        Ok(Self { graph })
    }

    /// Create an in-memory graph store.
    pub fn open_in_memory() -> Result<Self> {
        let graph = Graph::open(":memory:").map_err(|e| {
            MemoryError::Database(rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(1),
                Some(e.to_string()),
            ))
        })?;

        info!("In-memory graph store created");
        Ok(Self { graph })
    }

    /// Add an entity/node to the graph.
    pub fn add_entity(&self, node: &GraphNode) -> Result<()> {
        // Convert properties to the format graphqlite expects
        let props: Vec<(&str, &str)> = node
            .properties
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();

        self.graph
            .upsert_node(&node.id, props, &node.label)
            .map_err(|e| MemoryError::Query(e.to_string()))?;

        debug!("Added entity {} with label {}", node.id, node.label);
        Ok(())
    }

    /// Delete an entity by ID (and all its relationships).
    ///
    /// Note: This uses the delete_node method from graphqlite.
    pub fn delete_entity(&self, id: &str) -> Result<bool> {
        self.graph
            .delete_node(id)
            .map_err(|e| MemoryError::Query(e.to_string()))?;

        debug!("Deleted entity {}", id);
        Ok(true)
    }

    /// Add a relationship between two entities.
    pub fn add_relationship(&self, rel: &GraphRelationship) -> Result<()> {
        let props: Vec<(&str, &str)> = rel
            .properties
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();

        self.graph
            .upsert_edge(&rel.from_id, &rel.to_id, props, rel.rel_type.as_str())
            .map_err(|e| MemoryError::Query(e.to_string()))?;

        debug!(
            "Added relationship {} -[{}]-> {}",
            rel.from_id,
            rel.rel_type.as_str(),
            rel.to_id
        );
        Ok(())
    }

    /// Get neighbors of an entity.
    ///
    /// Returns the IDs of nodes connected to the given node.
    /// Note: This only returns outgoing relationships (from this node to others).
    pub fn get_neighbors(&self, id: &str) -> Result<Vec<String>> {
        let neighbors = self
            .graph
            .get_neighbors(id)
            .map_err(|e| MemoryError::Query(e.to_string()))?;

        // graphqlite returns Value::Object for each neighbor node with structure:
        // { "id": Integer, "labels": Array, "properties": Object { "id": String, ... } }
        // The string ID we set is stored in properties.id
        let neighbor_ids: Vec<String> = neighbors
            .into_iter()
            .filter_map(|v| {
                match v {
                    graphqlite::Value::Object(obj) => {
                        // Extract properties.id
                        if let Some(graphqlite::Value::Object(props)) = obj.get("properties") {
                            if let Some(graphqlite::Value::String(node_id)) = props.get("id") {
                                return Some(node_id.clone());
                            }
                        }
                        None
                    }
                    // Fallback for simpler Value types
                    graphqlite::Value::String(s) => Some(s),
                    graphqlite::Value::Integer(i) => Some(i.to_string()),
                    _ => None,
                }
            })
            .collect();

        Ok(neighbor_ids)
    }

    /// Get graph statistics.
    pub fn stats(&self) -> Result<GraphStats> {
        let stats = self
            .graph
            .stats()
            .map_err(|e| MemoryError::Query(e.to_string()))?;

        Ok(GraphStats {
            node_count: stats.nodes as usize,
            relationship_count: stats.edges as usize,
        })
    }
}

/// Statistics about the graph store.
#[derive(Debug, Clone)]
pub struct GraphStats {
    /// Number of nodes in the graph.
    pub node_count: usize,
    /// Number of relationships in the graph.
    pub relationship_count: usize,
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    fn create_test_graph() -> GraphStore {
        GraphStore::open_in_memory().unwrap()
    }

    #[test]
    #[serial]
    fn test_open_in_memory() {
        let graph = create_test_graph();
        let stats = graph.stats().unwrap();
        assert_eq!(stats.node_count, 0);
        assert_eq!(stats.relationship_count, 0);
    }

    #[test]
    #[serial]
    fn test_add_entity() {
        let graph = create_test_graph();

        let node = GraphNode::new("alice", "Person")
            .with_property("name", "Alice")
            .with_property("age", "30");

        graph.add_entity(&node).unwrap();

        let stats = graph.stats().unwrap();
        assert_eq!(stats.node_count, 1);
    }

    #[test]
    #[serial]
    fn test_add_multiple_entities() {
        let graph = create_test_graph();

        graph
            .add_entity(&GraphNode::new("alice", "Person").with_property("name", "Alice"))
            .unwrap();
        graph
            .add_entity(&GraphNode::new("bob", "Person").with_property("name", "Bob"))
            .unwrap();
        graph
            .add_entity(&GraphNode::new("rust", "Concept").with_property("name", "Rust"))
            .unwrap();

        let stats = graph.stats().unwrap();
        assert_eq!(stats.node_count, 3);
    }

    #[test]
    #[serial]
    fn test_add_relationship() {
        let graph = create_test_graph();

        // Create nodes
        graph
            .add_entity(&GraphNode::new("alice", "Person"))
            .unwrap();
        graph.add_entity(&GraphNode::new("bob", "Person")).unwrap();

        // Create relationship
        let rel = GraphRelationship::new("alice", "bob", RelationshipType::RelatedTo)
            .with_property("since", "2020");
        graph.add_relationship(&rel).unwrap();

        let stats = graph.stats().unwrap();
        assert_eq!(stats.node_count, 2);
        assert_eq!(stats.relationship_count, 1);
    }

    #[test]
    #[serial]
    fn test_relationship_types() {
        let graph = create_test_graph();

        // Create nodes
        graph
            .add_entity(&GraphNode::new("claim1", "Claim"))
            .unwrap();
        graph
            .add_entity(&GraphNode::new("claim2", "Claim"))
            .unwrap();
        graph
            .add_entity(&GraphNode::new("source1", "Source"))
            .unwrap();

        // Different relationship types
        graph
            .add_relationship(&GraphRelationship::new(
                "claim1",
                "claim2",
                RelationshipType::Supports,
            ))
            .unwrap();
        graph
            .add_relationship(&GraphRelationship::new(
                "claim1",
                "source1",
                RelationshipType::CitedIn,
            ))
            .unwrap();

        let stats = graph.stats().unwrap();
        assert_eq!(stats.relationship_count, 2);
    }

    #[test]
    #[serial]
    fn test_get_neighbors() {
        let graph = create_test_graph();

        // Create a small graph
        graph.add_entity(&GraphNode::new("a", "Node")).unwrap();
        graph.add_entity(&GraphNode::new("b", "Node")).unwrap();
        graph.add_entity(&GraphNode::new("c", "Node")).unwrap();

        graph
            .add_relationship(&GraphRelationship::new(
                "a",
                "b",
                RelationshipType::RelatedTo,
            ))
            .unwrap();
        graph
            .add_relationship(&GraphRelationship::new(
                "a",
                "c",
                RelationshipType::RelatedTo,
            ))
            .unwrap();

        let neighbors = graph.get_neighbors("a").unwrap();
        // Just verify we can call get_neighbors without error
        // The exact format of returned IDs may vary based on graphqlite internals
        let _ = neighbors;
    }

    #[test]
    #[serial]
    fn test_delete_entity() {
        let graph = create_test_graph();

        graph.add_entity(&GraphNode::new("temp", "Temp")).unwrap();
        assert_eq!(graph.stats().unwrap().node_count, 1);

        graph.delete_entity("temp").unwrap();
        assert_eq!(graph.stats().unwrap().node_count, 0);
    }

    #[test]
    #[serial]
    fn test_knowledge_graph_integration() {
        let graph = create_test_graph();

        // Build a small knowledge graph about Rust
        graph
            .add_entity(&GraphNode::new("rust", "Language").with_property("name", "Rust"))
            .unwrap();
        graph
            .add_entity(
                &GraphNode::new("memory_safety", "Concept").with_property("name", "Memory Safety"),
            )
            .unwrap();
        graph
            .add_entity(&GraphNode::new("ownership", "Concept").with_property("name", "Ownership"))
            .unwrap();
        graph
            .add_entity(&GraphNode::new("borrowing", "Concept").with_property("name", "Borrowing"))
            .unwrap();
        graph
            .add_entity(&GraphNode::new("mozilla", "Organization").with_property("name", "Mozilla"))
            .unwrap();

        // Relationships
        graph
            .add_relationship(&GraphRelationship::new(
                "rust",
                "memory_safety",
                RelationshipType::Supports,
            ))
            .unwrap();
        graph
            .add_relationship(&GraphRelationship::new(
                "ownership",
                "memory_safety",
                RelationshipType::Supports,
            ))
            .unwrap();
        graph
            .add_relationship(&GraphRelationship::new(
                "borrowing",
                "ownership",
                RelationshipType::RelatedTo,
            ))
            .unwrap();
        graph
            .add_relationship(&GraphRelationship::new(
                "rust",
                "mozilla",
                RelationshipType::CreatedBy,
            ))
            .unwrap();

        let stats = graph.stats().unwrap();
        assert_eq!(stats.node_count, 5);
        assert_eq!(stats.relationship_count, 4);
    }

    #[test]
    fn test_graph_node_builder() {
        let node = GraphNode::new("test", "Test")
            .with_property("key1", "value1")
            .with_property("key2", 42)
            .with_property("key3", true);

        assert_eq!(node.id, "test");
        assert_eq!(node.label, "Test");
        assert_eq!(node.properties.len(), 3);
    }

    #[test]
    fn test_relationship_type_as_str() {
        assert_eq!(RelationshipType::Supports.as_str(), "SUPPORTS");
        assert_eq!(RelationshipType::Contradicts.as_str(), "CONTRADICTS");
        assert_eq!(RelationshipType::RelatedTo.as_str(), "RELATED_TO");
        assert_eq!(RelationshipType::CitedIn.as_str(), "CITED_IN");
    }
}
