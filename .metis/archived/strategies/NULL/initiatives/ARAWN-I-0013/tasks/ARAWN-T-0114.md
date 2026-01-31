---
id: hook-dispatcher
level: task
title: "Hook dispatcher"
short_code: "ARAWN-T-0114"
created_at: 2026-02-02T01:54:20.066368+00:00
updated_at: 2026-02-02T03:59:21.336093+00:00
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

# Hook dispatcher

## Parent Initiative

[[ARAWN-I-0013]]

## Objective

Implement `HookDispatcher` that fires hooks at lifecycle events in the agent turn loop. Hooks are shell commands that receive context on stdin and can block tool execution (PreToolUse) or provide side effects (PostToolUse, SessionStart, SessionEnd, Stop).

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `HookDispatcher` struct holding registered hooks grouped by `HookEvent`
- [ ] `dispatch(event, context) -> HookOutcome` async method
- [ ] `HookOutcome` enum: `Allow`, `Block { reason: String }`, `Info { output: String }`
- [ ] PreToolUse: receives `{"tool": name, "params": {...}}` on stdin. Non-zero exit = block tool call. Stdout = reason.
- [ ] PostToolUse: receives `{"tool": name, "params": {...}, "result": {...}}` on stdin. Informational only.
- [ ] SessionStart: fires when a new session/turn begins. Receives `{"session_id": ...}`. Stdout injected as context.
- [ ] SessionEnd: fires when session/turn completes. Receives `{"session_id": ..., "turn_count": ...}`.
- [ ] Stop: fires when agent produces final response. Receives `{"response": ...}`. Can validate output.
- [ ] Hook matching: `tool_match` glob pattern on tool name, `match_pattern` regex on serialized params
- [ ] Hooks run sequentially per event (first blocker wins for PreToolUse)
- [ ] Timeout: kill hook subprocess after 10s default
- [ ] Tests with simple bash hook scripts in temp dirs
- [ ] `angreal check all` and `angreal test unit` pass

## Implementation Notes

### Technical Approach
- `arawn-plugin/src/hooks.rs`
- Uses `tokio::process::Command` like CliPluginTool
- `tool_match` uses `glob::Pattern` for matching
- `match_pattern` uses `regex::Regex` for matching
- Hook context serialized as JSON via serde
- Set `ARAWN_PLUGIN_DIR` and `ARAWN_SESSION_ID` env vars for hook subprocess

### Dependencies
- ARAWN-T-0110 (Hook and HookEvent types)

## Status Updates

*To be added during implementation*