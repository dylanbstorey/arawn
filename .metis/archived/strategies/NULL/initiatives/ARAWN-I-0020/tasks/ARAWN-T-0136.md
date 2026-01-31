---
id: delegatetool-struct-and-parameter
level: task
title: "DelegateTool struct and parameter schema"
short_code: "ARAWN-T-0136"
created_at: 2026-02-06T03:47:44.789743+00:00
updated_at: 2026-02-06T13:31:25.294182+00:00
parent: ARAWN-I-0020
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0020
---

# DelegateTool struct and parameter schema

## Parent Initiative

[[ARAWN-I-0020]] - Subagent Delegation

## Objective

Create the `DelegateTool` struct in `arawn-agent` that implements the `Tool` trait, with a complete JSON schema for the LLM to invoke subagents.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] `DelegateTool` struct created in `crates/arawn-agent/src/tools/delegate.rs`
- [x] Implements `Tool` trait with `name()` returning `"delegate"`
- [x] `parameters()` returns JSON schema with: `agent` (required), `task` (required), `context` (optional), `background` (optional, default false), `max_turns` (optional)
- [x] `description()` explains when/how to delegate to subagents
- [x] Struct holds `SharedSubagentSpawner` (trait-based, avoids cyclic dependency)
- [x] Unit tests for parameter validation (10 tests)

## Implementation Notes

### File Location
`crates/arawn-agent/src/tools/delegate.rs`

### Struct Design

```rust
pub struct DelegateTool {
    spawner: Arc<AgentSpawner>,
    configs: Arc<RwLock<HashMap<String, PluginAgentConfig>>>,
}
```

### Parameter Schema

```json
{
  "type": "object",
  "properties": {
    "agent": {
      "type": "string",
      "description": "Name of the subagent to delegate to (e.g., 'researcher', 'reviewer')"
    },
    "task": {
      "type": "string", 
      "description": "Task description for the subagent to execute"
    },
    "context": {
      "type": "string",
      "description": "Additional context from the current conversation to pass to the subagent"
    },
    "background": {
      "type": "boolean",
      "default": false,
      "description": "If true, run in background and return immediately"
    },
    "max_turns": {
      "type": "integer",
      "description": "Override maximum conversation turns for this delegation"
    }
  },
  "required": ["agent", "task"]
}
```

### Dependencies

- `AgentSpawner` from `arawn-plugin` (already exists)
- `PluginAgentConfig` from `arawn-plugin` (already exists)

## Status Updates

### Session 1 (2026-02-05)

**Completed:**
- [x] Created `SubagentSpawner` trait in `arawn-types/src/delegation.rs`
- [x] Added `SubagentInfo`, `SubagentResult`, `DelegationOutcome` types
- [x] Added `SharedSubagentSpawner` type alias (`Arc<dyn SubagentSpawner>`)
- [x] Created `DelegateTool` struct in `crates/arawn-agent/src/tools/delegate.rs`
- [x] Implements `Tool` trait with `name()` returning `"delegate"`
- [x] Full JSON schema with all parameters (agent, task, context, background, max_turns)
- [x] `description()` explains when/how to delegate to subagents
- [x] Custom `Debug` impl (trait object doesn't implement Debug)
- [x] 10 unit tests covering all functionality
- [x] All checks pass (`angreal check all`)
- [x] All unit tests pass (`angreal test unit`)

**Architecture Decision:**
Used trait-based abstraction to avoid cyclic dependency between `arawn-agent` and `arawn-plugin`:
- `SubagentSpawner` trait defined in `arawn-types` (shared crate)
- `DelegateTool` uses the trait via `SharedSubagentSpawner`
- Actual implementation will be in `arawn-plugin` (future task T-0137)

**Files Created:**
- `crates/arawn-types/src/delegation.rs` - Trait and types for subagent delegation
- `crates/arawn-agent/src/tools/delegate.rs` - DelegateTool implementation

**Files Modified:**
- `crates/arawn-types/src/lib.rs` - Export delegation module
- `crates/arawn-agent/src/tools/mod.rs` - Export DelegateTool