---
id: session-hydration-and-context
level: task
title: "Session Hydration and Context Assembly for Agent Turns"
short_code: "ARAWN-T-0063"
created_at: 2026-01-29T03:51:30.763940+00:00
updated_at: 2026-01-29T04:11:31.866644+00:00
parent: ARAWN-I-0018
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0018
---

# Session Hydration and Context Assembly for Agent Turns

## Parent Initiative

[[ARAWN-I-0018]]

## Objective

Build the context assembly pipeline that hydrates a workstream's history into the agent's conversation context for each turn. This bridges the workstream crate and the agent crate — given a workstream_id, produce the `Vec<Message>` the agent needs for its next LLM call.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `ContextAssembler` struct that takes a `WorkstreamManager` reference
- [ ] `assemble(workstream_id, max_tokens)` → returns `Vec<arawn_llm::Message>` ready for CompletionRequest
- [ ] Context includes: workstream summary (if exists) as system context, then recent session messages
- [ ] If history fits in `max_tokens`, include all messages from current session
- [ ] If history exceeds `max_tokens`, include workstream summary + most recent N messages that fit
- [ ] Maps `WorkstreamMessage` roles to `arawn_llm::Role` (User→User, Assistant→Assistant, AgentPush→Assistant, etc.)
- [ ] Agent builder accepts optional `WorkstreamManager` and `workstream_id` to enable hydration
- [ ] Agent turn loop calls assembler before building CompletionRequest (when workstream is configured)
- [ ] Unit tests: empty workstream, short history fits, long history truncated with summary, role mapping

## Implementation Notes

### Technical Approach

`ContextAssembler` in `crates/arawn-workstream/src/context.rs`. The assembler reads from `MessageStore` (JSONL) and `WorkstreamStore` (SQLite for summary). Token counting is approximate — use character count / 4 as a rough estimate until a proper tokenizer is wired in. The agent integration modifies `Agent::run_turn()` to prepend workstream context before the current user message. This is a read-only operation on the workstream — no writes during assembly.

### Dependencies

- ARAWN-T-0059 (message store for reading history)
- ARAWN-T-0060 (session manager for knowing current session boundaries)
- ARAWN-T-0062 (WorkstreamManager as the entry point)

## Status Updates

*To be added during implementation*