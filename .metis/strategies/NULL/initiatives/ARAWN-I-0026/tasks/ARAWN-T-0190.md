---
id: tui-context-indicator
level: task
title: "TUI context indicator"
short_code: "ARAWN-T-0190"
created_at: 2026-02-16T18:54:56.924819+00:00
updated_at: 2026-02-17T02:09:28.195517+00:00
parent: ARAWN-I-0026
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0026
---

# TUI context indicator

## Parent Initiative

[[ARAWN-I-0026]] - Context Management and Auto-Compaction

## Objective

Add context usage indicator to TUI status bar showing current token usage percentage with color-coded thresholds.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Status bar shows `[Context: XX%]` indicator
- [x] Green color when < 70%
- [x] Yellow color when 70-90%
- [x] Red color when > 90%
- [x] Optional: show `[tokens: ~85k/120k]` detail - shows `[~85k/120k 72%]`
- [x] Updates after each turn completes
- [x] Server sends context status in response metadata

## Implementation Notes

### Files to Modify
- `crates/arawn-tui/src/ui/status_bar.rs`
- `crates/arawn-tui/src/app.rs` - store context state

### Display Format

```
[workstream: arawn-dev] [session: abc123] [Context: 72%]
                                          ^^^^^^^^^^^^
                                          green/yellow/red
```

### Server Response Addition

```rust
// Add to chat/turn response
pub struct ContextInfo {
    pub usage_percent: f32,
    pub current_tokens: usize,
    pub max_tokens: usize,
    pub status: String,  // "ok" | "warning" | "critical"
}
```

### Dependencies
- ARAWN-T-0185 (ContextTracker)

## Tests

### Unit Tests
- `test_context_info_serialize` - ContextInfo serializes to JSON correctly
- `test_status_bar_format` - correct format string with percentage
- `test_status_bar_color_ok` - green when < 70%
- `test_status_bar_color_warning` - yellow when 70-90%
- `test_status_bar_color_critical` - red when > 90%

### Component Tests
- `test_status_bar_render` - status bar renders with context indicator
- `test_status_bar_updates` - indicator updates when context state changes

### Test File
- `crates/arawn-tui/src/ui/status_bar.rs` (inline `#[cfg(test)]` module)

## Status Updates

### Session 2026-02-17

**Completed implementation:**

1. **Server protocol** (`crates/arawn-server/src/routes/ws/protocol.rs`):
   - Added `ServerMessage::ContextInfo` with fields: session_id, current_tokens, max_tokens, percent, status
   - Added `context_info()` constructor with threshold logic (ok < 70%, warning 70-90%, critical >= 90%)
   - 4 new tests for serialization and boundary conditions

2. **TUI protocol** (`crates/arawn-tui/src/protocol.rs`):
   - Added matching `ServerMessage::ContextInfo` variant
   - Added deserialization test

3. **App state** (`crates/arawn-tui/src/app.rs`):
   - Added `ContextState` struct with current_tokens, max_tokens, percent, status
   - Added `context_info: Option<ContextState>` field to App
   - Added handler in `handle_server_message()` to update state on ContextInfo

4. **Status bar UI** (`crates/arawn-tui/src/ui/layout.rs`):
   - Updated `render_status_bar()` to display context indicator on right side
   - Added `format_context_indicator()` function with color coding:
     - Green for "ok" (< 70%)
     - Yellow for "warning" (70-90%)
     - Red for "critical" (>= 90%)
   - Format: `[~85k/120k 72%]` showing tokens and percentage

**Tests passing**: 12 WebSocket tests, 53 TUI tests