---
id: tui-workstream-panel-and-management
level: task
title: "TUI: Workstream panel and management"
short_code: "ARAWN-T-0167"
created_at: 2026-02-11T00:28:46.928805+00:00
updated_at: 2026-02-20T02:03:25.981547+00:00
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

# TUI: Workstream panel and management

## Objective

Implement the workstream list overlay (Ctrl+W) for browsing, searching, and switching between workstreams (persistent project contexts).

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Ctrl+W opens workstream navigation (sidebar-based design)
- [x] Workstreams listed with name and session count
- [x] Active workstream marked with green bullet (●)
- [x] Archived workstreams shown in separate section (dimmed)
- [x] Arrow keys navigate, Enter selects, Esc/Right closes
- [x] Typing filters workstreams
- [x] Current workstream shown in header bar
- [x] Tab switches focus between workstreams and sessions sections
- [x] Sessions update when workstream highlighted
- [x] Sidebar collapsible (Ctrl+W cycles: focus → collapse → hide)
- [x] Create workstream dialog with validation (empty name, duplicates, length)
- [x] Delete confirmation (press 'd' twice)

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
- [x] Connect to real workstream API (previously mock data) - DONE 2026-02-19
- [ ] Create workstream dialog
- [ ] Delete workstream confirmation
- [ ] Filter/search implementation

### 2026-02-19: Implementing Remaining Features

**Completed all remaining features:**

1. **Delete confirmation (press 'd' twice)** ✅
   - Added `pending_delete_workstream` and `pending_delete_session` fields to App
   - First 'd' press: shows confirmation message in status bar
   - Second 'd' press: executes delete
   - Clears pending on: Esc, navigation, Tab, Enter, other actions
   - Cannot delete: current workstream, scratch workstream, current session

2. **Create workstream validation** ✅
   - Empty name rejected with message
   - Length validation (max 100 chars)
   - Duplicate name check (case-insensitive)
   - Better status message: "New workstream: Enter name (Esc to cancel)"

3. **Archived workstreams section** ✅
   - Added `list_all_workstreams()` to WorkstreamManager
   - Added `include_archived` query param to server endpoint
   - Added `list_all()` to client API
   - Added `state` field to WorkstreamEntry
   - Sidebar shows archived workstreams in separate section with "─ archived ─" header
   - Archived items rendered with DarkGray + Italic styling

**Files Modified:**
- `crates/arawn-workstream/src/manager.rs` - Added list_all_workstreams()
- `crates/arawn-server/src/routes/workstreams.rs` - Added ListWorkstreamsQuery with include_archived
- `crates/arawn-client/src/api/workstreams.rs` - Added list_all() method
- `crates/arawn-tui/src/sidebar.rs` - Added state field, is_archived(), archived iteration methods
- `crates/arawn-tui/src/app.rs` - Delete confirmation logic, validation, uses list_all()
- `crates/arawn-tui/src/ui/sidebar.rs` - Archived section rendering with dimmed styling

All tests passing.

### 2026-02-19: Connected to Real API

Removed all mock data from sidebar - now uses real REST API:

**Changes**:
- Removed `populate_mock_workstreams()`, `populate_mock_sessions()`, `refresh_sessions_for_workstream()` from `sidebar.rs`
- Updated `select_prev()` and `select_next()` to return `Option<String>` (workstream ID) when selection changes
- Updated `app.rs` to trigger `FetchWorkstreamSessions` API call when workstream selection changes
- Workstreams loaded via `refresh_sidebar_data()` on WebSocket connect
- Sessions loaded via `do_fetch_workstream_sessions()` from real API

The sidebar now displays real workstreams and sessions from the server.