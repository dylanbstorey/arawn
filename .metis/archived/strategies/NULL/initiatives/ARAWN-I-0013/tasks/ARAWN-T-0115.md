---
id: plugin-agent-spawning
level: task
title: "Plugin agent spawning"
short_code: "ARAWN-T-0115"
created_at: 2026-02-02T01:54:20.861127+00:00
updated_at: 2026-02-02T04:17:41.442445+00:00
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

# Plugin agent spawning

## Parent Initiative

[[ARAWN-I-0013]]

## Objective

Implement plugin-defined agent spawning. Plugin agent configs (TOML) define subagents with custom system prompts, constrained tool sets, and optional model overrides. Provide an `AgentSpawner` that creates `Agent` instances from these configs.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Parse agent config TOML format: name, description, system_prompt, tools list, model override, max_iterations
- [ ] `AgentSpawner` that takes a `PluginAgentConfig` + parent's `ToolRegistry` + `SharedBackend` and produces an `Agent`
- [ ] Tool constraining: subagent only has access to tools listed in config (subset of parent registry)
- [ ] System prompt: uses the plugin agent's custom system prompt text
- [ ] Model override: if specified, use a different backend (requires backend resolver — store as metadata for now, actual model switching deferred to wiring task)
- [ ] max_iterations: cap the agent turn loop iterations
- [ ] Tests: spawn agent from config, verify tool constraint, verify system prompt
- [ ] `angreal check all` and `angreal test unit` pass

## Implementation Notes

### Technical Approach
- `arawn-plugin/src/agent_spawner.rs`
- Depends on `arawn-agent` for `Agent`, `AgentBuilder`, `ToolRegistry`
- Tool constraining: create new `ToolRegistry` with only the named tools from parent
- Model override stored as string in config — actual backend resolution happens in T-0118 (wiring)
- Uses existing `AgentBuilder` fluent API

### Dependencies
- ARAWN-T-0110 (PluginAgentConfig type)
- Depends on `arawn-agent` crate

## Status Updates

### Completed
- Created `agent_spawner.rs` with `AgentSpawner` struct
- Tool constraining from parent registry working
- System prompt and max_iterations from config
- Model override stored but deferred to wiring task
- Fixed API mismatches: `with_shared_backend()`, `agent.tools().names()`, removed `Debug` derive (SharedBackend not Debug)
- Added `arawn-llm` dependency for `SharedBackend` type
- 5 tests passing, 68 total across crate