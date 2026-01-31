---
id: integrate-gliner-local-ner-via
level: task
title: "Integrate GLiNER local NER via gline-rs for entity and relationship extraction"
short_code: "ARAWN-T-0106"
created_at: 2026-01-31T14:14:21.237234+00:00
updated_at: 2026-02-01T03:51:17.811883+00:00
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

# Integrate GLiNER local NER via gline-rs for entity and relationship extraction

## Parent Initiative

[[ARAWN-I-0017]]

## Objective

Replace the LLM-only entity and relationship extraction in the indexing pipeline with a hybrid approach: use GLiNER (via `gline-rs` crate) for local, fast, zero-cost entity recognition and relationship extraction, reserving LLM calls for fact extraction only (which requires reasoning about predicates/confidence).

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `gline-rs` added as dependency to `arawn-agent` with CoreML feature flag
- [ ] `GlinerExtractor` struct that loads a GLiNER ONNX model and runs NER inference
- [ ] Entity types mapped to Arawn's domain: person, tool, language, project, concept, organization, file, config
- [ ] Relationship extraction using GLiNER multitask model with defined relation schema
- [ ] Output maps to existing `ExtractedEntity` and `ExtractedRelationship` types from T-0101
- [ ] Model path configurable via `[memory.indexing]` config section (from T-0100)
- [ ] Fallback: if GLiNER model not available, fall back to LLM-only extraction (existing T-0101 path)
- [ ] Unit tests with mock/small model or pre-computed ONNX outputs
- [ ] `angreal check all` and `angreal test unit` pass

## Implementation Notes

### Technical Approach

**Crate**: `gline-rs` v1.0.1 (Apache-2.0), wraps ONNX Runtime via `ort` crate.

**Model options**:
- `gliner-multitask-large-v0.5` — supports both NER and relation extraction in token mode
- `gliner_small-v2.1` — NER only, ~600MB, minimal footprint (span mode)

**API pattern**:
```rust
use gliner::model::{GLiNER, input::text::TextInput, params::Parameters};
use gliner::model::pipeline::span::SpanMode;

let model = GLiNER::<SpanMode>::new(params, runtime_params, tokenizer_path, model_path)?;
let input = TextInput::from_str(&texts, &entity_labels)?;
let output = model.inference(input)?;
// output.spans -> Vec<Vec<Span>> with .text(), .class(), .probability()
```

**Hybrid pipeline**:
1. GLiNER extracts entities + relationships (local, ~ms, free)
2. LLM extracts facts only (needs reasoning about subject/predicate/object/confidence)
3. LLM prompt receives GLiNER entities as context for better fact extraction

**Integration point**: New `GlinerExtractor` in `arawn-agent/src/indexing/` alongside existing `ExtractionPrompt`. The `SessionIndexer` (T-0103) will orchestrate both.

### Dependencies

- T-0101 (extraction types) — completed, provides `ExtractedEntity`, `ExtractedRelationship`
- T-0100 (config) — completed, provides `IndexingConfig` for model path config
- `gline-rs` 1.0.1 crate + `ort` ONNX runtime
- ONNX model files (downloaded separately, not bundled)

### Risk Considerations

- **ONNX model size**: ~600MB-1.5GB depending on model choice. Must be downloaded separately, not embedded. Config should point to model directory.
- **`ort` build complexity**: ONNX Runtime has platform-specific build requirements. CoreML feature flag for macOS. May need CI adjustments.
- **Token mode vs Span mode**: Multitask model uses token mode for relation extraction. May need to support both pipeline types.

## Status Updates

### Session 1 — Complete

**Implementation approach**: Built NER as a trait-based abstraction rather than directly depending on gline-rs, because gline-rs pins `ort =2.0.0-rc.9` while the workspace uses `ort 2.0.0-rc.11` — an incompatible exact version conflict. The abstraction enables plugging in gline-rs (or any other NER backend) once the ort version aligns.

**What was built:**

1. **`ner.rs`** — New NER engine abstraction module:
   - `NerEngine` trait: `extract()` and `extract_relations()` methods
   - `NerSpan`, `NerRelation`, `NerOutput` types for raw NER inference output
   - `NerConfig` for model path + tokenizer path + threshold
   - `ner_output_to_extracted()` — converts NER output to Arawn's `ExtractedEntity`/`ExtractedRelationship` types with deduplication and threshold filtering
   - `ENTITY_LABELS` and `RELATION_LABELS` constants for Arawn's domain
   - 10 unit tests (entities, filtering, dedup, empty, mock engine, relations, labels, config, context cleanup)

2. **`extraction.rs`** — Added `FactsOnlyPrompt`:
   - Facts-only LLM prompt for hybrid mode (when NER handles entities/relationships)
   - Includes NER entity names as context for better fact extraction
   - 2 new tests

3. **`indexer.rs`** — Hybrid extraction pipeline:
   - Added `Option<Arc<dyn NerEngine>>` field to `SessionIndexer`
   - Added `with_ner_engine()` builder method
   - `run_hybrid_extraction()`: NER for entities/relationships, LLM for facts only
   - Graceful fallback: if NER fails, falls back to full LLM extraction
   - 3 new tests (hybrid entities, hybrid with graph, NER failure fallback)

4. **`arawn-config/types.rs`** — Extended `IndexingConfig`:
   - Added `ner_model_path: Option<String>`, `ner_tokenizer_path: Option<String>`, `ner_threshold: f32`

5. **Re-exports**: `NerConfig`, `NerEngine`, `NerExtraction`, `NerOutput`, `NerRelation`, `NerSpan` from `arawn-agent/lib.rs`

**Files modified/created:**
- `crates/arawn-agent/src/indexing/ner.rs` (NEW)
- `crates/arawn-agent/src/indexing/mod.rs`
- `crates/arawn-agent/src/indexing/extraction.rs`
- `crates/arawn-agent/src/indexing/indexer.rs`
- `crates/arawn-agent/src/lib.rs`
- `crates/arawn-config/src/types.rs`

**Test results**: `angreal check all` passes, `angreal test unit` passes (all tests green, 0 failures)

**Note on gline-rs**: The concrete gline-rs `NerEngine` implementation is deferred until the `ort` version conflict is resolved (gline-rs pins `=2.0.0-rc.9`, workspace uses `2.0.0-rc.11`). The trait abstraction is ready — a `GlinerEngine` impl wrapping `GLiNER::<SpanMode>` or `GLiNER::<TokenMode>` can be added behind a feature flag when versions align.

### Session 2 — ort rc.11 Port

Successfully ported both `orp` and `gline-rs` to `ort 2.0.0-rc.11`:

**orp changes** (`/tmp/gline-rs-ref/orp`):
- `SessionOutputs<'_, '_>` → `SessionOutputs<'_>` (1 lifetime in rc.11)
- Private fields → accessor methods: `session.inputs()`, `session.outputs()`, `input.name()`, `input.dtype()`
- `metadata.name()?` → `metadata.name().unwrap_or_default()` (returns Option)
- `fn run/inference(&self)` → `fn run/inference(&mut self)` (Session::run needs &mut)
- Decoupled lifetime on `check_schema` to avoid borrow conflict

**gline-rs changes** (`/tmp/gline-rs-ref/gline-rs`):
- ndarray 0.16 → 0.17 (ort rc.11 uses 0.17)
- `ort::inputs!{ KEY => array }` → `ort::inputs!{ KEY => Tensor::from_array(array)? }`
- `shape()?` → `shape().to_vec()` (returns &Shape not Result)
- `try_extract_tensor::<f32>()` → `try_extract_array::<f32>()` (for ndarray views)
- `try_extract_raw_tensor::<f32>()` → `try_extract_tensor::<f32>()` (raw slice)
- `SessionOutputs<'a, 'a>` → `SessionOutputs<'a>`
- `expected_inputs/outputs` signature updated for new Pipeline trait

**Status**: Complete. Both ported, vendored, and GlinerEngine implemented.

### Session 2 — Vendoring & GlinerEngine

- Copied orp and gline-rs into `crates/orp/` and `crates/gline-rs/`
- Added as workspace path deps (excluded from workspace members to avoid test failures from missing model files)
- Removed gline-rs examples (not needed, require model files)
- Added `gliner` feature flag to arawn-agent gating `gline-rs` + `orp` deps
- Created `GlinerEngine` (`crates/arawn-agent/src/indexing/gliner.rs`):
  - Implements `NerEngine` trait using `GLiNER<SpanMode>`
  - Uses `Mutex<GLiNER<SpanMode>>` for interior mutability (inference needs `&mut self`)
  - Loads from `NerConfig` (model_path, tokenizer_path, threshold)
  - Maps GLiNER `Span` output to `NerSpan` types
- All tests pass (`angreal test unit`), all checks pass (`angreal check all`)