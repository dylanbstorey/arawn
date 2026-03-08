---
id: add-tui-app-state-machine-and
level: task
title: "Add TUI app state machine and event handling tests"
short_code: "ARAWN-T-0284"
created_at: 2026-03-08T03:17:29.036824+00:00
updated_at: 2026-03-08T03:17:29.036824+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#tech-debt"


exit_criteria_met: false
initiative_id: NULL
---

# Add TUI app state machine and event handling tests

## Objective

`app.rs` is 2,219 lines with **zero tests**. It contains the main event loop, state management, message sending, sidebar interactions, session switching, and connection status handling. This is where most TUI bugs live (the keepalive fix, the read-only mode issue, the scroll overshoot). Extract testable logic and add tests for state transitions and event handling.

### Priority
- [x] P2 - Medium (TUI bugs are user-facing but the code changes frequently)
- **Size**: M

### Current Problems
- `App` struct is monolithic — 2,219 lines mixing state, I/O, and rendering
- Event handling (`handle_key_event`) is not independently testable
- State transitions (connected/disconnected, waiting/idle, read-only/owner) untested
- Message sending logic (with waiting state management) untested
- Connection status polling and recovery untested
- Every TUI bug fix is a "change and pray" situation

## Acceptance Criteria

- [ ] Testable state machine extracted from `App` (or App made constructable in tests without terminal)
- [ ] Tests for key event handling: Enter sends message, Ctrl+C quits, Tab switches focus
- [ ] Tests for connection state transitions: Connected → Disconnected → Reconnecting → Connected
- [ ] Tests for waiting state: send message → waiting=true → response received → waiting=false
- [ ] Tests for waiting state reset on disconnect (the bug we fixed)
- [ ] Tests for session switching (sidebar navigation → active session changes)
- [ ] Tests for command parsing from input (`:quit`, `:clear`, `/help`)
- [ ] At least 20 new test functions

## Implementation Notes

### Testability approach

The `App` struct currently requires a real terminal. Two approaches:

**Option A — Extract state machine:**
```rust
// New: app_state.rs
pub struct AppState {
    pub connection_status: ConnectionStatus,
    pub waiting: bool,
    pub active_session: Option<SessionId>,
    pub focus: FocusState,
    // ...
}

impl AppState {
    pub fn handle_connection_change(&mut self, status: ConnectionStatus) -> Vec<AppAction>;
    pub fn handle_message_sent(&mut self) -> Vec<AppAction>;
    pub fn handle_response_received(&mut self) -> Vec<AppAction>;
}
```

**Option B — Mock WsClient:**
```rust
#[cfg(test)]
impl App {
    fn test_new(ws: MockWsClient) -> Self { ... }
}
```

Option A is preferred — it decouples state logic from I/O, making every state transition testable without a terminal.

### Key state transitions to test

1. **Connection lifecycle**: `Disconnected → Connecting → Connected → Disconnected`
2. **Message flow**: `Idle → Waiting → ResponseReceived → Idle`
3. **Waiting + Disconnect**: `Waiting → Disconnected → Idle + error message`
4. **Focus navigation**: `Chat → Sidebar → Palette → Chat`
5. **Session management**: `Select session → subscribe → active`

### Key files
- `crates/arawn-tui/src/app.rs` — Extract state machine or add test constructor
- `crates/arawn-tui/src/app_state.rs` — New file (if Option A)

### Dependencies
- None — can be done independently

## Status Updates

*To be added during implementation*