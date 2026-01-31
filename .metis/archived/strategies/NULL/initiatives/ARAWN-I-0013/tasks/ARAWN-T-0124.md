---
id: implement-claude-plugin-root
level: task
title: "Implement ${CLAUDE_PLUGIN_ROOT} variable substitution"
short_code: "ARAWN-T-0124"
created_at: 2026-02-03T19:44:23.338283+00:00
updated_at: 2026-02-04T02:06:30.699856+00:00
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

# Implement ${CLAUDE_PLUGIN_ROOT} variable substitution

## Objective

Implement `${CLAUDE_PLUGIN_ROOT}` environment variable substitution in all plugin paths and commands, enabling plugins to work correctly after being cached/cloned to different locations.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] `${CLAUDE_PLUGIN_ROOT}` expanded to absolute plugin directory path
- [x] Substitution applied in: hook commands, CLI tool commands
- [x] Variable set as environment variable when executing plugin scripts
- [x] Works with both local and subscribed (cloned) plugins
- [x] Tests for variable substitution (6 tests + 1 doctest)

## Implementation Notes

### Usage Context

Claude plugins use `${CLAUDE_PLUGIN_ROOT}` because plugins get copied to a cache directory after installation. Relative paths like `./scripts/foo.sh` won't work once the plugin is moved. The variable provides the absolute path to the plugin's current location.

### Where to Substitute

1. **Hook commands**: `"command": "${CLAUDE_PLUGIN_ROOT}/scripts/format.sh"`
2. **MCP server configs**: `"command": "${CLAUDE_PLUGIN_ROOT}/servers/db-server"`
3. **MCP server args/env**: `"args": ["--config", "${CLAUDE_PLUGIN_ROOT}/config.json"]`

### Implementation Approach

```rust
fn expand_plugin_root(s: &str, plugin_dir: &Path) -> String {
    s.replace("${CLAUDE_PLUGIN_ROOT}", &plugin_dir.display().to_string())
}
```

Also set `CLAUDE_PLUGIN_ROOT` as an environment variable when spawning plugin processes.

### Files to Modify

- `crates/arawn-plugin/src/hooks.rs` - Expand in command paths
- `crates/arawn-plugin/src/cli_tool.rs` - Expand in tool commands, set env var
- `crates/arawn-plugin/src/manager.rs` - Central expansion function

## Status Updates

### Session 1 - 2026-02-03

**Implemented `${CLAUDE_PLUGIN_ROOT}` variable substitution:**

1. **lib.rs** - Added utility functions and constant:
   - `CLAUDE_PLUGIN_ROOT_VAR` constant = `"CLAUDE_PLUGIN_ROOT"`
   - `expand_plugin_root(s, plugin_dir)` - expands variable in strings
   - `expand_plugin_root_path(path, plugin_dir)` - expands variable in paths
   - Added 6 tests + 1 doctest for expansion functions

2. **hooks.rs** - Updated `run_hook_command()`:
   - Expands `${CLAUDE_PLUGIN_ROOT}` in command path before execution
   - Sets `CLAUDE_PLUGIN_ROOT` environment variable (alongside `ARAWN_PLUGIN_DIR`)

3. **cli_tool.rs** - Updated `execute()`:
   - Expands `${CLAUDE_PLUGIN_ROOT}` in command path before execution
   - Sets `CLAUDE_PLUGIN_ROOT` environment variable

**Usage:**
```json
{
  "hooks": {
    "PreToolUse": [{
      "hooks": [{
        "type": "command",
        "command": "${CLAUDE_PLUGIN_ROOT}/scripts/validate.sh"
      }]
    }]
  }
}
```

**Verification:**
- `angreal check all` passes
- All 106 plugin tests pass + 1 doctest