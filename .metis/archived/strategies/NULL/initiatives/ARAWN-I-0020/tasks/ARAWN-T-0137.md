---
id: wire-delegatetool-into-agent-and
level: task
title: "Wire DelegateTool into agent and ToolRegistry"
short_code: "ARAWN-T-0137"
created_at: 2026-02-06T03:47:46.159115+00:00
updated_at: 2026-02-06T13:29:07.918761+00:00
parent: ARAWN-I-0020
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: true
strategy_id: NULL
initiative_id: ARAWN-I-0020
---

# Wire DelegateTool into agent and ToolRegistry

## Parent Initiative

[[ARAWN-I-0020]] - Subagent Delegation

## Objective

Wire the `DelegateTool` into the agent's `ToolRegistry` and ensure agent configs from plugins are accessible at runtime.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] `DelegateTool` registered in `ToolRegistry` during agent build
- [x] `PluginManager` or `PluginWatcher` exposes agent configs to `start.rs`
- [x] `AgentSpawner` constructed with parent agent's backend and filtered tools
- [x] Agent configs passed to `DelegateTool` constructor
- [x] `delegate` tool appears in tool list when plugins define agents
- [ ] Integration test: agent with delegate tool can list available subagents (deferred - requires plugin fixtures)

## Implementation Notes

### Wiring in start.rs

```rust
// After loading plugins, collect agent configs
let agent_configs: HashMap<String, PluginAgentConfig> = {
    let state = watcher.state();
    let st = state.read().await;
    st.plugins()
        .flat_map(|p| p.agents.iter())
        .map(|a| (a.name.clone(), a.clone()))
        .collect()
};

// Create delegate tool if any agents defined
if !agent_configs.is_empty() {
    let spawner = AgentSpawner::new(tool_registry.clone(), backend.clone());
    let delegate_tool = DelegateTool::new(
        Arc::new(spawner),
        Arc::new(RwLock::new(agent_configs)),
    );
    tool_registry.register(delegate_tool);
}
```

### Files to Modify

- `crates/arawn-plugin/src/watcher.rs` - Expose agent configs from PluginState
- `crates/arawn/src/commands/start.rs` - Wire DelegateTool into registry
- `crates/arawn-agent/src/tools/mod.rs` - Export DelegateTool

### Dependencies

- [[ARAWN-T-0136]] - DelegateTool struct must exist first

## Status Updates

### Session 1 (2026-02-05)

**Analysis complete**:
- Examined `start.rs` - tool registration happens at lines 303-310, plugins loaded at ~492-536
- Examined `AgentSpawner` in arawn-plugin - spawns agents but doesn't implement `SubagentSpawner` trait
- DelegateTool created in T-0136 expects `SharedSubagentSpawner` (Arc<dyn SubagentSpawner>)

**Implementation plan**:
1. Create `PluginSubagentSpawner` wrapper in `agent_spawner.rs` that:
   - Holds `AgentSpawner` + `HashMap<String, PluginAgentConfig>` for agent configs
   - Implements `SubagentSpawner` trait from `arawn-types`
   
2. Wire in `start.rs`:
   - After plugins load, collect agent configs from all plugins
   - Create `PluginSubagentSpawner` with parent tools + backend + agent configs
   - Create `DelegateTool::new(spawner)` and register it

**Implementation complete.**

**Files modified**:
1. `crates/arawn-plugin/src/agent_spawner.rs`:
   - Added imports for `SubagentSpawner` trait, `DelegationOutcome`, etc.
   - Added `PluginSubagentSpawner` struct that wraps `AgentSpawner` with agent configs
   - Implemented `SubagentSpawner` trait with `list_agents`, `delegate`, `delegate_background`, `has_agent`
   - Added helper methods: `agent_count()`, `is_empty()`, `agent_names()`
   - Added 5 new unit tests for `PluginSubagentSpawner`

2. `crates/arawn-plugin/src/lib.rs`:
   - Exported `PluginSubagentSpawner` alongside `AgentSpawner`

3. `crates/arawn/src/commands/start.rs`:
   - Added collection of agent configs from loaded plugins
   - Added delegate tool wiring section that:
     - Collects `plugin_agent_configs` and `plugin_agent_sources` from plugins
     - Creates `PluginSubagentSpawner` with parent tools and backend
     - Registers `DelegateTool` with the spawner before agent build
   - Verbose logging when delegate tool is enabled

**Tests passing**:
- 10 delegate tool tests (arawn-agent)
- 10 agent_spawner tests (arawn-plugin)
- All workspace checks pass (angreal check all)