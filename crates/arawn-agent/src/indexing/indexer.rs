//! Session indexer orchestrator.
//!
//! Runs the full post-session indexing pipeline:
//! 1. Extract entities, facts, and relationships via LLM
//! 2. Store entities in the knowledge graph
//! 3. Store facts with contradiction detection and reinforcement
//! 4. Store relationships in the knowledge graph
//! 5. Generate and store a session summary

use std::sync::Arc;

use async_trait::async_trait;
use tracing::{info, warn};

use arawn_llm::backend::SharedBackend;
use arawn_llm::embeddings::SharedEmbedder;
use std::collections::HashMap;

use arawn_llm::types::{CompletionRequest, Message};
use arawn_memory::{
    Citation, ConfidenceSource, ContentType, GraphNode, GraphRelationship, Memory,
    MemoryConfidence, MemoryStore, Metadata, RelationshipType, StoreFactResult, StoreOptions,
};

use super::extraction::{ExtractionPrompt, FactsOnlyPrompt, parse_extraction};
use super::ner::{ENTITY_LABELS, NerEngine, RELATION_LABELS, ner_output_to_extracted};
use super::report::IndexReport;
use super::summarization::{SummarizationPrompt, clean_summary};
use super::types::{ExtractedEntity, ExtractedFact, ExtractedRelationship, ExtractionResult};

/// Configuration for the session indexer.
#[derive(Debug, Clone)]
pub struct IndexerConfig {
    /// Model name to use for LLM extraction/summarization.
    pub model: String,
    /// Maximum tokens for extraction response.
    pub max_extraction_tokens: u32,
    /// Maximum tokens for summarization response.
    pub max_summary_tokens: u32,
}

impl Default for IndexerConfig {
    fn default() -> Self {
        Self {
            model: String::new(),
            max_extraction_tokens: 4096,
            max_summary_tokens: 512,
        }
    }
}

/// Trait for LLM completion, enabling test mocking.
#[async_trait]
pub trait Completer: Send + Sync {
    async fn complete(&self, model: &str, prompt: &str, max_tokens: u32) -> Result<String, String>;
}

/// Production completer that uses the real LLM backend.
pub struct BackendCompleter {
    backend: SharedBackend,
}

impl BackendCompleter {
    pub fn new(backend: SharedBackend) -> Self {
        Self { backend }
    }
}

#[async_trait]
impl Completer for BackendCompleter {
    async fn complete(&self, model: &str, prompt: &str, max_tokens: u32) -> Result<String, String> {
        let request = CompletionRequest {
            model: model.to_string(),
            messages: vec![Message::user(prompt)],
            max_tokens,
            system: None,
            tools: vec![],
            tool_choice: None,
            stream: false,
            temperature: Some(0.3),
            top_p: None,
            top_k: None,
            stop_sequences: vec![],
            metadata: HashMap::new(),
        };
        let response = self
            .backend
            .complete(request)
            .await
            .map_err(|e| e.to_string())?;
        Ok(response.text())
    }
}

/// Orchestrates post-session indexing: extraction, graph storage, and summarization.
pub struct SessionIndexer {
    store: Arc<MemoryStore>,
    completer: Arc<dyn Completer>,
    embedder: Option<SharedEmbedder>,
    ner_engine: Option<Arc<dyn NerEngine>>,
    config: IndexerConfig,
}

impl SessionIndexer {
    /// Create a new SessionIndexer with the given dependencies.
    pub fn new(
        store: Arc<MemoryStore>,
        completer: Arc<dyn Completer>,
        embedder: Option<SharedEmbedder>,
        config: IndexerConfig,
    ) -> Self {
        Self {
            store,
            completer,
            embedder,
            ner_engine: None,
            config,
        }
    }

    /// Create a SessionIndexer using a real LLM backend.
    pub fn with_backend(
        store: Arc<MemoryStore>,
        backend: SharedBackend,
        embedder: Option<SharedEmbedder>,
        config: IndexerConfig,
    ) -> Self {
        Self::new(
            store,
            Arc::new(BackendCompleter::new(backend)),
            embedder,
            config,
        )
    }

    /// Get a reference to the underlying MemoryStore.
    pub fn store(&self) -> &Arc<MemoryStore> {
        &self.store
    }

    /// Set a local NER engine for hybrid extraction.
    ///
    /// When set, entity and relationship extraction uses the local NER engine
    /// instead of the LLM, and the LLM is only used for fact extraction.
    pub fn with_ner_engine(mut self, engine: Arc<dyn NerEngine>) -> Self {
        self.ner_engine = Some(engine);
        self
    }

    /// Run the full indexing pipeline for a session.
    ///
    /// `session_id` identifies the session being indexed.
    /// `messages` is the conversation history as `(role, content)` pairs.
    ///
    /// Individual pipeline steps are best-effort: errors are logged in the
    /// report but don't abort the remaining steps.
    pub async fn index_session(&self, session_id: &str, messages: &[(&str, &str)]) -> IndexReport {
        let mut report = IndexReport::default();

        if messages.is_empty() {
            return report;
        }

        // Step 1: Extract entities, facts, and relationships.
        // Hybrid mode: if NER engine is available, use it for entities/relationships
        // and use LLM only for facts. Otherwise, use LLM for everything.
        let extraction = if let Some(ner) = &self.ner_engine {
            self.run_hybrid_extraction(ner.as_ref(), messages).await
        } else {
            match self.run_extraction(messages).await {
                Ok(result) => result,
                Err(e) => {
                    warn!(error = %e, "Extraction failed, continuing with summarization only");
                    report.errors.push(format!("extraction: {e}"));
                    ExtractionResult::default()
                }
            }
        };

        // Step 2: Store entities in knowledge graph
        self.store_entities(session_id, &extraction.entities, &mut report);

        // Step 3: Store facts with contradiction detection
        self.store_facts(session_id, &extraction.facts, &mut report)
            .await;

        // Step 4: Store relationships in knowledge graph
        self.store_relationships(&extraction.relationships, &mut report);

        // Step 5: Generate and store summary
        self.store_summary(session_id, messages, &mut report).await;

        info!(
            session_id,
            entities = report.entities_stored,
            facts = report.total_facts(),
            relationships = report.relationships_stored,
            summary = report.summary_stored,
            errors = report.errors.len(),
            "Session indexing complete"
        );

        report
    }

    async fn run_extraction(&self, messages: &[(&str, &str)]) -> Result<ExtractionResult, String> {
        let prompt = ExtractionPrompt::build(messages);
        let raw = self
            .completer
            .complete(
                &self.config.model,
                &prompt,
                self.config.max_extraction_tokens,
            )
            .await?;
        Ok(parse_extraction(&raw))
    }

    /// Hybrid extraction: NER for entities/relationships, LLM for facts only.
    async fn run_hybrid_extraction(
        &self,
        ner: &dyn NerEngine,
        messages: &[(&str, &str)],
    ) -> ExtractionResult {
        // Concatenate message content for NER input
        let texts: Vec<String> = messages
            .iter()
            .map(|(_, content)| content.to_string())
            .collect();
        let text_refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();

        // Run NER for entities (and relationships if supported)
        let ner_output = if ner.supports_relations() {
            ner.extract_relations(&text_refs, ENTITY_LABELS, RELATION_LABELS)
        } else {
            ner.extract(&text_refs, ENTITY_LABELS)
        };

        let ner_result = match ner_output {
            Ok(output) => ner_output_to_extracted(&output, 0.5),
            Err(e) => {
                warn!(error = %e, "NER extraction failed, falling back to LLM-only");
                // Fallback: use LLM for everything
                return match self.run_extraction(messages).await {
                    Ok(result) => result,
                    Err(e2) => {
                        warn!(error = %e2, "LLM extraction also failed");
                        ExtractionResult::default()
                    }
                };
            }
        };

        // Run LLM for facts only, passing NER entities as context
        let entity_names: Vec<&str> = ner_result
            .entities
            .iter()
            .map(|e| e.name.as_str())
            .collect();
        let facts_prompt = FactsOnlyPrompt::build(messages, &entity_names);

        let facts = match self
            .completer
            .complete(
                &self.config.model,
                &facts_prompt,
                self.config.max_extraction_tokens,
            )
            .await
        {
            Ok(raw) => parse_extraction(&raw).facts,
            Err(e) => {
                warn!(error = %e, "Facts-only LLM extraction failed");
                vec![]
            }
        };

        info!(
            ner_entities = ner_result.entities.len(),
            ner_relationships = ner_result.relationships.len(),
            llm_facts = facts.len(),
            "Hybrid extraction complete"
        );

        ExtractionResult {
            entities: ner_result.entities,
            facts,
            relationships: ner_result.relationships,
        }
    }

    fn store_entities(
        &self,
        session_id: &str,
        entities: &[ExtractedEntity],
        report: &mut IndexReport,
    ) {
        let graph = match self.store.graph() {
            Some(g) => g,
            None => return,
        };

        for entity in entities {
            let mut node = GraphNode::new(
                entity.name.to_lowercase().replace(' ', "_"),
                &entity.entity_type,
            );
            if let Some(ctx) = &entity.context {
                node = node.with_property("context", ctx);
            }
            node = node.with_property("source_session", session_id);

            match graph.add_entity(&node) {
                Ok(()) => report.entities_stored += 1,
                Err(e) => {
                    warn!(entity = %entity.name, error = %e, "Failed to store entity");
                    report.errors.push(format!("entity '{}': {e}", entity.name));
                }
            }
        }
    }

    async fn store_facts(
        &self,
        session_id: &str,
        facts: &[ExtractedFact],
        report: &mut IndexReport,
    ) {
        for fact in facts {
            let content = format!("{} {} {}", fact.subject, fact.predicate, fact.object);

            let source = ConfidenceSource::from_db_str(&fact.confidence)
                .unwrap_or(ConfidenceSource::Inferred);

            // Create session citation for provenance tracking
            // message_index 0 indicates extracted from session as a whole
            let citation = Citation::session(session_id, 0);

            let memory = Memory::new(ContentType::Fact, &content)
                .with_session(session_id)
                .with_metadata(Metadata {
                    session_id: Some(session_id.to_string()),
                    subject: Some(fact.subject.clone()),
                    predicate: Some(fact.predicate.clone()),
                    ..Default::default()
                })
                .with_confidence(MemoryConfidence::with_source(source))
                .with_citation(citation);

            let embedding = self.embed_text(&content).await;
            let options = StoreOptions {
                embedding,
                entities: vec![],
            };

            match self.store.store_fact(&memory, options) {
                Ok(StoreFactResult::Inserted) => report.facts_inserted += 1,
                Ok(StoreFactResult::Reinforced { .. }) => report.facts_reinforced += 1,
                Ok(StoreFactResult::Superseded { superseded_ids }) => {
                    report.facts_superseded += superseded_ids.len();
                    report.facts_inserted += 1;
                }
                Err(e) => {
                    warn!(subject = %fact.subject, error = %e, "Failed to store fact");
                    report.errors.push(format!("fact '{}': {e}", fact.subject));
                }
            }
        }
    }

    fn store_relationships(
        &self,
        relationships: &[ExtractedRelationship],
        report: &mut IndexReport,
    ) {
        let graph = match self.store.graph() {
            Some(g) => g,
            None => return,
        };

        for rel in relationships {
            let from_id = rel.from.to_lowercase().replace(' ', "_");
            let to_id = rel.to.to_lowercase().replace(' ', "_");
            let rel_type = map_relationship_type(&rel.relation);

            let graph_rel = GraphRelationship::new(&from_id, &to_id, rel_type)
                .with_property("label", &rel.relation);

            match graph.add_relationship(&graph_rel) {
                Ok(()) => report.relationships_stored += 1,
                Err(e) => {
                    warn!(
                        from = %rel.from, to = %rel.to, relation = %rel.relation,
                        error = %e, "Failed to store relationship"
                    );
                    report
                        .errors
                        .push(format!("relationship '{} -> {}': {e}", rel.from, rel.to));
                }
            }
        }
    }

    async fn store_summary(
        &self,
        session_id: &str,
        messages: &[(&str, &str)],
        report: &mut IndexReport,
    ) {
        let prompt = match SummarizationPrompt::build(messages) {
            Some(p) => p,
            None => return,
        };

        let raw = match self
            .completer
            .complete(&self.config.model, &prompt, self.config.max_summary_tokens)
            .await
        {
            Ok(text) => text,
            Err(e) => {
                warn!(error = %e, "Summarization LLM call failed");
                report.errors.push(format!("summarization: {e}"));
                return;
            }
        };

        let summary = clean_summary(&raw);
        if summary.is_empty() {
            return;
        }

        // Create session citation for provenance tracking
        let citation = Citation::session(session_id, 0);

        let memory = Memory::new(ContentType::Summary, &summary)
            .with_session(session_id)
            .with_metadata(Metadata {
                session_id: Some(session_id.to_string()),
                ..Default::default()
            })
            .with_citation(citation);

        let embedding = self.embed_text(&summary).await;
        let options = StoreOptions {
            embedding,
            entities: vec![],
        };

        match self.store.store(&memory, options) {
            Ok(()) => report.summary_stored = true,
            Err(e) => {
                warn!(error = %e, "Failed to store summary");
                report.errors.push(format!("summary storage: {e}"));
            }
        }
    }

    async fn embed_text(&self, text: &str) -> Option<Vec<f32>> {
        match &self.embedder {
            Some(embedder) => match embedder.embed(text).await {
                Ok(v) => Some(v),
                Err(e) => {
                    warn!(error = %e, "Embedding failed");
                    None
                }
            },
            None => None,
        }
    }
}

/// Map an extracted relationship label to a `RelationshipType`.
fn map_relationship_type(label: &str) -> RelationshipType {
    match label.to_lowercase().as_str() {
        "uses" | "depends_on" | "requires" => RelationshipType::RelatedTo,
        "written_in" | "part_of" | "belongs_to" => RelationshipType::PartOf,
        "created_by" | "authored_by" | "maintained_by" => RelationshipType::CreatedBy,
        "is_a" | "type_of" | "kind_of" => RelationshipType::IsA,
        "supports" | "enables" | "provides" => RelationshipType::Supports,
        "contradicts" | "conflicts_with" | "replaces" => RelationshipType::Contradicts,
        "mentions" | "references" | "cites" => RelationshipType::Mentions,
        _ => RelationshipType::RelatedTo,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mock completer that returns pre-configured responses.
    struct MockCompleter {
        extraction_response: String,
        summary_response: String,
    }

    impl MockCompleter {
        fn new(extraction_json: &str, summary: &str) -> Self {
            Self {
                extraction_response: extraction_json.to_string(),
                summary_response: summary.to_string(),
            }
        }

        fn failing() -> Self {
            Self {
                extraction_response: String::new(),
                summary_response: String::new(),
            }
        }
    }

    #[async_trait]
    impl Completer for MockCompleter {
        async fn complete(
            &self,
            _model: &str,
            prompt: &str,
            _max_tokens: u32,
        ) -> Result<String, String> {
            // Distinguish extraction vs summarization by prompt content
            if prompt.contains("information extraction system") {
                if self.extraction_response.is_empty() {
                    Err("mock extraction failure".to_string())
                } else {
                    Ok(self.extraction_response.clone())
                }
            } else if prompt.contains("conversation summarizer") {
                if self.summary_response.is_empty() {
                    Err("mock summary failure".to_string())
                } else {
                    Ok(self.summary_response.clone())
                }
            } else {
                Err("unexpected prompt".to_string())
            }
        }
    }

    fn test_extraction_json() -> String {
        r#"{
            "entities": [
                {"name": "Rust", "entity_type": "language", "context": "Programming language"},
                {"name": "SQLite", "entity_type": "tool", "context": "Database"}
            ],
            "facts": [
                {"subject": "project.language", "predicate": "is", "object": "Rust", "confidence": "stated"},
                {"subject": "user.preference.db", "predicate": "prefers", "object": "SQLite", "confidence": "observed"}
            ],
            "relationships": [
                {"from": "Arawn", "relation": "written_in", "to": "Rust"},
                {"from": "Arawn", "relation": "uses", "to": "SQLite"}
            ]
        }"#
        .to_string()
    }

    fn test_indexer_config() -> IndexerConfig {
        IndexerConfig {
            model: "test-model".to_string(),
            ..Default::default()
        }
    }

    fn make_indexer(completer: impl Completer + 'static) -> SessionIndexer {
        let store = MemoryStore::open_in_memory().unwrap();
        SessionIndexer::new(
            Arc::new(store),
            Arc::new(completer),
            None,
            test_indexer_config(),
        )
    }

    fn make_indexer_with_graph(completer: impl Completer + 'static) -> SessionIndexer {
        let mut store = MemoryStore::open_in_memory().unwrap();
        store.init_graph().unwrap();
        SessionIndexer::new(
            Arc::new(store),
            Arc::new(completer),
            None,
            test_indexer_config(),
        )
    }

    #[tokio::test]
    async fn test_index_session_empty_messages() {
        let indexer = make_indexer(MockCompleter::new("{}", ""));
        let report = indexer.index_session("sess-1", &[]).await;
        assert_eq!(report.entities_stored, 0);
        assert_eq!(report.total_facts(), 0);
        assert!(!report.summary_stored);
        assert!(!report.has_errors());
    }

    #[tokio::test]
    async fn test_index_session_facts_stored() {
        let completer =
            MockCompleter::new(&test_extraction_json(), "Set up Rust project with SQLite.");
        let indexer = make_indexer(completer);
        let messages = &[
            ("user", "I'm building a Rust project with SQLite"),
            ("assistant", "Great, I'll help set that up."),
        ];

        let report = indexer.index_session("sess-1", messages).await;

        assert_eq!(report.facts_inserted, 2);
        assert_eq!(report.facts_reinforced, 0);
        assert_eq!(report.facts_superseded, 0);
        assert!(report.summary_stored);
        assert!(!report.has_errors());

        // Verify facts are in the store
        let facts = indexer
            .store
            .list_memories(Some(ContentType::Fact), 10, 0)
            .unwrap();
        assert_eq!(facts.len(), 2);

        // Verify summary is in the store
        let summaries = indexer
            .store
            .list_memories(Some(ContentType::Summary), 10, 0)
            .unwrap();
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].content, "Set up Rust project with SQLite.");
        assert_eq!(summaries[0].metadata.session_id, Some("sess-1".to_string()));
    }

    #[tokio::test]
    async fn test_index_session_with_graph() {
        let completer =
            MockCompleter::new(&test_extraction_json(), "Set up Rust project with SQLite.");
        let indexer = make_indexer_with_graph(completer);
        let messages = &[("user", "I use Rust and SQLite"), ("assistant", "Nice!")];

        let report = indexer.index_session("sess-1", messages).await;

        assert_eq!(report.entities_stored, 2);
        assert_eq!(report.relationships_stored, 2);
        assert_eq!(report.facts_inserted, 2);
        assert!(report.summary_stored);
        assert!(!report.has_errors());
    }

    #[tokio::test]
    async fn test_index_session_no_graph_skips_entities() {
        let completer = MockCompleter::new(&test_extraction_json(), "Summary text.");
        let indexer = make_indexer(completer);
        let messages = &[("user", "Hello"), ("assistant", "Hi")];

        let report = indexer.index_session("sess-1", messages).await;

        // No graph → entities and relationships silently skipped
        assert_eq!(report.entities_stored, 0);
        assert_eq!(report.relationships_stored, 0);
        // Facts still stored
        assert_eq!(report.facts_inserted, 2);
    }

    #[tokio::test]
    async fn test_index_session_extraction_failure_continues() {
        let completer = MockCompleter::failing();
        let indexer = make_indexer(completer);
        let messages = &[("user", "test"), ("assistant", "test")];

        let report = indexer.index_session("sess-1", messages).await;

        // Extraction failed but pipeline continued
        assert!(report.has_errors());
        assert!(report.errors.iter().any(|e| e.contains("extraction")));
        // Summarization also failed (mock returns empty for both)
        assert!(!report.summary_stored);
    }

    #[tokio::test]
    async fn test_index_session_fact_confidence_mapping() {
        let json = r#"{
            "entities": [],
            "facts": [
                {"subject": "x", "predicate": "is", "object": "y", "confidence": "stated"}
            ],
            "relationships": []
        }"#;
        let completer = MockCompleter::new(json, "Summary.");
        let indexer = make_indexer(completer);
        let messages = &[("user", "x is y")];

        indexer.index_session("sess-1", messages).await;

        let facts = indexer
            .store
            .list_memories(Some(ContentType::Fact), 10, 0)
            .unwrap();
        assert_eq!(facts.len(), 1);
        assert_eq!(facts[0].confidence.source, ConfidenceSource::Stated);
        assert_eq!(facts[0].metadata.subject, Some("x".to_string()));
        assert_eq!(facts[0].metadata.predicate, Some("is".to_string()));
    }

    #[tokio::test]
    async fn test_index_session_fact_reinforcement() {
        let json = r#"{
            "entities": [],
            "facts": [
                {"subject": "user.lang", "predicate": "is", "object": "Rust", "confidence": "stated"}
            ],
            "relationships": []
        }"#;
        // Share store across two indexer invocations
        let store = Arc::new(MemoryStore::open_in_memory().unwrap());

        let completer1 = MockCompleter::new(json, "Summary.");
        let indexer1 = SessionIndexer::new(
            store.clone(),
            Arc::new(completer1),
            None,
            test_indexer_config(),
        );

        // First session: insert
        let r1 = indexer1
            .index_session("sess-1", &[("user", "I use Rust")])
            .await;
        assert_eq!(r1.facts_inserted, 1);

        // Second session with same fact: reinforce
        let completer2 = MockCompleter::new(json, "Summary.");
        let indexer2 = SessionIndexer::new(
            store.clone(),
            Arc::new(completer2),
            None,
            test_indexer_config(),
        );
        let r2 = indexer2
            .index_session("sess-2", &[("user", "Still using Rust")])
            .await;
        assert_eq!(r2.facts_reinforced, 1);
        assert_eq!(r2.facts_inserted, 0);
    }

    #[tokio::test]
    async fn test_index_session_fact_supersession() {
        let json1 = r#"{
            "entities": [],
            "facts": [
                {"subject": "user.editor", "predicate": "is", "object": "Vim", "confidence": "stated"}
            ],
            "relationships": []
        }"#;
        let json2 = r#"{
            "entities": [],
            "facts": [
                {"subject": "user.editor", "predicate": "is", "object": "Neovim", "confidence": "stated"}
            ],
            "relationships": []
        }"#;

        // First indexing with Vim
        let completer1 = MockCompleter::new(json1, "Uses Vim.");
        let store = Arc::new(MemoryStore::open_in_memory().unwrap());
        let indexer1 = SessionIndexer::new(
            store.clone(),
            Arc::new(completer1),
            None,
            test_indexer_config(),
        );
        let r1 = indexer1
            .index_session("sess-1", &[("user", "I use Vim")])
            .await;
        assert_eq!(r1.facts_inserted, 1);

        // Second indexing with Neovim → supersedes Vim
        let completer2 = MockCompleter::new(json2, "Uses Neovim now.");
        let indexer2 = SessionIndexer::new(
            store.clone(),
            Arc::new(completer2),
            None,
            test_indexer_config(),
        );
        let r2 = indexer2
            .index_session("sess-2", &[("user", "Switched to Neovim")])
            .await;
        assert_eq!(r2.facts_superseded, 1);
        assert_eq!(r2.facts_inserted, 1);
    }

    use super::super::ner::{NerEngine, NerOutput, NerRelation, NerSpan};

    struct MockNer {
        output: NerOutput,
        supports_rels: bool,
    }

    impl NerEngine for MockNer {
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

    struct FailingNer;

    impl NerEngine for FailingNer {
        fn extract(&self, _texts: &[&str], _entity_labels: &[&str]) -> Result<NerOutput, String> {
            Err("NER model not loaded".to_string())
        }
    }

    fn make_indexer_with_ner(
        completer: impl Completer + 'static,
        ner: impl NerEngine + 'static,
    ) -> SessionIndexer {
        let store = MemoryStore::open_in_memory().unwrap();
        SessionIndexer::new(
            Arc::new(store),
            Arc::new(completer),
            None,
            test_indexer_config(),
        )
        .with_ner_engine(Arc::new(ner))
    }

    fn make_indexer_with_ner_and_graph(
        completer: impl Completer + 'static,
        ner: impl NerEngine + 'static,
    ) -> SessionIndexer {
        let mut store = MemoryStore::open_in_memory().unwrap();
        store.init_graph().unwrap();
        SessionIndexer::new(
            Arc::new(store),
            Arc::new(completer),
            None,
            test_indexer_config(),
        )
        .with_ner_engine(Arc::new(ner))
    }

    #[tokio::test]
    async fn test_hybrid_extraction_entities_from_ner() {
        let ner = MockNer {
            output: NerOutput {
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
            },
            supports_rels: false,
        };

        // LLM returns facts only (entities/relationships empty per hybrid contract)
        let facts_json = r#"{
            "entities": [],
            "facts": [
                {"subject": "project.language", "predicate": "is", "object": "Rust", "confidence": "stated"}
            ],
            "relationships": []
        }"#;
        let completer = MockCompleter::new(facts_json, "Summary.");
        let indexer = make_indexer_with_ner(completer, ner);

        let messages = &[("user", "I use Rust with SQLite"), ("assistant", "Nice!")];
        let report = indexer.index_session("sess-1", messages).await;

        // Facts from LLM
        assert_eq!(report.facts_inserted, 1);
        // Summary always from LLM
        assert!(report.summary_stored);
        assert!(!report.has_errors());
    }

    #[tokio::test]
    async fn test_hybrid_extraction_with_graph_stores_ner_entities() {
        let ner = MockNer {
            output: NerOutput {
                entities: vec![NerSpan {
                    text: "Rust".to_string(),
                    label: "language".to_string(),
                    score: 0.95,
                }],
                relations: vec![NerRelation {
                    subject: "Arawn".to_string(),
                    relation: "written_in".to_string(),
                    object: "Rust".to_string(),
                    score: 0.9,
                }],
            },
            supports_rels: true,
        };

        let facts_json = r#"{"entities": [], "facts": [], "relationships": []}"#;
        let completer = MockCompleter::new(facts_json, "Summary.");
        let indexer = make_indexer_with_ner_and_graph(completer, ner);

        let messages = &[("user", "Arawn is written in Rust")];
        let report = indexer.index_session("sess-1", messages).await;

        // NER entities stored in graph
        assert_eq!(report.entities_stored, 1);
        // NER relationships stored in graph
        assert_eq!(report.relationships_stored, 1);
    }

    #[tokio::test]
    async fn test_hybrid_extraction_ner_failure_falls_back_to_llm() {
        let completer = MockCompleter::new(&test_extraction_json(), "Summary.");
        let indexer = make_indexer_with_ner(completer, FailingNer);

        let messages = &[("user", "I use Rust"), ("assistant", "Nice!")];
        let report = indexer.index_session("sess-1", messages).await;

        // Fallback to LLM: should still get facts
        assert_eq!(report.facts_inserted, 2);
        assert!(report.summary_stored);
    }

    #[test]
    fn test_map_relationship_type() {
        assert!(matches!(
            map_relationship_type("uses"),
            RelationshipType::RelatedTo
        ));
        assert!(matches!(
            map_relationship_type("written_in"),
            RelationshipType::PartOf
        ));
        assert!(matches!(
            map_relationship_type("created_by"),
            RelationshipType::CreatedBy
        ));
        assert!(matches!(
            map_relationship_type("is_a"),
            RelationshipType::IsA
        ));
        assert!(matches!(
            map_relationship_type("supports"),
            RelationshipType::Supports
        ));
        assert!(matches!(
            map_relationship_type("contradicts"),
            RelationshipType::Contradicts
        ));
        assert!(matches!(
            map_relationship_type("mentions"),
            RelationshipType::Mentions
        ));
        // Unknown → RelatedTo fallback
        assert!(matches!(
            map_relationship_type("xyz_unknown"),
            RelationshipType::RelatedTo
        ));
    }
}
