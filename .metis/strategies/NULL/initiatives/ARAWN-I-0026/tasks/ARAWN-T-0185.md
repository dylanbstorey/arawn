---
id: contexttracker-implementation
level: task
title: "ContextTracker implementation"
short_code: "ARAWN-T-0185"
created_at: 2026-02-16T18:54:48.948272+00:00
updated_at: 2026-02-16T18:54:48.948272+00:00
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

# ContextTracker implementation

## Parent Initiative

[[ARAWN-I-0026]] - Context Management and Auto-Compaction

## Objective

Implement ContextTracker struct that tracks token usage per session, provides threshold status, and signals when compaction is needed.

## Acceptance Criteria

- [ ] `ContextTracker` struct with `for_model()` constructor
- [ ] `update(token_count)` method to set current usage
- [ ] `status()` returns `ContextStatus::Ok | Warning | Critical`
- [ ] `usage_percent()` returns current usage as percentage
- [ ] `should_compact()` returns true when critical threshold exceeded
- [ ] Configurable thresholds (default: 70% warning, 90% critical)
- [ ] Integrated with Session (per-session tracking)
- [ ] Unit tests for threshold logic

## Implementation Notes

### Files to Modify
- `crates/arawn-agent/src/context.rs` - Add ContextTracker
- `crates/arawn-agent/src/types.rs` - Add to Session struct

### Key Types

```rust
pub struct ContextTracker {
    max_tokens: usize,
    current_tokens: usize,
    warning_threshold: f32,
    critical_threshold: f32,
}

pub enum ContextStatus {
    Ok,
    Warning { percent: f32 },
    Critical { percent: f32 },
}
```

### Dependencies
- ARAWN-T-0184 (Model context limits in config)

## Tests

### Unit Tests
- `test_context_tracker_ok_status` - returns Ok when under warning threshold
- `test_context_tracker_warning_status` - returns Warning at 70%+ usage
- `test_context_tracker_critical_status` - returns Critical at 90%+ usage
- `test_context_tracker_should_compact` - true only when critical
- `test_context_tracker_usage_percent` - correct percentage calculation
- `test_context_tracker_custom_thresholds` - non-default thresholds work
- `test_context_tracker_update` - token count updates correctly

### Test File
- `crates/arawn-agent/src/context.rs` (inline `#[cfg(test)]` module)

## Status Updates

*To be added during implementation*