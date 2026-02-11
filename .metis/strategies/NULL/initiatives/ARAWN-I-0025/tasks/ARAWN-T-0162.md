---
id: tui-websocket-client-integration
level: task
title: "TUI: WebSocket client integration"
short_code: "ARAWN-T-0162"
created_at: 2026-02-11T00:28:42.732067+00:00
updated_at: 2026-02-11T00:28:42.732067+00:00
parent: ARAWN-I-0025
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0025
---

# TUI: WebSocket client integration

## Objective

Implement the WebSocket client that connects to the Arawn server, handles the chat protocol, and integrates with the TUI event loop.

## Acceptance Criteria

- [ ] `ArawnClient` struct connects to `ws://localhost:{port}/ws/chat`
- [ ] Sends `ClientMessage` enum (Send, Cancel, SwitchSession, etc.)
- [ ] Receives `ServerMessage` enum via tokio channel
- [ ] Reconnection on disconnect with backoff
- [ ] Connection status displayed in status bar
- [ ] `--server` flag to specify server URL

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

*To be added during implementation*