---
id: session-ownership-reconnect-tokens
level: task
title: "Session ownership reconnect tokens"
short_code: "ARAWN-T-0210"
created_at: 2026-02-19T13:43:41.579605+00:00
updated_at: 2026-02-19T13:51:17.857477+00:00
parent: ARAWN-I-0028
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0028
---

# Session ownership reconnect tokens

## Objective

Allow clients to reclaim session ownership after brief disconnections using reconnect tokens. When a client disconnects, ownership is held for a grace period. If the client reconnects with their token within that window, they reclaim ownership without losing it to another subscriber.

## Design

### Flow

```
1. Client A subscribes to session
   → SubscribeAck { owner: true, reconnect_token: "abc123" }

2. Client A disconnects (network hiccup)
   → Server stores pending reconnect: session → (token, expiry)
   → Ownership NOT immediately released

3. Client A reconnects within 30s
   → Subscribe { session_id, reconnect_token: "abc123" }
   → Token matches, within grace period
   → SubscribeAck { owner: true, reconnect_token: "def456" } (new token)

4. If grace period expires before reconnect
   → Pending reconnect cleared
   → Session ownership released
   → Next subscriber becomes owner
```

### Protocol Changes

Update `ClientMessage::Subscribe`:
```rust
Subscribe {
    session_id: String,
    reconnect_token: Option<String>,  // NEW
}
```

Update `ServerMessage::SubscribeAck`:
```rust
SubscribeAck {
    session_id: String,
    owner: bool,
    reconnect_token: Option<String>,  // NEW - only present if owner
}
```

### Server State

```rust
struct PendingReconnect {
    token: String,
    session_id: SessionId,
    expires_at: Instant,
}

// In AppState
pending_reconnects: Arc<RwLock<HashMap<SessionId, PendingReconnect>>>
```

### Grace Period

Default: 30 seconds (configurable via server config)

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Subscribe accepts optional `reconnect_token` field
- [x] SubscribeAck includes `reconnect_token` for owners
- [x] On disconnect, pending reconnect stored with token + expiry
- [x] Reconnect with valid token within grace period restores ownership
- [x] Expired tokens rejected, treated as new subscription
- [x] Invalid tokens rejected with error
- [x] Lazy cleanup of expired pending reconnects (during subscribe)
- [x] Grace period configurable in server config
- [x] Tests cover reconnect scenarios

## Implementation Notes

### Files to Modify

- `crates/arawn-server/src/routes/ws/protocol.rs` - Add reconnect_token fields
- `crates/arawn-server/src/routes/ws/handlers.rs` - Reconnect logic in handle_subscribe
- `crates/arawn-server/src/routes/ws/connection.rs` - Store pending on disconnect
- `crates/arawn-server/src/state.rs` - PendingReconnect storage + cleanup
- `crates/arawn-server/src/config.rs` - Grace period config

### Token Generation

Use `uuid::Uuid::new_v4().to_string()` - simple, unique, unguessable

### Cleanup Strategy

Options:
1. Lazy cleanup - check expiry on access
2. Background task - periodic sweep every 10s
3. Both - lazy for correctness, background for memory

Recommend: Lazy cleanup is sufficient given short grace periods

## Status Updates

### 2026-02-19: Implementation Complete

**Files Modified:**
- `crates/arawn-server/src/routes/ws/protocol.rs` - Added `reconnect_token` to Subscribe and SubscribeAck
- `crates/arawn-server/src/routes/ws/connection.rs` - Added `reconnect_tokens` HashMap to ConnectionState
- `crates/arawn-server/src/routes/ws/handlers.rs` - Reconnect logic in handle_subscribe
- `crates/arawn-server/src/state.rs` - PendingReconnect struct, storage, and management methods
- `crates/arawn-server/src/config.rs` - Added `reconnect_grace_period` config (default 30s)

**Implementation Details:**
- Token generated when client becomes owner, stored in ConnectionState
- On disconnect, tokens used to create PendingReconnect entries with expiry
- On reconnect with token: validates, restores ownership, issues new token
- Pending reconnects block other connections from claiming ownership
- Lazy cleanup of expired entries during subscribe operations
- Explicit unsubscribe does NOT create pending reconnect (intentional release)

**Tests Added:**
- `test_subscribe_ack_serialization` - Updated for reconnect_token
- `test_session_ownership_release_all_on_disconnect` - Updated with tokens
- `test_reconnect_token_wrong_token_rejected` - Wrong token denied
- `test_reconnect_token_new_connection_can_reclaim` - Different conn ID, same token works
- `test_reconnect_cleanup_expired` - Expired tokens cleaned up, ownership claimable

All 124+ tests pass.