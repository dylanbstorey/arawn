---
id: tui-command-palette
level: task
title: "TUI: Command palette"
short_code: "ARAWN-T-0168"
created_at: 2026-02-11T00:28:47.831875+00:00
updated_at: 2026-02-11T00:28:47.831875+00:00
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

# TUI: Command palette

## Objective

Implement the command palette (Ctrl+K) for discovering and executing actions via fuzzy search.

## Acceptance Criteria

- [ ] Ctrl+K opens command palette as centered overlay
- [ ] Actions listed with label, optional keyboard shortcut
- [ ] Typing filters actions (fuzzy match)
- [ ] Actions grouped by category with separators
- [ ] Enter executes selected action
- [ ] Esc closes palette
- [ ] Extensible action registry for future commands

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

*To be added during implementation*