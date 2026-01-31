---
id: runtime-protocol-types-and
level: task
title: "Runtime protocol types and workflow schema update"
short_code: "ARAWN-T-0080"
created_at: 2026-01-30T03:41:21.009338+00:00
updated_at: 2026-01-30T03:53:44.077023+00:00
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

# Runtime protocol types and workflow schema update

## Parent Initiative
[[ARAWN-I-0019]]

## Objective
Define the runtime protocol types (input envelope, output envelope) and update `WorkflowDefinition` / `TaskDefinition` in `arawn-pipeline` to use `runtime` + `config` fields instead of the current `action` type/name pattern.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria
- [ ] `RuntimeInput` struct: `{ config: Value, context: Value }` — serialized to JSON for stdin
- [ ] `RuntimeOutput` struct: `{ status: String, output: Option<Value>, error: Option<String> }` — parsed from stdout JSON
- [ ] `TaskDefinition` updated: `runtime: String`, `config: Value` fields replace or augment `action`
- [ ] `WorkflowFile::from_toml()` parses the new schema
- [ ] Existing tests updated, new tests for the `runtime` + `config` schema
- [ ] Backward compat: `action` field still works (maps `action.name` → `runtime`, `action.params` → `config`)
- [ ] `cargo test -p arawn-pipeline` passes

## Implementation Notes
- Protocol types go in a new `arawn-pipeline/src/protocol.rs`
- Update `definition.rs` `TaskDefinition` serde
- `arawn-script-sdk` `entry!` macro already reads stdin JSON and writes stdout JSON — the protocol types formalize what's expected

### Dependencies
None — this is the foundation task.

## Status Updates
### Session — completed
- Created `arawn-pipeline/src/protocol.rs` with `RuntimeInput` and `RuntimeOutput` types
- Updated `TaskDefinition`: `action` is now `Option<ActionDefinition>`, added `runtime: Option<String>` and `config: Option<Value>`
- Added `effective_runtime()` and `effective_config()` helper methods for backward compat
- Validation requires either `runtime` or `action` on each task
- `to_dynamic_tasks()` synthesizes an `ActionDefinition` for runtime-only tasks so factory still works
- Added `protocol` module to `lib.rs` with re-exports
- 7 new tests (protocol roundtrip, runtime schema parsing, effective methods, mixed tasks, neither-field validation, runtime-to-dynamic-tasks)
- All 80 arawn-pipeline tests pass, workspace compiles clean