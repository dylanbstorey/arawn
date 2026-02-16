---
id: tui-command-palette
level: task
title: "TUI: Command palette"
short_code: "ARAWN-T-0168"
created_at: 2026-02-11T00:28:47.831875+00:00
updated_at: 2026-02-12T01:28:08.302991+00:00
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

# TUI: Command palette

## Objective

Implement the command palette (Ctrl+K) for discovering and executing actions via fuzzy search.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Ctrl+K opens command palette as centered overlay
- [x] Actions listed with label, optional keyboard shortcut
- [x] Typing filters actions (fuzzy match)
- [x] Actions grouped by category with separators
- [x] Enter executes selected action
- [x] Esc closes palette
- [x] Extensible action registry for future commands

## Implementation Notes

### Files to Create
```
crates/arawn-tui/src/
├── state/
│   └── palette.rs     # CommandPalette state, Action registry
└── ui/
    └── palette.rs     # Palette rendering
```

### Panel Layout
```
┌─ command ────────────────────────────────────────────────────────┐
│ > ses                                                            │
├──────────────────────────────────────────────────────────────────┤
│   Sessions: Switch...                                   Ctrl+S   │
│   Sessions: New                                         Ctrl+N   │
│   Sessions: Delete current                                       │
│   ─────────────────────────────────────────────────────────────  │
│   Workstreams: Switch...                                Ctrl+W   │
│   Workstreams: Create                                            │
└──────────────────────────────────────────────────────────────────┘
```

### Action Registry
```rust
pub struct Action {
    id: &'static str,
    label: String,
    category: &'static str,
    shortcut: Option<KeyBinding>,
    handler: Box<dyn Fn(&mut App) -> Result<()>>,
}

pub struct CommandPalette {
    actions: Vec<Action>,
    filter: String,
    filtered_indices: Vec<usize>,
    selected: usize,
}
```

### Initial Actions
| Category | Action | Shortcut |
|----------|--------|----------|
| Sessions | Switch... | Ctrl+S |
| Sessions | New | Ctrl+N |
| Sessions | Delete current | - |
| Workstreams | Switch... | Ctrl+W |
| Workstreams | Create | - |
| View | Toggle tool pane | Ctrl+E |
| View | Open tool in editor | Ctrl+O |
| App | Quit | Ctrl+Q |

### Dependencies
- ARAWN-T-0161 (app shell)
- ARAWN-T-0166 (session actions)
- ARAWN-T-0167 (workstream actions)

## Status Updates

### 2026-02-11: Implementation Complete

**Files Created:**
- `crates/arawn-tui/src/palette.rs` - CommandPalette state, Action registry, ActionId enum
- `crates/arawn-tui/src/ui/palette.rs` - Palette overlay rendering

**Files Modified:**
- `crates/arawn-tui/src/lib.rs` - Export palette module
- `crates/arawn-tui/src/ui/mod.rs` - Export palette UI module
- `crates/arawn-tui/src/app.rs` - Added palette field, handle_palette_key, execute_action
- `crates/arawn-tui/src/ui/layout.rs` - Use palette rendering

**Features Implemented:**
- Ctrl+K opens command palette as centered overlay (60% width, 50% height)
- Actions listed with label and optional keyboard shortcut
- Typing filters actions with fuzzy matching
- Actions grouped by category with separators between groups
- Enter executes selected action
- Esc closes palette
- Up/Down/Home/End navigation
- Backspace removes filter characters
- Extensible ActionId enum for adding new commands

**Initial Actions:**
- Sessions: Switch... (Ctrl+S)
- Sessions: New (Ctrl+N)
- Sessions: Delete current
- Workstreams: Switch... (Ctrl+W)
- Workstreams: Create
- View: Toggle tool pane (Ctrl+E)
- App: Quit (Ctrl+Q)

**Action Execution:**
- All actions dispatch to appropriate handlers
- Unimplemented actions show status message

**Tests Added:**
- test_palette_filtering - fuzzy filter updates
- test_palette_navigation - up/down/first/last
- test_palette_action_selection - selecting specific actions
- test_category_grouping - verifies category separators

**Code passes:**
- All 17 tests passing
- Clippy with no warnings