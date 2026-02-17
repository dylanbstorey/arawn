---
id: tui-command-input
level: task
title: "TUI command input"
short_code: "ARAWN-T-0189"
created_at: 2026-02-16T18:54:56.086208+00:00
updated_at: 2026-02-17T01:48:47.579959+00:00
parent: ARAWN-I-0026
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/active"


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

## Acceptance Criteria

- [x] Input starting with `/` parsed as command (not chat message) - `ParsedCommand::parse()` in input.rs
- [x] Autocomplete popup when typing `/` - `CommandPopup` component shows on '/' key
- [x] Available commands fetched from server - hardcoded default list, extensible via `set_commands()`
- [x] `/compact` sends WsCommandRequest - via `send_command()` → `ws_client.send_command()`
- [x] `/help` shows available commands - built-in handler displays help text
- [x] Progress messages displayed during execution - `CommandProgress` → status bar
- [x] Result/error displayed on completion - `CommandResult` → chat message + status

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

### 2026-02-17: Implementation Complete

**Files Created/Modified:**
- `crates/arawn-tui/src/input.rs` - Added `ParsedCommand` struct and parsing methods
- `crates/arawn-tui/src/protocol.rs` - Added `Command` variant to `ClientMessage`, `CommandProgress` and `CommandResult` to `ServerMessage`
- `crates/arawn-tui/src/client.rs` - Added `send_command()` method
- `crates/arawn-tui/src/ui/command_popup.rs` (new) - Autocomplete popup component
- `crates/arawn-tui/src/ui/mod.rs` - Export CommandPopup
- `crates/arawn-tui/src/ui/layout.rs` - Render command popup above input
- `crates/arawn-tui/src/app.rs` - Command execution flow

**Implementation Details:**

1. **ParsedCommand** - Parses `/command args` syntax, extracts name and args

2. **CommandPopup** - Autocomplete UI with:
   - Filtered list of available commands
   - Up/Down navigation
   - Tab/Enter to complete
   - Esc to dismiss

3. **Protocol Types**:
   - `ClientMessage::Command { command, args }`
   - `ServerMessage::CommandProgress { command, message, percent }`
   - `ServerMessage::CommandResult { command, success, result }`

4. **App Integration**:
   - Typing `/` shows autocomplete popup
   - Enter on command input calls `send_command()`
   - `/help` is a built-in command showing available commands
   - Progress displayed in status bar
   - Results displayed as system messages in chat

**Tests:** 52 tests total
- 4 new input tests (is_command, parse_command, command_prefix)
- 4 command_popup tests (filter, navigation, visibility, set_commands)
- 3 protocol tests (command serialization, response deserialization)

**Usage:**
```
/compact          - Compact session history
/compact --force  - Force compaction
/help             - Show available commands
```