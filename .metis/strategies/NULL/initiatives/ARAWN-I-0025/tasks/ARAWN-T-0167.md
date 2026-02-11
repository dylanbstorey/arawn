---
id: tui-workstream-panel-and-management
level: task
title: "TUI: Workstream panel and management"
short_code: "ARAWN-T-0167"
created_at: 2026-02-11T00:28:46.928805+00:00
updated_at: 2026-02-11T00:28:46.928805+00:00
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

# TUI: Workstream panel and management

## Objective

Implement the workstream list overlay (Ctrl+W) for browsing, searching, and switching between workstreams (persistent project contexts).

## Acceptance Criteria

- [ ] Ctrl+W opens workstream list as centered overlay
- [ ] Workstreams listed with name, message count, last active
- [ ] Active workstream marked with star (★)
- [ ] Archived workstreams shown in separate section (dimmed)
- [ ] Arrow keys navigate, Enter selects, Esc closes
- [ ] Typing filters workstreams
- [ ] Current workstream shown in header bar

## Implementation Notes

### Files to Create
```
crates/arawn-tui/src/
├── state/
│   └── workstreams.rs  # WorkstreamList state
└── ui/
    └── workstreams.rs  # Workstream panel rendering
```

### Panel Layout
```
┌─ workstreams ───────────────────────────────────────────────────┐
│ > search...                                                      │
├──────────────────────────────────────────────────────────────────┤
│ > ★ Q4 Architecture Redesign                    127 msgs  2h   │
│     Frontend Migration                           89 msgs  1d   │
│     API Refactor                                 45 msgs  3d   │
│   ─────────────────────────────────────────────────────────     │
│     [archived] Old Project                      230 msgs  2mo  │
│                                                                  │
└────────────────────────────── ↑↓ navigate │ enter select │ esc ──┘
```

### Workstream State
```rust
pub struct WorkstreamList {
    items: Vec<WorkstreamSummary>,
    selected: usize,
    filter: String,
}

pub struct WorkstreamSummary {
    id: WorkstreamId,
    name: String,
    message_count: usize,
    last_active: DateTime<Utc>,
    is_current: bool,
    is_archived: bool,
}
```

### Header Integration
Current workstream shown in top-right: `ws:default` or `ws:Q4 Arch...`

### Dependencies
- ARAWN-T-0161 (app shell)
- ARAWN-T-0162 (WebSocket for ListWorkstreams, SwitchWorkstream)

## Status Updates

*To be added during implementation*