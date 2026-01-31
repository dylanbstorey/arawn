---
id: plugin-discovery-and-loading
level: task
title: "Plugin discovery and loading"
short_code: "ARAWN-T-0111"
created_at: 2026-02-02T01:54:17.507176+00:00
updated_at: 2026-02-02T03:20:58.082338+00:00
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

# Plugin discovery and loading

## Parent Initiative

[[ARAWN-I-0013]]

## Objective

Implement `PluginManager` that scans plugin directories (`~/.config/arawn/plugins/` and `./plugins/`), parses `plugin.toml` manifests, loads all component files (skills, hooks, agent configs) from disk, and resolves relative paths. Produces a collection of `LoadedPlugin` structs ready for registration.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `PluginManager` struct with configurable plugin directory list
- [ ] `LoadedPlugin` struct holding parsed manifest + loaded component content (skill markdown, agent TOML)
- [ ] Directory scanning: find all dirs containing `plugin.toml`
- [ ] Load and validate each manifest, skip invalid plugins with warning log
- [ ] Load skill files from disk (raw markdown content)
- [ ] Load agent config files from disk (parse TOML)
- [ ] Resolve all relative paths against plugin directory
- [ ] `PluginManager::load_all() -> Vec<LoadedPlugin>` returns all successfully loaded plugins
- [ ] Error handling: individual plugin failures don't prevent other plugins from loading
- [ ] Tests with temp directories containing sample plugin structures
- [ ] `angreal check all` and `angreal test unit` pass

## Implementation Notes

### Technical Approach
- Lives in `arawn-plugin` crate alongside manifest types from T-0110
- Uses `std::fs` for directory scanning and file reading
- Log warnings via `tracing` for invalid/missing plugins
- `LoadedPlugin` owns all loaded content (no lazy loading for v1)

### Dependencies
- ARAWN-T-0110 (manifest types must exist first)

## Status Updates

*To be added during implementation*