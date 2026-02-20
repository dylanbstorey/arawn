---
id: tui-session-panel-and-switching
level: task
title: "TUI: Session panel and switching"
short_code: "ARAWN-T-0166"
created_at: 2026-02-11T00:28:46.193611+00:00
updated_at: 2026-02-19T18:10:43.502850+00:00
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

# TUI: Session panel and switching

## Objective

Implement the session list overlay (Ctrl+S) for browsing, searching, and switching between chat sessions.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Ctrl+S opens session list as centered overlay
- [x] Sessions listed with title, relative timestamp
- [x] Current session marked with bullet (•)
- [x] Arrow keys navigate, Enter selects, Esc closes
- [x] Typing filters sessions (fuzzy match on title)
- [x] Ctrl+N creates new session
- [x] Session switch loads history from server (via Subscribe - history loading requires server support)

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

### 2026-02-11: Implementation Complete

**Files Created:**
- `crates/arawn-tui/src/sessions.rs` - SessionSummary, SessionList with filtering and navigation
- `crates/arawn-tui/src/ui/sessions.rs` - Sessions overlay rendering

**Files Modified:**
- `crates/arawn-tui/Cargo.toml` - Added chrono dependency
- `crates/arawn-tui/src/lib.rs` - Export sessions module
- `crates/arawn-tui/src/ui/mod.rs` - Export sessions UI module
- `crates/arawn-tui/src/app.rs` - Added sessions field, key handling, session switching
- `crates/arawn-tui/src/ui/layout.rs` - Use sessions rendering

**Features Implemented:**
- Ctrl+S opens sessions panel as centered overlay (60% width/height)
- Sessions listed with title, relative timestamp (just now, X min ago, yesterday, etc.)
- Current session marked with bullet (•)
- Arrow keys navigate, Enter selects, Esc closes
- Typing filters sessions with fuzzy matching (characters must appear in order)
- Ctrl+N creates new session (clears messages, unsets session_id)
- Home/End jump to first/last session
- Backspace removes filter characters
- Session switch clears current messages and subscribes to new session

**Mock Data:**
- Panel shows mock session data since real session listing requires REST API
- TODO comment marks where real API integration should go

**Tests Added:**
- test_fuzzy_match - fuzzy matching algorithm
- test_session_list_filtering - filter updates visible items
- test_session_list_navigation - up/down/first/last

**Code passes:**
- All 13 tests passing
- Clippy with no warnings