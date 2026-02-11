---
id: tui-client
level: initiative
title: "TUI Client"
short_code: "ARAWN-I-0025"
created_at: 2026-02-10T14:10:49.442536+00:00
updated_at: 2026-02-11T00:28:34.227161+00:00
parent: ARAWN-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/decompose"


exit_criteria_met: false
estimated_complexity: L
strategy_id: NULL
initiative_id: tui-client
---

# TUI Client Initiative

## Context

Terminal-based client for Arawn. Minimal, keyboard-driven interface for chat, session management, and workstream switching. Designed to establish core interaction patterns before building richer clients.

**Tech Stack:**
- ratatui (TUI framework)
- crossterm (terminal backend)
- tokio (async runtime)
- Integrated into main `arawn` binary as `arawn tui`

**Design Philosophy:**
- Minimal chrome, maximum content
- Keyboard-first, mouse-optional
- Every element earns its space
- Information density over decoration
- Works over SSH

## Goals & Non-Goals

**Goals:**
- Full chat with streaming responses
- Session list and switching
- Workstream management
- Tool execution visibility
- Claude Code-like interaction model

**Non-Goals:**
- Rich markdown rendering (plain text with minimal formatting)
- Images or media display
- Mouse-primary interaction
- Configuration UI (use config files)
- Vim-style modal editing

## UI Design

### Main Layout

```
┌─ arawn ──────────────────────────────────────────── ws:default ─┐
│                                                                  │
│ > Explain async/await in Rust                                   │
│                                                                  │
│ Async/await in Rust provides zero-cost asynchronous             │
│ programming. The key concepts are:                              │
│                                                                  │
│ • Future - a value that may not be ready yet                    │
│ • async fn - returns a Future instead of blocking               │
│ • .await - suspends until the Future resolves                   │
│                                                                  │
│ ┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄ │
│ [shell] cargo doc --open                              ✓ 0.3s   │
│ ┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄ │
│                                                                  │
│ The runtime (like tokio) manages scheduling...█                 │
│                                                                  │
├──────────────────────────────────────────────────────────────────┤
│ >                                                                │
└──────────────────────────────── ^K palette │ ^S sessions │ ^W ws ┘
```

### Interaction Model

No vim modes. Two focus states:

| Focus | Behavior |
|-------|----------|
| **Input** (default) | Type to compose, `Enter` to send, `↑` for history |
| **List** (sessions/workstreams) | Arrow keys to navigate, `Enter` to select, `Esc` to close |

Command palette (`Ctrl+K`) for discovery and actions.

### Command Palette (`Ctrl+K`)

```
┌─ command ────────────────────────────────────────────────────────┐
│ > ses                                                            │
├──────────────────────────────────────────────────────────────────┤
│   Sessions: Switch...                                   Ctrl+S   │
│   Sessions: New                                         Ctrl+N   │
│   Sessions: Delete current                                       │
│   ─────────────────────────────────────────────────────────────  │
│   Workstreams: Switch...                                Ctrl+W   │
│   Workstreams: Create                                            │
└──────────────────────────────────────────────────────────────────┘
```

Fuzzy filter as you type. Actions organized by category.

### Session List (`Ctrl+S`)

```
┌─ sessions ──────────────────────────────────────────────────────┐
│ > search...                                                      │
├──────────────────────────────────────────────────────────────────┤
│ > • async/await explanation                          2 min ago  │
│     Debug auth middleware                            yesterday  │
│     Rust workspace setup                               2 days   │
│     Memory indexing questions                          3 days   │
│                                                                  │
└────────────────────────────── ↑↓ navigate │ enter select │ esc ──┘
```

### Workstream List (`Ctrl+W`)

```
┌─ workstreams ───────────────────────────────────────────────────┐
│ > search...                                                      │
├──────────────────────────────────────────────────────────────────┤
│ > ★ Q4 Architecture Redesign                    127 msgs  2h   │
│     Frontend Migration                           89 msgs  1d   │
│     API Refactor                                 45 msgs  3d   │
│   ─────────────────────────────────────────────────────────     │
│     [archived] Old Project                      230 msgs  2mo  │
│                                                                  │
└────────────────────────────── ↑↓ navigate │ enter select │ esc ──┘
```

### Tool Execution Display

Inline - always compact, one line per tool:
```
┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄
[shell] ls -la /src                                      ✓ 0.1s
┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄
```

**Ctrl+O** - Open output in `$EDITOR` or `$PAGER` (decoupled view):
- Spawns external process with full tool output
- Returns to TUI when closed
- Good for large outputs, copying, searching

**Ctrl+E** - Expanded log pane (split view):
```
┌─ arawn ──────────────────────────────────────────── ws:default ─┐
│ > Explain async/await                                           │
│                                                                  │
│ ┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄ │
│ [shell] ls -la /src                              ✓ 0.1s        │
│ ┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄ │
│                                                                  │
├─ tool output ────────────────────────────────────────── [Ctrl+E] ┤
│ total 42                                                         │
│ drwxr-xr-x  5 user staff  160 Jan 15 10:00 .                    │
│ -rw-r--r--  1 user staff 1234 Jan 15 09:30 main.rs              │
│ -rw-r--r--  1 user staff  567 Jan 14 15:22 lib.rs               │
├──────────────────────────────────────────────────────────────────┤
│ >                                                                │
└──────────────────────────────────────────────────────────────────┘
```
- Bottom pane shows selected tool's output
- Navigate tools with `j/k`, pane scrolls with `J/K`
- `Ctrl+E` again to close pane
- Pane persists while navigating chat

### Streaming Indicator

```
The runtime manages scheduling...▌
```

Just a blinking cursor/block at the end of streaming text.

## Keybindings

### Input Focus (Default)
| Key | Action |
|-----|--------|
| `Enter` | Send message |
| `Shift+Enter` | Newline |
| `↑` | Previous input history |
| `↓` | Next input history |
| `Ctrl+C` | Cancel / stop generation |
| `Ctrl+K` | Open command palette |
| `Ctrl+S` | Open sessions |
| `Ctrl+W` | Open workstreams |
| `Ctrl+N` | New session |
| `Ctrl+O` | Open tool output in $EDITOR/$PAGER |
| `Ctrl+E` | Toggle expanded tool output pane |
| `Ctrl+Q` | Quit |

### List Focus (Sessions/Workstreams/Command Palette)
| Key | Action |
|-----|--------|
| `↑/↓` | Navigate items |
| `Enter` | Select item |
| `Esc` | Close panel, return to input |
| Type | Filter/search items |

### Tool Output Pane (when open via Ctrl+E)
| Key | Action |
|-----|--------|
| `↑/↓` | Scroll output |
| `PgUp/PgDn` | Page scroll |
| `Ctrl+E` | Close pane |

## Architecture

```
arawn-tui/
├── app.rs           # Main App state and event loop
├── ui/
│   ├── mod.rs
│   ├── chat.rs      # Chat view rendering
│   ├── input.rs     # Input area
│   ├── sessions.rs  # Session panel
│   ├── workstreams.rs # Workstream panel
│   └── tools.rs     # Tool execution display
├── state/
│   ├── mod.rs
│   ├── messages.rs  # Message buffer
│   ├── focus.rs     # Focus state (input/list/pane)
│   └── streaming.rs # Streaming handler
└── events.rs        # Keyboard/terminal events
```

### Focus States

```
                  Ctrl+K / Ctrl+S / Ctrl+W
              ┌────────────────────────────────┐
              │                                │
              ▼                                │
        ┌───────────┐                    ┌───────────┐
        │   List    │       Esc          │   Input   │
        │  Focus    │ ──────────────────▶│  Focus    │
        │           │       Enter        │ (default) │
        └───────────┘ ──────────────────▶└───────────┘
              │                                │
              │ (select action)                │ Ctrl+E
              ▼                                ▼
        ┌───────────┐                    ┌───────────┐
        │  Execute  │                    │   Tool    │
        │  Action   │                    │   Pane    │
        └───────────┘                    └───────────┘
```

Input focus is the default and where you spend most time. List focus is transient—select and return.

## Technical Design

### Core Types

```rust
/// Application state
pub struct App {
    focus: Focus,
    chat: ChatState,
    input: InputState,
    sessions: SessionList,
    workstreams: WorkstreamList,
    command_palette: CommandPalette,
    tool_pane: Option<ToolPane>,
    client: ArawnClient,
}

/// Focus determines which component handles input
pub enum Focus {
    Input,                    // Default - typing messages
    Sessions,                 // Session list overlay
    Workstreams,              // Workstream list overlay
    CommandPalette,           // Ctrl+K palette
    ToolPane,                 // Ctrl+E expanded view
}

/// Chat message for display
pub struct ChatMessage {
    id: MessageId,
    role: Role,
    content: String,
    tools: Vec<ToolExecution>,
    timestamp: DateTime<Utc>,
    streaming: bool,
}

/// Tool execution record
pub struct ToolExecution {
    id: ToolId,
    name: String,
    args_summary: String,      // Truncated for inline display
    output: String,            // Full output for Ctrl+O/E
    status: ToolStatus,
    duration: Duration,
}

pub enum ToolStatus {
    Running,
    Success,
    Error(String),
}
```

### WebSocket Protocol

Connect to `ws://localhost:{port}/ws/chat` with optional workstream:

```rust
// Client -> Server
pub enum ClientMessage {
    Send { content: String },
    Cancel,
    SwitchSession { id: SessionId },
    SwitchWorkstream { id: WorkstreamId },
    NewSession,
    ListSessions,
    ListWorkstreams,
}

// Server -> Client
pub enum ServerMessage {
    // Streaming
    Delta { content: String },
    ToolStart { id: ToolId, name: String, args: String },
    ToolEnd { id: ToolId, status: ToolStatus, output: String, duration_ms: u64 },
    Done { message_id: MessageId },
    
    // Lists
    Sessions { items: Vec<SessionSummary> },
    Workstreams { items: Vec<WorkstreamSummary> },
    
    // State
    Error { message: String },
    SessionChanged { id: SessionId, history: Vec<ChatMessage> },
    WorkstreamChanged { id: WorkstreamId },
}
```

### Event Loop

```rust
async fn run(mut app: App, mut terminal: Terminal) -> Result<()> {
    let mut events = EventStream::new();  // crossterm
    let mut ws_rx = app.client.subscribe();
    
    loop {
        // Render current state
        terminal.draw(|f| ui::render(&app, f))?;
        
        tokio::select! {
            // Terminal input
            Some(Ok(event)) = events.next() => {
                if let Event::Key(key) = event {
                    if handle_key(&mut app, key).await? == Action::Quit {
                        break;
                    }
                }
            }
            
            // WebSocket messages
            Some(msg) = ws_rx.recv() => {
                handle_server_message(&mut app, msg)?;
            }
        }
    }
    Ok(())
}
```

### Component Rendering

Each component implements a render trait:

```rust
pub trait Component {
    fn render(&self, area: Rect, buf: &mut Buffer, focused: bool);
    fn handle_key(&mut self, key: KeyEvent) -> Option<Action>;
}
```

**Layout calculation:**
```
┌─────────────────────────────────────────────┐
│ header (1 line)                             │
├─────────────────────────────────────────────┤
│                                             │
│ chat area (flex)                            │
│                                             │
├─────────────────────────────────────────────┤
│ tool pane (0 or 30% when open)              │
├─────────────────────────────────────────────┤
│ input (3 lines min, grows with content)     │
├─────────────────────────────────────────────┤
│ status bar (1 line)                         │
└─────────────────────────────────────────────┘
```

### Input Handling

```rust
async fn handle_key(app: &mut App, key: KeyEvent) -> Result<Action> {
    // Global shortcuts first
    match key.code {
        KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            return Ok(Action::Quit);
        }
        KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.focus = Focus::CommandPalette;
            return Ok(Action::Continue);
        }
        KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.focus = Focus::Sessions;
            app.sessions.refresh().await?;
            return Ok(Action::Continue);
        }
        // ... more global shortcuts
        _ => {}
    }
    
    // Delegate to focused component
    match app.focus {
        Focus::Input => app.input.handle_key(key),
        Focus::Sessions => app.sessions.handle_key(key),
        Focus::Workstreams => app.workstreams.handle_key(key),
        Focus::CommandPalette => app.command_palette.handle_key(key),
        Focus::ToolPane => app.tool_pane.as_mut().unwrap().handle_key(key),
    }
}
```

### Streaming Display

```rust
impl ChatState {
    fn append_delta(&mut self, content: &str) {
        if let Some(msg) = self.messages.last_mut() {
            if msg.streaming {
                msg.content.push_str(content);
                self.scroll_to_bottom();
            }
        }
    }
    
    fn render_message(&self, msg: &ChatMessage, area: Rect, buf: &mut Buffer) {
        // User messages: "> {content}"
        // Assistant messages: wrapped text
        // Tool blocks: dotted separator + compact line
        // Streaming: append blinking cursor "▌"
    }
}
```

### Error Handling

```rust
/// TUI-specific errors
pub enum TuiError {
    Connection(String),      // WebSocket connection failed
    Protocol(String),        // Unexpected message format
    Terminal(std::io::Error), // Terminal I/O
}

impl App {
    fn show_error(&mut self, err: TuiError) {
        // Display in status bar, auto-dismiss after 5s
        self.status_message = Some(StatusMessage {
            text: err.to_string(),
            level: StatusLevel::Error,
            expires: Instant::now() + Duration::from_secs(5),
        });
    }
}
```

### Testing Strategy

1. **Unit tests**: State transitions, input parsing, message formatting
2. **Snapshot tests**: UI rendering with insta crate
3. **Integration tests**: Mock WebSocket server, verify protocol
4. **Manual testing**: SSH, tmux, various terminal emulators

```rust
#[test]
fn test_focus_transitions() {
    let mut app = App::new_test();
    assert_eq!(app.focus, Focus::Input);
    
    app.handle_key(ctrl('k'));
    assert_eq!(app.focus, Focus::CommandPalette);
    
    app.handle_key(Key::Esc);
    assert_eq!(app.focus, Focus::Input);
}
```

## Alternatives Considered

1. **Separate binary vs subcommand**: Chose `arawn tui` subcommand - single binary distribution
2. **tui-rs vs ratatui**: ratatui is the maintained fork
3. **Async architecture**: tokio channels for streaming, crossterm for input

## Implementation Plan

1. Basic app shell with ratatui + crossterm
2. Chat view with message rendering
3. Input handling with focus states
4. WebSocket client for streaming
5. Session panel and switching
6. Workstream panel and management
7. Tool execution display
8. Polish and edge cases