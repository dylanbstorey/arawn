---
id: active-recall-injection-in-agent
level: task
title: "Active recall injection in agent turn loop"
short_code: "ARAWN-T-0093"
created_at: 2026-01-31T02:41:43.899282+00:00
updated_at: 2026-01-31T03:55:52.489282+00:00
parent: ARAWN-I-0014
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0014
---

# Active recall injection in agent turn loop

## Objective

Before the first LLM call per user message, automatically embed the user's message, query relevant memories via `recall()`, and inject matching results as a system message for the agent to use as context.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Active recall runs once per user message, before the first LLM call (not per tool-use iteration)
- [ ] User message is embedded via the existing embedding provider
- [ ] `RecallQuery` built with config `threshold` (via `with_min_score`) and `limit`
- [ ] If matches found, formatted context injected as system message at position 1 in message list
- [ ] Gracefully skipped if: recall disabled in config, embeddings not initialized, or embedding fails
- [ ] No recall on empty/whitespace-only messages
- [ ] Tests: injection with results, injection with no results, disabled config, no embeddings

## Implementation Notes

### Files
- `crates/arawn-agent/src/agent.rs` — add recall step in turn loop before first LLM call
- `crates/arawn-agent/src/context.rs` — add `format_recall_context()` helper to format RecallResult into a concise system message

### Technical Approach
- `recall()` takes a pre-computed `Vec<f32>` embedding, not raw text — so we need to call the embedding provider first
- The agent already has access to `MemoryStore` and config — thread `RecallConfig` through
- Format: concise bullet list of recalled memories with timestamps, capped at config limit
- Guard: wrap in `if let Some(embedder) = &self.embedder { ... }` so it's a no-op without embeddings

### Dependencies
- ARAWN-T-0090 (with_min_score on RecallQuery)
- ARAWN-T-0092 (RecallConfig in arawn-config)

## Status Updates

### Session 2
- Fixed 4 compilation errors in recall test module:
  1. Added `serial_test = "3.2"` to arawn-agent dev-dependencies
  2. Added `name()` method to `FixedEmbedder` mock (required by Embedder trait)
  3. Replaced private `store.conn` access with public `store.insert_memory_with_embedding()` API
  4. Removed unused `mut` from `let store`
- All checks pass (`angreal check all`)
- All tests pass (`angreal test unit`) — 4 new recall tests confirmed running:
  - `test_recall_injects_context` — verifies memories are recalled and injected
  - `test_recall_no_results` — empty store handled gracefully
  - `test_recall_disabled_config` — disabled config skips recall
  - `test_recall_no_embedder` — missing embedder skips recall silently

*To be added during implementation*