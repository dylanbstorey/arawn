---
id: wire-embedding-dimensions-from
level: task
title: "Wire embedding dimensions from config through to OpenAiEmbedder"
short_code: "ARAWN-T-0227"
created_at: 2026-02-26T01:44:59.506912+00:00
updated_at: 2026-02-26T16:35:13.939662+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#tech-debt"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Wire embedding dimensions from config through to OpenAiEmbedder

## Objective

`OpenAiEmbedder::new()` (`arawn-llm/src/embeddings.rs:202-207`) hardcodes a model→dimension lookup for only 3 models. If a user configures a custom embedding model (e.g. via `base_url` override pointing at a compatible API), the dimensions will be wrong and there's no way to override them at runtime.

The config system (`EmbeddingConfig`, `EmbeddingOpenAiConfig`) already has a `dimensions` field that users can set in `arawn.toml`, and `EmbedderSpec` carries `dimensions: Option<usize>` — but `OpenAiEmbedder::new()` ignores it entirely and uses the hardcoded map.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P2 - Medium (nice to have)

### Technical Debt Impact
- **Current Problems**: Users with custom/proxy embedding endpoints or non-standard models get silently wrong dimension values. The hardcoded map only covers `text-embedding-3-small`, `text-embedding-3-large`, and `text-embedding-ada-002`.
- **Benefits of Fixing**: Any OpenAI-compatible embedding API works correctly. Config is the single source of truth for dimensions, matching the pattern established for LLM model defaults (ARAWN-T-0223 Group 4).
- **Risk Assessment**: Low — localized change. Existing defaults still work for known models; explicit config takes precedence.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] `OpenAiEmbedder::new()` accepts an optional `dimensions` override via `OpenAiEmbedderConfig.dimensions`, using it instead of the hardcoded model lookup when provided
- [x] `build_embedder()` passes `EmbedderSpec.dimensions` through to `OpenAiEmbedderConfig`
- [x] `EmbeddingConfig::effective_dimensions()` remains the authoritative source for dimension defaults (unchanged)
- [x] Hardcoded model→dimension map kept as fallback (not removed), but config override wins
- [x] Existing tests pass (99 in arawn-llm), 3 new tests cover dimensions override

## Implementation Notes

### Key Files
- `arawn-llm/src/embeddings.rs` — `OpenAiEmbedder::new()` (lines 202-207), `build_embedder()` (line 584+)
- `arawn-config/src/types.rs` — `EmbeddingConfig::effective_dimensions()` (line 561), `EmbeddingOpenAiConfig` (line 592)

### Technical Approach
1. Add `dimensions: Option<usize>` param to `OpenAiEmbedder::new()` (or to `OpenAiEmbedderConfig`)
2. If provided, use it; otherwise fall back to existing model→dimension map
3. In `build_embedder()`, pass `spec.dimensions` through when constructing `OpenAiEmbedder`

## Status Updates

### Session 1 — 2026-02-26

#### Files Modified
- `crates/arawn-llm/src/embeddings.rs` — Added `dimensions: Option<usize>` to `OpenAiEmbedderConfig`, `with_dimensions()` builder, updated `OpenAiEmbedder::new()` to prefer config dimensions over model lookup, wired `spec.dimensions` through in `build_embedder()`, added 3 tests
- `crates/arawn-llm/src/openai.rs` — Fixed pre-existing test compilation error (missing `use crate::Message` import)

#### Data Flow
```
arawn.toml [embedding.dimensions] or [embedding.openai.dimensions]
  → EmbeddingConfig::effective_dimensions()
    → build_embedder_spec() sets spec.dimensions
      → build_embedder() passes to OpenAiEmbedderConfig::with_dimensions()
        → OpenAiEmbedder::new() uses config.dimensions.unwrap_or(model_lookup)
```

#### Test Results
- All 99 arawn-llm tests pass
- Full workspace compiles clean