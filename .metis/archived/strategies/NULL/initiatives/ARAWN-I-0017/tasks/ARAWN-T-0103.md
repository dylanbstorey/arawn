---
id: build-sessionindexer-orchestrator
level: task
title: "Build SessionIndexer orchestrator"
short_code: "ARAWN-T-0103"
created_at: 2026-01-31T04:09:07.638348+00:00
updated_at: 2026-02-01T03:51:16.447028+00:00
parent: ARAWN-I-0017
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0017
---

# Build SessionIndexer orchestrator

## Objective

Build the `SessionIndexer` struct that orchestrates the full post-session pipeline: extract entities/facts/relationships → store in knowledge graph → store facts with confidence/contradiction detection → generate and store summary.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `SessionIndexer` struct holding `Arc<MemoryStore>`, `Arc<dyn LlmBackend>`, `Option<SharedEmbedder>`, `IndexingConfig`
- [ ] `index_session(session_id) -> Result<IndexReport>` method that runs the full pipeline
- [ ] `IndexReport` struct: counts of entities/facts/relationships extracted, contradictions detected, summary generated
- [ ] Entities stored via knowledge graph `add_node()`
- [ ] Facts stored via `store_fact()` (triggers contradiction detection + reinforcement)
- [ ] Relationships stored via knowledge graph `add_edge()`
- [ ] Summary stored via summarization module
- [ ] Errors in individual steps logged but don't abort the pipeline (best-effort)
- [ ] Tests: mock LLM backend returns known extraction JSON, verify entities/facts/relationships stored correctly

## Implementation Notes

### Files
- `crates/arawn-agent/src/indexing/mod.rs` — SessionIndexer
- `crates/arawn-agent/src/indexing/report.rs` — IndexReport

### Dependencies
- ARAWN-T-0097 (contradiction detection / store_fact)
- ARAWN-T-0098 (reinforcement)
- ARAWN-T-0101 (extraction prompt + parser)
- ARAWN-T-0102 (summarization)

## Status Updates

### Session 1
- Created `crates/arawn-agent/src/indexing/report.rs` — `IndexReport` struct with entity/fact/relationship/summary counts, error tracking, Display impl
- Created `crates/arawn-agent/src/indexing/indexer.rs` — `SessionIndexer` orchestrator
  - `Completer` trait for LLM abstraction (enables test mocking)
  - `BackendCompleter` production impl wrapping `SharedBackend`
  - `IndexerConfig` (model, max_extraction_tokens, max_summary_tokens)
  - `SessionIndexer::new()` and `::with_backend()` constructors
  - `index_session(session_id, messages) -> IndexReport` — full pipeline:
    1. LLM extraction → parse entities/facts/relationships
    2. Store entities in knowledge graph via `GraphStore::add_entity()`
    3. Store facts via `MemoryStore::store_fact()` (triggers contradiction detection + reinforcement)
    4. Store relationships via `GraphStore::add_relationship()`
    5. LLM summarization → clean → store as `ContentType::Summary` memory
  - Best-effort: each step logs errors but doesn't abort remaining steps
  - Embeds facts and summaries if `SharedEmbedder` provided
  - `map_relationship_type()` maps extracted labels to `RelationshipType` enum
- Updated `indexing/mod.rs` to expose `SessionIndexer`, `IndexerConfig`, `Completer`, `IndexReport`
- 17 new tests (13 indexer + 4 report):
  - Empty messages, facts stored with correct metadata/confidence, graph storage, no-graph fallback
  - Extraction failure continues pipeline, confidence mapping, reinforcement across sessions, supersession
  - Relationship type mapping
- `angreal check all` passes, `angreal test unit` passes (741+ tests, 0 failures)