---
id: websocket-protocol-module-split
level: task
title: "WebSocket Protocol Module Split"
short_code: "ARAWN-T-0181"
created_at: 2026-02-13T16:39:56.125436+00:00
updated_at: 2026-02-13T21:21:36.756506+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#tech-debt"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# WebSocket Protocol Module Split

## Objective

Split `ws.rs` into separate modules for protocol, connection lifecycle, and message handlers to improve maintainability.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P2 - Medium (nice to have)

### Technical Debt Impact
- **Current Problems**: `ws.rs` handles protocol types, connection lifecycle, and all message handlers in one file. As WebSocket functionality grows, this becomes harder to navigate.
- **Benefits of Fixing**: Separation of concerns, easier testing of individual components, clearer code organization.
- **Risk Assessment**: LOW - Current file size is manageable; this is proactive cleanup.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Create `routes/ws/` directory structure
- [x] Extract protocol types to `routes/ws/protocol.rs`
- [x] Extract connection lifecycle to `routes/ws/connection.rs`
- [x] Extract message handlers to `routes/ws/handlers.rs`
- [x] Keep `routes/ws/mod.rs` as thin router
- [x] No behavior changes - pure refactor
- [x] All WebSocket tests pass

## Implementation Notes

### Proposed Structure
```
crates/arawn-server/src/routes/ws/
├── mod.rs           # Re-exports, ws_handler entry point
├── protocol.rs      # WsMessage, WsResponse, serialization
├── connection.rs    # WebSocket upgrade, ping/pong, lifecycle
└── handlers.rs      # handle_message, handle_chat, handle_subscribe
```

### Module Responsibilities

**protocol.rs**
- `WsMessage` enum (incoming messages)
- `WsResponse` enum (outgoing messages)
- Serialization/deserialization logic

**connection.rs**
- `ws_handler` - WebSocket upgrade and main loop
- Ping/pong handling
- Connection state management
- Graceful shutdown

**handlers.rs**
- `handle_message` - Dispatch to specific handlers
- `handle_chat` - Chat message processing
- `handle_subscribe` - Session subscription
- `handle_cancel` - Cancellation

### When to Do This
Defer until ws.rs exceeds ~500 lines or new WebSocket features are needed.

## Status Updates

### Completed
Split 644-line `ws.rs` into 4 focused modules:

| File | Lines | Purpose |
|------|-------|---------|
| `protocol.rs` | 203 | ClientMessage, ServerMessage enums + tests |
| `connection.rs` | 188 | ConnectionState, handle_socket, send_message |
| `handlers.rs` | 312 | handle_message, handle_chat, handle_auth, etc. |
| `mod.rs` | 35 | Re-exports, ws_handler entry point |

- Added Cancel message type to protocol for completeness
- All 57 arawn-server tests passing (3 ws::protocol tests)