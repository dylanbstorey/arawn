//! Types for the extraction pipeline.

use serde::{Deserialize, Serialize};

/// Result of LLM extraction from a conversation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExtractionResult {
    /// Named entities mentioned in the conversation.
    #[serde(default)]
    pub entities: Vec<ExtractedEntity>,
    /// Facts derived from the conversation.
    #[serde(default)]
    pub facts: Vec<ExtractedFact>,
    /// Relationships between entities.
    #[serde(default)]
    pub relationships: Vec<ExtractedRelationship>,
}

/// An entity extracted from conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedEntity {
    /// Entity name (e.g., "Rust", "Alice", "PostgreSQL").
    pub name: String,
    /// Entity type (e.g., "language", "person", "tool").
    pub entity_type: String,
    /// Brief context about the entity from this conversation.
    #[serde(default)]
    pub context: Option<String>,
}

/// A fact extracted from conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedFact {
    /// Subject of the fact (e.g., "user.preferred_model").
    pub subject: String,
    /// Predicate (e.g., "is", "prefers", "uses").
    pub predicate: String,
    /// Object/value (e.g., "Claude", "dark mode").
    pub object: String,
    /// How confident we are in this fact.
    #[serde(default = "default_confidence")]
    pub confidence: String,
}

fn default_confidence() -> String {
    "inferred".to_string()
}

/// A relationship between two entities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedRelationship {
    /// Source entity name.
    pub from: String,
    /// Relationship type (e.g., "uses", "depends_on", "authored_by").
    pub relation: String,
    /// Target entity name.
    pub to: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extraction_result_deserialize() {
        let json = r#"{
            "entities": [
                {"name": "Rust", "entity_type": "language", "context": "User's primary language"}
            ],
            "facts": [
                {"subject": "user.language", "predicate": "is", "object": "Rust", "confidence": "stated"}
            ],
            "relationships": [
                {"from": "user", "relation": "prefers", "to": "Rust"}
            ]
        }"#;

        let result: ExtractionResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.entities.len(), 1);
        assert_eq!(result.entities[0].name, "Rust");
        assert_eq!(result.facts.len(), 1);
        assert_eq!(result.facts[0].subject, "user.language");
        assert_eq!(result.facts[0].confidence, "stated");
        assert_eq!(result.relationships.len(), 1);
    }

    #[test]
    fn test_extraction_result_missing_sections_default() {
        let json = r#"{"entities": [{"name": "Go", "entity_type": "language"}]}"#;
        let result: ExtractionResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.entities.len(), 1);
        assert!(result.facts.is_empty());
        assert!(result.relationships.is_empty());
    }

    #[test]
    fn test_extraction_result_empty() {
        let json = r#"{}"#;
        let result: ExtractionResult = serde_json::from_str(json).unwrap();
        assert!(result.entities.is_empty());
        assert!(result.facts.is_empty());
        assert!(result.relationships.is_empty());
    }

    #[test]
    fn test_fact_default_confidence() {
        let json = r#"{"subject": "x", "predicate": "is", "object": "y"}"#;
        let fact: ExtractedFact = serde_json::from_str(json).unwrap();
        assert_eq!(fact.confidence, "inferred");
    }

    #[test]
    fn test_entity_optional_context() {
        let json = r#"{"name": "Rust", "entity_type": "language"}"#;
        let entity: ExtractedEntity = serde_json::from_str(json).unwrap();
        assert!(entity.context.is_none());
    }
}
