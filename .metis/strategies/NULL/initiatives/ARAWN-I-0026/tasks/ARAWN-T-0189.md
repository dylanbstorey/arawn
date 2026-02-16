---
id: tui-command-input
level: task
title: "TUI command input"
short_code: "ARAWN-T-0189"
created_at: 2026-02-16T18:54:56.086208+00:00
updated_at: 2026-02-16T18:54:56.086208+00:00
parent: ARAWN-I-0026
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0026
---

# TUI command input

## Parent Initiative

[[ARAWN-I-0026]] - Context Management and Auto-Compaction

## Objective

Add `/command` input handling to TUI - detect slash prefix, show autocomplete, send command requests via WebSocket.

## Acceptance Criteria

- [ ] Input starting with `/` parsed as command (not chat message)
- [ ] Autocomplete popup when typing `/`
- [ ] Available commands fetched from server
- [ ] `/compact` sends WsCommandRequest
- [ ] `/help` shows available commands
- [ ] Progress messages displayed during execution
- [ ] Result/error displayed on completion

## Implementation Notes

### Files to Modify
- `crates/arawn-tui/src/input.rs` - detect `/` prefix
- `crates/arawn-tui/src/ui/command_popup.rs` (new file)
- `crates/arawn-tui/src/app.rs` - handle command flow

### UX Flow

1. User types `/` → autocomplete popup appears
2. User selects or types command → popup filters
3. User presses Enter → send WsCommandRequest
4. Progress messages shown inline
5. Result displayed as system message

### Dependencies
- ARAWN-T-0188 (WebSocket command bridge)

## Tests

### Unit Tests
- `test_input_detects_slash_prefix` - "/" at start triggers command mode
- `test_input_normal_message` - text without "/" is chat message
- `test_command_parser` - parse command name and args from input
- `test_autocomplete_filter` - filter commands by typed prefix

### Component Tests
- `test_command_popup_render` - popup displays command list correctly
- `test_command_popup_selection` - arrow keys navigate, enter selects
- `test_progress_display` - progress messages render inline
- `test_result_display` - command result shown as system message

### Test File
- `crates/arawn-tui/src/input.rs` (inline `#[cfg(test)]` module)
- `crates/arawn-tui/src/ui/command_popup.rs` (inline tests)

## Status Updates

*To be added during implementation*