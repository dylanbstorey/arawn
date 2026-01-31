---
id: arawn-pipeline-crate-and
level: task
title: "arawn-pipeline crate and PipelineEngine core"
short_code: "ARAWN-T-0072"
created_at: 2026-01-29T18:34:32.443957+00:00
updated_at: 2026-01-30T00:50:05.290622+00:00
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

# arawn-pipeline crate and PipelineEngine core

## Parent Initiative

[[ARAWN-I-0012]]

## Objective

Create the `arawn-pipeline` crate and implement `PipelineEngine` — the core wrapper around Cloacina that provides Arawn's execution backbone. This is the foundational task that all other I-0012 tasks build upon.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `crates/arawn-pipeline` exists in workspace with `cloacina` as a dependency
- [ ] `PipelineEngine` struct wraps Cloacina's `DefaultRunner` with SQLite backend
- [ ] `PipelineEngine::new(db_path, config)` initializes the runner
- [ ] `PipelineEngine::execute(workflow_name, context)` runs a workflow and returns `ExecutionResult`
- [ ] `PipelineEngine::schedule_cron(workflow_name, cron_expr)` registers a cron schedule
- [ ] `PipelineEngine::trigger(workflow_name, context)` provides push-trigger execution
- [ ] `PipelineEngine::list_schedules()` returns active schedules
- [ ] `PipelineEngine::cancel_schedule(id)` cancels a schedule
- [ ] `PipelineEngine::shutdown()` gracefully drains running workflows
- [ ] Dynamic workflow construction via Cloacina core API (not macros) — workflows built at runtime from programmatic definitions
- [ ] Tests: engine init/shutdown lifecycle, execute a simple workflow, cron registration, trigger execution
- [ ] `cargo test -p arawn-pipeline` passes

## Implementation Notes

### Technical Approach
- Investigate Cloacina's core API (beneath the `#[task]`/`workflow!` macros) for dynamic workflow construction — similar to what the Python bindings (Cloaca) do
- `DefaultRunner::with_config(db_url, config)` for initialization
- `DefaultRunnerConfig` for cron/trigger settings
- `Context<Value>` for JSON data flow between tasks
- SQLite database at the Arawn data directory (alongside memory.db)

### Dependencies
- `cloacina` crate
- No other ARAWN-T-* dependencies — this is the foundation

## Status Updates

### Session 1
- Cloned cloacina repo, explored full API
- Key findings:
  - `cloacina-workflow` crate for lightweight types (Task trait, Context)
  - `cloacina` crate for full runtime (DefaultRunner, cron, triggers)
  - Dynamic workflow construction via `Workflow::builder("name").add_task(Arc::new(t)).build()`
  - Task trait: `execute(Context<Value>) -> Result<Context<Value>, TaskError>`, `id()`, `dependencies()`
  - Context is owned, passed between tasks, HashMap<String, Value> internally
  - DefaultRunner has full cron API: register_cron_workflow, list, update, delete, stats
  - Trigger trait: poll-based with Fire/Skip result, deduplication via context hash
  - Python bindings wrap Task trait with PythonTaskWrapper — confirms dynamic construction works
- Created crate structure: Cargo.toml, lib.rs, error.rs, task.rs, engine.rs
- Hit libsqlite3-sys version conflict: cloacina (diesel) needs 0.35, graphqlite (rusqlite 0.31) needs 0.28
- Resolution: upstream loosened version pins — graphqlite 0.3.1→0.3.2, refinery 0.8→0.9
- Fixed API mismatches against cloacina 0.3.1 (config fields are pub not setters, PipelineResult.error_message not .error, list_cron_schedules takes 3 args, UniversalUuid/UniversalBool types)
- Full workspace compiles clean

### Files Created/Modified
- `crates/arawn-pipeline/Cargo.toml` — new crate
- `crates/arawn-pipeline/src/lib.rs` — module root
- `crates/arawn-pipeline/src/error.rs` — PipelineError enum
- `crates/arawn-pipeline/src/task.rs` — DynamicTask (closure-based Task impl)
- `crates/arawn-pipeline/src/engine.rs` — PipelineEngine wrapping DefaultRunner
- `Cargo.toml` — added workspace member, rusqlite 0.37
- `crates/arawn-memory/Cargo.toml` — graphqlite 0.3.2
- `crates/arawn-workstream/Cargo.toml` — refinery 0.9
- `crates/arawn-pipeline/tests/engine_test.rs` — 6 integration tests

### Key Decisions
- Workflows registered in Cloacina's global registry via `register_workflow_constructor`
- DynamicTask dependencies use `__pending__` workflow placeholder, resolved at registration time
- Cloacina's `final_context` on PipelineResult reflects initial context; task outputs saved via task_execution_metadata (context merging is internal to Cloacina scheduler)
- Config fields on DefaultRunnerConfig are pub fields (not setters) in 0.3.1