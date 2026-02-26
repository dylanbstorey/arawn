---
id: wire-embedding-dimensions-from
level: task
title: "Wire embedding dimensions from config through to OpenAiEmbedder"
short_code: "ARAWN-T-0227"
created_at: 2026-02-26T01:44:59.506912+00:00
updated_at: 2026-02-26T01:44:59.506912+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#tech-debt"


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

- [ ] `OpenAiEmbedder::new()` accepts an optional `dimensions` override, using it instead of the hardcoded model lookup when provided
- [ ] `build_embedder()` passes `EmbedderSpec.dimensions` through to `OpenAiEmbedder`
- [ ] `EmbeddingConfig::effective_dimensions()` remains the authoritative source for dimension defaults
- [ ] Hardcoded model→dimension map kept as fallback (not removed), but config override wins
- [ ] Existing tests pass, new test covers custom dimensions override

## Implementation Notes

### Key Files
- `arawn-llm/src/embeddings.rs` — `OpenAiEmbedder::new()` (lines 202-207), `build_embedder()` (line 584+)
- `arawn-config/src/types.rs` — `EmbeddingConfig::effective_dimensions()` (line 561), `EmbeddingOpenAiConfig` (line 592)

### Technical Approach
1. Add `dimensions: Option<usize>` param to `OpenAiEmbedder::new()` (or to `OpenAiEmbedderConfig`)
2. If provided, use it; otherwise fall back to existing model→dimension map
3. In `build_embedder()`, pass `spec.dimensions` through when constructing `OpenAiEmbedder`

## Status Updates

*To be added during implementation*