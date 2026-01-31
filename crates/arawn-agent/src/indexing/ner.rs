//! Local NER (Named Entity Recognition) engine abstraction.
//!
//! Provides a trait-based interface for local NER engines like GLiNER,
//! mapping their output to Arawn's extraction types. This enables hybrid
//! extraction: fast local NER for entities/relationships, LLM for facts only.

use std::collections::HashMap;

use super::types::{ExtractedEntity, ExtractedRelationship};

/// A recognized entity span from NER inference.
#[derive(Debug, Clone)]
pub struct NerSpan {
    /// The entity text as it appears in the input.
    pub text: String,
    /// The entity class label (e.g., "person", "language", "tool").
    pub label: String,
    /// Confidence score from the model (0.0 to 1.0).
    pub score: f32,
}

/// A recognized relationship between two entities.
#[derive(Debug, Clone)]
pub struct NerRelation {
    /// Subject entity text.
    pub subject: String,
    /// Relation label (e.g., "uses", "founded", "written_in").
    pub relation: String,
    /// Object entity text.
    pub object: String,
    /// Confidence score from the model (0.0 to 1.0).
    pub score: f32,
}

/// Output from NER engine inference.
#[derive(Debug, Clone, Default)]
pub struct NerOutput {
    /// Recognized entity spans.
    pub entities: Vec<NerSpan>,
    /// Recognized relationships (if the engine supports relation extraction).
    pub relations: Vec<NerRelation>,
}

/// Entity labels used for NER inference in Arawn's domain.
pub const ENTITY_LABELS: &[&str] = &[
    "person",
    "tool",
    "language",
    "project",
    "concept",
    "organization",
    "file",
    "config",
];

/// Relation labels for relation extraction.
pub const RELATION_LABELS: &[&str] = &[
    "uses",
    "depends_on",
    "written_in",
    "part_of",
    "created_by",
    "supports",
    "is_a",
    "mentions",
];

/// Trait for local NER inference engines.
///
/// Implementations wrap specific NER backends (e.g., GLiNER via gline-rs)
/// and provide entity/relationship extraction without LLM calls.
pub trait NerEngine: Send + Sync {
    /// Run NER inference on the given texts.
    ///
    /// `texts` are the input text segments to analyze.
    /// `entity_labels` are the entity types to recognize.
    ///
    /// Returns extracted spans and optionally relationships.
    fn extract(&self, texts: &[&str], entity_labels: &[&str]) -> Result<NerOutput, String>;

    /// Whether this engine supports relation extraction.
    fn supports_relations(&self) -> bool {
        false
    }

    /// Run relation extraction on the given texts.
    ///
    /// Default implementation returns empty results.
    /// Engines that support relations should override this.
    fn extract_relations(
        &self,
        texts: &[&str],
        entity_labels: &[&str],
        relation_labels: &[&str],
    ) -> Result<NerOutput, String> {
        // Default: just run entity extraction, no relations
        let _ = relation_labels;
        self.extract(texts, entity_labels)
    }
}

/// Configuration for the NER engine.
#[derive(Debug, Clone)]
pub struct NerConfig {
    /// Path to the ONNX model file.
    pub model_path: String,
    /// Path to the tokenizer JSON file.
    pub tokenizer_path: String,
    /// Minimum confidence threshold for accepting spans (0.0 to 1.0).
    pub threshold: f32,
}

impl Default for NerConfig {
    fn default() -> Self {
        Self {
            model_path: String::new(),
            tokenizer_path: String::new(),
            threshold: 0.5,
        }
    }
}

/// Convert NER output to Arawn's extraction types.
///
/// Maps `NerSpan` → `ExtractedEntity` and `NerRelation` → `ExtractedRelationship`,
/// deduplicating entities by normalized name.
pub fn ner_output_to_extracted(output: &NerOutput, threshold: f32) -> NerExtraction {
    let mut entity_map: HashMap<String, ExtractedEntity> = HashMap::new();

    for span in &output.entities {
        if span.score < threshold {
            continue;
        }
        let key = span.text.to_lowercase();
        // Keep the highest-scoring occurrence of each entity
        let existing_score = entity_map
            .get(&key)
            .and_then(|e| e.context.as_ref())
            .and_then(|c| c.parse::<f32>().ok())
            .unwrap_or(0.0);

        if span.score > existing_score {
            entity_map.insert(
                key,
                ExtractedEntity {
                    name: span.text.clone(),
                    entity_type: span.label.clone(),
                    // Store score in context for dedup comparison
                    context: Some(format!("{:.3}", span.score)),
                },
            );
        }
    }

    // Clean context field — replace score with None for final output
    let entities: Vec<ExtractedEntity> = entity_map
        .into_values()
        .map(|mut e| {
            e.context = None;
            e
        })
        .collect();

    let relationships: Vec<ExtractedRelationship> = output
        .relations
        .iter()
        .filter(|r| r.score >= threshold)
        .map(|r| ExtractedRelationship {
            from: r.subject.clone(),
            relation: r.relation.clone(),
            to: r.object.clone(),
        })
        .collect();

    NerExtraction {
        entities,
        relationships,
    }
}

/// Entities and relationships extracted by the NER engine.
#[derive(Debug, Clone, Default)]
pub struct NerExtraction {
    pub entities: Vec<ExtractedEntity>,
    pub relationships: Vec<ExtractedRelationship>,
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockNerEngine {
        output: NerOutput,
        supports_rels: bool,
    }

    impl MockNerEngine {
        fn new(output: NerOutput) -> Self {
            Self {
                output,
                supports_rels: false,
            }
        }

        fn with_relations(mut self) -> Self {
            self.supports_rels = true;
            self
        }
    }

    impl NerEngine for MockNerEngine {
        fn extract(&self, _texts: &[&str], _entity_labels: &[&str]) -> Result<NerOutput, String> {
            Ok(self.output.clone())
        }

        fn supports_relations(&self) -> bool {
            self.supports_rels
        }

        fn extract_relations(
            &self,
            _texts: &[&str],
            _entity_labels: &[&str],
            _relation_labels: &[&str],
        ) -> Result<NerOutput, String> {
            Ok(self.output.clone())
        }
    }

    #[test]
    fn test_ner_output_to_extracted_entities() {
        let output = NerOutput {
            entities: vec![
                NerSpan {
                    text: "Rust".to_string(),
                    label: "language".to_string(),
                    score: 0.95,
                },
                NerSpan {
                    text: "SQLite".to_string(),
                    label: "tool".to_string(),
                    score: 0.88,
                },
            ],
            relations: vec![],
        };

        let result = ner_output_to_extracted(&output, 0.5);
        assert_eq!(result.entities.len(), 2);
        assert!(result.relationships.is_empty());

        let names: Vec<&str> = result.entities.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"Rust"));
        assert!(names.contains(&"SQLite"));
    }

    #[test]
    fn test_ner_output_filters_by_threshold() {
        let output = NerOutput {
            entities: vec![
                NerSpan {
                    text: "Rust".to_string(),
                    label: "language".to_string(),
                    score: 0.95,
                },
                NerSpan {
                    text: "maybe".to_string(),
                    label: "concept".to_string(),
                    score: 0.3,
                },
            ],
            relations: vec![
                NerRelation {
                    subject: "Arawn".to_string(),
                    relation: "uses".to_string(),
                    object: "Rust".to_string(),
                    score: 0.9,
                },
                NerRelation {
                    subject: "X".to_string(),
                    relation: "mentions".to_string(),
                    object: "Y".to_string(),
                    score: 0.2,
                },
            ],
        };

        let result = ner_output_to_extracted(&output, 0.5);
        assert_eq!(result.entities.len(), 1);
        assert_eq!(result.entities[0].name, "Rust");
        assert_eq!(result.relationships.len(), 1);
        assert_eq!(result.relationships[0].from, "Arawn");
    }

    #[test]
    fn test_ner_output_deduplicates_entities() {
        let output = NerOutput {
            entities: vec![
                NerSpan {
                    text: "Rust".to_string(),
                    label: "language".to_string(),
                    score: 0.7,
                },
                NerSpan {
                    text: "Rust".to_string(),
                    label: "language".to_string(),
                    score: 0.95,
                },
            ],
            relations: vec![],
        };

        let result = ner_output_to_extracted(&output, 0.5);
        assert_eq!(result.entities.len(), 1);
    }

    #[test]
    fn test_ner_output_empty() {
        let output = NerOutput::default();
        let result = ner_output_to_extracted(&output, 0.5);
        assert!(result.entities.is_empty());
        assert!(result.relationships.is_empty());
    }

    #[test]
    fn test_mock_ner_engine_extract() {
        let output = NerOutput {
            entities: vec![NerSpan {
                text: "Python".to_string(),
                label: "language".to_string(),
                score: 0.92,
            }],
            relations: vec![],
        };
        let engine = MockNerEngine::new(output);

        let result = engine.extract(&["I use Python"], ENTITY_LABELS).unwrap();
        assert_eq!(result.entities.len(), 1);
        assert_eq!(result.entities[0].text, "Python");
    }

    #[test]
    fn test_mock_ner_engine_relations() {
        let output = NerOutput {
            entities: vec![
                NerSpan {
                    text: "Bill".to_string(),
                    label: "person".to_string(),
                    score: 0.99,
                },
                NerSpan {
                    text: "Microsoft".to_string(),
                    label: "organization".to_string(),
                    score: 0.98,
                },
            ],
            relations: vec![NerRelation {
                subject: "Bill".to_string(),
                relation: "created_by".to_string(),
                object: "Microsoft".to_string(),
                score: 0.97,
            }],
        };
        let engine = MockNerEngine::new(output).with_relations();

        assert!(engine.supports_relations());
        let result = engine
            .extract_relations(&["Bill founded Microsoft"], ENTITY_LABELS, RELATION_LABELS)
            .unwrap();
        assert_eq!(result.entities.len(), 2);
        assert_eq!(result.relations.len(), 1);
    }

    #[test]
    fn test_entity_labels_defined() {
        assert!(ENTITY_LABELS.contains(&"person"));
        assert!(ENTITY_LABELS.contains(&"tool"));
        assert!(ENTITY_LABELS.contains(&"language"));
        assert!(ENTITY_LABELS.contains(&"project"));
        assert_eq!(ENTITY_LABELS.len(), 8);
    }

    #[test]
    fn test_relation_labels_defined() {
        assert!(RELATION_LABELS.contains(&"uses"));
        assert!(RELATION_LABELS.contains(&"written_in"));
        assert_eq!(RELATION_LABELS.len(), 8);
    }

    #[test]
    fn test_ner_config_default() {
        let config = NerConfig::default();
        assert!(config.model_path.is_empty());
        assert_eq!(config.threshold, 0.5);
    }

    #[test]
    fn test_context_cleaned_in_output() {
        let output = NerOutput {
            entities: vec![NerSpan {
                text: "Rust".to_string(),
                label: "language".to_string(),
                score: 0.9,
            }],
            relations: vec![],
        };

        let result = ner_output_to_extracted(&output, 0.5);
        // Context should be None in final output (score was only used internally for dedup)
        assert!(result.entities[0].context.is_none());
    }
}
