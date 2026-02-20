---
id: websocket-notification-for-file
level: task
title: "WebSocket notification for file promotion conflicts"
short_code: "ARAWN-T-0208"
created_at: 2026-02-18T19:52:00.547316+00:00
updated_at: 2026-02-19T01:29:17.053411+00:00
parent: ARAWN-I-0028
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/active"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0028
---

# WebSocket notification for file promotion conflicts

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[ARAWN-I-0028]]

## Objective

Send a WebSocket notification to connected clients when a file promotion results in a conflict rename (e.g., `file.txt` → `file(1).txt`).

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Add `FileAlert` variant to `ServerMessage` enum in protocol.rs
- [ ] Broadcast notification to workstream-subscribed clients on conflict rename
- [ ] Message includes: workstream_id, original_path, renamed_path, operation ("promote")
- [ ] Unit tests for new message serialization
- [ ] Integration with `promote_file_handler` in workstreams.rs

## Implementation Notes

### Technical Approach

1. Add new `ServerMessage::FileAlert` variant:
```rust
FileAlert {
    workstream_id: String,
    operation: String,       // "promote", "export", etc.
    original_path: String,   // The intended destination
    actual_path: String,     // The actual path after rename
    renamed: bool,           // Whether rename occurred
}
```

2. In `promote_file_handler`, after successful promotion with rename:
   - Get the connection manager from AppState
   - Broadcast `FileAlert` to clients subscribed to the workstream

### Dependencies
- ARAWN-T-0197 (File promote operation) - completed
- WebSocket connection/subscription system already exists

### Files to Modify
- `crates/arawn-server/src/routes/ws/protocol.rs` - Add FileAlert variant
- `crates/arawn-server/src/routes/workstreams.rs` - Broadcast on conflict
- `crates/arawn-server/src/state.rs` - May need broadcast channel access

## Status Updates **[REQUIRED]**

*To be added during implementation*