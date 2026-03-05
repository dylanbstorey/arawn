# Arawn

A personal research agent for edge computing. Arawn runs as a single binary with embedded storage, providing an LLM-powered agent with tool use, memory, workstreams, and plugin extensibility.

## Features

- **Agent loop** with streaming responses, tool execution, and context management
- **Built-in tools** &mdash; file read/write, shell, glob, grep, web search/fetch, notes, memory
- **Memory system** with SQLite-backed storage and optional vector search via local or OpenAI embeddings
- **Workstream management** for organizing work into isolated, persistent environments
- **Plugin system** with skills, hooks, agents, and prompt fragments
- **MCP integration** for connecting to Model Context Protocol servers
- **Sandboxed shell execution** via macOS sandbox-exec and Linux bubblewrap
- **Multiple LLM backends** &mdash; Anthropic, OpenAI, Groq, and OpenAI-compatible endpoints
- **OAuth 2.0 PKCE** authentication support
- **HTTP API + WebSocket** server with REST, streaming, and rate limiting
- **Terminal UI** for a keyboard-driven interactive experience

## Quick Start

### Install

```bash
curl -fsSL https://raw.githubusercontent.com/colliery-io/arawn/main/scripts/install.sh | sh
```

### Configure an LLM backend

```bash
# Anthropic (Claude)
export ANTHROPIC_API_KEY="sk-ant-..."

# Or OpenAI
export OPENAI_API_KEY="sk-..."

# Or Groq
export GROQ_API_KEY="gsk_..."
```

### Use it

```bash
# Interactive chat
arawn chat

# One-shot question
arawn ask "Explain the builder pattern in Rust"

# Start as an HTTP server
arawn start
```

## Building from Source

### Prerequisites

- Rust 1.93+ with cargo
- C compiler (for SQLite bindings)
- Linux only: bubblewrap and socat for sandboxed shell execution

### Build

```bash
git clone https://github.com/dstorey/arawn.git
cd arawn
cargo build --release

# Binary at ./target/release/arawn
```

### Development

The project uses [angreal](https://github.com/angreal/angreal) for task automation:

```bash
# Run all checks (clippy + fmt)
angreal check all

# Run unit tests
angreal test unit

# Build documentation
angreal docs build
```

## Architecture

Arawn is a Rust workspace with 18 crates organized in layers:

```
CLI / UI              Server                 TUI
  arawn ──────────── arawn-server ──────── arawn-tui
    │                    │
    └────────┬───────────┘
             ▼
        arawn-domain         (orchestration facade)
             │
    ┌────────┼────────┐
    ▼        ▼        ▼
arawn-agent  arawn-mcp  arawn-plugin
    │
    ├── arawn-llm        (LLM provider abstraction)
    ├── arawn-memory     (SQLite storage + embeddings)
    ├── arawn-sandbox    (OS-level sandboxing)
    └── arawn-pipeline   (workflow engine)
```

### Crate Map

| Crate | Description |
|-------|-------------|
| `arawn` | CLI binary &mdash; commands, REPL, client |
| `arawn-agent` | Agent loop, tool framework, context management, compaction |
| `arawn-client` | HTTP client SDK for the Arawn API |
| `arawn-config` | TOML-based configuration, LLM settings, API key resolution |
| `arawn-domain` | Domain facade orchestrating agent execution and session management |
| `arawn-llm` | LLM client abstraction for multiple providers with streaming and tool calling |
| `arawn-mcp` | Model Context Protocol client for external tool servers |
| `arawn-memory` | Persistent storage for memories, sessions, and notes (SQLite + vector search) |
| `arawn-oauth` | OAuth 2.0 PKCE proxy for Claude MAX authentication |
| `arawn-pipeline` | Workflow orchestration engine for resilient async task pipelines |
| `arawn-plugin` | Plugin system with skills, hooks, agents, and manifest loading |
| `arawn-sandbox` | OS-level sandboxing for shell commands (macOS sandbox-exec, Linux bubblewrap) |
| `arawn-script-sdk` | Utilities for agent-generated Rust scripts compiled to WASM |
| `arawn-server` | HTTP API and WebSocket server with auth and rate limiting |
| `arawn-session` | Session cache with LRU eviction and optional TTL |
| `arawn-tui` | Terminal User Interface |
| `arawn-types` | Shared types for config, delegation, filesystem gating, hooks |
| `arawn-workstream` | Workstream management with persistent message history |

### WASM Runtimes

The `runtimes/` directory contains WASM-based tool runtimes: `file_read`, `file_write`, `http`, `shell`, `passthrough`, and `transform`.

## Configuration

Arawn uses TOML configuration. Create `arawn.toml` in the working directory or `~/.config/arawn/arawn.toml`:

```toml
[llm]
backend = "anthropic"
model = "claude-sonnet-4-20250514"

[agent.default]
max_tokens = 65536

[server]
host = "127.0.0.1"
port = 8080
```

See the [configuration reference](docs/src/configuration/reference.md) for all options.

## Documentation

Full documentation is available as an mdbook:

```bash
angreal docs serve
```

Topics covered:
- [Installation](docs/src/getting-started/installation.md)
- [Quick Start](docs/src/getting-started/quickstart.md)
- [Architecture](docs/src/architecture/README.md)
- [Agent Loop](docs/src/core-systems/agent-loop.md)
- [Tools](docs/src/tools/README.md)
- [Plugins](docs/src/extensibility/plugins.md)
- [Configuration Reference](docs/src/configuration/reference.md)
- [REST API](docs/src/reference/api.md)

## License

MIT
