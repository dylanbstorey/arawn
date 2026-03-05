---
id: make-error-messages-actionable-for
level: task
title: "Make error messages actionable for end users"
short_code: "ARAWN-T-0249"
created_at: 2026-03-04T13:23:50.072834+00:00
updated_at: 2026-03-04T16:49:44.453002+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#feature"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
---

# Make error messages actionable for end users

## Objective

Replace raw `anyhow` error propagation in CLI commands with user-friendly error messages that explain what went wrong and suggest how to fix it.

## Backlog Item Details

### Type
- [x] Feature - New functionality or enhancement

### Priority
- [x] P1 - High (important for user experience)

### Business Justification
- **User Value**: Users currently see internal error messages like "connection refused" without guidance on what to do
- **Effort Estimate**: M

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] CLI commands wrap errors with actionable context (e.g., "Could not connect to server at localhost:8080. Is the server running? Start it with `arawn start`")
- [ ] Error messages follow a consistent format across all commands
- [ ] Internal/debug details only shown with `--verbose`
- [ ] Common failure modes (server not running, no API key, bad config) have dedicated messages

## Implementation Notes

### Key Files
- All files in `crates/arawn/src/commands/`
- Consider a shared error formatting helper

## Status Updates

### Session — 2026-03-04

**Implementation complete.** Created shared error formatting infrastructure and applied across all CLI commands.

#### What was done

1. **Created `format_user_error()` in `commands/mod.rs`** — pattern-matches on error messages to detect common failure modes and return actionable messages with suggestions:
   - Connection refused → "Could not connect to server... Start it with: arawn start"
   - DNS errors → "Could not resolve server address... set ARAWN_SERVER_URL"
   - Authentication (401) → "Check your API token or run: arawn auth login"
   - 403 Forbidden → "Access denied... authenticate with: arawn auth login"
   - 404 Not Found → "Resource not found... may have been deleted"
   - 5xx Server errors → "Check server logs... try restarting: arawn start"
   - Timeouts → "Server may be overloaded... try again"
   - TOML parse errors → "Check your config with: arawn config show"
   - WebSocket handshake → "Server may not support WebSocket..."
   - Unknown errors → passthrough original message

2. **Created `print_cli_error()`** — formats and prints errors; shows full error chain only with `--verbose`

3. **Updated commands to use shared helpers**:
   - `ask.rs` — wraps both connection errors and streaming errors
   - `memory.rs` — `cmd_search` uses `print_cli_error` instead of raw red text
   - `notes.rs` — all 5 error branches (add, list, search, show, delete) updated
   - `repl.rs` — `send_message`, `search_memory`, `add_note` all use `format_user_error`; added `server_url` field to Repl struct
   - `chat.rs` — passes `server_url` to Repl constructor

4. **Updated `main.rs`** — top-level error handler catches unformatted errors with styled output and `exit(1)`

5. **Added 14 unit tests** for `format_user_error` covering all patterns + fallback + URL inclusion

#### Files modified
- `crates/arawn/src/commands/mod.rs` — new `format_user_error()`, `print_cli_error()`, 14 tests
- `crates/arawn/src/commands/ask.rs` — use shared error formatter
- `crates/arawn/src/commands/memory.rs` — use shared error formatter
- `crates/arawn/src/commands/notes.rs` — use shared error formatter (5 sites)
- `crates/arawn/src/commands/chat.rs` — pass server_url to Repl
- `crates/arawn/src/commands/repl.rs` — add server_url field, use format_user_error
- `crates/arawn/src/main.rs` — styled top-level error handler

#### Verification
- `angreal check all` — clippy + fmt clean
- `angreal test unit` — all tests pass including 14 new tests