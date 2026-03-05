---
id: resolve-agentconfig-naming
level: task
title: "Resolve AgentConfig naming collision between crates"
short_code: "ARAWN-T-0264"
created_at: 2026-03-04T13:24:13.087273+00:00
updated_at: 2026-03-05T04:09:47.203031+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#tech-debt"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
---

# Resolve AgentConfig naming collision between crates

## Objective

Rename or namespace the `AgentConfig` type that exists in both `arawn-config` and `arawn-agent` to avoid confusion when both are in scope.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P3 - Low (when time permits)

### Technical Debt Impact
- **Current Problems**: Two different `AgentConfig` types exist — `arawn_config::AgentConfig` (TOML config) and `arawn_agent::AgentConfig` (runtime config). Imports require aliasing.
- **Benefits of Fixing**: Clearer code, no aliasing needed, less confusion for contributors
- **Risk Assessment**: Low — rename + find/replace

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] No two public types share the same unqualified name across crates
- [ ] All call sites updated
- [ ] Suggested names: `AgentProfileConfig` (config crate) vs `AgentConfig` (agent crate), or similar

## Status Updates

### Implementation
- Renamed `AgentConfig` → `AgentProfileConfig` in `arawn-config/src/types.rs`
- 3 occurrences replaced: struct definition + 2 `HashMap<String, _>` field types
- No external crate referenced `arawn_config::AgentConfig` by name — only used internally in types.rs
- `arawn_agent::AgentConfig` (runtime config) keeps its name unchanged — it's the more widely used type
- `angreal check all` — clean
- `angreal test unit` — all 1,488 tests pass