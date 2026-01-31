---
id: hot-reload-workflow-file-watcher
level: task
title: "Hot-reload workflow file watcher"
short_code: "ARAWN-T-0075"
created_at: 2026-01-29T18:34:41.666349+00:00
updated_at: 2026-01-30T01:41:26.254522+00:00
parent: ARAWN-I-0012
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0012
---

# Hot-reload workflow file watcher

## Parent Initiative

[[ARAWN-I-0012]]

## Objective

Implement a file watcher for the workflow definitions directory so that new, modified, or deleted workflow TOML files are picked up at runtime without restarting the Arawn server.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `WorkflowLoader` struct that manages a directory of workflow TOML files
- [ ] `WorkflowLoader::load_all()` — initial scan of workflow directory, returns all parsed `WorkflowDefinition`s
- [ ] File watcher (e.g., `notify` crate) monitors the workflow directory for create/modify/delete events
- [ ] On file change: re-parse the affected TOML file, validate, and update the engine's workflow registry
- [ ] On file delete: unregister the workflow from the engine (cancel any active schedules)
- [ ] Debounce rapid file changes (e.g., editor save-swap patterns)
- [ ] Error handling: log parse errors for invalid files without crashing; keep previous valid version loaded
- [ ] Tests: load directory of TOML files, detect new file, detect modification, detect deletion, invalid file doesn't crash loader
- [ ] Tests pass

## Implementation Notes

### Technical Approach
- Use the `notify` crate for cross-platform filesystem event watching (macOS FSEvents, Linux inotify)
- Debounce with ~200ms window to handle editor save patterns
- Maintain an in-memory `HashMap<String, WorkflowDefinition>` keyed by workflow name
- On change, re-parse only the affected file, diff against current state, and update PipelineEngine registration
- Default workflow directory: `~/.arawn/workflows/`

### Dependencies
- ARAWN-T-0073 (parser produces `WorkflowDefinition` from TOML files)
- ARAWN-T-0072 (PipelineEngine provides workflow registration/unregistration)

## Status Updates

### Completed
- Created `loader.rs` with `WorkflowLoader`, `WorkflowEvent`, `WatcherHandle`
- `load_all()` scans directory, `load_file()`/`remove_file()` for individual ops
- `watch()` uses `notify-debouncer-mini` with 300ms debounce, returns `mpsc::Receiver<WorkflowEvent>`
- Wired into `lib.rs` with public exports
- All 9 unit tests passing, workspace compiles clean
- Added `notify = "7"` and `notify-debouncer-mini = "0.5"` dependencies