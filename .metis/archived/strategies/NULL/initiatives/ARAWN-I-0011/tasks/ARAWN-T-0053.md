---
id: wire-interactionlogger-into-agent
level: task
title: "Wire InteractionLogger into Agent Turn Loop"
short_code: "ARAWN-T-0053"
created_at: 2026-01-29T02:40:02.453426+00:00
updated_at: 2026-01-29T02:58:55.982376+00:00
parent: ARAWN-I-0011
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0011
---

# Wire InteractionLogger into Agent Turn Loop

## Parent Initiative

[[ARAWN-I-0011]]

## Objective

Replace the ad-hoc TRACE-level logging in `agent.rs` with structured `InteractionRecord` writes via `InteractionLogger`. Every LLM call in the agent turn loop produces a complete interaction record capturing the full request/response cycle.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `Agent` accepts an `Arc<InteractionLogger>` (injected via `AgentBuilder` or constructor)
- [ ] Each LLM call in the turn loop creates an `InteractionRecord` with: session_id, turn_id, iteration number, full messages array, system prompt, tools available, model requested
- [ ] Response fields populated: model_used, response_content, stop_reason, input/output tokens, has_tool_use
- [ ] `ToolCallRecord` populated for each tool call in the response (tool_name, call_id, arguments)
- [ ] Tool execution results backfilled into the record (result_success, truncated result_content)
- [ ] Duration captured (time from LLM call start to response received)
- [ ] Existing TRACE-level message/response logging in agent.rs removed (replaced by interaction log)
- [ ] Existing INFO/DEBUG tracing for turn start/end/tool execution preserved (those are for console, not training)
- [ ] `cargo test` passes
- [ ] Manual test: run `arawn start`, send a chat, verify JSONL file contains interaction records

## Implementation Notes

### Technical Approach
- Add `interaction_logger: Option<Arc<InteractionLogger>>` to `Agent` struct
- In the turn loop, wrap the `self.llm.complete()` call with timing and record construction
- Tool results come after execution — need to collect them and either: (a) write a second "tool results" record, or (b) hold the record open and backfill. Option (a) is simpler.
- Consider two record types: `InteractionRecord` (LLM call) and a lighter `ToolExecutionRecord` that references the parent interaction by ID

### Dependencies
- ARAWN-T-0052 (InteractionLogger must exist first)

## Status Updates

*To be added during implementation*