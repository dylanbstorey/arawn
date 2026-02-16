---
id: websocket-command-bridge
level: task
title: "WebSocket command bridge"
short_code: "ARAWN-T-0188"
created_at: 2026-02-16T18:54:55.136367+00:00
updated_at: 2026-02-16T18:54:55.136367+00:00
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

# WebSocket command bridge

## Parent Initiative

[[ARAWN-I-0026]] - Context Management and Auto-Compaction

## Objective

Bridge WebSocket messages to REST command handlers, enabling active sessions to execute commands via WS and receive streaming progress.

## Acceptance Criteria

- [ ] `WsCommandRequest` message type (client → server)
- [ ] `WsCommandProgress` message type (server → client)
- [ ] `WsCommandResult` message type (server → client)
- [ ] WS handler routes command messages to REST handlers
- [ ] Progress streamed back over WS connection
- [ ] Session context automatically included (no session_id needed in WS)

## Implementation Notes

### Files to Modify
- `crates/arawn-server/src/routes/ws.rs` - add command message handling

### Message Types

```rust
// Client → Server
pub struct WsCommandRequest {
    pub command: String,
    pub args: serde_json::Value,
}

// Server → Client
pub struct WsCommandProgress {
    pub command: String,
    pub message: String,
    pub percent: Option<f32>,
}

pub struct WsCommandResult {
    pub command: String,
    pub success: bool,
    pub result: serde_json::Value,
}
```

### Dependencies
- ARAWN-T-0187 (Command REST API)

## Tests

### Unit Tests
- `test_ws_command_request_deserialize` - parse WsCommandRequest from JSON
- `test_ws_command_progress_serialize` - WsCommandProgress serializes correctly
- `test_ws_command_result_serialize` - WsCommandResult serializes correctly

### Integration Tests
- `test_ws_command_routes_to_handler` - WS message triggers correct command handler
- `test_ws_command_progress_streaming` - progress messages sent over WS
- `test_ws_command_session_context` - session_id derived from WS connection
- `test_ws_unknown_command_error` - unknown command returns error result

### Test File
- `crates/arawn-server/src/routes/ws.rs` (inline `#[cfg(test)]` module)

## Status Updates

*To be added during implementation*