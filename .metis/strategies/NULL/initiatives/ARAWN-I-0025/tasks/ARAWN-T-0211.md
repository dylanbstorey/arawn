---
id: tui-session-ownership-and
level: task
title: "TUI: Session ownership and reconnect token support"
short_code: "ARAWN-T-0211"
created_at: 2026-02-19T13:57:31.801395+00:00
updated_at: 2026-02-19T14:05:14.126861+00:00
parent: ARAWN-I-0025
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0025
---

# TUI: Session ownership and reconnect token support

## Objective

Update the TUI to support the new session ownership model with reconnect tokens. The TUI should handle owner/reader modes, store reconnect tokens for session recovery after disconnects, and provide clear UI feedback about session state.

## Background

Server now implements session ownership (ARAWN-T-0209, ARAWN-T-0210):
- First subscriber to a session becomes the owner (can send Chat)
- Subsequent subscribers are readers (receive messages, cannot send Chat)
- Owners receive a `reconnect_token` for reclaiming ownership after disconnect
- 30-second grace period for reconnection

## Protocol Changes Required

### ClientMessage::Subscribe
```rust
Subscribe {
    session_id: String,
    reconnect_token: Option<String>,  // NEW
}
```

### ServerMessage (add new variant)
```rust
SubscribeAck {
    session_id: String,
    owner: bool,
    reconnect_token: Option<String>,
}
```

## UI Requirements

### Reader Mode
- Show "READ ONLY" badge/indicator when `owner: false`
- Disable input field (grayed out, non-interactive)
- Show tooltip/status message explaining reader mode

### Owner Mode  
- Normal operation (current behavior)
- Store `reconnect_token` in App state

### Auto-Reconnect
- On WebSocket disconnect, attempt reconnect
- Include stored `reconnect_token` in Subscribe message
- If token valid → restore owner mode
- If token expired/invalid → enter reader mode (if session still exists)

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Protocol updated: Subscribe has reconnect_token field
- [x] Protocol updated: SubscribeAck variant added to ServerMessage
- [x] App stores reconnect tokens per session
- [x] SubscribeAck handler updates owner status and stores token
- [x] Reader mode: "READ ONLY" badge displayed
- [x] Reader mode: Input disabled with visual indicator
- [x] Reconnect logic uses stored token
- [x] Handle session_not_owned error gracefully

## Implementation Notes

### Files to Modify

- `crates/arawn-tui/src/protocol.rs` - Add reconnect_token to Subscribe, add SubscribeAck
- `crates/arawn-tui/src/app.rs` - Add reconnect_tokens storage, owner state per session
- `crates/arawn-tui/src/client.rs` - Send token on reconnect, handle SubscribeAck
- `crates/arawn-tui/src/ui/layout.rs` or similar - Reader mode badge
- `crates/arawn-tui/src/ui/input.rs` or similar - Disable input in reader mode

### State Changes

```rust
// In App
reconnect_tokens: HashMap<String, String>,  // session_id -> token
is_session_owner: bool,  // current session ownership status
```

### Dependencies

- ARAWN-T-0209: Session ownership (completed)
- ARAWN-T-0210: Reconnect tokens (completed)

## Status Updates

### Session 2 - Continued
- Initialized `reconnect_tokens` and `is_session_owner` fields in `App::new()`
- Added `SubscribeAck` handler in `handle_server_message()` - updates ownership state and stores reconnect token
- Updated `switch_to_session()` to pass reconnect token when subscribing
- Modified `render_input()` in `ui/input.rs` to show "READ ONLY" badge when not owner
- Added read-only check to `send_message()` and `send_command()` (except /help)
- Build passes for both arawn-tui and arawn-server

### Completed
- Added specific handling for `session_not_owned` error - automatically switches to read-only mode
- All acceptance criteria met
- Build passes