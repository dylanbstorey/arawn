---
id: prompt-fragment-injection
level: task
title: "Prompt fragment injection"
short_code: "ARAWN-T-0116"
created_at: 2026-02-02T01:54:21.707522+00:00
updated_at: 2026-02-02T13:01:48.363944+00:00
parent: ARAWN-I-0013
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0013
---

# Prompt fragment injection

## Parent Initiative

[[ARAWN-I-0013]]

## Objective

Wire plugin `[prompt].system` sections into `SystemPromptBuilder` so plugin-provided prompt fragments are included in the agent's system prompt.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `SystemPromptBuilder` gains a `with_plugin_prompts(fragments: &[PromptFragment])` method
- [ ] Plugin prompt fragments appended as a distinct section in `build()` output
- [ ] Section header: `## Plugin: {plugin_name}` followed by the fragment text
- [ ] Multiple plugins' fragments concatenated in load order
- [ ] Empty fragments skipped
- [ ] Tests: builder with plugin fragments produces expected output
- [ ] `angreal check all` and `angreal test unit` pass

## Implementation Notes

### Technical Approach
- Modify `arawn-agent/src/prompt/builder.rs` to accept plugin prompt fragments
- Add `plugin_prompts: Vec<(String, String)>` field (plugin name, prompt text) to builder
- Append after bootstrap context section in `build()`
- `arawn-plugin` provides the fragments; `arawn-agent` consumes them — no circular dependency since arawn-agent doesn't depend on arawn-plugin, the fragments are passed as simple strings

### Dependencies
- ARAWN-T-0110 (PromptFragment type)

## Status Updates

### Completed
- Added `plugin_prompts: Vec<(String, String)>` field to `SystemPromptBuilder`
- Added `with_plugin_prompts()` builder method
- Plugin fragments rendered as `## Plugin: {name}` sections after bootstrap context
- Empty fragments skipped
- 3 new tests, all 16 builder tests pass