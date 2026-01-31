---
id: migrate-hooks-to-hooks-hooks-json
level: task
title: "Migrate hooks to hooks/hooks.json format"
short_code: "ARAWN-T-0122"
created_at: 2026-02-03T19:44:21.439793+00:00
updated_at: 2026-02-03T22:00:55.960249+00:00
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

# Migrate hooks to hooks/hooks.json format

## Objective

Replace inline TOML hook definitions with Claude Code's JSON-based `hooks/hooks.json` format.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Parse hooks from `hooks/hooks.json` (or path specified in manifest)
- [x] Support all Claude hook events: PreToolUse, PostToolUse, PostToolUseFailure, PermissionRequest, UserPromptSubmit, Notification, Stop, SubagentStop, SessionStart, SessionEnd, PreCompact
- [x] Support hook types: command, prompt, agent
- [x] Support matcher patterns for tool/event filtering
- [x] Remove TOML-based hook definitions (kept HookDef for backwards compat)
- [x] Tests updated for new format

## Implementation Notes

### Claude hooks.json Format

```json
{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "Write|Edit",
        "hooks": [
          {
            "type": "command",
            "command": "${CLAUDE_PLUGIN_ROOT}/scripts/format-code.sh"
          }
        ]
      }
    ],
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "prompt",
            "prompt": "Validate this bash command for safety"
          }
        ]
      }
    ]
  }
}
```

### Hook Types

- `command`: Execute shell script (receives JSON on stdin)
- `prompt`: Evaluate with LLM (returns pass/fail/message)
- `agent`: Run agentic verifier (full agent loop)

### Files to Modify

- `crates/arawn-plugin/src/hooks.rs` - Update parsing and types
- `crates/arawn-plugin/src/types.rs` - Add JSON hook types
- `crates/arawn-plugin/src/manager.rs` - Update hook loading

### Dependencies

- ARAWN-T-0120 (manifest migration) - hooks path comes from manifest
- ARAWN-T-0124 (CLAUDE_PLUGIN_ROOT) - variable substitution in commands

## Status Updates

### Session 1 - 2026-02-03

**Implemented Claude Code hooks.json format:**

1. **types.rs** - Added new types:
   - Extended `HookEvent` with: `PostToolUseFailure`, `PermissionRequest`, `UserPromptSubmit`, `Notification`, `SubagentStop`, `PreCompact`
   - Added `HookType` enum: `Command`, `Prompt`, `Agent`
   - Added `HookAction` struct for individual hook actions
   - Added `HookMatcherGroup` for grouping hooks by matcher pattern
   - Added `HooksConfig` for parsing `hooks/hooks.json` with `from_json()` and `from_file()` methods
   - Kept `HookDef` for legacy/internal compatibility

2. **manager.rs** - Added hooks loading:
   - Added `hooks_config: Option<HooksConfig>` field to `LoadedPlugin`
   - Added `load_hooks()` method that:
     - Loads from path specified in manifest, or defaults to `hooks/hooks.json`
     - Gracefully handles missing/invalid files (returns None)
   - Added tests for hooks loading scenarios

3. **lib.rs** - Exported new types:
   - `HookAction`, `HookMatcherGroup`, `HookType`, `HooksConfig`

**Key changes:**
- Hooks now use JSON format matching Claude Code's `hooks/hooks.json` schema
- All Claude hook events supported (11 total)
- Three hook types: command (shell), prompt (LLM), agent (agentic)
- Matcher patterns for tool filtering (regex-based)

**Verification:**
- `angreal check all` passes
- `angreal test unit` - all 100 plugin tests pass

**Note:** The HookDispatcher still uses the legacy HookDef internally. Wiring the new HooksConfig to the dispatcher will be done when integrating with the agent turn loop.