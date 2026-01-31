---
id: auto-update-subscribed-plugins-on
level: task
title: "Auto-update subscribed plugins on startup"
short_code: "ARAWN-T-0127"
created_at: 2026-02-03T19:44:25.774551+00:00
updated_at: 2026-02-04T13:21:45.355447+00:00
parent: ARAWN-I-0013
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0013
---

# Auto-update subscribed plugins on startup

## Objective

Automatically update subscribed plugins on startup by pulling latest changes from their git repositories.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] On startup, iterate all subscribed plugins and pull updates
- [x] Run updates in parallel for performance
- [x] Log update results (updated, already current, failed)
- [x] Continue startup even if some updates fail
- [x] Environment variable to disable auto-update: `ARAWN_DISABLE_PLUGIN_UPDATES=1`
- [x] Integration with start command

## Implementation Notes

### Startup Flow

```
1. Load subscription config
2. For each subscription:
   a. Check if cache directory exists
   b. If not: clone
   c. If exists: git pull
3. Load plugins from cache + local directories
4. Continue normal startup
```

### Parallel Updates

Use `tokio::spawn` or `rayon` to update multiple plugins concurrently:

```rust
let updates: Vec<_> = subscriptions
    .iter()
    .map(|sub| tokio::spawn(update_plugin(sub)))
    .collect();

for result in futures::future::join_all(updates).await {
    match result {
        Ok(Ok(name)) => info!("Updated plugin: {}", name),
        Ok(Err(e)) => warn!("Plugin update failed: {}", e),
        Err(e) => warn!("Update task panicked: {}", e),
    }
}
```

### Environment Variables

- `ARAWN_DISABLE_PLUGIN_UPDATES=1` - Skip all plugin updates
- `ARAWN_PLUGIN_UPDATE_TIMEOUT=30` - Timeout per plugin in seconds

### Files to Modify

- `crates/arawn/src/commands/start.rs` - Add update step before plugin load
- `crates/arawn-plugin/src/subscription.rs` - Add update_all function

### Dependencies

- ARAWN-T-0125 (subscription config)
- ARAWN-T-0126 (git clone/update)

## Status Updates

### Completed 2026-02-04

**arawn-plugin/src/subscription.rs:**
- Added `sync_all_async()` - parallel async sync using `tokio::task::spawn`
- Each subscription runs in its own task with `spawn_blocking` for git ops
- Per-subscription timeout via `ARAWN_PLUGIN_UPDATE_TIMEOUT` (default 30s)
- Added `is_auto_update_disabled()` - checks `ARAWN_DISABLE_PLUGIN_UPDATES=1`
- Added `update_timeout_secs()` - reads timeout from env var

**arawn/src/commands/start.rs:**
- Integrated subscription sync into startup flow
- Runs after config load, before plugin loading
- Syncs subscriptions if `plugins.auto_update = true` and env not disabled
- Logs results: cloned/updated/skipped/failed
- Adds synced plugin directories to plugin_dirs for loading
- Continues startup even if some syncs fail

**Environment variables:**
- `ARAWN_DISABLE_PLUGIN_UPDATES=1` - skip all updates
- `ARAWN_PLUGIN_UPDATE_TIMEOUT=<seconds>` - per-plugin timeout

**Tests:** 5 new tests for env var parsing and async sync
All 131 tests pass.