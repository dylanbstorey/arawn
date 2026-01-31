---
id: wire-factory-into-workflowtool-and
level: task
title: "Wire factory into WorkflowTool and server startup"
short_code: "ARAWN-T-0085"
created_at: 2026-01-30T03:41:24.949427+00:00
updated_at: 2026-01-30T04:12:34.666508+00:00
parent: ARAWN-I-0019
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0019
---

# Wire factory into WorkflowTool and server startup

## Parent Initiative

[[ARAWN-I-0019]] — WASM Script Runtime System

## Objective

Replace the temporary no-op `ActionExecutorFactory` in `WorkflowTool.action_create()` with the real factory implementation. Wire `ScriptExecutor` and `RuntimeCatalog` through from server startup (`start.rs`) so that `action_create` builds workflows backed by real WASM execution and `action_run` dispatches tasks to actual runtimes.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `start.rs` creates `RuntimeCatalog` (loading from data directory) and `ScriptExecutor` (with catalog reference)
- [ ] Both are passed into `WorkflowTool` constructor via `Arc`
- [ ] `action_create` uses the real `ActionExecutorFactory` instead of the no-op stub
- [ ] `action_run` executes workflows with real task dispatch through WASM runtimes
- [ ] The temporary no-op factory code is removed
- [ ] Server boots cleanly with `cargo run`
- [ ] `cargo test` passes with no regressions

## Implementation Notes

### Dependencies
- ARAWN-T-0081 (catalog) — RuntimeCatalog instance created at startup
- ARAWN-T-0084 (ActionExecutorFactory) — the real factory being wired in

### Approach
In `start.rs`, initialize `RuntimeCatalog::load(data_dir)` and `ScriptExecutor::new(catalog)`, wrap both in `Arc`. Pass `Arc<ScriptExecutor>` to `WorkflowTool::new()`. Inside `WorkflowTool`, replace the no-op factory with `ActionExecutorFactory::new(script_executor)`. Remove all no-op/placeholder factory code and any `todo!()` markers related to runtime execution.

## Status Updates

### Session — completed
- `WorkflowTool` now holds `Arc<ScriptExecutor>` + `Arc<RuntimeCatalog>`; constructor updated to 4 args
- `action_create` uses `build_executor_factory()` instead of the no-op stub — workflows are now backed by real WASM execution
- `start.rs` creates `RuntimeCatalog::load(data_dir/runtimes)` and `ScriptExecutor::new(data_dir/wasm-cache)`, passes both to `WorkflowTool`
- Verbose logging for catalog path and executor cache dir
- Removed all no-op factory code
- All 11 workflow tool tests pass, full workspace (595+ tests) green with 0 failures