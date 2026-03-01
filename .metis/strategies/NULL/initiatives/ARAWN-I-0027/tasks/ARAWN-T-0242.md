---
id: implement-exploretool
level: task
title: "Implement ExploreTool"
short_code: "ARAWN-T-0242"
created_at: 2026-03-01T16:27:47.805659+00:00
updated_at: 2026-03-01T19:22:51.015404+00:00
parent: ARAWN-I-0027
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/active"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0027
---

# Implement ExploreTool

## Parent Initiative

[[ARAWN-I-0027]] — RLM Exploration Agent

## Objective

Implement the `explore` tool that the main agent calls to trigger RLM exploration. This is a standard `Tool` trait implementation that wraps `RlmSpawner` and formats the result for injection into the main agent's context.

## Acceptance Criteria

## Acceptance Criteria

- [ ] `ExploreTool` struct implementing the `Tool` trait
- [ ] Tool name: `explore`
- [ ] Tool description: "Explore and research to answer a question. Returns compressed findings."
- [ ] Single required parameter: `query` (string)
- [ ] Calls `RlmSpawner::explore(query)` and returns the summary as tool output
- [ ] Includes metadata (iterations, tokens, sources) in tool output as a compact footer
- [ ] Handles errors gracefully (returns error result, doesn't panic)
- [ ] Tool is registerable in `ToolRegistry` like any other tool
- [ ] Tests: tool definition schema is correct, tool calls spawner and returns summary
- [ ] `angreal check all` passes

## Implementation Notes

### Files
- `crates/arawn-agent/src/tools/explore.rs` (new)
- `crates/arawn-agent/src/tools/mod.rs` (add module)

### Approach
1. `ExploreTool` holds an `Arc<RlmSpawner>`
2. `Tool::execute()` extracts `query` from input JSON, calls `spawner.explore(query).await`
3. Formats `ExplorationResult` as tool output: summary text + metadata footer
4. The `explore` tool must NOT be included in the RLM's own filtered tool registry (prevents recursive exploration)

### Dependencies
- ARAWN-T-0241 (RLM module / RlmSpawner)

## Status Updates

### Session 1
- Created `crates/arawn-agent/src/tools/explore.rs` with full `ExploreTool` implementation
  - `ExploreTool` struct holding `Arc<RlmSpawner>`
  - `Tool` trait impl: name="explore", single required "query" parameter
  - `execute()` calls `spawner.explore(query)`, formats summary + metadata footer (iterations, tokens, compactions, truncation)
  - Graceful error handling for missing/empty query and spawner failures
- Added `mod explore;` and `pub use explore::ExploreTool;` to `tools/mod.rs`
- Added `ExploreTool` re-export to `lib.rs`
- Fixed type mismatch in `test_explore_registerable` (`registry.names()` returns `Vec<&str>`)
- All 5 tests pass: tool_definition, explore_returns_summary, explore_missing_query, explore_empty_query, explore_registerable
- `angreal check all` passes clean
- `angreal test unit` passes: 437 arawn-agent tests, 1706 total workspace tests, 0 failures