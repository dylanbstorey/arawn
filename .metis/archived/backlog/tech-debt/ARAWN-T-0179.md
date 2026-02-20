---
id: tui-focus-management-extraction
level: task
title: "TUI Focus Management Extraction"
short_code: "ARAWN-T-0179"
created_at: 2026-02-13T16:39:54.329855+00:00
updated_at: 2026-02-13T21:12:32.389020+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#tech-debt"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# TUI Focus Management Extraction

## Objective

Extract focus management logic from `app.rs` into a dedicated `FocusManager` component to improve TUI maintainability.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P2 - Medium (nice to have)

### Technical Debt Impact
- **Current Problems**: Focus state, transitions, and input routing are interleaved throughout `app.rs`. Adding new panels requires touching many places.
- **Benefits of Fixing**: Centralized focus logic, easier to add new panels, clearer state machine for focus transitions.
- **Risk Assessment**: LOW - Current approach works but will become harder to maintain as TUI grows.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Create `crates/arawn-tui/src/focus.rs` with `FocusManager` struct
- [x] Define `FocusTarget` enum for all focusable panels
- [x] Implement focus transition rules (what can focus from where)
- [x] Extract input routing logic based on current focus
- [x] Handle overlay state (command palette takes focus priority)
- [x] Update `app.rs` to use `FocusManager`
- [x] No behavior changes - pure refactor

## Implementation Notes

### Technical Approach

```rust
pub enum FocusTarget {
    Chat,
    Sidebar,
    Sessions,
    Tools,
    Input,
    CommandPalette,  // Overlay - takes priority
}

pub struct FocusManager {
    current: FocusTarget,
    previous: Option<FocusTarget>,  // For returning from overlays
    overlay_stack: Vec<FocusTarget>,
}

impl FocusManager {
    pub fn focus(&mut self, target: FocusTarget);
    pub fn cycle_next(&mut self);
    pub fn cycle_prev(&mut self);
    pub fn push_overlay(&mut self, overlay: FocusTarget);
    pub fn pop_overlay(&mut self);
    pub fn current(&self) -> FocusTarget;
}
```

### Input Routing
Move keyboard handling dispatch from `App::handle_key_event` into `FocusManager::route_input()` or keep in App but make it cleaner with match on `focus.current()`.

### When to Do This
Defer until TUI has more panels or focus logic becomes unwieldy. Current state is manageable.

## Status Updates

### Completed
- Created `focus.rs` with `FocusTarget` enum and `FocusManager` struct
- Implemented overlay stack for command palette, sessions, workstreams overlays
- Added `push_overlay()` / `pop_overlay()` for proper overlay dismiss behavior
- Added `toggle()`, `cycle_next()`, `cycle_prev()` for panel navigation
- Refactored `app.rs` to use `FocusManager` instead of bare `Focus` enum
- Updated `ui/layout.rs` to use `FocusTarget`
- All 43 tests passing (12 new focus tests)