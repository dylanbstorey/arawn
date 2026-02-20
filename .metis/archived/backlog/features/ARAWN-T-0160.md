---
id: wire-workstreammanager-into-server
level: task
title: "Wire WorkstreamManager into server startup"
short_code: "ARAWN-T-0160"
created_at: 2026-02-09T17:17:49.395896+00:00
updated_at: 2026-02-09T21:31:51.173288+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#feature"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Wire WorkstreamManager into server startup

## Objective

Connect the `arawn-workstream` crate to the server so the workstream API endpoints (`/api/v1/workstreams/*`) function correctly instead of returning "Workstreams not configured".

## Backlog Item Details

### Type
- [x] Feature - New functionality or enhancement  

### Priority
- [ ] P1 - High (important for user experience)

### Business Justification
- **User Value**: Workstreams provide persistent conversation contexts that span multiple sessions
- **Business Value**: Core feature for long-running agent workflows
- **Effort Estimate**: M

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Add `WorkstreamConfig` to `arawn-config` crate
- [ ] Wire `WorkstreamManager` initialization in `start.rs`
- [ ] Pass manager to `AppState::with_workstreams()`
- [ ] All `/api/v1/workstreams/*` endpoints return success responses
- [ ] `test-api.sh` workstream tests pass

## Implementation Notes

### Current State
- `arawn-workstream` crate is fully implemented with `WorkstreamManager`
- `AppState::with_workstreams()` builder exists but is never called
- No config type in `arawn-config` for workstreams

### Technical Approach

1. **Add config type** (`crates/arawn-config/src/types.rs`):
```rust
#[derive(Debug, Clone, Deserialize, Default)]
pub struct WorkstreamConfig {
    pub enabled: bool,
    pub database: Option<PathBuf>,  // defaults to workstreams.db
    pub data_dir: Option<PathBuf>,  // defaults to workstreams/
    pub session_timeout_minutes: Option<i64>,  // defaults to 60
}
```

2. **Wire in start.rs** (`crates/arawn/src/commands/start.rs`):
```rust
let ws_cfg = config.workstream.clone().unwrap_or_default();
if ws_cfg.enabled {
    let ws_config = arawn_workstream::WorkstreamConfig {
        db_path: resolve_path(ws_cfg.database, "workstreams.db"),
        data_dir: resolve_path(ws_cfg.data_dir, "workstreams"),
        session_timeout_minutes: ws_cfg.session_timeout_minutes.unwrap_or(60),
    };
    match arawn_workstream::WorkstreamManager::new(&ws_config) {
        Ok(mgr) => {
            app_state = app_state.with_workstreams(mgr);
            println!("Workstreams: enabled");
        }
        Err(e) => eprintln!("warning: failed to init workstreams: {}", e),
    }
}
```

3. **Update arawn.toml** for testing:
```toml
[workstream]
enabled = true
```

### Dependencies
- `arawn-workstream` crate (already complete)
- `arawn-config` changes

## Status Updates

*To be added during implementation*