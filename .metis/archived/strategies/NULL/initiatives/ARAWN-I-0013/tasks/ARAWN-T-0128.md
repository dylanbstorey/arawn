---
id: cli-commands-arawn-plugin-add
level: task
title: "CLI commands: arawn plugin add/update/remove/list"
short_code: "ARAWN-T-0128"
created_at: 2026-02-03T19:44:26.714063+00:00
updated_at: 2026-02-04T13:28:57.772087+00:00
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

# CLI commands: arawn plugin add/update/remove/list

## Objective

Add CLI subcommands for managing plugin subscriptions: add, update, remove, and list.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] `arawn plugin add <url>` - Subscribe to a plugin (clone + add to config)
- [x] `arawn plugin update [name]` - Update one or all subscribed plugins
- [x] `arawn plugin remove <name>` - Unsubscribe and delete cached plugin
- [x] `arawn plugin list` - Show all plugins (local + subscribed, enabled/disabled)
- [x] Pretty output with plugin name, version, source, status
- [x] Error handling for invalid URLs, missing plugins, etc.

## Implementation Notes

### CLI Structure

```
arawn plugin <subcommand>

Subcommands:
  add <url>       Subscribe to a plugin from git URL or GitHub shorthand
  update [name]   Update subscribed plugins (all if no name given)
  remove <name>   Unsubscribe and remove a plugin
  list            List all installed plugins
```

### Example Usage

```bash
# Add from GitHub
arawn plugin add dstorey/arawn-journal-plugin

# Add from full URL with specific tag
arawn plugin add https://github.com/author/plugin.git --ref v1.0.0

# Update all
arawn plugin update

# Update specific plugin
arawn plugin update journal

# Remove
arawn plugin remove journal

# List
arawn plugin list
```

### List Output Format

```
NAME            VERSION   SOURCE                              STATUS
journal         0.1.0     local                               enabled
github-tools    2.1.0     github.com/dstorey/github-plugin    enabled
home-auto       1.0.0     gitlab.com/team/home-automation     disabled
```

### Files to Create/Modify

- `crates/arawn/src/commands/plugin.rs` - New file for plugin subcommands
- `crates/arawn/src/commands/mod.rs` - Add plugin module
- `crates/arawn/src/main.rs` - Wire up plugin command

### Dependencies

- ARAWN-T-0125 (subscription config)
- ARAWN-T-0126 (git clone/update)

## Status Updates

### Completed 2026-02-04

**New file: `crates/arawn/src/commands/plugin.rs`**

Implements four subcommands:

**`arawn plugin add <source> [--ref <ref>] [--project]`**
- Parses GitHub shorthand (`owner/repo`) or full git URLs
- Clones plugin to cache directory
- Saves subscription to global or project config

**`arawn plugin update [name]`**
- Updates all subscribed plugins (or just one if name given)
- Uses async parallel sync from T-0127
- Reports cloned/updated/skipped/failed counts

**`arawn plugin remove <name> [--project] [--delete-cache]`**
- Removes subscription from config
- Optionally deletes cached files
- Matches by name or subscription ID

**`arawn plugin list [--subscribed] [--local]`**
- Lists subscribed and local plugins in table format
- Shows ID, ref, source, status for subscribed
- Shows name, version, path for local
- JSON output with `--json` flag

**Files modified:**
- `commands/mod.rs` - Added plugin module
- `main.rs` - Added Plugin command and dispatch

All 131 tests pass.