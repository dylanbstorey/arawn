---
id: wire-sessionindexer-into-start
level: task
title: "Wire SessionIndexer into start command for production use"
short_code: "ARAWN-T-0107"
created_at: 2026-02-01T02:49:27.496967+00:00
updated_at: 2026-02-01T02:53:06.521830+00:00
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

# Wire SessionIndexer into start command for production use

## Parent Initiative

[[ARAWN-I-0017]]

## Objective

Connect the session intelligence pipeline (T-0095–T-0106) to the production server so `SessionIndexer` actually runs when sessions close. Currently all indexing code is library-only — `start.rs` never creates a `MemoryStore` or `SessionIndexer`, and `AppState` always has `indexer: None`.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `MemoryStore` created and initialized in start command
- [ ] `SessionIndexer` constructed with configured LLM backend + embedder
- [ ] Indexer passed to `AppState` via `with_indexer()`
- [ ] Embedder wired (replaces `let _embedder = embedder;`)
- [ ] `memory.database` config field for SQLite path
- [ ] NER engine created when `ner_model_path` is configured (behind `gliner` feature)
- [ ] `angreal check all` and `angreal test unit` pass
- [ ] Verbose startup logs show indexer status

## Implementation Plan

### 1. Add `database` field to `MemoryConfig` (`crates/arawn-config/src/types.rs`)

Add `pub database: Option<PathBuf>` to `MemoryConfig`. Default `None`, resolved to `memory.db` relative to data dir in start.rs.

### 2. Add `Server::from_state()` constructor (`crates/arawn-server/src/lib.rs`)

```rust
pub fn from_state(state: AppState) -> Self {
    Self { state }
}
```

`Server::new()` creates `AppState` internally with `indexer: None`. We need `from_state()` so start.rs can build `AppState` with the indexer attached before constructing `Server`.

### 3. Wire indexer in `start.rs` (`crates/arawn/src/commands/start.rs`)

After building agent, before creating server:

1. Read `config.memory` (or defaults)
2. If `memory.indexing.enabled`:
   a. Open `MemoryStore::open(memory_db_path)` + `init_graph()`
   b. Look up `memory.indexing.backend` in the `backends` HashMap (already built), fall back to `"default"`
   c. Build `IndexerConfig { model: memory.indexing.model, ..Default::default() }`
   d. Create `SessionIndexer::with_backend(store, backend, Some(embedder), config)`
   e. If `memory.indexing.ner_model_path` set + `#[cfg(feature = "gliner")]`: create `GlinerEngine`, call `.with_ner_engine()`
3. Build `AppState::new(agent, server_config).with_indexer(indexer)`, use `Server::from_state(state)`
4. Replace `let _embedder = embedder;` — embedder goes into SessionIndexer

### 4. Add `gliner` feature passthrough (`crates/arawn/Cargo.toml`)

Add `gliner = ["arawn-agent/gliner"]` feature flag.

## Files to Modify

| File | Change |
|------|--------|
| `crates/arawn-config/src/types.rs` | Add `database: Option<PathBuf>` to `MemoryConfig` |
| `crates/arawn-server/src/lib.rs` | Add `Server::from_state(AppState)` constructor |
| `crates/arawn/src/commands/start.rs` | Wire MemoryStore + SessionIndexer + embedder |
| `crates/arawn/Cargo.toml` | Add `gliner` feature passthrough |

## Key Type Signatures (working memory)

- `MemoryStore::open(path: impl AsRef<Path>) -> Result<Self>`
- `MemoryStore::init_graph(&mut self) -> Result<()>`
- `SessionIndexer::with_backend(store: Arc<MemoryStore>, backend: SharedBackend, embedder: Option<SharedEmbedder>, config: IndexerConfig) -> Self`
- `SessionIndexer::with_ner_engine(self, engine: Arc<dyn NerEngine>) -> Self`
- `IndexerConfig { model: String, max_extraction_tokens: u32, max_summary_tokens: u32 }`
- `AppState::new(agent, config).with_indexer(indexer) -> AppState`
- `Server::from_state(state: AppState) -> Self` (new)
- `GlinerEngine::new(config: &NerConfig) -> Result<Self, String>`
- `config.memory: Option<MemoryConfig>` — `memory.indexing.enabled`, `.backend`, `.model`, `.ner_model_path`, `.ner_tokenizer_path`, `.ner_threshold`

## Progress

*To be updated during implementation*