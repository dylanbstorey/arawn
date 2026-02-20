---
id: tui-chat-view-with-streaming
level: task
title: "TUI: Chat view with streaming"
short_code: "ARAWN-T-0163"
created_at: 2026-02-11T00:28:43.552077+00:00
updated_at: 2026-02-19T18:10:42.380589+00:00
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

# TUI: Chat view with streaming

## Objective

Implement the main chat view that displays conversation history and handles streaming responses with visual feedback.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] User messages displayed with `> ` prefix
- [x] Assistant messages with word wrapping
- [x] Streaming text appends in real-time with cursor indicator `▌`
- [x] Auto-scroll to bottom during streaming
- [x] Manual scroll through history (arrow keys when not in input)
- [x] Message buffer manages memory for long conversations (Vec<ChatMessage> in App)

## Implementation Notes

### Files to Create
```
crates/arawn-tui/src/
├── state/
│   ├── mod.rs
│   ├── messages.rs    # ChatMessage, message buffer
│   └── streaming.rs   # Streaming state handler
└── ui/
    └── chat.rs        # Chat view rendering
```

### Message Rendering
```
> User message here

Assistant response that wraps nicely across
multiple lines when the terminal is narrow.

┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄
[shell] ls -la                    ✓ 0.1s
┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄

More assistant text...▌
```

### Streaming Handler
```rust
fn handle_delta(&mut self, content: &str) {
    if let Some(msg) = self.messages.last_mut() {
        if msg.streaming {
            msg.content.push_str(content);
            self.scroll_to_bottom();
        }
    }
}
```

### Dependencies
- ARAWN-T-0161 (app shell)
- ARAWN-T-0162 (WebSocket client)

## Status Updates

### 2026-02-11: Implementation Complete

**Files Created:**
- `crates/arawn-tui/src/ui/chat.rs` - Chat view rendering with streaming support

**Files Modified:**
- `crates/arawn-tui/src/ui/mod.rs` - Export chat module
- `crates/arawn-tui/src/ui/layout.rs` - Use render_chat for content area
- `crates/arawn-tui/src/app.rs` - Added chat_scroll and chat_auto_scroll fields, scroll handling

**Features Implemented:**
- User messages displayed with cyan `> ` prefix
- Assistant messages with word wrapping
- Streaming cursor `▌` shown while message is being received
- Auto-scroll to bottom during streaming (enabled by default)
- Manual scroll with:
  - ↑/↓ when input is empty
  - PageUp/PageDown always
  - Ctrl+Home to scroll to top
  - Ctrl+End to re-enable auto-scroll
- Tool executions displayed with status indicators (◐ running, ✓ success, ✗ failure)
- Welcome screen shown when no messages
- Scroll state preserved when manually scrolling

**Code passes:**
- All tests (4/4 passing)
- Clippy with no warnings