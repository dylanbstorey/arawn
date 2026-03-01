---
id: add-read-only-toolregistry
level: task
title: "Add read-only ToolRegistry filtering"
short_code: "ARAWN-T-0239"
created_at: 2026-03-01T16:27:45.717371+00:00
updated_at: 2026-03-01T16:33:00.341294+00:00
parent: ARAWN-I-0027
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0027
---

# Add read-only ToolRegistry filtering

## Parent Initiative

[[ARAWN-I-0027]] — RLM Exploration Agent

## Objective

Add a `filtered_by_names()` method to `ToolRegistry` that produces a new registry containing only the specified tools. The RLM exploration agent needs a restricted tool set (read-only tools only), and this is the mechanism to create it.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `ToolRegistry::filtered_by_names(&self, names: &[&str]) -> ToolRegistry` method exists
- [ ] Returned registry contains only tools whose names match the allowlist
- [ ] Tools not in the allowlist are excluded
- [ ] Names not matching any registered tool are silently ignored
- [ ] Original registry is unchanged (non-destructive)
- [ ] Tool definitions (for LLM schema generation) are also filtered
- [ ] Existing tests pass unchanged
- [ ] New tests: filter includes correct tools, excludes others, handles unknown names
- [ ] `angreal check all` passes

## Implementation Notes

### File
`crates/arawn-agent/src/tool.rs`

### Approach
1. Add `filtered_by_names(&self, names: &[&str]) -> ToolRegistry` to `ToolRegistry`
2. Iterate `self.tools`, clone `Arc` refs for matching names into a new `HashMap`
3. The `Arc<dyn Tool>` refs are cheap to clone — no deep copies
4. Also filter `tool_definitions()` output so the LLM only sees allowed tools

### Scope
Small — one method, a few tests. No API changes for existing callers.

## Status Updates

### Implementation Complete
- Added `filtered_by_names(&self, names: &[&str]) -> ToolRegistry` to `ToolRegistry`
- Clones matching `Arc<dyn Tool>` refs and output config overrides into a new registry
- Unknown names silently ignored, original registry unchanged
- LLM tool definitions are naturally filtered (only registered tools appear)
- 7 new tests: includes matching, excludes non-matching, ignores unknown, preserves original, carries output overrides, filters LLM definitions, empty allowlist
- `angreal check all` clean, all tests pass