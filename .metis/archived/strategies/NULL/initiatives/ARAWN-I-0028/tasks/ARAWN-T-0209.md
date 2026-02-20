---
id: session-ownership-for-websocket
level: task
title: "Session ownership for WebSocket connections"
short_code: "ARAWN-T-0209"
created_at: 2026-02-19T01:39:59.313057+00:00
updated_at: 2026-02-19T02:56:53.791873+00:00
parent: ARAWN-I-0028
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0028
---

# Session ownership for WebSocket connections

## Objective

Implement reader/writer model for WebSocket session subscriptions to prevent multiple clients from simultaneously writing to the same chat session ("thrashing").

## Design

### Model: Reader vs Writer

- **First subscriber** to a session becomes the **owner** (writer)
- Subsequent subscribers are **readers** - they receive all messages but cannot send Chat
- Readers can still execute **read-only commands** (e.g., list sessions, get status)
- Ownership released on **Unsubscribe** or **disconnect**
- No explicit ownership transfer - just release and re-claim

### Server State

```rust
// In AppState
session_owners: Arc<RwLock<HashMap<SessionId, ConnectionId>>>
```

```rust
// In ConnectionState
pub id: ConnectionId,  // Unique per connection
```

### Message Flow

```
Client A: Subscribe(session-1) → owner, gets SubscribeAck { owner: true }
Client B: Subscribe(session-1) → reader, gets SubscribeAck { owner: false }
Client B: Chat(...) → rejected: "Session owned by another client"
Client A: Unsubscribe → ownership released
Client B: Chat(...) → B becomes owner, succeeds
```

### Protocol Changes

New response for Subscribe:
```rust
ServerMessage::SubscribeAck {
    session_id: String,
    owner: bool,
}
```

New error code for Chat rejection:
```rust
ServerMessage::Error {
    code: "session_not_owned",
    message: "Session is owned by another client",
}
```

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] ConnectionState has unique ConnectionId
- [x] AppState tracks session ownership globally
- [x] First subscriber becomes owner
- [x] Subscribe returns SubscribeAck with owner flag
- [x] Chat from non-owner rejected with clear error
- [x] Read-only commands work for non-owners (commands don't check ownership)
- [x] Ownership released on Unsubscribe
- [x] Ownership released on disconnect (cleanup)
- [x] Tests cover ownership scenarios

## Implementation Notes

### Files to Modify

- `crates/arawn-server/src/routes/ws/connection.rs` - Add ConnectionId
- `crates/arawn-server/src/routes/ws/handlers.rs` - Ownership logic in handle_subscribe/handle_chat
- `crates/arawn-server/src/routes/ws/protocol.rs` - SubscribeAck message type
- `crates/arawn-server/src/state.rs` - Session ownership tracking

### Read-Only Commands

Commands that should work for non-owners:
- list_sessions
- list_workstreams
- get_session
- status/health checks

Commands that require ownership:
- Chat messages
- Potentially: delete_session, rename_session

## Status Updates

### 2026-02-19: Implementation Complete

**Files Modified:**
- `crates/arawn-server/src/routes/ws/protocol.rs` - Added `SubscribeAck` message type
- `crates/arawn-server/src/routes/ws/connection.rs` - Added `ConnectionId` type, cleanup on disconnect
- `crates/arawn-server/src/routes/ws/handlers.rs` - Updated Subscribe/Unsubscribe/Chat handlers for ownership
- `crates/arawn-server/src/routes/ws/mod.rs` - Re-exported `ConnectionId`
- `crates/arawn-server/src/state.rs` - Added `SessionOwners` type and ownership methods

**Implementation Details:**
- Each WebSocket connection gets a unique `ConnectionId`
- `AppState.session_owners` tracks which connection owns each session
- First subscriber to a session becomes the owner
- Subscribe returns `SubscribeAck { session_id, owner: bool }`
- Chat from non-owner rejected with `session_not_owned` error
- Ownership released on Unsubscribe or disconnect

**Tests Added:**
- `test_subscribe_ack_serialization` - Protocol serialization
- `test_session_ownership_first_claimer_wins` - First subscriber wins
- `test_session_ownership_release` - Owner release, non-owner cannot
- `test_session_ownership_release_all_on_disconnect` - Cleanup on disconnect
- `test_session_ownership_same_connection_reclaim` - Idempotent re-claim

All 57 tests pass.