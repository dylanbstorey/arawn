---
id: fix-high-priority-data-integrity
level: task
title: "Fix High Priority Data Integrity Issues (Phase 2)"
short_code: "ARAWN-T-0171"
created_at: 2026-02-13T13:14:59.859045+00:00
updated_at: 2026-02-13T13:19:25.050356+00:00
parent: ARAWN-I-0025
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0025
---

# Fix High Priority Data Integrity Issues (Phase 2)

Fix 4 HIGH severity data integrity issues from the codebase audit.

## Parent Initiative

[[ARAWN-I-0025]]

## Objective

Ensure chat conversations persist correctly across REST API and WebSocket, and fix TUI workstream management bugs.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] REST API chat handler persists turns to workstream storage
- [ ] WebSocket session state captured after stream completes
- [ ] Workstream rename updates sidebar by ID (not name)
- [ ] Session cleared when switching workstreams (already done - verify)

## Issues Detail

### 7. Chat Handler Doesn't Persist Turns (chat.rs:85-139)
**Impact**: REST API conversations not saved to workstream storage.
**Fix**: After agent turn completes, get workstream_id from session cache and call `save_turn()`.

### 8. WebSocket Session State Not Captured (ws.rs:387-399)
**Impact**: Tool results and state changes during streaming not persisted.
**Fix**: After stream completes, update session in cache and save the turn.

### 12. Workstream Rename ID Lookup Bug (app.rs:355)
**Impact**: After rename, workstream not found in sidebar.
**Fix**: Change `ws.name == id` to `ws.id == id`.

### 13. Session Creation on New Workstream (app.rs:1342)
**Status**: Already fixed - `switch_to_workstream()` clears `session_id` at line 1342.

## Implementation Notes

### Technical Approach

1. **chat.rs**: Add workstream lookup and save_turn after line 115
2. **ws.rs**: Need to persist turn after streaming completes
3. **app.rs**: Simple fix - change lookup key from name to id

## Status Updates

### 2026-02-13: All 4 Issues Fixed

**Completed:**

1. **REST API chat handler persists turns** (chat.rs:85-139)
   - Added code after `state.update_session()` to get workstream_id and call `save_turn()`
   - Gets completed turn before session is consumed, then persists via session_cache

2. **WebSocket session state captured** (ws.rs:387-399)  
   - Added tool_calls and tool_results accumulators during streaming
   - On StreamChunk::Done, constructs a Turn object with all accumulated data
   - Calls `session_cache.save_turn()` to persist the complete turn
   - Removed old message-based storage in favor of proper turn-based persistence

3. **Workstream rename lookup fixed** (app.rs:355)
   - Changed `ws.name == id` to `ws.id == id`
   - Now correctly looks up workstream entry by ID instead of name

4. **Session clearing on workstream switch** (app.rs:1342)
   - Already implemented - `switch_to_workstream()` clears `session_id` at line 1342
   - Verified working correctly

**Files Modified:**
- `crates/arawn-server/src/routes/chat.rs` - Added turn persistence after agent response
- `crates/arawn-server/src/routes/ws.rs` - Added imports, tool tracking, turn persistence
- `crates/arawn-tui/src/app.rs` - Fixed workstream rename lookup

**All tests pass**: `angreal test unit` completes successfully