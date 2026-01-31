---
id: hot-reload-via-file-watcher
level: task
title: "Hot-reload via file watcher"
short_code: "ARAWN-T-0117"
created_at: 2026-02-02T01:54:22.609228+00:00
updated_at: 2026-02-02T13:05:26.928229+00:00
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

# Hot-reload via file watcher

## Parent Initiative

[[ARAWN-I-0013]]

## Objective

Add `notify`-based file watching to `PluginManager` for hot-reload. When plugin files change on disk, atomically unload the old plugin state, reload from disk, and re-register all components.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `PluginManager` starts a background file watcher on all plugin directories
- [ ] Detects changes to `plugin.toml`, skill files, hook scripts, agent configs
- [ ] On change: identify which plugin was affected, reload that entire plugin
- [ ] Atomic swap: build new `LoadedPlugin`, then swap it in (no partial state)
- [ ] On plugin directory added: load new plugin
- [ ] On plugin directory removed: unload plugin
- [ ] Debounce: coalesce rapid file changes (e.g., 500ms debounce window)
- [ ] Log reload events at INFO level, errors at WARN
- [ ] `PluginManager` exposes a channel/callback for notifying consumers of reloads
- [ ] Tests: modify plugin files in temp dir, verify reload triggered
- [ ] `angreal check all` and `angreal test unit` pass

## Implementation Notes

### Technical Approach
- Use `notify` crate (already used by arawn-pipeline for workflow watching)
- `PluginManager` holds `Arc<RwLock<HashMap<String, LoadedPlugin>>>` for concurrent access during reload
- Background tokio task watches for events, debounces, and triggers reload
- Consumers (ToolRegistry, SkillRegistry, HookDispatcher) need a way to pick up changes — either re-register on reload callback, or hold `Arc` references that get swapped
- Consider `tokio::sync::watch` channel to broadcast reload events

### Dependencies
- ARAWN-T-0111 (PluginManager loading logic)
- `notify` crate

## Status Updates

### Completed
- Created `watcher.rs` with `PluginWatcher`, `PluginState`, `PluginEvent`, `WatcherHandle`
- `PluginState` behind `Arc<RwLock>` for concurrent access during reload
- `notify`+`notify-debouncer-mini` for file watching with configurable debounce (default 500ms)
- Recursive watching of plugin directories
- Per-plugin reload: identifies affected plugin dir from changed file path
- Atomic swap: loads new plugin fully before replacing in state
- Add/remove detection via `plugin.toml` existence check
- `load_initial()`, `reload_plugin()`, `remove_plugin()` for programmatic control
- `watch()` returns mpsc channel + WatcherHandle
- 8 new tests, 76 total across crate