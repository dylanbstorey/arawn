---
id: 001-multi-crate-workspace-architecture
level: adr
title: "Multi-Crate Workspace Architecture"
number: 1
short_code: "ARAWN-A-0005"
created_at: 2026-03-05T05:16:01.932054+00:00
updated_at: 2026-03-05T05:19:00.176636+00:00
decision_date: 
decision_maker: 
parent: 
archived: false

tags:
  - "#adr"
  - "#phase/decided"


exit_criteria_met: false
initiative_id: NULL
---

# ADR-5: Multi-Crate Workspace Architecture

## Context

Arawn is a complex system with multiple subsystems — LLM communication, agent orchestration, tool execution, configuration management, session caching, memory storage, plugin loading, workstream management, sandboxing, TUI, HTTP server, and CLI. These subsystems have very different dependency profiles:

- The LLM client needs HTTP and provider-specific SDKs
- The TUI needs crossterm and ratatui
- The server needs axum and tower
- Memory storage needs SQLite and vector search
- Sandboxing needs platform-specific OS APIs

Putting all of this in a single crate would create a monolith where every consumer pulls in every dependency. Compile times would be poor, and the API surface would be unwieldy.

## Decision

Organize the project as a **Cargo workspace with 18 crates** arranged in a strict 5-layer dependency hierarchy. Dependencies flow strictly downward — higher layers depend on lower layers, never vice versa.

### Layer Architecture

```
Layer 0 (Foundation):    arawn-types
Layer 1 (Infrastructure): arawn-config, arawn-llm, arawn-memory, arawn-session,
                          arawn-mcp, arawn-sandbox, arawn-script-sdk, arawn-oauth
Layer 2 (Integration):   arawn-agent, arawn-workstream, arawn-pipeline, arawn-plugin
Layer 3 (Orchestration): arawn-domain
Layer 4 (Interface):     arawn-server, arawn-tui, arawn-client, arawn (CLI binary)
```

### Key Design Rules

1. **`arawn-types` is the universal foundation** — shared types, traits, and error definitions imported by all other crates. It has zero internal dependencies.

2. **Infrastructure crates are single-purpose** — each owns one concern (config, LLM, memory, sessions, etc.) and depends only on `arawn-types` plus external crates.

3. **Integration crates combine infrastructure** — `arawn-agent` depends on `arawn-llm`, `arawn-memory`, `arawn-mcp`, etc. to implement the agent loop.

4. **`arawn-domain` is the facade** — single orchestration point that wires together all services. Transport layers (server, CLI) depend only on this.

5. **Interface crates are independent** — `arawn-server`, `arawn-tui`, and `arawn-client` don't depend on each other. The TUI connects to the server via `arawn-client`, not by importing server internals.

6. **No circular dependencies** — strictly enforced by the layer hierarchy.

### Crate Inventory

| Crate | Layer | Purpose |
|-------|-------|---------|
| `arawn-types` | 0 | Shared traits, types, errors |
| `arawn-config` | 1 | TOML config, secret store, provider resolution |
| `arawn-llm` | 1 | LLM client abstraction (Anthropic, OpenAI, Groq, Ollama) |
| `arawn-memory` | 1 | SQLite-based memory with vector search |
| `arawn-session` | 1 | LRU session cache with TTL |
| `arawn-mcp` | 1 | Model Context Protocol client |
| `arawn-sandbox` | 1 | OS-level command sandboxing |
| `arawn-oauth` | 1 | OAuth 2.0 PKCE authentication |
| `arawn-script-sdk` | 1 | SDK for WASM script plugins |
| `arawn-agent` | 2 | Agent loop, tool registry, prompt building |
| `arawn-workstream` | 2 | Persistent conversation contexts |
| `arawn-pipeline` | 2 | Workflow orchestration engine |
| `arawn-plugin` | 2 | Plugin system (skills, hooks, agents) |
| `arawn-domain` | 3 | Domain facade orchestrating all services |
| `arawn-server` | 4 | HTTP API and WebSocket server |
| `arawn-tui` | 4 | Terminal UI client |
| `arawn-client` | 4 | HTTP client SDK |
| `arawn` | 4 | CLI binary entry point |

## Alternatives Analysis

| Option | Pros | Cons | Risk Level |
|--------|------|------|------------|
| **Multi-crate workspace (chosen)** | Clear boundaries, independent compilation, enforced layering | More crates to manage, inter-crate API design overhead | Low |
| **Single crate with modules** | Simpler Cargo.toml, easier refactoring | All deps pulled everywhere, slow compilation, no enforced boundaries | Medium |
| **Feature-gated single crate** | One crate, optional deps | Feature flag combinatorial explosion, conditional compilation complexity | High |
| **Separate repositories** | Maximum independence | Versioning nightmare, cross-repo changes painful, no shared workspace | High |

## Rationale

- **Enforced boundaries**: Cargo's dependency system makes circular dependencies a compile error, not a code review comment. The layer hierarchy is structurally guaranteed.
- **Incremental compilation**: Changing `arawn-tui` doesn't recompile `arawn-agent`. During development, only the affected crate and its dependents rebuild.
- **Dependency isolation**: The TUI doesn't pull in SQLite. The server doesn't pull in crossterm. Each binary/library gets only what it needs.
- **Independent testability**: Each crate has its own test suite with focused mocks. `arawn-session` tests don't need an LLM backend.
- **API clarity**: Crate boundaries force explicit public APIs via `pub use` re-exports. Internal implementation details stay private.
- **Server/client symmetry**: `arawn-client` has zero internal dependencies — it's a standalone HTTP SDK that external consumers can use without pulling in the entire Arawn dependency tree.

## Consequences

### Positive
- Compile times scale with change scope, not project size
- Each crate can be reasoned about independently
- Clear ownership — "where does X live?" has one answer
- External consumers can depend on specific crates (e.g., `arawn-client` only)
- Dependency graph is auditable and acyclic

### Negative
- Inter-crate API changes require coordinated updates across multiple Cargo.toml files
- Traits must live in `arawn-types` to avoid circular dependencies, even when they're only implemented in one place
- New contributors must understand the layer hierarchy before adding dependencies
- 18 crates means 18 Cargo.toml files to maintain

### Neutral
- Vendored crates (`orp-vendored`, `gline-rs-vendored`) are excluded from the workspace to avoid polluting the dependency graph
- The workspace uses a shared `[workspace.dependencies]` table to keep versions consistent