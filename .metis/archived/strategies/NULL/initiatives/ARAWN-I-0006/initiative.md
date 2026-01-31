---
id: cli-command-interface-and-repl
level: initiative
title: "CLI: Command Interface and REPL"
short_code: "ARAWN-I-0006"
created_at: 2026-01-28T01:37:35.773229+00:00
updated_at: 2026-01-28T14:06:59.749610+00:00
parent: ARAWN-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: S
strategy_id: NULL
initiative_id: cli-command-interface-and-repl
---

# CLI: Command Interface and REPL Initiative

*This template includes sections for various types of initiatives. Delete sections that don't apply to your specific use case.*

## Context

The CLI is the primary interface for interacting with Arawn on the local machine. It provides both one-shot commands for quick interactions and a REPL for extended conversations. The CLI connects to the local server (arawn-server) or can run in embedded mode for single-user scenarios.

## Goals & Non-Goals

**Goals:**
- Fast startup (<500ms for simple commands)
- Rich REPL with history, completion, and streaming output
- Scriptable commands for automation
- Clear feedback on long-running task status
- Config management without editing files

**Non-Goals:**
- Full TUI dashboard (future post-MVP)
- GUI components
- Direct LLM interaction (goes through agent or server)

## Command Structure

```
arawn
├── start              # Start server in foreground or daemon mode
│   ├── --daemon       # Detach and run in background
│   ├── --port <n>     # Override default port
│   └── --bind <addr>  # Bind address (localhost default)
├── stop               # Stop running daemon
├── status             # Show server status, active tasks, resource usage
│   └── --json         # Output as JSON for scripting
├── ask "<prompt>"     # One-shot question (starts session if needed)
│   ├── --session <id> # Continue existing session
│   └── --no-memory    # Skip memory context
├── chat               # Enter REPL mode
│   ├── --session <id> # Resume session
│   └── --new          # Force new session
├── research "<topic>" # Start long-running research task
│   ├── --depth <n>    # Research depth (shallow/medium/deep)
│   └── --notify       # Push notification when complete
├── tasks              # List running and recent tasks
│   ├── list           # Show all tasks
│   ├── status <id>    # Detailed task status
│   ├── cancel <id>    # Cancel running task
│   └── result <id>    # Show task output
├── memory             # Memory operations
│   ├── search <query> # Semantic search
│   ├── recent         # Recent memories
│   ├── stats          # Memory database stats
│   └── export         # Export memories (JSON)
├── notes              # Note management
│   ├── add "<text>"   # Quick note
│   ├── list           # List notes
│   ├── search <query> # Search notes
│   └── show <id>      # Display note
├── config             # Configuration
│   ├── show           # Current config
│   ├── set <k> <v>    # Set value
│   ├── get <k>        # Get value
│   └── edit           # Open in $EDITOR
├── auth               # Authentication
│   ├── login          # OAuth flow for Claude
│   ├── status         # Show auth status
│   └── logout         # Clear credentials
└── version            # Version info
```

## REPL Design

Interactive chat mode with streaming responses:

```
arawn> How does the memory module work?
[streaming response...]

arawn> /status          # Slash commands for quick actions
arawn> /tasks
arawn> /memory search "sqlite-vec"
arawn> /help
arawn> /quit
```

**REPL Features:**
- rustyline for line editing with history
- Streaming output with proper line wrapping
- Slash commands for common operations without leaving chat
- Session persistence (auto-saves, can resume)
- Interrupt with Ctrl+C (cancels current request, doesn't exit)
- Ctrl+D to exit

## API Surface

```rust
// Main CLI entry point
pub fn main() -> Result<()>;

// Command handlers
mod commands {
    pub async fn start(opts: StartOpts) -> Result<()>;
    pub async fn stop() -> Result<()>;
    pub async fn status(opts: StatusOpts) -> Result<()>;
    pub async fn ask(prompt: &str, opts: AskOpts) -> Result<()>;
    pub async fn chat(opts: ChatOpts) -> Result<()>;
    pub async fn research(topic: &str, opts: ResearchOpts) -> Result<()>;
    pub async fn tasks(cmd: TasksCmd) -> Result<()>;
    pub async fn memory(cmd: MemoryCmd) -> Result<()>;
    pub async fn notes(cmd: NotesCmd) -> Result<()>;
    pub async fn config(cmd: ConfigCmd) -> Result<()>;
    pub async fn auth(cmd: AuthCmd) -> Result<()>;
}

// REPL
pub struct Repl {
    client: Client,
    session_id: String,
    history: History,
}

impl Repl {
    pub fn new(client: Client) -> Self;
    pub async fn run(&mut self) -> Result<()>;
    async fn handle_input(&mut self, line: &str) -> Result<ControlFlow>;
    async fn handle_slash(&mut self, cmd: &str) -> Result<()>;
}

// Client for server communication
pub struct Client {
    base_url: Url,
    token: Option<String>,
}

impl Client {
    pub fn connect(url: &str) -> Result<Self>;
    pub async fn chat(&self, msg: ChatRequest) -> Result<impl Stream<Item = ChatEvent>>;
    pub async fn tasks(&self) -> Result<Vec<Task>>;
    pub async fn memory_search(&self, query: &str) -> Result<Vec<Memory>>;
    // ...
}
```

## Output Formatting

- **Streaming text**: Print tokens as received, handle line wrapping
- **Structured data**: Tables for lists (tasks, memories, notes)
- **JSON mode**: `--json` flag for scriptable output
- **Colors**: Minimal, respects NO_COLOR env var
- **Progress**: Simple spinner for waiting states

## Dependencies

- clap (derive) - argument parsing
- rustyline - REPL line editing
- tokio - async runtime
- reqwest - HTTP client
- tokio-tungstenite - WebSocket for streaming
- serde_json - JSON serialization
- console - terminal formatting (colors, width detection)
- indicatif - progress indicators

## Alternatives Considered

**Full TUI with ratatui**: Considered building a dashboard-style interface. Rejected for MVP - adds complexity, harder to script. Can add later as `arawn tui` command.

**Embedded-only mode**: Considered running agent directly in CLI process. Rejected because server architecture enables mobile clients and better long-running task handling. CLI can still work offline if server is embedded.

## Implementation Plan

1. Scaffold with clap derive macros
2. Implement Client for server communication
3. Basic commands: start, stop, status, config
4. REPL with rustyline
5. Streaming chat integration
6. Task management commands
7. Memory and notes commands
8. Auth flow for Claude OAuth