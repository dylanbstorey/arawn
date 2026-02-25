# Crate Structure

Arawn is organized as a layered Rust workspace with clear dependencies.

## Dependency Graph

Arrows show compile-time dependencies from each crate's `Cargo.toml`.

```
                        ┌─────────────────────────────────────────────┐
  Layer 4 (binary)      │                   arawn                     │
                        └──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──────────┘
                           │  │  │  │  │  │  │  │  │  │  │
                           │  │  │  │  │  │  │  │  │  │  └──────────────┐
                           │  │  │  │  │  │  │  │  │  │                 │
                           ▼  │  │  │  │  │  │  │  │  │                 ▼
  Layer 3 (transport)   ┌─────────────┐ │  │  │  │  │  │          ┌──────────┐
                        │arawn-server │ │  │  │  │  │  │          │arawn-tui │
                        └──┬──────────┘ │  │  │  │  │  │          └──┬───────┘
                           │            │  │  │  │  │  │             │
                           ▼            │  │  │  │  │  │             ▼
  Layer 2 (business)    ┌─────────────┐ │  │  │  │  │  │    ┌────────────┐
                        │arawn-domain │ │  │  │  │  │  │    │arawn-client│
                        └┬─┬─┬─┬─┬─┬─┘ │  │  │  │  │  │    └────────────┘
                         │ │ │ │ │ │    │  │  │  │  │  │
       ┌─────────────────┘ │ │ │ │ │    │  │  │  │  │  │
       │  ┌────────────────┘ │ │ │ │    │  │  │  │  │  │
       │  │  ┌───────────────┘ │ │ └────┼──┼──┼──┘  │  │
       ▼  ▼  ▼                 │ │      │  │  │     │  │
    ┌──────────┐  ┌────────────▼─▼──┐   │  │  │     │  │
    │arawn-    │  │arawn-agent      │   │  │  │     │  │
    │workstrm  │  │                 │   │  │  │     │  │
    └──┬───────┘  └┬──┬──┬──┬──────┘   │  │  │     │  │
       │           │  │  │  │           │  │  │     │  │
       │           │  │  │  │    ┌──────┘  │  │     │  │
       │           │  │  │  │    │         │  │     │  │
       │           │  │  │  │    ▼         │  │     │  │
       │           │  │  │  │ ┌──────────┐ │  │     │  │
       │           │  │  │  │ │arawn-    │ │  │     │  │
       │           │  │  │  │ │plugin    │ │  │     │  │
       │           │  │  │  │ └┬─────────┘ │  │     │  │
       │           │  │  │  │  │           │  │     │  │
  Layer 1          │  │  │  │  │           │  │     │  │
  (services)       ▼  │  ▼  ▼  │           ▼  │     ▼  │
    ┌──────────┐ ┌────▼──┐ ┌───────────┐ ┌────▼──┐ ┌───▼─────┐
    │arawn-    │ │arawn- │ │arawn-     │ │arawn- │ │arawn-   │
    │memory    │ │llm    │ │pipeline   │ │mcp    │ │session  │
    └──┬───────┘ └──┬────┘ └──┬────────┘ └───────┘ └─────────┘
       │            │         │
  Layer 0           ▼         ▼
  (foundation)   ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐
                 │arawn-    │ │arawn-    │ │arawn-    │ │arawn-    │ │arawn-    │
                 │types     │ │config    │ │oauth     │ │sandbox   │ │script-sdk│
                 └──────────┘ └──────────┘ └──────────┘ └──────────┘ └──────────┘
```

**Key dependency paths:**

| Crate | Depends on |
|-------|-----------|
| `arawn` | server, tui, domain, agent, config, types, memory, llm, oauth, pipeline, plugin, mcp, workstream |
| `arawn-server` | domain, agent, config, types, llm, memory, mcp, sandbox, session, workstream |
| `arawn-domain` | agent, config, types, llm, memory, mcp, sandbox, session, workstream |
| `arawn-tui` | client, config |
| `arawn-agent` | types, memory, llm, pipeline, mcp |
| `arawn-plugin` | agent, config, llm, types |
| `arawn-workstream` | llm |
| `arawn-pipeline` | config |
| `arawn-config` | types |
| `arawn-llm` | types |
| `arawn-memory` | types |
| `arawn-types` | *(none)* |
| `arawn-client` | *(none)* |
| `arawn-mcp` | *(none)* |
| `arawn-oauth` | *(none)* |
| `arawn-sandbox` | *(none)* |
| `arawn-script-sdk` | *(none)* |
| `arawn-session` | *(none)* |

## Layer Descriptions

### Layer 0: Foundation

Core types and utilities with no internal dependencies.

| Crate | Purpose |
|-------|---------|
| `arawn-types` | Shared types: Message, Session, Memory, etc. |
| `arawn-config` | Configuration loading, TOML parsing, secret resolution |
| `arawn-oauth` | OAuth PKCE flow for Claude MAX authentication |
| `arawn-script-sdk` | Pre-compiled SDK for WASM script execution |
| `arawn-sandbox` | OS-level sandboxing for shell execution (macOS sandbox-exec, Linux bubblewrap) |

### Layer 1: Services

Domain-specific functionality depending only on foundation.

| Crate | Purpose |
|-------|---------|
| `arawn-llm` | LLM backends (Anthropic, OpenAI, Groq, Ollama), embeddings |
| `arawn-memory` | Memory store with vector search, graph, confidence scoring |
| `arawn-pipeline` | Workflow execution via Cloacina engine |
| `arawn-plugin` | Plugin system: skills, hooks, agents, CLI tools |
| `arawn-mcp` | MCP (Model Context Protocol) client for tool servers |
| `arawn-session` | Session cache with LRU eviction and TTL |

### Layer 2: Business Logic

Core agent functionality combining services.

| Crate | Purpose |
|-------|---------|
| `arawn-agent` | Agentic loop, tools, context building, session indexing |
| `arawn-workstream` | Persistent conversation contexts with JSONL history |
| `arawn-domain` | Domain facade orchestrating agent, session, memory, and MCP |

### Layer 3: Transport

HTTP/WebSocket server and client layers.

| Crate | Purpose |
|-------|---------|
| `arawn-server` | Axum-based HTTP server, REST API, WebSocket, OpenAPI |
| `arawn-client` | HTTP/WebSocket client for connecting to Arawn servers |

### Layer 4: Binaries

The main executable and TUI.

| Crate | Purpose |
|-------|---------|
| `arawn` | CLI binary, commands, REPL |
| `arawn-tui` | Terminal UI using Ratatui/Crossterm |

## Design Rules

1. **No upward dependencies** — Lower layers never depend on higher layers
2. **Foundation has no internal deps** — Layer 0 crates only use external crates
3. **Services are independent** — Layer 1 crates don't depend on each other
4. **Types are shared** — `arawn-types` provides common data structures
5. **Config is pervasive** — `arawn-config` used across all layers

## External Dependencies

Key external crates by layer:

| Layer | Key Dependencies |
|-------|------------------|
| Foundation | `serde`, `tokio`, `keyring` |
| Services | `reqwest`, `rusqlite`, `sqlite-vec` |
| Business | `async-trait`, `tracing` |
| Transport | `axum`, `tower`, `tokio-tungstenite` |
| Binary | `clap`, `ratatui` |
