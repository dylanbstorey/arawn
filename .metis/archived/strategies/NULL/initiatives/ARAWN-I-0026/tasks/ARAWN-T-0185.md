---
id: contexttracker-implementation
level: task
title: "ContextTracker implementation"
short_code: "ARAWN-T-0185"
created_at: 2026-02-16T18:54:48.948272+00:00
updated_at: 2026-02-16T21:23:55.801206+00:00
parent: ARAWN-I-0026
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


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

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] `ContextTracker` struct with `for_model()` constructor
- [x] `update(token_count)` method to set current usage
- [x] `status()` returns `ContextStatus::Ok | Warning | Critical`
- [x] `usage_percent()` returns current usage as percentage
- [x] `should_compact()` returns true when critical threshold exceeded
- [x] Configurable thresholds (default: 70% warning, 90% critical)
- [x] Integrated with Session (per-session tracking)
- [x] Unit tests for threshold logic

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

### Session 1 (2026-02-16)
- Added `ContextStatus` enum with `Ok`, `Warning { percent }`, `Critical { percent }` variants
- Added `ContextTracker` struct with all required methods:
  - `for_model(max_tokens)` constructor
  - `update(token_count)` and `add(tokens)` methods
  - `status()` returns threshold-based status
  - `usage_percent()` returns 0.0-1.0 percentage
  - `should_compact()` returns true when critical
  - `with_warning_threshold()` and `with_critical_threshold()` for custom thresholds
  - Helper methods: `current_tokens()`, `max_tokens()`, `remaining_tokens()`, `reset()`
- Added 17 unit tests for ContextTracker
- Exported `ContextTracker` and `ContextStatus` from `arawn-agent` crate
- All 32 context-related tests pass

### Session 2 (2026-02-16)
- Added `context_tracker: Option<ContextTracker>` to Session struct with `#[serde(skip)]`
- Added `init_context_tracker(max_tokens)` method to initialize tracking
- Added `context_tracker()` and `context_tracker_mut()` accessors
- Added 2 unit tests for Session context tracker integration
- All 382 arawn-agent tests pass

### Session 3 (2026-02-16)
- Improved `ContextStatus` enum per user feedback: now carries `current` and `max` token counts instead of just percentage
- This provides better context (95% of 10k vs 95% of 200k are very different situations)
- Added helper methods to ContextStatus: `current()`, `max()`, `percent()`, `remaining()`
- Added 2 more tests for ContextStatus helper methods
- All 26 context tests pass

**All acceptance criteria complete.**