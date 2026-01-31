---
id: context-template-resolver
level: task
title: "Context template resolver"
short_code: "ARAWN-T-0074"
created_at: 2026-01-29T18:34:38.751334+00:00
updated_at: 2026-01-30T01:35:30.702777+00:00
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

# Context template resolver

## Parent Initiative

[[ARAWN-I-0012]]

## Objective

Implement the context template resolution system that enables data flow between workflow tasks. Every task receives the full workflow context; template expressions like `{{task_id.output.field}}` are resolved before a task executes, allowing upstream outputs to feed into downstream inputs.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `ContextResolver` struct that takes a `Context<Value>` and resolves template expressions
- [ ] Template syntax: `{{task_id.output}}` for full output, `{{task_id.output.field}}` for nested access, `{{input.field}}` for workflow-level input
- [ ] Nested field access via dot notation (e.g., `{{extract.output.entities[0].name}}`)
- [ ] Resolves templates in action params (tool params, script args, LLM prompts)
- [ ] Graceful error on missing keys — clear error message identifying which expression failed
- [ ] Works with `serde_json::Value` — no custom types required for context data
- [ ] Tests: simple resolution, nested access, missing key error, multiple expressions in one string, input context access
- [ ] Tests pass

## Implementation Notes

### Technical Approach
- Parse `{{...}}` expressions from string values in action params
- Walk the `serde_json::Value` tree using dot-separated path segments
- Replace expressions in-place, producing resolved params before task execution
- Consider using a lightweight template engine (e.g., handlebars-like) or rolling a simple one since the syntax is constrained

### Dependencies
- ARAWN-T-0073 (workflow definitions define where templates appear)

## Status Updates

### Session 1
- Created `context.rs` module with `ContextResolver` struct
- Template syntax: `{{path.to.field}}`, `{{task.output.items[0].name}}`, `{{input.field}}`
- Type-preserving: sole `{{expr}}` preserves JSON type (number, object, array); mixed text stringifies
- Recursive resolution through objects and arrays via `resolve_value()`
- Dot-separated path navigation with array index support (`[N]`)
- Clear error messages identifying which expression and segment failed
- Convenience functions: `resolve_params()` for action param maps, `resolve_template_string()` for prompts
- 25 unit tests covering: simple/nested/array access, type preservation, mixed strings, missing keys, out-of-bounds, whitespace trimming, unclosed braces, null/boolean stringification
- All 49 crate tests pass