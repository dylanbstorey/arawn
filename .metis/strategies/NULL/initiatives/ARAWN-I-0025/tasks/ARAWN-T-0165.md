---
id: tui-tool-execution-display
level: task
title: "TUI: Tool execution display"
short_code: "ARAWN-T-0165"
created_at: 2026-02-11T00:28:45.275031+00:00
updated_at: 2026-02-11T00:28:45.275031+00:00
parent: ARAWN-I-0025
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0025
---

# TUI: Tool execution display

## Objective

Implement inline tool execution display with compact one-line summaries, plus Ctrl+O (external editor) and Ctrl+E (split pane) for viewing full output.

## Acceptance Criteria

- [ ] Tool executions render inline as compact one-liners with dotted separators
- [ ] Running tools show spinner, completed show ✓/✗ and duration
- [ ] Ctrl+O opens selected tool's output in `$EDITOR` or `$PAGER`
- [ ] Ctrl+E toggles split pane showing tool output
- [ ] Arrow keys navigate between tools when pane is open
- [ ] Pane scrolls independently with PgUp/PgDn

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

*To be added during implementation*