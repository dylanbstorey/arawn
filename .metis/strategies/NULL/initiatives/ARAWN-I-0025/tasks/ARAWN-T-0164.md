---
id: tui-input-component-and-history
level: task
title: "TUI: Input component and history"
short_code: "ARAWN-T-0164"
created_at: 2026-02-11T00:28:44.431996+00:00
updated_at: 2026-02-11T00:28:44.431996+00:00
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

# TUI: Input component and history

## Objective

Implement the input area with text editing, multi-line support, input history navigation, and message submission.

## Acceptance Criteria

- [ ] Text input with cursor movement (left/right, home/end)
- [ ] Multi-line input with Shift+Enter
- [ ] Enter sends message (when not empty)
- [ ] Up/Down arrows navigate input history
- [ ] Ctrl+C cancels current generation
- [ ] Input area grows with content (3 lines min, up to ~30% of screen)
- [ ] Visual prompt `> ` prefix

## Implementation Notes

### Files to Create
```
crates/arawn-tui/src/
├── state/
│   └── input.rs       # InputState, history ring buffer
└── ui/
    └── input.rs       # Input area rendering
```

### Input State
```rust
pub struct InputState {
    content: String,
    cursor: usize,          // Byte position
    history: VecDeque<String>,
    history_index: Option<usize>,
    draft: Option<String>,  // Saved when browsing history
}
```

### Key Handling
| Key | Action |
|-----|--------|
| Printable | Insert at cursor |
| Backspace | Delete before cursor |
| Delete | Delete at cursor |
| Left/Right | Move cursor |
| Home/End | Jump to start/end |
| Up | Previous history (save draft first time) |
| Down | Next history or restore draft |
| Enter | Submit if content, else newline |
| Shift+Enter | Always newline |

### Dependencies
- ARAWN-T-0161 (app shell)

## Status Updates

*To be added during implementation*