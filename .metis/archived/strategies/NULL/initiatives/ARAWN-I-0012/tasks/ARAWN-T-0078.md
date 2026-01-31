---
id: workflowtool-agent-facing-workflow
level: task
title: "WorkflowTool: agent-facing workflow management"
short_code: "ARAWN-T-0078"
created_at: 2026-01-29T18:34:51.852678+00:00
updated_at: 2026-01-30T02:44:49.039024+00:00
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

# WorkflowTool: agent-facing workflow management

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[ARAWN-I-0012]]

## Objective

Implement the `WorkflowTool` — the agent-facing tool that lets the agent autonomously create, schedule, run, monitor, and cancel workflows. This is how the agent interacts with the pipeline engine during conversations.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `WorkflowTool` implements the Arawn tool trait (same pattern as existing tools)
- [ ] Action: `create` — writes a new workflow TOML definition file to the workflow directory
- [ ] Action: `run` — executes a named workflow immediately with provided context, returns execution ID
- [ ] Action: `schedule` — registers a cron schedule for a workflow (cron expression + timezone)
- [ ] Action: `list` — returns all known workflows, active schedules, and recent execution history
- [ ] Action: `cancel` — cancels a cron schedule by ID
- [ ] Action: `status` — returns execution status/history for a given execution ID or workflow name
- [ ] Tool schema (JSON Schema) defines all actions and their parameters for the agent
- [ ] Input validation: reject invalid cron expressions, unknown workflow names, etc.
- [ ] Returns structured JSON responses the agent can reason about
- [ ] Tests: each action exercised, invalid input rejected, integration with PipelineEngine
- [ ] Tests pass

## Implementation Notes

### Technical Approach
- Follows existing tool pattern in arawn (tool trait, JSON schema, execute method)
- `create` action writes TOML to the workflow directory — hot-reload (T-0075) picks it up
- `run` delegates to `PipelineEngine::execute()`
- `schedule` delegates to `PipelineEngine::schedule_cron()`
- `list`/`status` query Cloacina's execution history via PipelineEngine
- `cancel` delegates to `PipelineEngine::cancel_schedule()`

### Dependencies
- ARAWN-T-0072 (PipelineEngine API)
- ARAWN-T-0073 (workflow definition format for `create` action)
- ARAWN-T-0075 (hot-reload picks up created files)

## Status Updates

### Completed
- Created `workflow.rs` in `arawn-agent/src/tools/` implementing `Tool` trait
- 6 actions: `create`, `run`, `schedule`, `list`, `cancel`, `status`
- `create` validates TOML via `WorkflowFile::from_toml()` + `validate()`, writes to workflow dir
- `run` builds `Context<Value>`, delegates to `engine.execute()`
- `schedule` delegates to `engine.schedule_cron()` with default UTC timezone
- `list` returns workflows + schedules (gracefully handles cron-disabled)
- `cancel` delegates to `engine.cancel_schedule()`
- `status` returns registration status + matching schedules
- JSON Schema with action enum + conditional params
- Added `arawn-pipeline` and `cloacina-workflow` deps to arawn-agent
- Wired into `tools/mod.rs` with public export
- 11 tests passing, workspace compiles clean