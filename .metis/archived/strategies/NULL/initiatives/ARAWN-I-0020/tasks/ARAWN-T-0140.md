---
id: subagent-lifecycle-event-types
level: task
title: "Subagent lifecycle event types"
short_code: "ARAWN-T-0140"
created_at: 2026-02-06T03:47:49.362053+00:00
updated_at: 2026-02-06T13:51:32.508213+00:00
parent: ARAWN-I-0020
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0020
---

# Subagent lifecycle event types

## Parent Initiative

[[ARAWN-I-0020]] - Subagent Delegation

## Objective

Add `SubagentStarted` and `SubagentCompleted` event types to the hook system, enabling plugins to react to subagent lifecycle events.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Add `SubagentStarted` variant to `HookEvent` enum in `arawn-types`
- [x] Add `SubagentCompleted` variant to `HookEvent` enum
- [x] Add `dispatch_subagent_started()` method to `HookDispatch` trait
- [x] Add `dispatch_subagent_completed()` method to `HookDispatch` trait
- [x] Implement dispatchers in `HookDispatcher`
- [x] JSON context includes: parent_session_id, subagent_name, task_preview, duration_ms, success
- [x] Unit tests for event serialization
- [x] Hook scripts can subscribe to `SubagentStarted` / `SubagentCompleted`

## Implementation Notes

### Event Types in arawn-types/src/hooks.rs

```rust
pub enum HookEvent {
    // ... existing ...
    SubagentStarted,
    SubagentCompleted,
}
```

### Trait Methods

```rust
#[async_trait]
pub trait HookDispatch: Send + Sync {
    // ... existing ...
    
    async fn dispatch_subagent_started(
        &self,
        parent_session_id: &str,
        subagent_name: &str,
        task_preview: &str,
    ) -> HookOutcome;
    
    async fn dispatch_subagent_completed(
        &self,
        parent_session_id: &str,
        subagent_name: &str,
        result_preview: &str,
        duration_ms: u64,
        success: bool,
    ) -> HookOutcome;
}
```

### JSON Context for Hooks

```json
{
  "event": "SubagentCompleted",
  "parent_session_id": "abc-123",
  "subagent_name": "researcher",
  "result_preview": "Found 5 relevant papers...",
  "duration_ms": 15234,
  "success": true
}
```

### Files to Modify

- `crates/arawn-types/src/hooks.rs` - Add event variants and trait methods
- `crates/arawn-plugin/src/hooks.rs` - Implement trait methods in HookDispatcher

### Dependencies

- ARAWN-T-0134 (Hook system wiring) - ✅ Complete

## Status Updates

### 2026-02-06: Already Implemented in ARAWN-T-0139

This task was fully implemented as part of ARAWN-T-0139 (Background subagent execution with events). The event types were required to make background execution useful.

**Implementation verified in:**

1. **`crates/arawn-types/src/hooks.rs`** (lines 37, 39):
   - `SubagentStarted` variant added to `HookEvent` enum
   - `SubagentCompleted` variant added to `HookEvent` enum
   - Display impl for both events (lines 56-57)
   - `dispatch_subagent_started()` method in `HookDispatch` trait (line 178)
   - `dispatch_subagent_completed()` method in `HookDispatch` trait (line 188)
   - Unit test `test_subagent_events_serde` for event serialization (line 235)

2. **`crates/arawn-plugin/src/hooks.rs`**:
   - `SubagentStartedContext` struct (line 485) with fields: `parent_session_id`, `subagent_name`, `task_preview`
   - `SubagentCompletedContext` struct (line 492) with fields: `parent_session_id`, `subagent_name`, `result_preview`, `duration_ms`, `success`
   - `dispatch_subagent_started()` implementation (line 174)
   - `dispatch_subagent_completed()` implementation (line 190)
   - Trait implementation in `impl HookDispatch for HookDispatcher` (lines 415, 424)
   - 4 unit tests: `test_subagent_started_event`, `test_subagent_completed_event`, `test_subagent_completed_failure_event`, `test_subagent_events_no_hooks_registered`

3. **`crates/arawn-plugin/src/types.rs`**:
   - Added `SubagentStarted` and `SubagentCompleted` to `test_new_hook_events_serde` test

**JSON Context format:**
```json
// SubagentStarted
{"parent_session_id":"sess-123","subagent_name":"researcher","task_preview":"Find papers..."}

// SubagentCompleted
{"parent_session_id":"sess-123","subagent_name":"researcher","result_preview":"Found 5...","duration_ms":1500,"success":true}
```

**Hook subscription:** Plugins can now subscribe to `SubagentStarted` and `SubagentCompleted` events in their `hooks/hooks.json` file, just like any other hook event.

All tests pass (verified in T-0139 completion).