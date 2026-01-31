---
id: built-in-wasm-runtimes-file-read
level: task
title: "Built-in WASM runtimes: file_read, file_write, shell, transform"
short_code: "ARAWN-T-0087"
created_at: 2026-01-30T03:41:26.572416+00:00
updated_at: 2026-01-30T04:22:51.108637+00:00
parent: ARAWN-I-0019
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0019
---

# Built-in WASM runtimes: file_read, file_write, shell, transform

## Parent Initiative

[[ARAWN-I-0019]] — WASM Script Runtime System

## Objective

Build the remaining four built-in WASM runtimes: `file_read` (reads a file path from config and returns its contents), `file_write` (writes content to a file path), `shell` (executes a command with args and configurable timeout), and `transform` (applies jq-style dot-path or template transformations to context data).

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] All four runtimes compile to `wasm32-wasip1` target
- [ ] All follow the RuntimeInput/RuntimeOutput JSON protocol over stdin/stdout
- [ ] All are registered as builtin entries in the catalog
- [ ] `file_read` reads the path specified in config and returns file contents in output
- [ ] `file_write` writes `content` from config/context to the specified path, returns success/bytes-written
- [ ] `file_read` and `file_write` respect WASI sandbox filesystem constraints (only preopened directories)
- [ ] `shell` executes `command` with `args` from config, has a configurable `timeout_secs` (default 30s), returns stdout/stderr/exit_code
- [ ] `transform` supports dot-path context references (e.g., `context.task1.body`) and simple template string interpolation
- [ ] Unit tests for each runtime

## Implementation Notes

### Dependencies
- ARAWN-T-0080 (protocol types) — shared RuntimeInput/RuntimeOutput
- ARAWN-T-0081 (catalog) — registration as builtins

### Approach
Each runtime is a separate crate under `runtimes/` like the passthrough/http runtimes. `file_read`/`file_write` use WASI filesystem APIs with preopened dirs. `shell` uses `wasi:cli/command` to spawn processes (or falls back to a host-provided command execution capability). `transform` parses dot-path expressions and performs string interpolation against the context JSON — no full jq engine needed, just nested key traversal and `{{expression}}` template syntax.

## Status Updates

*No updates yet.*