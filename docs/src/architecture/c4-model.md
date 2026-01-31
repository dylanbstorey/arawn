# C4 Model

Arawn's architecture documented using C4 model diagrams.

## Level 1: System Context

```
┌──────────────────────────────────────────────────────────────────┐
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

### Context Elements

| Element | Description |
|---------|-------------|
| **Users** | Developers interacting via CLI or HTTP API |
| **Arawn** | Agentic platform with persistent memory |
| **External LLM Providers** | Cloud APIs or local Ollama for inference |
| **Local Storage** | SQLite + graphqlite + ONNX Runtime |

## Level 2: Container Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│ Arawn Platform                                                          │
│                                                                         │
│  ┌─────────────┐     ┌──────────────────────────────────────────────┐  │
│  │  CLI Binary  │     │  HTTP/WS Server (arawn-server)              │  │
│  │  (arawn)     │────▶│                                              │  │
│  │              │     │  Routes: /health, /api/v1/chat,             │  │
│  │  Commands:   │     │    /sessions, /memory, /notes,              │  │
│  │  start, chat │     │    /workstreams, /ws                        │  │
│  │  ask, config │     │                                              │  │
│  │  memory      │     │  Middleware: Auth → Rate Limit → Logging    │  │
│  └─────────────┘     └──────────────┬───────────────────────────────┘  │
│                                      │                                  │
│                                      ▼                                  │
│                      ┌───────────────────────────────┐                  │
│                      │  Agent (arawn-agent)           │                  │
│                      │                                │                  │
│                      │  ┌──────────┐ ┌────────────┐  │                  │
│                      │  │ Agentic  │ │  Tool      │  │                  │
│                      │  │ Loop     │ │  Registry  │  │                  │
│                      │  └────┬─────┘ └──────┬─────┘  │                  │
│                      │       │              │        │                  │
│                      │  ┌────┴─────┐ ┌──────┴─────┐  │                  │
│                      │  │ Context  │ │ Session    │  │                  │
│                      │  │ Builder  │ │ Indexer    │  │                  │
│                      │  └──────────┘ └────────────┘  │                  │
│                      └───────┬──────────┬────────────┘                  │
│                              │          │                               │
│              ┌───────────────┤          ├───────────────┐               │
│              ▼               ▼          ▼               ▼               │
│  ┌──────────────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────┐      │
│  │ LLM Backends     │ │ Memory   │ │ Pipeline │ │ Workstream   │      │
│  │ (arawn-llm)      │ │ Store    │ │ Engine   │ │ Manager      │      │
│  │                   │ │(arawn-   │ │(arawn-   │ │(arawn-       │      │
│  │ Anthropic,OpenAI, │ │ memory)  │ │ pipeline)│ │ workstream)  │      │
│  │ Groq,Ollama       │ │          │ │          │ │              │      │
│  │                   │ │ SQLite + │ │ Cloacina │ │ JSONL +      │      │
│  │ Embeddings:       │ │ Vec0 +   │ │ + WASM   │ │ SQLite cache │      │
│  │ Local ONNX,OpenAI │ │ Graph    │ │ sandbox  │ │              │      │
│  └──────────────────┘ └──────────┘ └──────────┘ └──────────────┘      │
│                                                                         │
│  ┌──────────────────┐ ┌──────────────────┐                              │
│  │ Config            │ │ OAuth Proxy      │                              │
│  │ (arawn-config)    │ │ (arawn-oauth)    │                              │
│  │                   │ │                  │                              │
│  │ TOML, keyring,    │ │ PKCE flow for    │                              │
│  │ secret resolution │ │ Claude MAX       │                              │
│  └──────────────────┘ └──────────────────┘                              │
└─────────────────────────────────────────────────────────────────────────┘
```

### Container Responsibilities

| Container | Crate | Responsibility |
|-----------|-------|----------------|
| CLI Binary | `arawn` | User commands, REPL, server startup |
| HTTP Server | `arawn-server` | REST API, WebSocket, middleware |
| Agent | `arawn-agent` | Agentic loop, tools, context building |
| LLM Backends | `arawn-llm` | Provider abstraction, streaming |
| Memory Store | `arawn-memory` | Vector search, graph, fact storage |
| Pipeline Engine | `arawn-pipeline` | Workflow execution via Cloacina |
| Workstream Manager | `arawn-workstream` | Persistent contexts |
| Config | `arawn-config` | TOML parsing, secret resolution |
| OAuth Proxy | `arawn-oauth` | PKCE authentication flow |
