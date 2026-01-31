---
id: migrate-agents-to-markdown-format
level: task
title: "Migrate agents to markdown format"
short_code: "ARAWN-T-0123"
created_at: 2026-02-03T19:44:22.228322+00:00
updated_at: 2026-02-04T02:02:30.489301+00:00
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

# Migrate agents to markdown format

## Objective

Replace TOML-based agent configs with Claude Code's markdown format for agent definitions.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Parse agents from `agents/<name>.md` markdown files (implemented in T-0120)
- [x] Support YAML frontmatter with: description, tools (implemented in T-0120)
- [x] Agent body content becomes the system prompt / detailed instructions (implemented in T-0120)
- [x] Remove TOML agent config parsing (production code uses markdown; TOML test kept for struct serde verification)
- [x] Tests updated for new format (done in T-0120)
- [x] Documentation updated to reflect Claude format

## Implementation Notes

### Claude Agent Format

```markdown
---
description: What this agent specializes in
capabilities: ["task1", "task2", "task3"]
tools: ["shell", "file_read", "grep"]
---

# Agent Name

Detailed description of agent expertise and when Claude should invoke it.

## Capabilities
- Specialized task
- Another capability

## Context and examples
When to use and what it solves.
```

### Files to Modify

- `crates/arawn-plugin/src/types.rs` - Update agent config types
- `crates/arawn-plugin/src/manager.rs` - Update agent loading
- `crates/arawn-plugin/src/agent_spawner.rs` - Adapt to new config format

### Dependencies

- ARAWN-T-0120 (manifest migration) - agents path comes from manifest

## Status Updates

### Session 1 - 2026-02-03

**Agent markdown format was already implemented in T-0120.** This task verified the implementation and updated documentation.

**Changes made:**

1. **lib.rs** - Updated module documentation:
   - Changed plugin structure docs from TOML to Claude format
   - Updated paths: `.claude-plugin/plugin.json`, `skills/<name>/SKILL.md`, `agents/<name>.md`, `hooks/hooks.json`

2. **types.rs** - Updated doc comments:
   - `PluginAgentDef`: Updated to reference markdown format
   - `PluginAgentConfig`: Updated to reference markdown format
   - `AgentSection`: Removed TOML reference

**Already implemented in T-0120:**
- `parse_agent_markdown()` function parses YAML frontmatter from `agents/<name>.md`
- `discover_agents()` scans agents directory for .md files
- Frontmatter fields: `description`, `tools`
- Body content becomes system prompt

**Verification:**
- All 100 plugin tests pass