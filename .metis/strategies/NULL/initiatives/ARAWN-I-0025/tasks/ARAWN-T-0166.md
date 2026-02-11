---
id: tui-session-panel-and-switching
level: task
title: "TUI: Session panel and switching"
short_code: "ARAWN-T-0166"
created_at: 2026-02-11T00:28:46.193611+00:00
updated_at: 2026-02-11T00:28:46.193611+00:00
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

# TUI: Session panel and switching

## Objective

Implement the session list overlay (Ctrl+S) for browsing, searching, and switching between chat sessions.

## Acceptance Criteria

- [ ] Ctrl+S opens session list as centered overlay
- [ ] Sessions listed with title, relative timestamp
- [ ] Current session marked with bullet (•)
- [ ] Arrow keys navigate, Enter selects, Esc closes
- [ ] Typing filters sessions (fuzzy match on title)
- [ ] Ctrl+N creates new session
- [ ] Session switch loads history from server

## Implementation Notes

### Files to Create
```
crates/arawn-tui/src/
├── state/
│   └── sessions.rs    # SessionList state
└── ui/
    └── sessions.rs    # Session panel rendering
```

### Panel Layout
```
┌─ sessions ──────────────────────────────────────────────────────┐
│ > search...                                                      │
├──────────────────────────────────────────────────────────────────┤
│ > • async/await explanation                          2 min ago  │
│     Debug auth middleware                            yesterday  │
│     Rust workspace setup                               2 days   │
│     Memory indexing questions                          3 days   │
│                                                                  │
└────────────────────────────── ↑↓ navigate │ enter select │ esc ──┘
```

### Session State
```rust
pub struct SessionList {
    items: Vec<SessionSummary>,
    selected: usize,
    filter: String,
    filtered_indices: Vec<usize>,
}

pub struct SessionSummary {
    id: SessionId,
    title: String,
    last_active: DateTime<Utc>,
    message_count: usize,
    is_current: bool,
}
```

### Dependencies
- ARAWN-T-0161 (app shell)
- ARAWN-T-0162 (WebSocket for ListSessions, SwitchSession)

## Status Updates

*To be added during implementation*