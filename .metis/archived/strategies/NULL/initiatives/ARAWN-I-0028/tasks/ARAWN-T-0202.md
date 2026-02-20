---
id: filesystem-monitoring
level: task
title: "Filesystem monitoring"
short_code: "ARAWN-T-0202"
created_at: 2026-02-18T19:03:21.738936+00:00
updated_at: 2026-02-18T21:14:12.759842+00:00
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

# Filesystem monitoring

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[ARAWN-I-0028]]

## Objective

Implement filesystem monitoring using the `notify` crate to watch workstream directories and broadcast changes via WebSocket.

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

- [x] `FileWatcher` struct using `notify` crate for cross-platform watching
- [x] Watch `production/` and `work/` directories for each workstream
- [x] 500ms debounce for rapid file changes (configurable via FileWatcherConfig)
- [x] Polling fallback handled by notify crate internally
- [x] WebSocket `fs_change` events (ServerMessage::FsChange variant added)
- [x] Events include: workstream, path, action (created/modified/deleted), timestamp
- [x] Integration tests for file change detection (test_watcher_start_and_detect_changes)

## WebSocket Event

```json
{ 
  "event": "fs_change", 
  "workstream": "my-blog", 
  "path": "production/post.md", 
  "action": "modified",
  "timestamp": "2026-02-18T12:00:00Z"
}
```

## Implementation Notes

### Location
- `crates/arawn-workstream/src/watcher.rs` (new file)

### Core Logic

```rust
use notify::{Watcher, RecursiveMode, watcher};
use std::sync::mpsc::channel;
use std::time::Duration;

pub struct FileWatcher {
    watcher: notify::RecommendedWatcher,
    debounce_ms: u64,
}

impl FileWatcher {
    pub fn new(debounce_ms: u64) -> Result<Self, WatcherError> {
        let (tx, rx) = channel();
        let watcher = watcher(tx, Duration::from_millis(debounce_ms))?;
        Ok(Self { watcher, debounce_ms })
    }
    
    pub fn watch_workstream(&mut self, path: &Path) -> Result<(), WatcherError> {
        self.watcher.watch(path.join("production"), RecursiveMode::Recursive)?;
        self.watcher.watch(path.join("work"), RecursiveMode::Recursive)?;
        Ok(())
    }
    
    pub async fn run(&self, broadcast: impl Fn(FsChangeEvent)) {
        // Event loop with debouncing
    }
}

pub struct FsChangeEvent {
    pub workstream: String,
    pub path: PathBuf,
    pub action: FsAction,
}

pub enum FsAction {
    Created,
    Modified,
    Deleted,
}
```

### Debouncing Strategy
- Collect events for 500ms window
- Coalesce multiple events for same path
- Emit single event per path per window

### Dependencies
- ARAWN-T-0194 (DirectoryManager)
- `notify` crate

## Status Updates

### Session 1 - Implementation Complete

**Completed:**

1. Added dependencies to `arawn-workstream/Cargo.toml`:
   - `notify = "7"` - Cross-platform filesystem watching
   - `notify-debouncer-mini = "0.5"` - Event debouncing

2. Created `crates/arawn-workstream/src/watcher.rs` with:
   - `WatcherError` enum for error handling
   - `FsAction` enum (Created, Modified, Deleted) with serde serialization
   - `FsChangeEvent` struct with workstream, path, action, timestamp
   - `FileWatcherConfig` struct with debounce_ms (default 500) and buffer_size
   - `FileWatcher` struct for managing filesystem watching
   - `WatcherHandle` for managing the background watcher thread
   - Helper functions for path resolution

3. `FileWatcher` implementation:
   - `new()` / `with_config()` constructors
   - `start(workstreams)` - starts watching and returns event receiver
   - `get_watch_paths()` - returns paths to watch for a workstream
   - `watched_workstreams()` - lists currently watched workstreams
   - Watches `production/` and `work/` for named workstreams
   - Watches `sessions/` for scratch workstream
   - Uses background thread with debounced notify watcher
   - Properly handles path → workstream mapping

4. Added WebSocket `FsChange` message variant to `ServerMessage`:
   - Fields: workstream, path, action, timestamp
   - `fs_change()` helper method for creating from FsChangeEvent

5. Added 11 watcher unit tests:
   - `test_fs_action_display` - action formatting
   - `test_fs_change_event_new` - event creation
   - `test_fs_change_event_serialization` - JSON round-trip
   - `test_file_watcher_config_default` - default config values
   - `test_get_watch_paths_named_workstream` - prod/work paths
   - `test_get_watch_paths_scratch` - sessions path
   - `test_get_watch_paths_nonexistent` - error handling
   - `test_get_watch_paths_invalid_name` - validation
   - `test_find_workstream_for_path` - path resolution
   - `test_calculate_relative_path` - relative path calculation
   - `test_watcher_start_and_detect_changes` - integration test

6. Added 1 protocol test:
   - `test_fs_change_serialization` - WebSocket message format

7. Exported from `lib.rs`:
   - `FileWatcher`, `FileWatcherConfig`, `FsAction`, `FsChangeEvent`
   - `WatcherError`, `WatcherHandle`, `WatcherResult`
   - `DEFAULT_DEBOUNCE_MS`, `DEFAULT_POLL_INTERVAL_SECS`

**Tests:** 135 workstream tests pass, 115 server tests pass