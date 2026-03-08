---
id: add-websocket-e2e-integration
level: task
title: "Add WebSocket E2E integration tests for chat flow"
short_code: "ARAWN-T-0281"
created_at: 2026-03-08T03:17:25.921539+00:00
updated_at: 2026-03-08T03:17:25.921539+00:00
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

# Add WebSocket E2E integration tests for chat flow

## Objective

The WebSocket chat flow — the primary way the TUI talks to the server — has **zero E2E test coverage**. Only protocol parsing is unit-tested. Write integration tests that exercise the full flow: WS connect → authenticate → subscribe → chat → receive response → unsubscribe.

This would have caught the session ownership bug, the idle timeout disconnect, and the read-only mode issue we just fixed.

### Priority
- [x] P0 - Critical (this is the primary user-facing communication path)
- **Size**: L

### Current Problems
- Session ownership logic only tested with unit-level state tests (no real WS connections)
- Keepalive/ping-pong behavior untested
- Reconnection behavior untested
- Concurrent client behavior (ownership conflict) untested
- Chat message routing through workstreams untested over WS

## Acceptance Criteria

- [ ] Test file: `crates/arawn-server/tests/websocket_integration.rs`
- [ ] Tests use `TestWsClient` from `arawn-test-utils` (or inline tokio-tungstenite)
- [ ] Scenarios covered:
  - [ ] Connect and authenticate (no-auth mode + token mode)
  - [ ] Subscribe to session → receive `subscribe_ack` with `is_owner: true`
  - [ ] Send chat message → receive agent response
  - [ ] Multi-turn conversation in same session
  - [ ] Second client subscribes → gets `is_owner: false`
  - [ ] Owner disconnects → second client can claim ownership
  - [ ] Idle timeout (5min) sends error and closes connection
  - [ ] Ping/pong keepalive prevents idle timeout
  - [ ] Reconnect with token → reclaims ownership within grace period
  - [ ] Reconnect after grace period → gets new ownership
  - [ ] Concurrent chat from non-owner → `session_not_owned` error
  - [ ] Invalid session ID → error response
  - [ ] Unauthenticated chat → `unauthorized` error

## Implementation Notes

### Test infrastructure needed

Use `tokio-tungstenite` directly or create `TestWsClient` wrapper:

```rust
let server = TestServer::builder().build().await;
let (mut ws, _) = tokio_tungstenite::connect_async(server.ws_url()).await.unwrap();

// Send subscribe
ws.send(Message::Text(json!({"type": "subscribe", "session_id": "..."}).to_string())).await;
let ack: ServerMessage = read_next(&mut ws).await;
assert!(ack.is_owner);

// Send chat
ws.send(Message::Text(json!({"type": "chat", "message": "hello"}).to_string())).await;
let response: ServerMessage = read_next(&mut ws).await;
assert_eq!(response.type_, "chat_response");
```

### Key scenarios by complexity

**Basic (must have):**
- Connect → auth → subscribe → chat → response
- Unsubscribe → re-subscribe

**Ownership (critical — these bugs happened):**
- Two clients, ownership transfer on disconnect
- Dead connection detection (active_connections tracking)
- Pending reconnect grace period behavior

**Edge cases:**
- Binary frame with valid JSON (accepted)
- Binary frame with invalid UTF-8 (rejected)
- Malformed JSON → parse_error
- Very large message handling

### Dependencies
- Depends on ARAWN-T-0279 (shared test utils) for TestServer, or can inline TestServer temporarily

## Status Updates

*To be added during implementation*