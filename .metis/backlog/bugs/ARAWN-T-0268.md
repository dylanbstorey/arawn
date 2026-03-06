---
id: startup-errors-logged-to-stderr
level: task
title: "Startup errors logged to stderr without timestamps"
short_code: "ARAWN-T-0268"
created_at: 2026-03-06T02:27:45.186703+00:00
updated_at: 2026-03-06T02:27:45.186703+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#bug"


exit_criteria_met: false
initiative_id: NULL
---

# Startup errors logged to stderr without timestamps

## Objective

Startup errors from `arawn start` are written to stderr via `anyhow`'s default `Display` — bare text with no timestamps, no log level, no structure. This makes it impossible to distinguish old errors from new ones when inspecting `launchd-stderr.log`, since the same line repeats with no temporal context.

## Bug Details

### Type
- [x] Bug - Production issue that needs fixing

### Priority
- [ ] P1 - High (important for user experience)

### Impact Assessment
- **Affected Users**: All users running arawn as a launchd/systemd service
- **Reproduction Steps**: 
  1. Start arawn with a missing API key (or any startup error condition)
  2. Fix the issue and restart
  3. Check `~/.config/arawn/logs/launchd-stderr.log`
  4. Old errors and new errors are indistinguishable — no timestamps
- **Expected vs Actual**: Errors should be timestamped and structured (like stdout logs via `tracing`). Instead they are bare `Error: <message>` lines.

### Root Cause

`crates/arawn/src/commands/start.rs` returns `anyhow::Result` from `run()`. When this errors, clap/main prints it to stderr as plain text. The tracing subscriber is configured for stdout only. Stderr gets raw `eprintln!`/`anyhow` output with no formatting.

## Acceptance Criteria

- [ ] Startup errors are logged with timestamps and log level (ERROR) matching the tracing format used on stdout
- [ ] `launchd-stderr.log` entries have timestamps so old vs new errors can be distinguished
- [ ] Existing stdout tracing output is unaffected

## Implementation Notes

### Technical Approach
Route fatal startup errors through the `tracing` infrastructure (`tracing::error!`) before exiting, or configure the tracing subscriber to also write to stderr for ERROR level. Alternatively, ensure the main entrypoint catches errors and logs them via tracing before printing to stderr.

### Key Files
- `crates/arawn/src/commands/start.rs` — startup error path
- `crates/arawn/src/main.rs` — top-level error handling

## Status Updates

*To be added during implementation*