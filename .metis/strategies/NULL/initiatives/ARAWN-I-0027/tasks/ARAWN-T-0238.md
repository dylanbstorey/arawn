---
id: add-cumulative-token-budget
level: task
title: "Add cumulative token budget enforcement to Agent turn loop"
short_code: "ARAWN-T-0238"
created_at: 2026-03-01T15:57:39.831617+00:00
updated_at: 2026-03-01T16:25:05.260078+00:00
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

# Add cumulative token budget enforcement to Agent turn loop

## Parent Initiative

[[ARAWN-I-0027]] — RLM Exploration Agent

## Objective

Add a cumulative token budget as a safety valve on the `Agent` turn loop. The existing loop checks `max_iterations` (line 160 of `agent.rs`) but has no guard on total token consumption. A sub-agent (like the RLM) running with tools can burn through tokens quickly — we need a hard stop that says "you've used X tokens total, wrap it up."

This is generic infrastructure on `Agent`, not RLM-specific. Any agent type can set a token budget.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `AgentConfig` has an optional `max_total_tokens: Option<usize>` field
- [ ] When set, the turn loop checks `total_input_tokens + total_output_tokens` after each LLM response (after line 228)
- [ ] When budget is exceeded, the loop returns gracefully with the last response (same pattern as `max_iterations` exceeded)
- [ ] When not set (`None`), no token budget is enforced (no behavior change for existing callers)
- [ ] `AgentBuilder` has a `.with_max_total_tokens(n)` method
- [ ] Existing tests pass unchanged
- [ ] New test: token budget exceeded triggers graceful return
- [ ] New test: no budget set means no limit enforced
- [ ] `angreal check all` passes

## Implementation Notes

### Files
- `crates/arawn-agent/src/agent.rs` — `AgentConfig`, `AgentBuilder`, and the turn loop in `Agent::turn()`

### Approach
1. Add `max_total_tokens: Option<usize>` to `AgentConfig`, default `None`
2. Add `.with_max_total_tokens(n: usize)` to `AgentBuilder`
3. In the turn loop, after line 228 (where `total_input_tokens` and `total_output_tokens` are updated), add:
   ```rust
   if let Some(max) = self.config.max_total_tokens {
       if total_input_tokens + total_output_tokens > max {
           tracing::warn!(%session_id, %turn_id, tokens = total_input_tokens + total_output_tokens, max, "Token budget exceeded");
           // return gracefully, same pattern as max_iterations
       }
   }
   ```
4. The return on budget exceeded should mirror the existing `max_iterations` exceeded path — return the last response text, log a warning, not an error

### Scope
Small change — one config field, one builder method, one check in the loop, two tests.

## Status Updates

### Implementation Complete
- Added `max_total_tokens: Option<usize>` to `AgentConfig` with `None` default
- Added `with_max_total_tokens()` builder methods on both `AgentConfig` and `AgentBuilder`
- Added budget check in `Agent::turn()` after usage update — mirrors `max_iterations` exceeded pattern (graceful return with `truncated: true`)
- Streaming path (`create_turn_stream`) doesn't track tokens — out of scope, separate concern
- Added `test_turn_token_budget_exceeded` — verifies truncation when budget exceeded (50 token budget, 30 tokens/iteration, triggers on iteration 2)
- Added `test_turn_no_token_budget` — verifies no limit enforced when `None`
- `angreal check all` clean
- `angreal test unit` all pass
- No behavior change for existing callers (None = unlimited)