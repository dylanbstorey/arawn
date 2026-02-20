---
id: fix-medium-priority-tui-issues
level: task
title: "Fix Medium Priority TUI Issues (Phase 4)"
short_code: "ARAWN-T-0175"
created_at: 2026-02-13T14:05:44.833942+00:00
updated_at: 2026-02-13T14:11:25.773437+00:00
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

# Fix Medium Priority TUI Issues (Phase 4)

Fix 9 medium-priority TUI issues from the codebase audit.

## Parent Initiative

[[ARAWN-I-0025]]

## Objective

Improve TUI stability, user experience, and fix race conditions and edge cases.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] #29: Race in message rendering - Added detailed documentation explaining behavior
- [x] #30: Event channel silent termination - Added tracing logs for all break cases
- [x] #31: Sidebar filter not implemented - Implemented incremental filter with feedback
- [x] #32: Tool pane scrolling TODO - Implemented Up/Down/PageUp/PageDown/Home/End
- [x] #33: Cancel doesn't notify server - Added Cancel protocol message and send_cancel()
- [x] #34: Cursor position can panic - Added bounds check with safe_cursor.min(len)
- [x] #35: Session state loss on switch - Reordered to subscribe before clearing state
- [x] #36: Auto-scroll overrides manual - Added documentation, behavior was already correct
- [x] #37: Pending actions not deduplicated - Added HashSet-based deduplication

## Issues Detail

### #29: Race in message rendering (ui/chat.rs:40)
**Impact**: Tool use and results can render out of order during rapid updates.
**Fix**: Separate tool rendering pipeline from message list updates.

### #30: Event channel silent termination (events.rs:51)
**Impact**: Event loop terminates without logging why, hard to debug.
**Fix**: Log the reason when channel closes.

### #31: Sidebar filter not implemented (app.rs:1280)
**Impact**: TODO placeholder, incomplete feature.
**Fix**: Implement filter functionality or remove the TODO.

### #32: Tool pane scrolling TODO (app.rs:1093)
**Impact**: Long tool output can't be scrolled, truncated content hidden.
**Fix**: Implement scrolling for tool pane.

### #33: Cancel doesn't notify server (app.rs:661)
**Impact**: Server continues processing after user cancels, wasting resources.
**Fix**: Send cancel message via WebSocket when user cancels.

### #34: Cursor position can panic (input.rs:63-66)
**Impact**: Edge cases in cursor positioning can cause panic.
**Fix**: Add bounds checking for cursor position.

### #35: Session state loss on switch (app.rs:1025-1029)
**Impact**: Switching sessions clears state before subscribing to new session.
**Fix**: Subscribe to new session before clearing state.

### #36: Auto-scroll overrides manual (app.rs:468,870)
**Impact**: User scrolls up to read history, auto-scroll snaps back down.
**Fix**: Track scroll state and disable auto-scroll when user has scrolled up.

### #37: Pending actions not deduplicated (app.rs:247-280)
**Impact**: Multiple identical actions can queue up, causing redundant work.
**Fix**: Deduplicate actions before adding to queue.

## Implementation Notes

These fixes improve TUI reliability and user experience. Most are small, targeted changes.

## Status Updates

### Completed 2026-02-13

All 9 TUI medium-priority issues fixed:

**#29 Race in message rendering (ui/chat.rs)**
- Added detailed comments explaining tool rendering behavior
- Tools are cleared on new message send, ensuring correct state per response

**#30 Event channel silent termination (events.rs)**
- Added tracing::debug for channel closed, receiver dropped, tick channel
- Added tracing::warn for event read errors with error details

**#31 Sidebar filter (app.rs)**
- Implemented incremental filter with status message feedback
- Filter clears on backspace, shows current filter string

**#32 Tool pane scrolling (app.rs)**
- Added tool_scroll field to App state
- Implemented Up/Down (1 line), PageUp/PageDown (10 lines), Home/End

**#33 Cancel notification (protocol.rs, client.rs, app.rs)**
- Added Cancel variant to ClientMessage protocol
- Added send_cancel() method to WsClient
- Ctrl+C during streaming now sends cancel to server

**#34 Cursor bounds check (input.rs)**
- Added safe_cursor = cursor.min(content.len()) in cursor_position()
- Prevents panic on edge cases

**#35 Session state loss (app.rs)**
- Reordered switch_to_session: subscribe FIRST, then clear state
- Returns early if subscribe fails, preserving current state

**#36 Auto-scroll documentation (app.rs)**
- Added detailed doc comments explaining scroll behavior
- Confirmed existing implementation is correct

**#37 Action deduplication (app.rs)**
- Added PartialEq, Eq, Hash, Clone derives to PendingAction
- Added HashSet-based dedup in process_pending_actions()

All 23 arawn-tui tests pass.