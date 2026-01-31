---
id: implement-contextbuilder-for
level: task
title: "Implement ContextBuilder for context window management"
short_code: "ARAWN-T-0012"
created_at: 2026-01-28T03:20:08.772300+00:00
updated_at: 2026-01-28T03:39:12.890767+00:00
parent: ARAWN-I-0004
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0004
---

# Implement ContextBuilder for context window management

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[ARAWN-I-0004]]

## Objective

Implement `ContextBuilder` to construct LLM request context from session history. Manages fitting conversation history within token budgets and preparing messages for the LLM.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `ContextBuilder` struct with configurable max context tokens
- [ ] `build()` method: takes Session + user message, returns `CompletionRequest`
- [ ] Includes system prompt configuration
- [ ] Converts Session turns to LLM Message format
- [ ] Handles tool calls and tool results in message history
- [ ] Truncates old messages when exceeding token budget (keeps recent)
- [ ] Attaches tool definitions from registry to request
- [ ] Unit tests for context building with various session states
- [ ] `cargo test -p arawn-agent` passes

## Implementation Notes

### Features Implemented

- Token estimation (rough approximation: chars / 4)
- Session history to LLM message conversion
- Token budget management with truncation of old messages
- Tool call and tool result handling in message history
- Diagnostic methods: `count_messages()`, `estimate_session_tokens()`

### Files Created

- `crates/arawn-agent/src/context.rs` - ContextBuilder

## Status Updates

- Implemented ContextBuilder with token budget management
- History truncation keeps most recent turns
- All 45 tests passing