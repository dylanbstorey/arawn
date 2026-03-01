---
id: build-rlm-module-rlmspawner-and
level: task
title: "Build RLM module — RlmSpawner and types"
short_code: "ARAWN-T-0241"
created_at: 2026-03-01T16:27:47.024268+00:00
updated_at: 2026-03-01T19:22:27.967922+00:00
parent: ARAWN-I-0027
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0027
---

# Build RLM module — RlmSpawner and types

## Parent Initiative

[[ARAWN-I-0027]] — RLM Exploration Agent

## Objective

Create the `arawn-agent/src/rlm/` module that ties together the compaction orchestrator, filtered tool registry, and RLM-specific configuration into a cohesive exploration API. This is the public interface that `ExploreTool` (T-0242) will call.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `arawn-agent/src/rlm/mod.rs` module with public API
- [ ] `RlmSpawner` struct: holds backend, tool registry, default budget, compaction config
- [ ] `RlmSpawner::explore(query: &str) -> Result<ExplorationResult>` method
  - Creates an `Agent` with RLM system prompt
  - Filters tool registry to read-only tools (using T-0239)
  - Wraps in `CompactionOrchestrator` (from T-0240)
  - Runs to completion, returns summary + metadata
- [ ] `ExplorationResult` type: summary string + `ExplorationMetadata`
- [ ] `ExplorationMetadata` type: iterations_used, sources_explored, tokens_used, compactions_performed, model_used
- [ ] `RlmConfig` type: model, max_iterations, max_tokens, compaction_threshold, compaction_model, compaction_prompt
- [ ] RLM system prompt defined as a constant (research-focused, instructs agent to summarize and cite sources)
- [ ] Max 1 level of recursion — RLM cannot spawn sub-RLMs (no `explore` tool in filtered registry)
- [ ] Tests: spawner creates agent with correct config, exploration returns result, system prompt is set
- [ ] `angreal check all` passes

## Implementation Notes

### Files
- `crates/arawn-agent/src/rlm/mod.rs` (new)
- `crates/arawn-agent/src/rlm/types.rs` (new — ExplorationResult, ExplorationMetadata, RlmConfig)
- `crates/arawn-agent/src/rlm/prompt.rs` (new — RLM_SYSTEM_PROMPT constant)
- `crates/arawn-agent/src/lib.rs` (add `pub mod rlm;`)

### Dependencies
- ARAWN-T-0239 (ToolRegistry filtering)
- ARAWN-T-0240 (CompactionOrchestrator)

## Status Updates

### Session 1 — Implementation complete
- Created `crates/arawn-agent/src/rlm/` module with 3 files:
  - `mod.rs`: `RlmSpawner` struct with `explore()` method, `DEFAULT_READ_ONLY_TOOLS` constant
  - `types.rs`: `RlmConfig`, `ExplorationResult`, `ExplorationMetadata`
  - `prompt.rs`: `RLM_SYSTEM_PROMPT` constant (research-focused instructions)
- `RlmSpawner::explore()` wires together:
  - `ToolRegistry::filtered_by_names(DEFAULT_READ_ONLY_TOOLS)` for read-only access
  - `Agent::builder()` with RLM system prompt, max_iterations=1, optional model/budget
  - `CompactionOrchestrator` for explore→compact→continue cycles
- Read-only tools: file_read, glob, grep, web_fetch, web_search, memory_search, think
- Excluded (write/mutation): shell, file_write, delegate, note, workflow, catalog
- Builder methods: `with_config()`, `with_compaction_backend()`
- 6 tests: simple query, tool calls, tool filtering, custom config, metadata tokens, system prompt content
- Added `pub mod rlm` and re-exports to `lib.rs`
- `angreal check all` — clean (only pre-existing warnings)
- `angreal test unit` — full suite passes