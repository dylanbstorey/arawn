---
id: implement-workflow-hot-reload-via
level: task
title: "Implement workflow hot-reload via file watcher"
short_code: "ARAWN-T-0230"
created_at: 2026-02-27T00:05:03.809480+00:00
updated_at: 2026-02-27T00:05:03.809480+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#feature"


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

*To be added during implementation*