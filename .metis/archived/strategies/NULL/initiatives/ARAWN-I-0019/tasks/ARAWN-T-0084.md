---
id: actionexecutorfactory-real
level: task
title: "ActionExecutorFactory: real implementation with context propagation"
short_code: "ARAWN-T-0084"
created_at: 2026-01-30T03:41:24.162826+00:00
updated_at: 2026-01-30T04:08:49.098386+00:00
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

# ActionExecutorFactory: real implementation with context propagation

## Parent Initiative

[[ARAWN-I-0019]] — WASM Script Runtime System

## Objective

Implement the real `ActionExecutorFactory` that bridges workflow task definitions to WASM runtime execution via `ScriptExecutor.execute_runtime()`. Each task's RuntimeOutput is inserted into the pipeline context under its task ID, so downstream tasks in the DAG receive the full accumulated context snapshot when they execute.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Factory creates a `TaskFn` closure for each task definition in a workflow
- [ ] Each `TaskFn` calls `execute_runtime` with the task's runtime name, config, and a snapshot of the current pipeline context
- [ ] RuntimeOutput from each task is stored in `context[task_id]`
- [ ] Context accumulates correctly across DAG execution order — each task sees outputs from all predecessors
- [ ] Errors from `execute_runtime` propagate with the originating task ID in the error message
- [ ] Unit tests verify context propagation across a simple two-task chain (passthrough -> passthrough)

## Implementation Notes

### Dependencies
- ARAWN-T-0080 (protocol types) — RuntimeInput/RuntimeOutput used to build inputs and parse outputs
- ARAWN-T-0083 (ScriptExecutor.execute_runtime) — the execution method this factory wraps

### Approach
`ActionExecutorFactory` holds an `Arc<ScriptExecutor>`. Its `create(task_def: &TaskDefinition) -> TaskFn` method captures the runtime name and config from the task definition, then returns a closure that: (1) clones the current context snapshot, (2) builds a `RuntimeInput { config, context }`, (3) calls `execute_runtime`, (4) merges the output into the shared context map under `task_def.id`. The DAG executor calls these `TaskFn`s in topological order.

## Status Updates

### Session — completed
- Created `arawn-pipeline/src/factory.rs` with `build_executor_factory(executor, catalog) -> ActionExecutorFactory`
- Factory produces `TaskFn` closures that: snapshot context → build `RuntimeInput` → call `execute_runtime` → store output under `context[task_id]`
- Context accumulates across DAG: each downstream task sees all predecessor outputs
- Errors propagate as `TaskError::ExecutionFailed { task_id, message, timestamp }`
- 3 tests: working task fn, context propagation across 2-step chain, unknown runtime error with task_id
- All pass, workspace clean