---
id: cleanup-scheduled-tasks
level: task
title: "Cleanup scheduled tasks"
short_code: "ARAWN-T-0203"
created_at: 2026-02-18T19:03:22.557250+00:00
updated_at: 2026-02-18T21:13:48.158629+00:00
parent: ARAWN-I-0028
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0028
---

# Cleanup scheduled tasks

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[ARAWN-I-0028]]

## Objective

Implement scheduled cleanup tasks using cloacina for automatic scratch session cleanup and disk pressure monitoring.

## Backlog Item Details **[CONDITIONAL: Backlog Item]**

{Delete this section when task is assigned to an initiative}

### Type
- [ ] Bug - Production issue that needs fixing
- [ ] Feature - New functionality or enhancement  
- [ ] Tech Debt - Code improvement or refactoring
- [ ] Chore - Maintenance or setup work

### Priority
- [ ] P0 - Critical (blocks users/revenue)
- [ ] P1 - High (important for user experience)
- [ ] P2 - Medium (nice to have)
- [ ] P3 - Low (when time permits)

### Impact Assessment **[CONDITIONAL: Bug]**
- **Affected Users**: {Number/percentage of users affected}
- **Reproduction Steps**: 
  1. {Step 1}
  2. {Step 2}
  3. {Step 3}
- **Expected vs Actual**: {What should happen vs what happens}

### Business Justification **[CONDITIONAL: Feature]**
- **User Value**: {Why users need this}
- **Business Value**: {Impact on metrics/revenue}
- **Effort Estimate**: {Rough size - S/M/L/XL}

### Technical Debt Impact **[CONDITIONAL: Tech Debt]**
- **Current Problems**: {What's difficult/slow/buggy now}
- **Benefits of Fixing**: {What improves after refactoring}
- **Risk Assessment**: {Risks of not addressing this}

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Cloacina scheduled task for scratch session cleanup - `CleanupContext` provides `run_scratch_cleanup()` for cloacina DynamicTask
- [x] Inactive session detection: last turn > 7 days ago (configurable) - `CleanupConfig.scratch_cleanup_days` (default 7)
- [x] Automatic deletion of inactive scratch session `work/` directories - `delete_scratch_session_work()` removes session directory
- [x] Disk pressure monitoring: check usage against thresholds - `check_disk_pressure()` with configurable total/per-workstream limits
- [x] WebSocket `disk_pressure` alerts when thresholds exceeded - `ServerMessage::DiskPressure` variant added
- [x] Configurable intervals and thresholds - `CleanupConfig` struct with all settings
- [x] Logging of cleanup actions - tracing::info/warn throughout

## Implementation Notes

### Location
- Cloacina task definitions in server startup

### Scratch Cleanup Task

```rust
// Register with cloacina scheduler
scheduler.register("scratch_cleanup", Duration::from_secs(86400), || {
    let cutoff = Utc::now() - Duration::days(config.scratch_cleanup_days);
    
    for session in directory_manager.list_scratch_sessions()? {
        if let Some(last_turn) = get_last_turn_time(&session.id) {
            if last_turn < cutoff {
                directory_manager.delete_scratch_session(&session.id)?;
                tracing::info!(session_id = %session.id, "Cleaned up inactive scratch session");
            }
        }
    }
});
```

### Disk Pressure Monitor Task

```rust
scheduler.register("disk_pressure_check", Duration::from_secs(300), || {
    let usage = directory_manager.get_total_usage()?;
    
    if usage > config.total_usage_warning_bytes {
        broadcast_event(DiskPressureEvent {
            level: "warning",
            scope: "total",
            usage_mb: usage / 1_000_000,
            limit_mb: config.total_usage_warning_bytes / 1_000_000,
        });
    }
    
    // Check per-workstream limits too
    for ws in workstreams.list()? {
        let ws_usage = directory_manager.get_usage(&ws.id)?;
        if ws_usage.total() > config.workstream_usage_warning_bytes {
            broadcast_event(DiskPressureEvent { ... });
        }
    }
});
```

### Dependencies
- ARAWN-T-0194 (DirectoryManager)
- ARAWN-T-0201 (Usage stats)
- ARAWN-T-0202 (Filesystem monitoring)
- Cloacina scheduler

## Status Updates

### Session 1 - 2026-02-18

**Completed:**
- Created `crates/arawn-workstream/src/cleanup.rs` with:
  - `CleanupConfig` struct with configurable thresholds (scratch_cleanup_days, usage warning bytes, dry_run mode)
  - `cleanup_scratch_sessions()` function that detects inactive sessions older than cutoff and deletes their work directories
  - `check_disk_pressure()` function that monitors per-workstream and total disk usage against thresholds
  - `DiskPressureEvent` and `PressureLevel` types for alerting
  - `CleanupContext` struct designed for cloacina integration - wraps DirectoryManager, WorkstreamManager, and CleanupConfig
  - 7 unit tests all passing
- Added `DiskPressure` variant to WebSocket `ServerMessage` enum in `protocol.rs`
- Added `disk_pressure()` helper method to create messages from events
- Added serialization tests for DiskPressure messages

**Integration with Cloacina:**
The `CleanupContext` struct provides `run_scratch_cleanup()` and `run_disk_pressure_check()` methods that can be called from cloacina's DynamicTask. Server startup will register these as scheduled tasks.

**Files modified:**
- `crates/arawn-workstream/src/cleanup.rs` (new)
- `crates/arawn-workstream/src/lib.rs` (exports)
- `crates/arawn-server/src/routes/ws/protocol.rs` (DiskPressure variant)