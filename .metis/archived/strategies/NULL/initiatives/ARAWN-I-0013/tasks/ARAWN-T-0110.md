---
id: plugin-manifest-and-core-types
level: task
title: "Plugin manifest and core types"
short_code: "ARAWN-T-0110"
created_at: 2026-02-02T01:54:16.614365+00:00
updated_at: 2026-02-02T02:16:04.592511+00:00
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

# Plugin manifest and core types

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[ARAWN-I-0013]]

## Objective

Create the `arawn-plugin` crate with all core types for the plugin system: `PluginManifest`, `Skill`, `Hook`, `PluginAgentConfig`, `CliToolDef`, `PromptFragment`, and `HookEvent`. Implement TOML deserialization for the manifest format defined in ARAWN-I-0013.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] New `arawn-plugin` crate added to workspace
- [ ] `PluginManifest` struct with TOML deserialization matching the `plugin.toml` schema from ARAWN-I-0013
- [ ] `Skill` struct: name, description, file path, uses_tools, args
- [ ] `SkillArg` struct: name, description, required flag
- [ ] `Hook` struct: event, tool_match (glob), match_pattern (regex), command path
- [ ] `HookEvent` enum: PreToolUse, PostToolUse, SessionStart, SessionEnd, Stop
- [ ] `CliToolDef` struct: name, description, command path, JSON Schema parameters
- [ ] `PluginAgentConfig` struct: name, description, system_prompt, tools list, model override
- [ ] `PromptFragment` struct: system prompt text
- [ ] Round-trip tests: serialize/deserialize a full plugin.toml manifest
- [ ] Validation: manifest must have name and version, component paths are relative
- [ ] `angreal check all` and `angreal test unit` pass

## Implementation Notes

### Technical Approach
- New crate at `crates/arawn-plugin/`
- Depends on `serde`, `toml` for deserialization
- All paths in manifest are relative to the plugin directory — store `plugin_dir: PathBuf` alongside parsed manifest
- Use `#[serde(default)]` for optional sections (skills, hooks, agents, tools, prompt)
- Validate after parsing: check required fields, normalize paths

### Files to Create/Modify
- `crates/arawn-plugin/Cargo.toml` — new crate
- `crates/arawn-plugin/src/lib.rs` — re-exports
- `crates/arawn-plugin/src/manifest.rs` — PluginManifest + TOML parsing
- `crates/arawn-plugin/src/types.rs` — Skill, Hook, HookEvent, CliToolDef, PluginAgentConfig, PromptFragment
- `Cargo.toml` (workspace) — add to members and dependencies

## Status Updates

*To be added during implementation*