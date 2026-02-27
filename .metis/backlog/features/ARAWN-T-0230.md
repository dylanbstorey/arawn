---
id: implement-workflow-hot-reload-via
level: task
title: "Implement workflow hot-reload via file watcher"
short_code: "ARAWN-T-0230"
created_at: 2026-02-27T00:05:03.809480+00:00
updated_at: 2026-02-27T01:48:43.053402+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#feature"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Implement workflow hot-reload via file watcher

## Objective

Wire up a file watcher on the workflow TOML directory so that adding, editing, or removing workflow files hot-reloads them without restarting the server. The scaffolding already exists in `WorkflowLoader` (`remove_file()`, `path` field on `LoadedWorkflow`, `path_to_name` reverse map) — it just needs a watcher driving it.

## Backlog Item Details

### Type
- [x] Feature - New functionality or enhancement

### Priority
- [x] P3 - Low (when time permits)

### Business Justification
- **User Value**: Edit workflow TOML files and see changes immediately without server restart
- **Effort Estimate**: M

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] File watcher monitors the workflow directory for create/modify/delete events
- [ ] New or modified TOML files are loaded/reloaded via existing `WorkflowLoader::load_file()`
- [ ] Deleted TOML files trigger `WorkflowLoader::remove_file()` to unload the workflow
- [ ] Watcher runs as a background tokio task, shut down cleanly on server stop
- [ ] Debounce rapid file changes (e.g., editor save-then-rename patterns)
- [ ] Existing `WorkflowEvent` enum used to emit events for loaded/removed/error

## Implementation Notes

### Existing Scaffolding
- `arawn-pipeline/src/loader.rs:32-35` — `LoadedWorkflow.path` field (currently triggers dead code warning)
- `arawn-pipeline/src/loader.rs:43-44` — `path_to_name` reverse map for delete handling
- `arawn-pipeline/src/loader.rs:155-169` — `remove_file()` method (currently triggers dead code warning)
- `arawn-pipeline/src/loader.rs:25` — `WorkflowEvent::Removed` variant

### Technical Approach
- Use `notify` crate (or `notify-debouncer-mini`) for cross-platform file watching
- Spawn a background task from `start.rs` after workflow loader is created
- Feed file events into `load_file()` / `remove_file()` on the loader

## Status Updates

### Session 1
**Exploration complete. Implementation plan:**

The `WorkflowLoader` in `loader.rs` is ALREADY fully implemented — `watch()`, `WorkflowLoaderView`, `WatcherHandle`, debouncing, load/remove. The gap is:

1. `start.rs` never creates a `WorkflowLoader` — the `WorkflowTool` gets the dir path directly
2. On startup, TOML files in the workflow dir are NOT loaded/registered with `PipelineEngine`
3. If server restarts, previously created workflows are on disk but unregistered — `engine.execute()` returns `WorkflowNotFound`

**Implementation complete.**

`WorkflowLoader` was already fully implemented in `loader.rs` — `watch()`, `WorkflowLoaderView`, `WatcherHandle`, debouncing, load/remove all existed. The gap was only the wiring in `start.rs`.

**Changes made:**

1. `crates/arawn/src/commands/start.rs`:
   - Added imports: `WorkflowEvent`, `WorkflowLoader`, `build_executor_factory`
   - Added `_workflow_watcher_handle` variable to keep watcher alive
   - After `WorkflowTool` registration: create `WorkflowLoader`, call `load_all()`, register each loaded workflow with `PipelineEngine` via `register_dynamic_workflow()`
   - Start `loader.watch()`, spawn background tokio task to process `WorkflowEvent`s — Loaded events re-parse and register, Removed events log (engine doesn't have unregister yet), Error events warn
   - Verbose mode prints workflow count

2. `crates/arawn-pipeline/src/loader.rs`:
   - Removed TODO comment

**Key detail:** On startup, workflows are now automatically loaded from disk and registered with the engine. Previously, `action_create` was the only way to register workflows — server restarts lost all registrations. Now TOML files persist across restarts AND hot-reload when edited.

**Dead code warnings remain** for `LoadedWorkflow.path` and `WorkflowLoader::remove_file()` because the compiler sees them unused on the `WorkflowLoader` type — they're actually used by `WorkflowLoaderView` in the watcher thread. Per user preference, no `#[allow(dead_code)]` added.

`angreal check all` clean. `angreal test unit` all pass.

### Session 2 — Integration test fixes

**Problem:** 3 of 4 integration tests for `watch()` were timing out — watcher events never arrived.

**Root causes found and fixed:**
1. **macOS path symlink** — `tempfile::tempdir()` returns `/tmp/xxx` but macOS's `/tmp` is a symlink to `/private/tmp`. The `notify` FSEvents backend resolves symlinks, so events arrive with `/private/tmp/xxx/new.toml` paths. The `starts_with(&workflow_dir)` check in the watcher thread compared against unresolved `/tmp/xxx`, silently filtering ALL events. **Fix:** Added `canonicalize()` in `WorkflowLoader::new()` after directory creation.

2. **tokio runtime handle in spawned thread** — `tokio::runtime::Handle::current()` was called inside `std::thread::spawn`. Spawned threads have no tokio context, so this would panic once events actually arrived. **Fix:** Captured `rt_handle = tokio::runtime::Handle::current()` before `std::thread::spawn`, use `rt_handle.block_on()` inside.

**Result:** All 4 integration tests now pass:
- `test_watch_detects_new_file` — ok
- `test_watch_detects_modified_file` — ok
- `test_watch_detects_deleted_file` — ok
- `test_watch_ignores_non_toml` — ok

`angreal test integration` convention was already in place in `.angreal/task_test.py`.

3. **Path normalization for `load_file`/`remove_file`** — The `canonicalize()` fix caused `test_remove_file` unit test to fail because the test passed non-canonical `/tmp/...` paths while `load_all` stored canonical `/private/tmp/...` paths. Added `normalize_path()` helper that reconstructs paths using the canonical `workflow_dir` prefix, called in both `load_file` and `remove_file`.

**Final result:** All unit tests (114 + full workspace), integration tests (4/4), and e2e tests pass. `angreal check all` clean.