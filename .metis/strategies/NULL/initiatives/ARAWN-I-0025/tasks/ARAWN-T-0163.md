---
id: tui-chat-view-with-streaming
level: task
title: "TUI: Chat view with streaming"
short_code: "ARAWN-T-0163"
created_at: 2026-02-11T00:28:43.552077+00:00
updated_at: 2026-02-11T00:28:43.552077+00:00
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

# TUI: Chat view with streaming

## Objective

Implement the main chat view that displays conversation history and handles streaming responses with visual feedback.

## Acceptance Criteria

- [ ] User messages displayed with `> ` prefix
- [ ] Assistant messages with word wrapping
- [ ] Streaming text appends in real-time with cursor indicator `▌`
- [ ] Auto-scroll to bottom during streaming
- [ ] Manual scroll through history (arrow keys when not in input)
- [ ] Message buffer manages memory for long conversations

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

*To be added during implementation*