# Architecture Overview

Arawn is built as a modular Rust workspace with clear separation of concerns.

## Design Principles

- **Edge-First** — Single binary deployment with embedded storage
- **Memory-Centric** — Persistent context via vector + graph database
- **Tool-Native** — LLM as orchestrator, tools as capabilities
- **Extensible** — Plugin system for customization

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         External Systems                         │
│                                                                  │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌───────────────┐   │
│  │  Groq    │  │  OpenAI  │  │ Anthropic│  │ Ollama (local)│   │
│  │  Cloud   │  │  Cloud   │  │  Cloud   │  │               │   │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └──────┬────────┘   │
│       └──────────────┴──────────────┴───────────────┘            │
│                              │                                   │
└──────────────────────────────┼───────────────────────────────────┘
                               │ HTTPS / OpenAI-compat API
                               ▼
                    ┌─────────────────────┐
                    │                     │
      CLI ────────▶ │    Arawn Agent      │ ◀──── HTTP/WS Clients
      (arawn)       │    Platform         │
                    │                     │
                    └─────────┬───────────┘
                              │
              ┌───────────────┼───────────────┐
              ▼               ▼               ▼
        ┌──────────┐   ┌──────────┐   ┌──────────┐
        │ SQLite   │   │ SQLite   │   │ ONNX     │
        │ memory.db│   │ graph.db │   │ Runtime  │
        │ (facts,  │   │ (knowledge│  │ (local   │
        │ sessions)│   │  graph)  │   │ embeddings│
        └──────────┘   └──────────┘   │ local NER)│
                                      └──────────┘
```

## Key Interactions

| Actor | Description |
|-------|-------------|
| **Users** | Developers interacting via CLI (`arawn chat`, `arawn ask`) or HTTP API |
| **Arawn** | Agentic platform using LLMs with tool-calling for research tasks |
| **LLM Providers** | Cloud APIs (Groq, OpenAI, Anthropic) or local models (Ollama) |
| **Local Storage** | SQLite for memories/sessions, graphqlite for knowledge graph |

## Section Contents

- [C4 Model](c4-model.md) — System context and container diagrams
- [Components](components.md) — Agent, Memory, LLM component details
- [Crate Structure](crate-structure.md) — Dependency graph and layers
- [Data Flow](data-flow.md) — Sequence diagrams for key operations
