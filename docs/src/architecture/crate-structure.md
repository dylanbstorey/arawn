# Crate Structure

Arawn is organized as a layered Rust workspace with clear dependencies.

## Dependency Graph

```
Layer 4  ┌─────────────────────────────────┐
(binary) │ arawn                            │
         └──┬──┬──┬──┬──┬──┬──────────────┘
            │  │  │  │  │  │
Layer 3     │  │  │  │  │  ▼
(transport) │  │  │  │  │  ┌──────────────┐
            │  │  │  │  │  │ arawn-server │
            │  │  │  │  │  └──┬─────┬─────┘
            │  │  │  │  │     │     │
Layer 2     │  │  │  │  ▼     ▼     ▼
(business)  │  │  │  │  ┌─────────┐ ┌──────────────┐
            │  │  │  │  │ arawn-  │ │ arawn-       │
            │  │  │  │  │ agent   │ │ workstream   │
            │  │  │  │  └┬─┬─┬─┬─┘ └──────┬───────┘
            │  │  │  │   │ │ │ │           │
Layer 1     │  │  ▼  ▼   ▼ │ ▼ ▼           ▼
(services)  │  │  ┌──────────┐ ┌──────────┐ ┌──────────┐
            │  │  │ arawn-   │ │ arawn-   │ │ arawn-   │
            │  │  │ memory   │ │ pipeline │ │ llm      │
            │  │  └────┬─────┘ └────┬─────┘ └────┬─────┘
            │  │       │            │             │
Layer 0     ▼  ▼       ▼            ▼             ▼
(foundation)┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐
            │ arawn-   │ │ arawn-   │ │ arawn-   │ │ arawn-   │
            │ oauth    │ │ types    │ │ config   │ │ script-  │
            └──────────┘ └──────────┘ └──────────┘ │ sdk      │
                                                   └──────────┘
```

## Layer Descriptions

### Layer 0: Foundation

Core types and utilities with no internal dependencies.

| Crate | Purpose |
|-------|---------|
| `arawn-types` | Shared types: Message, Session, Memory, etc. |
| `arawn-config` | Configuration loading, TOML parsing, secret resolution |
| `arawn-oauth` | OAuth PKCE flow for Claude MAX authentication |
| `arawn-script-sdk` | SDK for WASM script execution |

### Layer 1: Services

Domain-specific functionality depending only on foundation.

| Crate | Purpose |
|-------|---------|
| `arawn-llm` | LLM backends (Anthropic, OpenAI, Groq, Ollama), embeddings |
| `arawn-memory` | Memory store with vector search, graph, confidence scoring |
| `arawn-pipeline` | Workflow execution via Cloacina engine |

### Layer 2: Business Logic

Core agent functionality combining services.

| Crate | Purpose |
|-------|---------|
| `arawn-agent` | Agentic loop, tools, context building, session indexing |
| `arawn-workstream` | Persistent conversation contexts |

### Layer 3: Transport

HTTP/WebSocket server layer.

| Crate | Purpose |
|-------|---------|
| `arawn-server` | Axum-based HTTP server, REST API, WebSocket |

### Layer 4: Binary

The main executable.

| Crate | Purpose |
|-------|---------|
| `arawn` | CLI binary, commands, REPL |

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
