---
id: embedding-configuration-and
level: task
title: "Embedding Configuration and Provider Selection"
short_code: "ARAWN-T-0068"
created_at: 2026-01-29T04:43:47.120341+00:00
updated_at: 2026-01-29T14:45:26.781509+00:00
parent: ARAWN-I-0015
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0015
---

# Embedding Configuration and Provider Selection

## Parent Initiative

[[ARAWN-I-0015]]

## Objective

Add `[embedding]` configuration to `arawn-config` and wire up provider selection in `arawn/src/commands/start.rs` so the correct `SharedEmbedder` is constructed from config and passed to the agent/memory system.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `EmbeddingConfig` struct in `crates/arawn-config/src/types.rs` with provider, model, dimensions, batch settings
- [ ] Config supports `[embedding]`, `[embedding.openai]`, `[embedding.gemini]`, `[embedding.batch]` TOML sections
- [ ] Sensible defaults: provider=local (ONNX, offline-first), dimensions=384
- [ ] `build_embedder(config: &EmbeddingConfig) -> Result<SharedEmbedder>` factory function in arawn-llm
- [ ] `start.rs` reads embedding config, constructs embedder, passes to MemoryStore/Agent
- [ ] API key resolution via config secret store (keyring) for openai/gemini providers
- [ ] Unit tests for config parsing and provider construction
- [ ] `LocalEmbedder` overrides `embed_batch()` with true ONNX batch inference (batched input tensors in single `session.run()`)
- [ ] `OpenAiEmbedder::embed_batch()` already uses native batch API (verify, no change needed)
- [ ] All existing tests pass

## Implementation Notes

### Technical Approach
- Add `EmbeddingConfig` to `crates/arawn-config/src/types.rs` alongside existing `LlmConfig`
- Add `build_embedder()` factory in `crates/arawn-llm/src/embeddings.rs` — matches on provider string, constructs the right embedder
- Wire into `start.rs`: load config → build embedder → pass to memory store initialization
- Default to `LocalEmbedder` (ONNX, MiniLM-L6-v2, 384d) — zero-config, offline-first
- Fall back to `MockEmbedder` only if local-embeddings feature is disabled and no API keys configured

### Dependencies
- Existing arawn-config infrastructure (layered config, keyring)

## Status Updates

*To be added during implementation*