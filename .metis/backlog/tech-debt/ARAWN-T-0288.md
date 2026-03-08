---
id: add-session-ownership-and
level: task
title: "Add session ownership and WebSocket reconnection tests"
short_code: "ARAWN-T-0288"
created_at: 2026-03-08T03:17:33.020566+00:00
updated_at: 2026-03-08T03:17:33.020566+00:00
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

# Add session ownership and WebSocket reconnection tests

## Objective

The session ownership system (`try_claim_session_ownership`, `release_all_session_ownerships`, `active_connections` tracking, pending reconnects with grace period) is tested only with unit-level state tests — no real WebSocket connections are involved. We just fixed two bugs in this system (stale ownership from dead connections, idle timeout disconnects). Add focused unit and integration tests that specifically exercise ownership transitions and reconnection scenarios.

This complements ARAWN-T-0281 (WebSocket E2E) but focuses specifically on the ownership state machine in `state.rs` and the reconnection protocol.

### Priority
- [x] P1 - High (this system has had multiple production bugs)
- **Size**: M

### Current Problems
- `state.rs` has 22 tests but they were written before `active_connections` tracking was added
- No tests verify that dead connections are evicted from ownership
- No tests for the pending reconnect grace period expiration
- `release_all_session_ownerships` creates pending reconnects but the full cycle (release → pending → expire → new claim) is untested
- `register_connection`/`unregister_connection` not tested
- The interaction between `active_connections` and `try_claim_session_ownership` not tested

## Acceptance Criteria

- [ ] Unit tests in `state.rs` for the new `active_connections` tracking:
  - [ ] `register_connection` adds to set
  - [ ] `unregister_connection` removes from set
  - [ ] `is_connection_active` returns correct values
- [ ] Tests for dead owner eviction in `try_claim_session_ownership`:
  - [ ] Live owner blocks claim → returns false
  - [ ] Dead owner (not in active_connections) evicted → returns true
  - [ ] Dead owner eviction logged correctly
- [ ] Tests for the full ownership lifecycle:
  - [ ] Claim → release → pending reconnect → reclaim with token
  - [ ] Claim → release → pending reconnect → grace period expires → new client claims
  - [ ] Claim → connection dies → unregister → new client claims via dead-owner eviction
- [ ] Tests for the chat handler ownership check:
  - [ ] Owner sends chat → succeeds
  - [ ] Non-owner sends chat, live owner exists → `session_not_owned` error
  - [ ] Non-owner sends chat, dead owner exists → ownership transferred, chat succeeds
  - [ ] No owner → ownership claimed, chat succeeds
- [ ] At least 15 new test functions

## Implementation Notes

### Unit test pattern (state.rs)

```rust
#[tokio::test]
async fn test_dead_owner_evicted_on_claim() {
    let state = test_state();
    let conn_a = ConnectionId::new();
    let conn_b = ConnectionId::new();
    let session = SessionId::new();
    
    // Register A as active, claim ownership
    state.register_connection(conn_a).await;
    assert!(state.try_claim_session_ownership(session, conn_a).await);
    
    // Unregister A (simulating disconnect)
    state.unregister_connection(conn_a).await;
    
    // Register B, try to claim — should succeed because A is dead
    state.register_connection(conn_b).await;
    assert!(state.try_claim_session_ownership(session, conn_b).await);
    
    // B is now the owner
    assert!(state.is_session_owner(session, conn_b).await);
}
```

### Integration test pattern (handlers)

The chat handler tests could be inline in `handlers.rs` or in a separate integration test using real WebSocket connections (overlaps with ARAWN-T-0281).

### Key files
- `crates/arawn-server/src/state.rs` — Add unit tests for active_connections + ownership
- `crates/arawn-server/src/routes/ws/handlers.rs` — Add chat handler ownership tests

### Dependencies
- None — can be done immediately since the code already exists

## Status Updates

*To be added during implementation*