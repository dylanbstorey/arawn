---
id: cli-subagent-list-command
level: task
title: "CLI subagent list command"
short_code: "ARAWN-T-0143"
created_at: 2026-02-06T03:47:52.804961+00:00
updated_at: 2026-02-07T13:04:17.427692+00:00
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

# CLI subagent list command

## Parent Initiative

[[ARAWN-I-0020]] - Subagent Delegation

## Objective

Add CLI command to list available subagents and their tool constraints, helping users understand what agents are available for delegation.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] `arawn agent list` command shows all available subagents
- [x] Output includes: name, description, allowed tools, source plugin
- [x] `arawn agent info <name>` shows detailed agent config
- [x] Works with current plugin configuration
- [x] Helpful message when no agents configured
- [x] JSON output option (`--json`) for scripting

## Implementation Notes

### Command Structure

```
arawn agent list              # List all available subagents
arawn agent info <name>       # Show details for specific agent
arawn agent list --json       # JSON output
```

### Example Output

```
$ arawn agent list
Available Subagents:
  researcher    Web research specialist      [web_fetch, web_search]     (github plugin)
  reviewer      Code review agent            [file_read, grep, glob]     (builtin)
  sandbox       Safe execution environment   [file_read, think]          (builtin)

$ arawn agent info researcher
Name: researcher
Description: Web research specialist for finding and summarizing information
Plugin: github
Allowed Tools:
  - web_fetch
  - web_search
  - memory
System Prompt: You are a research assistant...
```

### Files to Create/Modify

- `crates/arawn/src/commands/agent.rs` - New command module
- `crates/arawn/src/commands/mod.rs` - Add agent subcommand
- `crates/arawn/src/main.rs` - Wire agent command

### Dependencies

- [[ARAWN-T-0137]] - Agent configs must be accessible

## Status Updates

### 2026-02-06: Implementation Complete

**Files created/modified:**

1. **`crates/arawn/src/commands/agent.rs`** - New command module
   - `AgentArgs` and `AgentCommand` clap types
   - `arawn agent list` - Lists agents with name, description, tools, source plugin
   - `arawn agent info <name>` - Shows detailed info including system prompt
   - Case-insensitive partial name matching for `info`
   - JSON output via `--json` flag
   - `--plugin` filter for list command
   - Helpful message when no agents available
   - Loads agents from both subscribed and local plugins

2. **`crates/arawn/src/commands/mod.rs`** - Added `pub mod agent`

3. **`crates/arawn/src/main.rs`** - Wired `Agent` command variant

**Features:**
- Table output shows: NAME, DESCRIPTION, TOOLS (truncated), PLUGIN
- Verbose mode (`-v`) shows model and max_iterations
- JSON output provides full agent details for scripting
- Info command shows full system prompt (truncated at 500 chars)
- Multiple match disambiguation for info command

**Tests:** All workspace tests pass, command help verified working.