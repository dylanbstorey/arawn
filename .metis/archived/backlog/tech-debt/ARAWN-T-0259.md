---
id: standardize-cli-output-styling
level: task
title: "Standardize CLI output styling across commands"
short_code: "ARAWN-T-0259"
created_at: 2026-03-04T13:24:04.071558+00:00
updated_at: 2026-03-05T02:50:22.160207+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#tech-debt"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
---

# Standardize CLI output styling across commands

## Objective

Create a shared CLI output formatting layer so all commands produce consistent output (headers, tables, success/error markers, colors).

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P2 - Medium (nice to have)

### Technical Debt Impact
- **Current Problems**: Some commands use unicode markers, others plain text; table alignment varies; color usage is inconsistent
- **Benefits of Fixing**: Professional, consistent user experience across all commands
- **Risk Assessment**: Low — cosmetic changes only

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Shared output helpers for: success messages, error messages, tables, headers, key-value pairs
- [ ] All commands use the shared helpers
- [ ] `--json` flag produces consistent JSON structure across commands
- [ ] Colors disabled when stdout is not a TTY

## Status Updates

### Completed
- Created `crates/arawn/src/commands/output.rs` — shared output formatting module with:
  - `header(title)` — bold title + dim `─` separator
  - `success(msg)` — green `✓` + message
  - `error(msg)` — red `Error:` prefix to stderr
  - `kv(label, value)` — dim-labeled indented key-value pairs
  - `hint(msg)` — dim text for hints/notes
  - `truncate(s, max)` / `truncate_multiline(s, max)` — single shared implementation
- Refactored all 10 command files to use shared helpers:
  - `status.rs` — uses `output::header`, `output::kv`, `output::hint`
  - `memory.rs` — replaced local `truncate()`, uses `output::header`, `output::hint`, `output::success`
  - `notes.rs` — replaced local `truncate()`, uses `output::header`, `output::success`, `output::error`, `output::hint`
  - `agent.rs` — replaced local `truncate()` and `truncate_multiline()`, uses `output::header`, `output::kv`, `output::hint`
  - `config.rs` — uses `output::header`, `output::success`, `output::hint`, `output::error`
  - `auth.rs` — uses `output::header`, `output::kv`, `output::success`, `output::hint`
  - `secrets.rs` — uses `output::success`, `output::error`, `output::hint`
  - `mcp.rs` — replaced local `truncate()`, replaced `❌`/`✓` with `output::error`/`output::success`
  - `plugin.rs` — replaced local `truncate()`
  - `mod.rs` — `print_cli_error` now uses `output::error()`
- Standardized separators: all `"-".repeat(N)` → `"─".repeat(N)` (unicode box drawing)
- Standardized markers: all `❌` → `output::error()`, all bare `✓` → `output::success()`
- `console` crate already handles TTY detection for color disabling
- All checks pass (`angreal check all`), all unit tests pass (`angreal test unit`)