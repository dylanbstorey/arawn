---
id: complete-tui-placeholder-todos
level: task
title: "Complete TUI placeholder TODOs"
short_code: "ARAWN-T-0263"
created_at: 2026-03-04T13:24:12.192199+00:00
updated_at: 2026-03-05T03:55:51.021924+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#feature"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
---

# Complete TUI placeholder TODOs

## Objective

Implement the 15+ TODO/placeholder items in the TUI crate. Many panels and keybindings are stubbed out.

## Backlog Item Details

### Type
- [x] Feature - New functionality or enhancement

### Priority
- [x] P3 - Low (when time permits)

### Business Justification
- **User Value**: TUI is partially functional; completing TODOs makes it a usable interface
- **Effort Estimate**: L

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Audit all TODO comments in `crates/arawn-tui/src/`
- [ ] Implement or remove each TODO
- [ ] All TUI panels render real data (not placeholders)
- [ ] Keybindings documented in help panel

## Status Updates

### Audit Results
- Found 3 actual TODOs, all in `app.rs:1614-1622` (workstreams overlay handler)
- Found "placeholder" references in `tools.rs` — these are legitimate empty-state UI, not stubs
- Sessions overlay already fully implemented (nav, filter, enter, Ctrl+N)
- Command palette already fully implemented
- Help panel already documents keybindings
- Sidebar navigation already fully implemented

### Implementation
- **`app.rs` `handle_overlay_key()`**: Replaced 3 TODOs with working code:
  - Enter: selects workstream via `switch_to_workstream()`, clears filter, closes overlay
  - Up/Down: navigates workstream list via `sidebar.select_prev/next()`
  - Char: pushes to `sidebar.filter` for live filtering
  - Backspace: pops from filter
  - Esc: clears filter and closes overlay
- **`ui/layout.rs` `render_workstreams_overlay()`**: Replaced hardcoded placeholder with real data:
  - Renders from `app.sidebar.visible_workstreams()` (filtered)
  - Shows ★ for current workstream, › for selected
  - Shows session count per workstream
  - Highlights selected item in cyan/bold
  - Shows search prompt with current filter text
  - Footer with navigation hints

### Verification
- `angreal check all` — clean (clippy + fmt)
- `angreal test unit` — all tests pass