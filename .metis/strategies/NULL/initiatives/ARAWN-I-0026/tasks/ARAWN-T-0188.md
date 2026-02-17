---
id: websocket-command-bridge
level: task
title: "WebSocket command bridge"
short_code: "ARAWN-T-0188"
created_at: 2026-02-16T18:54:55.136367+00:00
updated_at: 2026-02-17T02:48:11.031404+00:00
parent: ARAWN-I-0026
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


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

## Acceptance Criteria

## Acceptance Criteria

- [x] `WsCommandRequest` message type (client → server) - `ClientMessage::Command { command, args }`
- [x] `WsCommandProgress` message type (server → client) - `ServerMessage::CommandProgress { command, message, percent }`
- [x] `WsCommandResult` message type (server → client) - `ServerMessage::CommandResult { command, success, result }`
- [x] WS handler routes command messages to REST handlers - via `handle_command()` in handlers.rs
- [x] Progress streamed back over WS connection - using `MessageResponse::Stream`
- [x] Session context automatically included (no session_id needed in WS) - via `inject_session_context()`

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

### 2026-02-17: Implementation Complete

**Files Modified:**
- `crates/arawn-server/src/routes/ws/protocol.rs` - Added `Command` variant to `ClientMessage`, `CommandProgress` and `CommandResult` variants to `ServerMessage`, plus helper constructors
- `crates/arawn-server/src/routes/ws/handlers.rs` - Added `handle_command()` function and `inject_session_context()` helper

**Implementation Details:**

1. **ClientMessage::Command** - New message type with `command` (name) and `args` (JSON value)

2. **ServerMessage::CommandProgress** - Progress updates with `command`, `message`, and optional `percent`

3. **ServerMessage::CommandResult** - Final result with `command`, `success` boolean, and `result` JSON

4. **handle_command()** - Routes to CommandRegistry, executes handler, streams progress and result

5. **inject_session_context()** - If args don't include `session_id` and connection has subscriptions, auto-injects the first subscribed session ID

**Tests:** 10 tests total
- 6 protocol tests (message parsing and serialization)
- 4 handler tests (session context injection)

**Usage Example:**
```json
// Client sends:
{"type": "command", "command": "compact", "args": {"force": true}}

// Server responds (streamed):
{"type": "command_progress", "command": "compact", "message": "Starting...", "percent": 0}
{"type": "command_result", "command": "compact", "success": true, "result": {"compacted": true, "turns_compacted": 3}}