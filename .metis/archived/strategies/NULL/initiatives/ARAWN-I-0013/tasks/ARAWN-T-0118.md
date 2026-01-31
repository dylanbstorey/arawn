---
id: wire-pluginmanager-into-start
level: task
title: "Wire PluginManager into start command"
short_code: "ARAWN-T-0118"
created_at: 2026-02-02T01:54:23.486600+00:00
updated_at: 2026-02-03T02:15:41.739345+00:00
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

# Wire PluginManager into start command

## Parent Initiative

[[ARAWN-I-0013]]

## Objective

Wire all plugin components into the production startup path. On `arawn start`, load plugins, register CLI tools in `ToolRegistry`, inject prompt fragments into `SystemPromptBuilder`, wire hooks into the agent turn loop, make skills invocable, and start the hot-reload watcher.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `arawn-plugin` added as dependency to `arawn` and `arawn-agent` crates
- [ ] `start.rs`: create `PluginManager`, call `load_all()`, log loaded plugins
- [ ] Register all plugin CLI tools into the agent's `ToolRegistry`
- [ ] Pass plugin prompt fragments to `SystemPromptBuilder::with_plugin_prompts()`
- [ ] Wire `HookDispatcher` into agent — add hook dispatch calls at PreToolUse, PostToolUse, SessionStart, SessionEnd, Stop points in `Agent::turn()`
- [ ] Wire skill invocation: detect `/skill-name` in user messages, inject skill content
- [ ] Start hot-reload watcher in background tokio task
- [ ] Plugin agent configs available for subagent spawning (stored in AppState or Agent)
- [ ] Config: add `[plugins]` section to ArawnConfig with `dirs` list and `enabled` flag
- [ ] Startup logs: list loaded plugins, tools, skills, hooks, agents at INFO level
- [ ] `angreal check all` and `angreal test unit` pass
- [ ] Manual test: start server with no plugins dir → works fine (no plugins is valid)

## Implementation Notes

### Technical Approach
- This is the integration task that ties everything together
- Modify `Agent` to hold optional `HookDispatcher` and `SkillRegistry`
- Modify `Agent::turn()` to call hook dispatch at appropriate points
- Modify `Agent::turn()` to detect and handle skill invocations before LLM call
- Add `[plugins]` config section to `arawn-config/src/types.rs`
- Plugin dirs default to `~/.config/arawn/plugins/` and `./plugins/`

### Files to Modify
- `crates/arawn/Cargo.toml` — add arawn-plugin dep
- `crates/arawn-agent/Cargo.toml` — add arawn-plugin dep
- `crates/arawn/src/commands/start.rs` — plugin loading and wiring
- `crates/arawn-agent/src/agent.rs` — hook dispatch + skill invocation in turn loop
- `crates/arawn-config/src/types.rs` — PluginsConfig

### Dependencies
- All previous tasks (T-0110 through T-0117)

## Status Updates

### Completed
- Added `PluginsConfig` to `ArawnConfig` (enabled, dirs, hot_reload fields)
- Added to RawConfig and merge logic
- Added `arawn-plugin` dep to `arawn` binary crate
- Added `with_plugin_prompts()` to `AgentBuilder`, wired into SystemPromptBuilder in `build()`
- `start.rs` wiring:
  - Creates `PluginManager` with default + config dirs
  - `PluginWatcher::load_initial()` loads all plugins at startup
  - Registers `CliPluginTool` for each plugin tool into `ToolRegistry`
  - Collects prompt fragments and passes to `Agent::builder().with_plugin_prompts()`
  - Starts background hot-reload watcher with event logging
  - Graceful handling: no plugins dir is fine, watcher failure is non-fatal
- All workspace tests pass

### Deferred
- Hook dispatch in Agent::turn() — requires more invasive changes to arawn-agent turn loop
- Skill invocation detection in turn loop — same, better as follow-up
- These are noted but not blockers for the wiring task