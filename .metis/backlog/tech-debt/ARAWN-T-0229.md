---
id: make-per-tool-output-limits
level: task
title: "Make per-tool output limits configurable via arawn.toml"
short_code: "ARAWN-T-0229"
created_at: 2026-02-26T01:51:55.237208+00:00
updated_at: 2026-02-26T19:35:56.897939+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#tech-debt"
  - "#phase/completed"


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

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Per-tool output limits configurable in `arawn.toml` under `[tools.output]`
- [x] Current defaults preserved: shell=100KB, file_read=500KB, web_fetch=200KB, search=50KB
- [x] `start.rs` wires all per-tool limits from config (not just shell and web_fetch)
- [x] Remove duplicate limits: consolidate `ShellConfig.max_output_size` and `WebFetchConfig.max_text_length` with `OutputConfig` limits
- [x] Existing tests pass, new tests cover per-tool config override

## Implementation Notes

### Current State (3 overlapping limit systems)
1. **`OutputConfig::for_*()`** (`tool.rs:612-629`) — per-tool limits, hardcoded, used by `ToolRegistry::output_config_for()`
2. **`ToolOutputConfig.max_size_bytes`** (`config/types.rs:1360`) — single global config value (100KB default)
3. **`ShellConfig.max_output_size`** (`shell.rs:37`, 1MB) and **`WebFetchConfig.max_text_length`** (`web.rs:34`, 50K chars) — tool-internal limits

### Target Config Shape
```toml
[tools.output]
max_size_bytes = 102400   # default fallback
shell = 102400            # 100KB
file_read = 512000        # 500KB
web_fetch = 204800        # 200KB
search = 51200            # 50KB
```

### Key Finding from Exploration

**`output_config_for()` is dead code.** The actual execution path (`Agent::execute_tools` line 578 → `ToolRegistry::execute` line 1142) always uses `OutputConfig::default()` (100KB) for all tools. The per-tool factory methods exist but are never called in production. This means file_read is truncated at 100KB instead of the intended 500KB.

### Implementation Plan

**1. Expand `ToolOutputConfig`** (`arawn-config/src/types.rs:1375-1388`)
- Add `shell: Option<usize>`, `file_read: Option<usize>`, `web_fetch: Option<usize>`, `search: Option<usize>`
- `None` → falls back to existing hardcoded defaults from `OutputConfig::for_*()`
- Keep `max_size_bytes` as the default/fallback for unknown tools

**2. Add output overrides to `ToolRegistry`** (`arawn-agent/src/tool.rs`)
- Add `output_overrides: HashMap<String, OutputConfig>` field
- Add `set_output_config(&mut self, name: &str, config: OutputConfig)` method
- Update `output_config_for()` to check overrides first, then hardcoded defaults

**3. Fix `execute_tools()` to use per-tool limits** (`arawn-agent/src/agent.rs:578`)
- Change `self.tools.execute(...)` → `self.tools.execute_with_config(..., &self.tools.output_config_for(&tool_use.name))`
- This is the critical bug fix — makes per-tool limits actually apply

**4. Wire config → ToolRegistry** (`arawn/src/commands/start.rs:319-330`)
- After creating `tool_registry`, apply per-tool overrides from config
- Remove `max_text_length: tools_cfg.output.max_size_bytes` from `WebFetchConfig` (was incomplete wiring)

**5. Remove duplicate tool-internal truncation**
- `ShellConfig.max_output_size` (shell.rs): Default to `usize::MAX` — let `OutputConfig` be the single source of truth. Wire config value to both places if set.
- `WebFetchConfig.max_text_length` (web.rs): Same — default to `usize::MAX`, let `OutputConfig` handle it.

**6. Update `HasToolConfig` trait** (`arawn-types/src/config.rs`)
- Add per-tool output accessors for forward compatibility

### Key Files
- `arawn-agent/src/tool.rs:575-629` — `OutputConfig` struct and factory methods
- `arawn-agent/src/tool.rs:1182-1190` — `ToolRegistry::output_config_for()`
- `arawn-agent/src/agent.rs:576-578` — `execute_tools()` call site (the bug)
- `arawn-agent/src/tools/shell.rs:37,60` — `ShellConfig.max_output_size`
- `arawn-agent/src/tools/web.rs:34,44` — `WebFetchConfig.max_text_length`
- `arawn-config/src/types.rs:1375-1388` — `ToolOutputConfig`
- `arawn-types/src/config.rs:32-44` — `HasToolConfig` trait
- `arawn/src/commands/start.rs:314-341` — wiring

## Status Updates

### Session 1 (2026-02-26)
**All work completed.**

**Key finding:** `output_config_for()` was dead code — `Agent::execute_tools()` always called `ToolRegistry::execute()` which hardcodes `OutputConfig::default()` (100KB). Per-tool limits (file_read=500KB, web_fetch=200KB, search=50KB) were defined but never applied.

**Config layer (`arawn-config/src/types.rs`):**
- Expanded `ToolOutputConfig` with `shell`, `file_read`, `web_fetch`, `search` (`Option<usize>`) fields
- All default to `None` (falls back to hardcoded per-tool defaults)

**Registry overrides (`arawn-agent/src/tool.rs`):**
- Added `output_overrides: HashMap<String, OutputConfig>` to `ToolRegistry`
- Added `set_output_config()` method
- Updated `output_config_for()` to check overrides first, then hardcoded defaults

**Critical bug fix (`arawn-agent/src/agent.rs`):**
- Changed `execute_tools()` from `self.tools.execute(...)` to `self.tools.execute_with_config(..., &self.tools.output_config_for(&tool_use.name))`
- Per-tool limits now actually apply in production

**Wiring (`arawn/src/commands/start.rs`):**
- Per-tool config overrides wired to `ToolRegistry` via `set_output_config()` for all aliases
- Shell config limit wired to `ShellConfig.max_output_size`
- Web config limit wired to `WebFetchConfig.max_text_length`
- Consolidated: one config knob controls both internal tool limit and post-execution sanitization

**Tests:**
- `test_registry_output_config_override` — override takes precedence over defaults
- `test_registry_output_config_override_all_aliases` — both aliases get overridden
- `test_tool_output_config_per_tool_fields` — TOML round-trip with per-tool values
- `test_tool_output_config_defaults_none` — per-tool fields default to None
- All existing tests pass, `angreal check all` clean