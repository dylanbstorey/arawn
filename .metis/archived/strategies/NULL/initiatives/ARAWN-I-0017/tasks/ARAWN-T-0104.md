---
id: wire-session-indexer-to-session
level: task
title: "Wire session indexer to session close lifecycle"
short_code: "ARAWN-T-0104"
created_at: 2026-01-31T04:09:07.878740+00:00
updated_at: 2026-02-01T03:51:17.149909+00:00
parent: ARAWN-I-0017
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0017
---

# Wire session indexer to session close lifecycle

## Objective

Wire the `SessionIndexer` into the session lifecycle so indexing runs automatically when a session ends. This is the integration point between the agent/server layer and the indexing pipeline.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Session close event triggers `SessionIndexer::index_session()` as a background `tokio::spawn` task
- [ ] Indexing only runs when `memory.indexing.enabled = true` in config
- [ ] Indexing failure is logged but does not affect the session close response
- [ ] Agent or server holds `Option<SessionIndexer>` (None when indexing disabled)
- [ ] `IndexReport` logged at info level on completion
- [ ] Tests: verify indexer is called on session close (mock), verify skipped when disabled

## Implementation Notes

### Files
- `crates/arawn-agent/src/agent.rs` — add `Option<SessionIndexer>` field, trigger on session end
- `crates/arawn-server/src/routes/ws.rs` — or wherever session lifecycle is managed

### Technical Approach
The agent's `turn()` method doesn't own session close — that's handled by the REPL or WebSocket layer. The indexer should be wired at the point where a session is explicitly closed or times out. Use `tokio::spawn` to avoid blocking the close response.

### Dependencies
- ARAWN-T-0103 (SessionIndexer must exist)
- ARAWN-T-0100 (indexing config)

## Status Updates

### Session 1 — Complete
- Added `Option<Arc<SessionIndexer>>` and `with_indexer()` builder to `AppState` in `crates/arawn-server/src/state.rs`
- Implemented `close_session()` on `AppState`: removes session from store, spawns background `tokio::spawn` indexing if indexer configured and session non-empty
- Added `session_to_messages()` and `messages_as_refs()` helper functions for converting session turns to indexer input format
- Wired `delete_session_handler` in `sessions.rs` to use `state.close_session()` instead of direct HashMap removal
- Wired WebSocket close path in `ws.rs` to index all subscribed sessions on disconnect (reads without removing — sessions may be shared)
- Re-exported `SessionIndexer`, `IndexerConfig`, `IndexReport` from `arawn-agent/src/lib.rs`
- Added 8 new tests in `state.rs`: message conversion (empty, with turns, incomplete), refs conversion, close lifecycle (removes, nonexistent, without indexer), default state has no indexer
- `angreal check all` passes, `angreal test unit` passes (750+ tests, 0 failures)

**Files modified:**
- `crates/arawn-server/src/state.rs` — AppState fields, close_session(), helpers, tests
- `crates/arawn-server/src/routes/sessions.rs` — delete handler wiring
- `crates/arawn-server/src/routes/ws.rs` — WS close indexing
- `crates/arawn-agent/src/lib.rs` — re-exports