---
id: tui-workstream-panel-and-management
level: task
title: "TUI: Workstream panel and management"
short_code: "ARAWN-T-0167"
created_at: 2026-02-11T00:28:46.928805+00:00
updated_at: 2026-02-12T03:05:49.166515+00:00
parent: ARAWN-I-0025
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/active"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0025
---

# TUI: Workstream panel and management

## Objective

Implement the workstream list overlay (Ctrl+W) for browsing, searching, and switching between workstreams (persistent project contexts).

## Acceptance Criteria

## Acceptance Criteria

- [x] Ctrl+W opens workstream navigation (sidebar-based design)
- [x] Workstreams listed with name and session count
- [x] Active workstream marked with green bullet (●)
- [ ] Archived workstreams shown in separate section (dimmed)
- [x] Arrow keys navigate, Enter selects, Esc/Right closes
- [ ] Typing filters workstreams
- [x] Current workstream shown in header bar
- [x] Tab switches focus between workstreams and sessions sections
- [x] Sessions update when workstream highlighted
- [x] Sidebar collapsible (Ctrl+W cycles: focus → collapse → hide)

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

### 2026-02-11: Sidebar Implementation

Implemented combined sidebar approach instead of overlay (user-approved design):

**Design Decision**: User preferred left sidebar (Option A) + collapsible (Option B) over centered overlay. Sessions list updates when workstream is highlighted.

**Files Created**:
- `crates/arawn-tui/src/sidebar.rs` - Sidebar state with WorkstreamEntry, SidebarSection
- `crates/arawn-tui/src/ui/sidebar.rs` - Sidebar rendering with collapsed/expanded modes

**Files Modified**:
- `crates/arawn-tui/src/app.rs` - Added Sidebar state, Focus::Sidebar, keybinding handlers
- `crates/arawn-tui/src/ui/layout.rs` - Integrated sidebar into horizontal layout
- `crates/arawn-tui/src/lib.rs`, `src/ui/mod.rs` - Module exports

**Features Implemented**:
- Left sidebar with workstreams section at top, sessions below
- Sessions list updates when workstream selection changes
- Ctrl+W toggles: focus → collapse → hide → show expanded
- Tab switches focus between workstreams and sessions sections
- ↑↓ navigation, Enter select, n new, d delete (placeholders)
- Collapsed mode shows icons only (W/S)
- Current workstream/session marked with green bullet

**Remaining**:
- [ ] Connect to real workstream API (currently mock data)
- [ ] Create workstream dialog
- [ ] Delete workstream confirmation
- [ ] Filter/search implementation