---
id: pipeline-config-and-server
level: task
title: "Pipeline config and server lifecycle integration"
short_code: "ARAWN-T-0079"
created_at: 2026-01-29T18:34:55.438622+00:00
updated_at: 2026-01-30T02:57:13.944737+00:00
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

# Pipeline config and server lifecycle integration

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[ARAWN-I-0012]]

## Objective

Wire the PipelineEngine into the Arawn server lifecycle and add minimal pipeline configuration to `arawn-config`. This is the integration task that makes everything work end-to-end: engine boots on server start, workflows load, cron starts, triggers are available, and graceful shutdown drains running work.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `[pipeline]` config section in `arawn-config` types: `database` (path), `workflow_dir` (path), `enabled` (bool)
- [ ] Default values: `pipeline.db` alongside memory.db, `workflows/` in config dir, enabled=true
- [ ] `start.rs` initializes `PipelineEngine` on server boot (if enabled)
- [ ] `WorkflowLoader` loads all workflow definitions from `workflow_dir` on startup
- [ ] File watcher starts for hot-reload
- [ ] Cron scheduler starts for workflows with `[schedule]` config
- [ ] `Arc<PipelineEngine>` passed to agent builder for `WorkflowTool` access
- [ ] Push trigger method available to other server components (e.g., `engine.trigger("session_close", ctx)`)
- [ ] Graceful shutdown: `engine.shutdown()` called on server stop, drains running workflows
- [ ] Tests: server starts with pipeline enabled, server starts with pipeline disabled, shutdown is clean
- [ ] `cargo test --workspace` passes

## Implementation Notes

### Technical Approach
- Add `PipelineConfig` to `arawn-config/src/types.rs`
- In `start.rs`: conditionally init PipelineEngine based on config
- Pass `Arc<PipelineEngine>` via the existing shared state / agent builder pattern
- Register `WorkflowTool` alongside existing tools
- Shutdown hook: tokio signal handler calls `engine.shutdown()` before process exit

### Dependencies
- ARAWN-T-0072 (PipelineEngine)
- ARAWN-T-0075 (WorkflowLoader)
- ARAWN-T-0078 (WorkflowTool to register)
- This is the final integration task — depends on most other I-0012 tasks

## Status Updates

### Completed
- Added `PipelineSection` config type to `arawn-config/src/types.rs` with: `enabled`, `database`, `workflow_dir`, `max_concurrent_tasks`, `task_timeout_secs`, `pipeline_timeout_secs`, `cron_enabled`, `triggers_enabled`
- Wired into `RawConfig`/`ArawnConfig` with TOML parsing, merge support, and defaults
- Added `arawn-pipeline` dependency to `arawn` crate
- Updated `start.rs`: conditionally initializes `PipelineEngine` based on `[pipeline]` config
- Resolves `database` and `workflow_dir` paths relative to XDG config dir
- Creates workflow directory on startup
- Registers `WorkflowTool` with the agent's `ToolRegistry` when pipeline is enabled
- Graceful shutdown: `Arc::try_unwrap` + `engine.shutdown()` after server stops
- 5 new config tests (defaults, parsing, disabled, no-section default, merge override)
- All 30 config tests pass, full workspace compiles clean, `cargo test --workspace` all green

### Files Modified
- `crates/arawn-config/src/types.rs` — Added `PipelineSection` struct + tests
- `crates/arawn/Cargo.toml` — Added `arawn-pipeline` dependency
- `crates/arawn/src/commands/start.rs` — Pipeline engine init, WorkflowTool registration, graceful shutdown