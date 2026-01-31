---
id: declarative-workflow-definition
level: task
title: "Declarative workflow definition parser"
short_code: "ARAWN-T-0073"
created_at: 2026-01-29T18:34:35.630527+00:00
updated_at: 2026-01-30T01:31:09.813832+00:00
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

# Declarative workflow definition parser

## Parent Initiative

[[ARAWN-I-0012]]

## Objective

Build the TOML-based declarative workflow definition format and parser. This translates workflow files on disk into Cloacina core API calls at runtime, enabling agents (and users) to define workflows without Rust compilation of workflow definitions.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `WorkflowDefinition` type: name, description, tasks, schedule, triggers, runtime config
- [ ] `TaskDefinition` type: id, action (tool/script/llm), dependencies, retry config, capabilities
- [ ] `ActionDefinition` enum: `Tool { name, params }`, `Script { language, source_file }`, `Llm { prompt, model }`
- [ ] TOML deserialization via serde for all definition types
- [ ] `WorkflowDefinition::to_cloacina()` — converts declarative definition to Cloacina core API workflow (dynamic construction, not macros)
- [ ] Validation: cycle detection in task dependencies, unknown action types, missing required fields
- [ ] Per-workflow config sections: `[workflow.schedule]`, `[workflow.runtime]`, `[workflow.triggers]`
- [ ] Tests: parse valid TOML, reject invalid TOML, cycle detection, round-trip serialize/deserialize
- [ ] Tests pass

## Implementation Notes

### Technical Approach
- Serde derive for TOML deserialization
- Validate dependency DAG (topological sort, detect cycles)
- `to_cloacina()` uses Cloacina's core builder API to construct workflows at runtime
- Action types are an enum — `script` variant defers to T-0076 (Wasmtime), `tool` dispatches to existing Arawn tools

### Dependencies
- ARAWN-T-0072 (PipelineEngine must exist to register constructed workflows)

## Status Updates

### Session 1
- Created `definition.rs` module with full TOML parsing, validation, and conversion to DynamicTasks
- Types: `WorkflowFile`, `WorkflowDefinition`, `TaskDefinition`, `ActionDefinition` (Tool/Script/Llm), `ScheduleConfig`, `RuntimeConfig`, `TriggerConfig`, `Capabilities`
- Validation: empty name, empty tasks, duplicate IDs, unknown dependencies, cycle detection (Kahn's algorithm), unsupported script languages
- `to_dynamic_tasks()` converts definitions to Cloacina tasks via pluggable `ActionExecutorFactory`
- `from_toml()` and `from_file()` for parsing
- Roundtrip serialization support via Serialize
- Added `toml = "0.8"` dependency
- 18 unit tests covering: all action types, all validation errors, DAG shapes (diamond, self-cycle, mutual cycle), roundtrip, minimal workflow
- All 24 tests pass (18 definition + 6 engine)