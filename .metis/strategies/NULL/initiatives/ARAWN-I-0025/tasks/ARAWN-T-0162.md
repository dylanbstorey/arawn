---
id: tui-websocket-client-integration
level: task
title: "TUI: WebSocket client integration"
short_code: "ARAWN-T-0162"
created_at: 2026-02-11T00:28:42.732067+00:00
updated_at: 2026-02-11T20:31:12.363747+00:00
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

# TUI: WebSocket client integration

## Objective

Implement the WebSocket client that connects to the Arawn server, handles the chat protocol, and integrates with the TUI event loop.

## Acceptance Criteria

## Acceptance Criteria

- [x] `ArawnClient` struct connects to `ws://localhost:{port}/ws`
- [x] Sends `ClientMessage` enum (Chat, Subscribe, Unsubscribe, Ping, Auth)
- [x] Receives `ServerMessage` enum via tokio channel
- [x] Reconnection on disconnect with exponential backoff
- [x] Connection status displayed in header bar
- [x] Server URL passed to App::new() (CLI flag handled by main.rs)

## Implementation Notes

### Files to Create
```
crates/arawn-tui/src/
├── client.rs          # ArawnClient, connection management
└── protocol.rs        # ClientMessage, ServerMessage enums
```

### Dependencies
```toml
tokio-tungstenite = "0.21"
```

### Protocol
```rust
// Client -> Server
enum ClientMessage {
    Send { content: String },
    Cancel,
    SwitchSession { id: SessionId },
    SwitchWorkstream { id: WorkstreamId },
    NewSession,
    ListSessions,
    ListWorkstreams,
}

// Server -> Client  
enum ServerMessage {
    Delta { content: String },
    ToolStart { id: ToolId, name: String, args: String },
    ToolEnd { id: ToolId, status: ToolStatus, output: String, duration_ms: u64 },
    Done { message_id: MessageId },
    Sessions { items: Vec<SessionSummary> },
    Workstreams { items: Vec<WorkstreamSummary> },
    Error { message: String },
    SessionChanged { id: SessionId, history: Vec<ChatMessage> },
    WorkstreamChanged { id: WorkstreamId },
}
```

### Integration with Event Loop
```rust
tokio::select! {
    Some(Ok(event)) = events.next() => { /* terminal */ }
    Some(msg) = ws_rx.recv() => { /* websocket */ }
}
```

## Status Updates

### 2026-02-11: Implementation Complete

**Files Created:**
- `crates/arawn-tui/src/protocol.rs` - ClientMessage/ServerMessage enums matching server protocol
- `crates/arawn-tui/src/client.rs` - ArawnClient with connection management and reconnection

**Files Modified:**
- `crates/arawn-tui/Cargo.toml` - Added tokio-tungstenite, url, serde, serde_json dependencies
- `crates/arawn-tui/src/lib.rs` - Export new modules
- `crates/arawn-tui/src/app.rs` - Integrated WebSocket client with event loop, added ChatMessage/ToolExecution structs
- `crates/arawn-tui/src/ui/layout.rs` - Connection status indicator in header

**Key Implementation Details:**
- Protocol types mirror `arawn-server/src/routes/ws.rs` exactly
- ConnectionStatus enum: Disconnected, Connecting, Connected, Reconnecting
- Exponential backoff reconnection (100ms * 2^attempt, max 30s)
- `tokio::select!` for concurrent terminal + WebSocket event handling
- Connection status shown in header: ● green (connected), ◐ yellow (connecting), ○ red (disconnected)
- HTTP/HTTPS URLs auto-converted to WS/WSS with /ws path

**Tests:** All 4 tests passing (URL conversion, connection status display, protocol serialization)