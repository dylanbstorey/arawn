---
id: tui-input-component-and-history
level: task
title: "TUI: Input component and history"
short_code: "ARAWN-T-0164"
created_at: 2026-02-11T00:28:44.431996+00:00
updated_at: 2026-02-12T00:46:20.670582+00:00
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

# TUI: Input component and history

## Objective

Implement the input area with text editing, multi-line support, input history navigation, and message submission.

## Acceptance Criteria

## Acceptance Criteria

- [x] Text input with cursor movement (left/right, home/end)
- [x] Multi-line input with Shift+Enter
- [x] Enter sends message (when not empty)
- [x] Up/Down arrows navigate input history
- [x] Ctrl+C cancels current generation
- [x] Input area grows with content (3 lines min, up to ~30% of screen)
- [x] Visual prompt `> ` prefix

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

### 2026-02-11: Implementation Complete

**Files Created:**
- `crates/arawn-tui/src/input.rs` - InputState with history, cursor movement, multi-line support
- `crates/arawn-tui/src/ui/input.rs` - Input area rendering with dynamic height

**Files Modified:**
- `crates/arawn-tui/src/lib.rs` - Export input module
- `crates/arawn-tui/src/ui/mod.rs` - Export input UI module
- `crates/arawn-tui/src/app.rs` - Use InputState, updated key handling
- `crates/arawn-tui/src/ui/layout.rs` - Dynamic input height calculation

**Features Implemented:**
- Text input with full cursor movement (left/right, home/end)
- Multi-line input with Shift+Enter to insert newlines
- Enter sends message when not empty and not waiting
- Up/Down navigates input history when at boundaries
  - First line: Up goes to previous history
  - Last line: Down goes to next history or restores draft
- Draft preserved when browsing history
- History stores up to 100 entries, deduplicates consecutive entries
- Input area grows dynamically (3 lines min, 30% max)
- Visual prompt `> ` on first line, `  ` continuation on subsequent lines
- Ctrl+C cancels current generation (existing)
- Auto-scroll enabled when sending message

**Tests Added:**
- test_basic_input - character insertion
- test_cursor_movement - left/right/home/end
- test_backspace - character deletion
- test_history - history navigation
- test_multiline - newline handling and line count
- test_history_with_draft - draft preservation

**Code passes:**
- All 10 tests passing
- Clippy with no warnings