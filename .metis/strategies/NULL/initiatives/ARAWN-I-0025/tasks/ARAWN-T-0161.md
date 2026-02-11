---
id: tui-app-shell-and-event-loop
level: task
title: "TUI: App shell and event loop"
short_code: "ARAWN-T-0161"
created_at: 2026-02-11T00:28:41.800120+00:00
updated_at: 2026-02-11T00:28:41.800120+00:00
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

# TUI: App shell and event loop

## Objective

Create the foundational TUI application structure with ratatui, crossterm, and the async event loop that handles both terminal input and WebSocket messages.

## Acceptance Criteria

- [ ] New `arawn-tui` crate in workspace
- [ ] `arawn tui` subcommand launches the TUI
- [ ] Basic layout renders: header, content area, input, status bar
- [ ] Event loop handles keyboard input (Ctrl+Q quits)
- [ ] Terminal state properly restored on exit (normal or panic)
- [ ] Works in common terminals: iTerm2, Terminal.app, tmux

## Implementation Notes

### Files to Create
```
crates/arawn-tui/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── app.rs         # App state struct
│   ├── events.rs      # Event loop with tokio::select!
│   └── ui/
│       ├── mod.rs
│       └── layout.rs  # Main layout rendering
```

### Dependencies
```toml
[dependencies]
ratatui = "0.26"
crossterm = "0.27"
tokio = { version = "1", features = ["full"] }
```

### Key Patterns
- Use `crossterm::terminal::enable_raw_mode()` on startup
- Wrap terminal restore in `Drop` impl and panic hook
- Use `futures::StreamExt` for crossterm's `EventStream`

## Status Updates

*To be added during implementation*