---
id: background-subagent-execution-with
level: task
title: "Background subagent execution with events"
short_code: "ARAWN-T-0139"
created_at: 2026-02-06T03:47:48.459624+00:00
updated_at: 2026-02-06T13:51:31.929116+00:00
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

# Background subagent execution with events

## Parent Initiative

[[ARAWN-I-0020]] - Subagent Delegation

## Objective

Implement background (non-blocking) subagent execution. When `background: true`, spawn the subagent in a tokio task and return immediately. Fire events when subagent starts and completes.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] When `background: true`, spawn subagent in `tokio::spawn()`
- [x] Return immediately with "delegated in background" message
- [x] Dispatch `SubagentStarted` event when background task begins
- [x] Dispatch `SubagentCompleted` event when background task finishes
- [x] Include parent session ID in events for correlation
- [x] Handle subagent errors in background (log, fire error event)
- [ ] Track active background subagents (optional: for status queries)
- [x] Integration test: background delegation fires events correctly

## Implementation Notes

### Background Execution Path

```rust
if background {
    let spawner = Arc::clone(&self.spawner);
    let config = config.clone();
    let task = task.to_string();
    let agent_name = agent_name.to_string();
    let parent_session_id = ctx.session_id.clone();
    let hook_dispatcher = self.hook_dispatcher.clone();
    
    tokio::spawn(async move {
        // Fire SubagentStarted event
        if let Some(ref dispatcher) = hook_dispatcher {
            dispatcher.dispatch_subagent_started(
                &parent_session_id,
                &agent_name,
                &task[..100.min(task.len())],
            ).await;
        }
        
        let start = std::time::Instant::now();
        let subagent = spawner.spawn(&config)?;
        let mut session = Session::new();
        
        let result = subagent.turn(&mut session, &task).await;
        let duration_ms = start.elapsed().as_millis() as u64;
        
        // Fire SubagentCompleted event
        if let Some(ref dispatcher) = hook_dispatcher {
            let (success, preview) = match &result {
                Ok(r) => (true, r.text[..200.min(r.text.len())].to_string()),
                Err(e) => (false, e.to_string()),
            };
            dispatcher.dispatch_subagent_completed(
                &parent_session_id,
                &agent_name,
                &preview,
                duration_ms,
                success,
            ).await;
        }
    });
    
    return Ok(ToolResult::text(format!(
        "Delegated to '{}' in background. You'll be notified when complete.",
        agent_name
    )));
}
```

### Dependencies

- [[ARAWN-T-0138]] - Blocking execution (shared code)
- [[ARAWN-T-0140]] - Subagent event types

## Status Updates

### 2026-02-06: Starting Implementation

**Current state**: `delegate_background()` exists with TODO comments for events.

**Plan**:
1. Add `SubagentStarted`/`SubagentCompleted` to HookEvent (partial T-0140 work)
2. Add dispatch methods to HookDispatch trait
3. Implement dispatchers in HookDispatcher  
4. Add SharedHookDispatcher to PluginSubagentSpawner
5. Wire events in delegate_background
6. Add optional tracking of active background subagents
7. Tests

### 2026-02-06: Implementation Complete

**Files modified:**

1. **`crates/arawn-types/src/hooks.rs`**
   - Added `SubagentStarted` and `SubagentCompleted` variants to `HookEvent` enum
   - Added Display impl for new events
   - Added `dispatch_subagent_started()` and `dispatch_subagent_completed()` to `HookDispatch` trait
   - Added test for new event serialization

2. **`crates/arawn-plugin/src/hooks.rs`**
   - Added `SubagentStartedContext` and `SubagentCompletedContext` structs for JSON serialization
   - Implemented `dispatch_subagent_started()` and `dispatch_subagent_completed()` in `HookDispatcher`
   - Implemented trait methods in `impl HookDispatch for HookDispatcher`
   - Added 4 new tests: `test_subagent_started_event`, `test_subagent_completed_event`, `test_subagent_completed_failure_event`, `test_subagent_events_no_hooks_registered`

3. **`crates/arawn-plugin/src/agent_spawner.rs`**
   - Added `hook_dispatcher: Option<SharedHookDispatcher>` field to `PluginSubagentSpawner`
   - Added `with_hook_dispatcher()` builder method
   - Updated `delegate_background()` to:
     - Fire `SubagentStarted` event when background task begins
     - Fire `SubagentCompleted` event on success (with result preview, duration, success=true)
     - Fire `SubagentCompleted` event on failure (with error message, duration, success=false)

4. **`crates/arawn-plugin/src/types.rs`**
   - Added `SubagentStarted` and `SubagentCompleted` to test for new hook events serde

5. **`crates/arawn/src/commands/start.rs`**
   - Moved `SharedHookDispatcher` creation before delegate tool setup
   - Wired hook dispatcher into `PluginSubagentSpawner::with_hook_dispatcher()`

**Event JSON format:**
```json
// SubagentStarted
{"parent_session_id":"sess-123","subagent_name":"researcher","task_preview":"Find papers..."}

// SubagentCompleted (success)
{"parent_session_id":"sess-123","subagent_name":"researcher","result_preview":"Found 5...","duration_ms":1500,"success":true}

// SubagentCompleted (failure)
{"parent_session_id":"sess-123","subagent_name":"researcher","result_preview":"Error: timeout","duration_ms":500,"success":false}
```

**Tests passing:** All 142 arawn-plugin tests pass, all 5 arawn-types tests pass.

**Note:** The "Track active background subagents" criterion is marked optional and deferred - it would require additional state management (e.g., `DashMap<String, JoinHandle>`) which adds complexity without immediate benefit.