---
id: tui-context-indicator
level: task
title: "TUI context indicator"
short_code: "ARAWN-T-0190"
created_at: 2026-02-16T18:54:56.924819+00:00
updated_at: 2026-02-16T18:54:56.924819+00:00
parent: ARAWN-I-0026
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


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

- [ ] Status bar shows `[Context: XX%]` indicator
- [ ] Green color when < 70%
- [ ] Yellow color when 70-90%
- [ ] Red color when > 90%
- [ ] Optional: show `[tokens: ~85k/120k]` detail
- [ ] Updates after each turn completes
- [ ] Server sends context status in response metadata

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

*To be added during implementation*