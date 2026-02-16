---
id: context-management-integration
level: task
title: "Context management integration testing"
short_code: "ARAWN-T-0191"
created_at: 2026-02-16T18:54:57.891392+00:00
updated_at: 2026-02-16T18:54:57.891392+00:00
parent: ARAWN-I-0026
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0026
---

# Context management integration testing

## Parent Initiative

[[ARAWN-I-0026]] - Context Management and Auto-Compaction

## Objective

End-to-end integration tests for context tracking, auto-compaction, and `/compact` command flow.

## Acceptance Criteria

- [ ] Test: Long session triggers auto-compaction at 90% threshold
- [ ] Test: Compaction preserves recent N turns verbatim
- [ ] Test: `/compact` via REST API works
- [ ] Test: `/compact` via WebSocket works
- [ ] Test: Context indicator updates correctly after turns
- [ ] Test: Progress streaming during compaction
- [ ] Test: Cancellation produces partial results

## Test Cases

### TC-001: Auto-compaction at threshold
- **Preconditions**: Server running, session with max_context_tokens=1000
- **Steps**: 
  1. Create new session
  2. Send messages until ContextTracker reports >90%
  3. Send one more message
- **Expected**: SessionCompactor triggers automatically, older turns summarized, recent 3 turns preserved verbatim

### TC-002: Turn preservation during compaction
- **Preconditions**: Session with 10+ completed turns
- **Steps**: 
  1. Trigger compaction (manual or automatic)
  2. Inspect session state after compaction
- **Expected**: Last 3 turns unchanged, turns 1-7 replaced with summary, total token count reduced

### TC-003: REST /compact command
- **Preconditions**: Active session with history
- **Steps**: 
  1. POST /api/v1/commands/compact with session_id
  2. Consume SSE stream
- **Expected**: Progress events received, final result contains CompactionResult (turns_compacted, tokens_before, tokens_after)

### TC-004: WebSocket /compact command
- **Preconditions**: WebSocket connection to active session
- **Steps**: 
  1. Send WsCommandRequest { command: "compact", args: {} }
  2. Receive messages until WsCommandResult
- **Expected**: WsCommandProgress messages received, final WsCommandResult.success=true

### TC-005: Context indicator updates
- **Preconditions**: TUI connected to session
- **Steps**: 
  1. Send multiple messages
  2. Observe status bar after each turn
- **Expected**: Context percentage increases, color changes at 70% (yellow) and 90% (red)

### TC-006: Compaction progress streaming
- **Preconditions**: Session with many turns (for observable progress)
- **Steps**: 
  1. Trigger compaction
  2. Observe progress messages
- **Expected**: Progress messages show incremental percent, message describes current operation

### TC-007: Compaction cancellation
- **Preconditions**: Long-running compaction in progress
- **Steps**: 
  1. Start compaction
  2. Send cancel request mid-stream
- **Expected**: Partial result returned, session in consistent state, no data corruption

## Implementation Notes

### Test Scenarios

1. **Auto-compaction trigger**
   - Create session with low context limit (e.g., 1000 tokens)
   - Send messages until 90% reached
   - Verify compaction triggered automatically

2. **Turn preservation**
   - Compact session with 10 turns
   - Verify last 3 turns intact
   - Verify older turns summarized

3. **Command flow (REST)**
   - POST to /api/v1/commands/compact
   - Verify SSE progress stream
   - Verify final result

4. **Command flow (WS)**
   - Send WsCommandRequest
   - Verify WsCommandProgress messages
   - Verify WsCommandResult

### Dependencies
- All other ARAWN-I-0026 tasks

## Status Updates

*To be added during implementation*