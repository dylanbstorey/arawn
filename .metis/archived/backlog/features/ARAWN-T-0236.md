---
id: wire-plugin-cli-tool-discovery
level: task
title: "Wire plugin CLI tool discovery into load_plugin()"
short_code: "ARAWN-T-0236"
created_at: 2026-03-01T13:35:39.428365+00:00
updated_at: 2026-03-01T13:35:39.428365+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/backlog"
  - "#feature"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Wire plugin CLI tool discovery into load_plugin()

## Objective

`CliPluginTool` executor exists and is tested (`arawn-plugin/src/cli_tool.rs`), but `load_plugin()` in `arawn-plugin/src/manager.rs` never discovers commands from a plugin's `commands/` directory or constructs `CliPluginTool` instances. Plugins can define skills, agents, and hooks — but not CLI tools.

## Backlog Item Details

### Type
- [x] Feature - New functionality or enhancement

### Priority
- [x] P3 - Low (when time permits)

### Business Justification
- **User Value**: Plugin authors can expose CLI commands that integrate into arawn's tool registry
- **Effort Estimate**: M — requires changes to `LoadedPlugin` struct, `load_plugin()` discovery, and tool registry wiring

## Acceptance Criteria

## Acceptance Criteria

- [ ] `LoadedPlugin` has a field for discovered CLI tools
- [ ] `load_plugin()` scans `commands/` directory and constructs `CliPluginTool` instances
- [ ] Discovered CLI tools are registered in the tool registry
- [ ] Existing tests pass, new tests cover command discovery

## Implementation Notes

### Key Files
- `arawn-plugin/src/manager.rs` — `LoadedPlugin` struct and `load_plugin()` function
- `arawn-plugin/src/cli_tool.rs` — `CliPluginTool` executor (already working)

### Context
Identified during ARAWN-T-0225 codebase cleanup audit. Separated as standalone feature work rather than cleanup.

## Status Updates

*To be added during implementation*