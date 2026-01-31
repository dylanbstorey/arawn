---
id: cli-wrapper-tool-adapter
level: task
title: "CLI-wrapper tool adapter"
short_code: "ARAWN-T-0112"
created_at: 2026-02-02T01:54:18.315258+00:00
updated_at: 2026-02-02T03:26:46.556158+00:00
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

# CLI-wrapper tool adapter

## Parent Initiative

[[ARAWN-I-0013]]

## Objective

Implement `CliPluginTool` — an adapter that wraps a CLI executable as an Arawn `Tool`. The adapter spawns the subprocess, sends JSON parameters on stdin, reads JSON response from stdout, and maps it to `ToolResult`.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `CliPluginTool` struct implementing `arawn_agent::Tool` trait
- [ ] Spawns subprocess from `CliToolDef.command` path
- [ ] Sends tool parameters as JSON on stdin
- [ ] Reads JSON response from stdout: `{"success": bool, "content": string}` or `{"error": string}`
- [ ] Maps to `ToolResult::Text` on success, `ToolResult::Error` on failure
- [ ] Timeout support: kill subprocess after configurable duration (default 30s)
- [ ] Stderr captured and logged via tracing
- [ ] Non-zero exit code treated as error with stderr as message
- [ ] `name()`, `description()`, `parameters()` delegated from `CliToolDef`
- [ ] Tests with a simple bash script tool (echo back input)
- [ ] Tests for timeout, non-zero exit, malformed JSON output
- [ ] `angreal check all` and `angreal test unit` pass

## Implementation Notes

### Technical Approach
- Lives in `arawn-plugin` crate (e.g., `cli_tool.rs`)
- Depends on `arawn-agent` for `Tool` trait
- Uses `tokio::process::Command` for async subprocess management
- Set working directory to plugin dir when spawning
- Pass `ARAWN_PLUGIN_DIR` env var to subprocess

### Dependencies
- ARAWN-T-0110 (CliToolDef type)

## Status Updates

*To be added during implementation*