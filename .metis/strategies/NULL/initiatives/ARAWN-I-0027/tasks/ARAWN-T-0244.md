---
id: rlm-integration-tests
level: task
title: "RLM integration tests"
short_code: "ARAWN-T-0244"
created_at: 2026-03-01T16:27:49.497372+00:00
updated_at: 2026-03-01T16:27:49.497372+00:00
parent: ARAWN-I-0027
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0027
---

# RLM integration tests

## Parent Initiative

[[ARAWN-I-0027]] — RLM Exploration Agent

## Objective

Write integration tests that exercise the full RLM exploration pipeline end-to-end, validating that all components work together correctly.

## Acceptance Criteria

- [ ] End-to-end exploration test: mock LLM + real tools (glob/grep on test fixtures), verify summary returned
- [ ] Compaction cycle test: mock LLM that generates enough output to trigger compaction, verify context is compressed and exploration continues
- [ ] Budget enforcement test: mock LLM that keeps calling tools, verify iteration and token limits stop execution
- [ ] Cancellation test: trigger cancellation mid-exploration, verify partial results returned gracefully
- [ ] Tool filtering test: verify write tools (shell, file_write) are excluded from RLM's registry, explore tool itself is excluded (no recursion)
- [ ] Config wiring test: verify `arawn.toml` `[rlm]` values flow through to RlmSpawner
- [ ] All tests in `angreal test unit` or `angreal test integration` as appropriate
- [ ] `angreal check all` passes

## Implementation Notes

### Files
- `crates/arawn-agent/tests/rlm_integration.rs` (new, or `src/rlm/tests.rs` for unit-level)

### Approach
- Use `MockBackend` for LLM responses — sequence tool-use responses followed by text responses
- Use real `ToolRegistry` with `MockTool` instances to simulate tool execution
- Test fixtures: small test files for glob/grep tests
- For compaction cycle: mock backend returns large tool results that push context past threshold, then compaction backend returns summary, then exploration continues

### Dependencies
- All other T-0239 through T-0243 tasks must be complete

## Status Updates

*To be added during implementation*