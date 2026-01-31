---
id: example-plugin-github
level: task
title: "Example plugin: GitHub"
short_code: "ARAWN-T-0119"
created_at: 2026-02-02T01:54:24.339845+00:00
updated_at: 2026-02-03T19:22:23.421376+00:00
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

# Example plugin: GitHub

## Parent Initiative

[[ARAWN-I-0013]]

## Objective

Create a "Notes/Journal" example plugin that exercises all plugin component types: a CLI tool for note CRUD, a skill for guided journal entries, a hook for session-end journaling prompts, and an agent config for a journal-focused subagent. This validates the entire plugin system end-to-end.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Plugin directory at `examples/plugins/journal/` with `plugin.toml` manifest
- [ ] **Tool**: `journal` CLI wrapper (bash or Python script) with actions: `create`, `list`, `search`, `tag`
  - Stores notes as JSON files in `~/.local/share/arawn/journal/`
  - JSON stdin/stdout protocol per CLI tool spec
- [ ] **Skill**: `/journal-entry` — guided daily journal skill with prompts for mood, accomplishments, tomorrow's goals
  - Markdown with TOML frontmatter, uses `journal` tool
  - Args: `date` (optional, defaults to today)
- [ ] **Skill**: `/journal-review` — review journal entries for a time period
  - Args: `period` (required: "today", "week", "month")
- [ ] **Hook**: SessionEnd hook that suggests journaling if no journal entry exists for today
- [ ] **Agent**: `journal-assistant` subagent config — focused on journaling with constrained tools (`journal`, `shell`)
- [ ] **Prompt fragment**: System prompt text describing journal capabilities
- [ ] Plugin loads successfully via `PluginManager`
- [ ] Tool executes and returns valid JSON responses
- [ ] Skills invoke correctly with argument substitution
- [ ] README.md in plugin directory explaining usage

## Implementation Notes

### Technical Approach
- Tool script in bash for simplicity and portability
- Journal storage: one JSON file per entry in `~/.local/share/arawn/journal/YYYY-MM-DD.json`
- Tool actions: `create` (write entry), `list` (list entries by date range), `search` (full-text grep), `tag` (add/remove tags)
- This is primarily a validation exercise — if anything in the plugin system doesn't work, this task surfaces it

### Dependencies
- All previous tasks (T-0110 through T-0118) — full plugin system must be functional

## Status Updates

### Completed
- Created `examples/plugins/journal/` with full plugin structure
- **plugin.toml**: Manifest with all component types declared
- **tools/journal.sh**: Bash CLI tool with create, list, search, tag actions
  - JSON stdin/stdout protocol
  - Stores entries in `~/.local/share/arawn/journal/YYYY-MM-DD.json`
  - Fixed bash compatibility (shebang, jq syntax)
- **skills/journal-entry.md**: Guided daily journal skill with mood, accomplishments, goals
- **skills/journal-review.md**: Review entries for today/week/month periods
- **hooks/session-end.sh**: Suggests journaling if no entry for today
- **agents/journal-assistant.toml**: Focused subagent with constrained tools
- **README.md**: Usage documentation
- All tool actions tested and working
- Hook returns correct outcome (allow vs info)
- All workspace checks pass