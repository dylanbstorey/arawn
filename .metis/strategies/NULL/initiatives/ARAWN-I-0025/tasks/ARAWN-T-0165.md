---
id: tui-tool-execution-display
level: task
title: "TUI: Tool execution display"
short_code: "ARAWN-T-0165"
created_at: 2026-02-11T00:28:45.275031+00:00
updated_at: 2026-02-18T02:21:17.626312+00:00
parent: ARAWN-I-0025
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0025
---

# TUI: Tool execution display

## Objective

Implement inline tool execution display with compact one-line summaries, plus Ctrl+O (external editor) and Ctrl+E (split pane) for viewing full output.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Tool executions render inline as compact one-liners with dotted separators
- [x] Running tools show spinner, completed show ✓/✗ and duration
- [x] Ctrl+O opens selected tool's output in `$PAGER` (suspends TUI, runs pager, restores)
- [x] Ctrl+E toggles split pane showing tool output
- [x] Arrow keys navigate between tools when pane is open
- [x] Pane scrolls independently with PgUp/PgDn

## Implementation Notes

### Files to Create
```
crates/arawn-tui/src/
├── state/
│   └── tools.rs       # ToolExecution state, selection
└── ui/
    └── tools.rs       # Inline + pane rendering
```

### Inline Display
```
┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄
[shell] ls -la /src                      ✓ 0.1s
┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄
```

### External Editor (Ctrl+O)
```rust
fn open_in_editor(&self, output: &str) -> Result<()> {
    let editor = env::var("EDITOR")
        .or_else(|_| env::var("PAGER"))
        .unwrap_or_else(|_| "less".to_string());
    
    let mut tmp = tempfile::NamedTempFile::new()?;
    tmp.write_all(output.as_bytes())?;
    
    // Restore terminal, run editor, re-init terminal
    terminal::disable_raw_mode()?;
    Command::new(&editor).arg(tmp.path()).status()?;
    terminal::enable_raw_mode()?;
    Ok(())
}
```

### Split Pane Layout
When Ctrl+E is active, bottom 30% of screen shows tool output with scroll.

### Dependencies
- ARAWN-T-0163 (chat view)
- ARAWN-T-0162 (WebSocket for ToolStart/ToolEnd messages)

## Status Updates

### 2026-02-17: Implementation Complete

**Files Created:**
- `crates/arawn-tui/src/ui/tools.rs` - Tool pane UI with tool selector and output display

**Files Modified:**
- `crates/arawn-tui/src/app.rs`:
  - Added `args`, `started_at`, `duration_ms` to `ToolExecution` struct
  - Added `selected_tool_index` and `show_tool_pane` to App state
  - Updated `handle_tool_pane_key()` with ←/→ tool navigation, ↑↓ scroll
  - Added `open_tool_in_editor()` placeholder for Ctrl+O functionality
  - Duration tracking: record start time on ToolStart, calculate on ToolEnd
- `crates/arawn-tui/src/ui/chat.rs`:
  - New one-liner format with dotted separators `┄┄┄`
  - Added `format_duration()` for human-readable durations (ms/s/m)
  - Added `truncate_str()` helper for args/output preview
- `crates/arawn-tui/src/ui/layout.rs`:
  - Split view: 70% chat / 30% tool pane when `show_tool_pane` is true
  - Updated status bar with tool pane keybindings
- `crates/arawn-tui/src/ui/mod.rs` - Export tools module

**Features Implemented:**
- [x] Tool executions render inline as compact one-liners with dotted separators
- [x] Running tools show ◐ spinner, completed show ✓/✗ and duration (e.g., "1.2s")
- [x] Ctrl+E toggles split pane showing tool output (bottom 30%)
- [x] ←/→ arrows navigate between tools when pane is open
- [x] ↑↓/PgUp/PgDn scrolls tool output independently
- [x] Tool selector in pane title shows all tools with status indicators
- [x] Ctrl+O opens output in $PAGER (less by default), suspends/restores TUI correctly

**Additional (2026-02-17):**
- Added `tempfile` dependency for pager integration
- Implemented `run_pager()` method with proper TUI suspend/restore

**Tests:** All 53 tests passing