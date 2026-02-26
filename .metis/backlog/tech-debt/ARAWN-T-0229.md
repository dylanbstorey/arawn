---
id: make-per-tool-output-limits
level: task
title: "Make per-tool output limits configurable via arawn.toml"
short_code: "ARAWN-T-0229"
created_at: 2026-02-26T01:51:55.237208+00:00
updated_at: 2026-02-26T01:51:55.237208+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#tech-debt"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Make per-tool output limits configurable via arawn.toml

## Objective

Tool output truncation limits are hardcoded per-tool in `OutputConfig::for_*()` methods (`arawn-agent/src/tool.rs:612-629`). The config system has `[tools.output] max_size_bytes` but it's a single global value that only partially applies — `start.rs` wires it to shell and web_fetch but file_read and search use hardcoded defaults.

Make per-tool output limits configurable so users can tune truncation for their use case.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P3 - Low (when time permits)

### Technical Debt Impact
- **Current Problems**: Four hardcoded limits (shell 100KB, file_read 500KB, web_fetch 200KB, search 50KB) in `OutputConfig` factory methods. Config has a single global `max_size_bytes` that doesn't map to per-tool limits. `ShellConfig.max_output_size` (1MB) and `WebFetchConfig.max_text_length` (50K chars) are additional separate limits not connected to config. Confusing overlap.
- **Benefits of Fixing**: Single source of truth for all output limits. Users can tune limits per tool type in `arawn.toml`. Eliminates the disconnect between config and runtime.
- **Risk Assessment**: Low — defaults stay the same, just becomes configurable.

## Acceptance Criteria

- [ ] Per-tool output limits configurable in `arawn.toml` under `[tools.output]`
- [ ] Current defaults preserved: shell=100KB, file_read=500KB, web_fetch=200KB, search=50KB
- [ ] `start.rs` wires all per-tool limits from config (not just shell and web_fetch)
- [ ] Remove duplicate limits: consolidate `ShellConfig.max_output_size` and `WebFetchConfig.max_text_length` with `OutputConfig` limits
- [ ] Existing tests pass, new tests cover per-tool config override

## Implementation Notes

### Current State (3 overlapping limit systems)
1. **`OutputConfig::for_*()`** (`tool.rs:612-629`) — per-tool limits, hardcoded, used by `ToolRegistry::output_config_for()`
2. **`ToolOutputConfig.max_size_bytes`** (`config/types.rs:1360`) — single global config value (100KB default)
3. **`ShellConfig.max_output_size`** (`shell.rs:37`, 1MB) and **`WebFetchConfig.max_text_length`** (`web.rs:34`, 50K chars) — tool-internal limits

### Target Config Shape
```toml
[tools.output]
shell = 102400        # 100KB
file_read = 512000    # 500KB
web_fetch = 204800    # 200KB
search = 51200        # 50KB
```

### Key Files
- `arawn-agent/src/tool.rs:575-629` — `OutputConfig` struct and factory methods
- `arawn-agent/src/tool.rs:1182-1190` — `ToolRegistry::output_config_for()`
- `arawn-agent/src/tools/shell.rs:37,60` — `ShellConfig.max_output_size`
- `arawn-agent/src/tools/web.rs:34,44` — `WebFetchConfig.max_text_length`
- `arawn-config/src/types.rs:1357-1371` — `ToolOutputConfig`
- `arawn/src/commands/start.rs:319-330` — wiring

## Status Updates

*To be added during implementation*