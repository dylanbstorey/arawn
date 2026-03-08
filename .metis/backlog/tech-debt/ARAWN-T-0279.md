---
id: create-shared-test-utilities-crate
level: task
title: "Create shared test utilities crate (arawn-test-utils)"
short_code: "ARAWN-T-0279"
created_at: 2026-03-08T03:17:23.703939+00:00
updated_at: 2026-03-08T03:17:23.703939+00:00
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

# Create shared test utilities crate (arawn-test-utils)

## Objective

Create a shared `arawn-test-utils` crate that consolidates test infrastructure currently duplicated or missing across the workspace. This is a **prerequisite** for most other testing tasks — it provides the foundation that makes writing cross-crate tests practical.

### Priority
- [x] P0 - Critical (blocks other testing work)
- **Size**: M

### Current Problems
- `TestServer` harness lives in `arawn-server/tests/common/mod.rs` — not reusable by other crates
- `MockBackend` requires feature-gating (`arawn-llm/testing`) and is limited (no streaming)
- Each crate re-invents setup helpers (`setup()`, `test_manager()`, `test_engine()`)
- No shared fixtures for configs, workstreams, sessions, or agent contexts
- No WebSocket test client for E2E testing

## Acceptance Criteria

- [ ] `arawn-test-utils` crate added to workspace
- [ ] `TestServer` extracted and generalized (configurable: with/without workstreams, auth, plugins)
- [ ] `TestWsClient` — WebSocket test client that can connect, subscribe, send chat, receive responses
- [ ] `TestFixtures` module — factory functions for common test data (config, workstream, session, agent)
- [ ] `StreamingMockBackend` — MockBackend variant that yields streaming chunks (for SSE tests)
- [ ] Existing `arawn-server/tests/common/mod.rs` refactored to use the shared crate
- [ ] At least 3 other crates consume `arawn-test-utils` as dev-dependency

## Implementation Notes

### What to extract/create

```
crates/arawn-test-utils/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── server.rs        # TestServer (from arawn-server/tests/common)
    ├── ws_client.rs     # TestWsClient (new)
    ├── fixtures.rs      # Factory functions for test data
    ├── mock_backend.rs  # Re-export + StreamingMockBackend
    └── assertions.rs    # Custom assert macros (assert_json_contains!, etc.)
```

### Key components

1. **TestServer** — generalize from current `tests/common/mod.rs`:
   - `TestServer::builder().with_auth("token").with_workstreams().with_plugins(&[]).build().await`
   - Returns base URL + pre-configured HTTP client + WS URL

2. **TestWsClient** — wrap `tokio-tungstenite`:
   - `ws.connect(&url).await`
   - `ws.authenticate("token").await`
   - `ws.subscribe("session-id").await -> SubscribeAck`
   - `ws.chat("hello", session_id, workstream_id).await -> Vec<ServerMessage>`
   - `ws.send_ping().await`

3. **StreamingMockBackend** — extends MockBackend:
   - Returns `ResponseStream` instead of `CompletionResponse`
   - Configurable chunk size and delay
   - Supports tool_use in stream

4. **Fixtures**:
   - `fixtures::config()` → valid ServerConfig
   - `fixtures::workstream_manager()` → in-memory WorkstreamManager
   - `fixtures::agent()` → Agent with MockBackend and basic tools
   - `fixtures::session()` → Session with pre-loaded turns

### Dependencies
- None — this is foundational

## Status Updates

*To be added during implementation*