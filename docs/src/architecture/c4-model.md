# C4 Model

Arawn's architecture documented using the C4 model (Context, Containers, Components, Code).

## Level 1: System Context

Shows Arawn and its relationships with users and external systems.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              External Systems                               │
│                                                                             │
│   ┌───────────┐  ┌───────────┐  ┌───────────┐  ┌───────────┐                │
│   │ Anthropic │  │  OpenAI   │  │   Groq    │  │  Ollama   │                │
│   │   Cloud   │  │   Cloud   │  │   Cloud   │  │  (local)  │                │
│   └─────┬─────┘  └─────┬─────┘  └─────┬─────┘  └─────┬─────┘                │
│         └──────────────┴──────────────┴──────────────┘                      │
│                                   │                                         │
│                    HTTPS (OpenAI-compatible API)                            │
│                                   │                                         │
│   ┌───────────────────────────────┼───────────────────────────────────┐     │
│   │                               │                                   │     │
│   │  ┌───────────┐  ┌─────────────┴─────────────┐  ┌───────────────┐  │     │
│   │  │   MCP     │  │    stdio / SSE / HTTP     │  │   External    │  │     │
│   │  │  Servers  │◀─┤                           ├─▶│     APIs      │  │     │ 
│   │  │(file,git, │  │                           │  │  (web search, │  │     │
│   │  │ browser)  │  │                           │  │   calendar)   │  │     │
│   │  └───────────┘  └───────────────────────────┘  └───────────────┘  │     │
│   │                                                                   │     │
│   └───────────────────────────────────────────────────────────────────┘     │
│                                   │                                         │
└───────────────────────────────────┼─────────────────────────────────────────┘
                                    │
                ┌───────────────────┼───────────────────┐
                │                   │                   │
                ▼                   ▼                   ▼
        ┌─────────────┐    ┌─────────────────┐    ┌─────────────┐
        │  Developer  │    │   Arawn Agent   │    │  HTTP/WS    │
        │    (CLI)    │───▶│    Platform     │◀───│   Clients   │
        │             │    │                 │    │   (TUI)     │
        └─────────────┘    └────────┬────────┘    └─────────────┘
                                    │
                    ┌───────────────┼───────────────┐
                    ▼               ▼               ▼
              ┌──────────┐   ┌──────────┐   ┌──────────┐
              │ SQLite   │   │  JSONL   │   │   ONNX   │
              │ Databases│   │  Logs    │   │ Runtime  │
              │(memory,  │   │(session  │   │(embeddings│
              │ cache)   │   │ history) │   │  NER)    │
              └──────────┘   └──────────┘   └──────────┘
```

### Context Elements

| Element | Type | Description |
|---------|------|-------------|
| **Developer** | Person | Interacts via CLI (`arawn`) commands |
| **HTTP/WS Clients** | Person | External applications using REST API or WebSocket |
| **Arawn Platform** | System | AI agent platform with persistent memory |
| **LLM Providers** | External | Anthropic, OpenAI, Groq (cloud) or Ollama (local) |
| **MCP Servers** | External | Model Context Protocol servers for tool integration |
| **SQLite** | Database | Operational state, memory storage, session cache |
| **JSONL** | Storage | Append-only session history and workstream logs |
| **ONNX Runtime** | Runtime | Local embeddings and NER models |

---

## Level 2: Container Diagram

Shows the high-level containers (crates) within Arawn.

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│ Arawn Platform                                                                  │
│                                                                                 │
│  ┌────────────────────────────────────────────────────────────────────────────┐ │
│  │                           User Interfaces                                  │ │
│  │                                                                            │ │
│  │  ┌─────────────┐     ┌─────────────┐     ┌─────────────┐                   │ │
│  │  │  CLI Binary │     │     TUI     │     │ API Client  │                   │ │
│  │  │   (arawn)   │     │ (arawn-tui) │     │(arawn-client│                   │ │
│  │  │             │     │             │     │             │                   │ │
│  │  │ start,chat, │     │ Real-time   │     │ Typed HTTP  │                   │ │
│  │  │ config,ask  │     │ streaming   │     │ SDK         │                   │ │
│  │  └──────┬──────┘     └──────┬──────┘     └──────┬──────┘                   │ │
│  │         └────────────────────┴────────────────────┘                        │ │
│  └──────────────────────────────┬─────────────────────────────────────────────┘ │
│                                 │                                               │
│                                 ▼                                               │
│  ┌────────────────────────────────────────────────────────────────────────────┐ │
│  │                         HTTP/WS Server (arawn-server)                      │ │
│  │                                                                            │ │
│  │  ┌─────────────────────────────────────────────────────────────────────┐   │ │
│  │  │ Routes: /health, /api/v1/chat, /sessions, /memory, /workstreams, /ws│   │ │
│  │  └─────────────────────────────────────────────────────────────────────┘   │ │
│  │  ┌─────────────────────────────────────────────────────────────────────┐   │ │
│  │  │ Middleware: Auth → Rate Limit → Logging → Validation                │   │ │
│  │  └─────────────────────────────────────────────────────────────────────┘   │ │
│  │  ┌─────────────────────────────────────────────────────────────────────┐   │ │
│  │  │ WebSocket: Protocol → Connection → Handlers (streaming responses)   │   │ │
│  │  └─────────────────────────────────────────────────────────────────────┘   │ │
│  └──────────────────────────────┬─────────────────────────────────────────────┘ │
│                                 │                                               │
│          ┌──────────────────────┼──────────────────────┐                        │
│          ▼                      ▼                      ▼                        │
│  ┌───────────────┐    ┌─────────────────┐    ┌─────────────────┐                │
│  │ Session Cache │    │      Agent      │    │   Workstream    │                │
│  │(arawn-session)│◀──▶│  (arawn-agent)  │◀──▶│   Manager       │                │
│  │               │    │                 │    │(arawn-workstream│                │
│  │ LRU + TTL     │    │ Agentic loop,   │    │                 │                │
│  │ eviction      │    │ tool execution  │    │ JSONL + SQLite  │                │
│  └───────────────┘    └────────┬────────┘    └─────────────────┘                │
│                                │                                                │
│          ┌─────────────────────┼─────────────────────┐                          │
│          ▼                     ▼                     ▼                          │
│  ┌───────────────┐    ┌───────────────┐    ┌───────────────┐                    │
│  │  LLM Backends │    │    Memory     │    │  MCP Client   │                    │
│  │  (arawn-llm)  │    │(arawn-memory) │    │ (arawn-mcp)   │                    │
│  │               │    │               │    │               │                    │
│  │ Anthropic,    │    │ SQLite-vec,   │    │ stdio/SSE/    │                    │
│  │ OpenAI,Groq,  │    │ graphqlite,   │    │HTTP transports│                    │
│  │ Ollama        │    │ fact storage  │    │               │                    │
│  └───────────────┘    └───────────────┘    └───────────────┘                    │
│                                                                                 │
│  ┌────────────────────────────────────────────────────────────────────────────┐ │
│  │                          Supporting Crates                                 │ │
│  │                                                                            │ │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │ │
│  │  │   Config    │  │   Plugin    │  │   Types     │  │  Pipeline   │        │ │
│  │  │(arawn-config│  │(arawn-plugin│  │(arawn-types)│  │(arawn-      │        │ │
│  │  │             │  │             │  │             │  │ pipeline)   │        │ │
│  │  │ TOML,keyring│  │ Skills,hooks│  │ Shared DTOs │  │ Workflow    │        │ │
│  │  │ secrets     │  │ agents,tools│  │ and traits  │  │ execution   │        │ │
│  │  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘        │ │
│  └────────────────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### Container Responsibilities

| Container | Crate | Responsibility |
|-----------|-------|----------------|
| **CLI Binary** | `arawn` | User commands, REPL, server startup, configuration |
| **TUI** | `arawn-tui` | Terminal interface with real-time streaming, workstream/session management |
| **API Client** | `arawn-client` | Typed HTTP SDK for REST API integration |
| **HTTP Server** | `arawn-server` | REST API, WebSocket, middleware stack, rate limiting |
| **Agent** | `arawn-agent` | Agentic loop, tool execution, context building, streaming |
| **Session Cache** | `arawn-session` | LRU cache with TTL eviction, persistence hooks |
| **Workstream** | `arawn-workstream` | Persistent contexts, JSONL history, SQLite state |
| **LLM Backends** | `arawn-llm` | Provider abstraction (Anthropic, OpenAI, Groq, Ollama) |
| **Memory** | `arawn-memory` | Vector search (sqlite-vec), knowledge graph, fact storage |
| **MCP Client** | `arawn-mcp` | Model Context Protocol integration for external tools |
| **Config** | `arawn-config` | TOML parsing, keyring secrets, environment resolution |
| **Plugin** | `arawn-plugin` | Skills, hooks, agents, CLI tools, prompt fragments |
| **Types** | `arawn-types` | Shared DTOs and traits across crates |
| **Pipeline** | `arawn-pipeline` | Workflow execution via Cloacina WASM runtime |
| **Domain** | `arawn-domain` | Domain facade orchestrating agent, session, memory, MCP |
| **OAuth** | `arawn-oauth` | OAuth PKCE flow for Claude MAX authentication |
| **Sandbox** | `arawn-sandbox` | OS-level sandboxing (macOS sandbox-exec, Linux bubblewrap) |
| **Script SDK** | `arawn-script-sdk` | Pre-compiled SDK for WASM script execution |

---

## Level 3: Component Diagrams

### 3.1 Agent Core (arawn-agent)

```
┌────────────────────────────────────────────────────────────────────────┐
│ arawn-agent                                                            │
│                                                                        │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                        Public API                               │   │
│  │                                                                 │   │
│  │  Agent::new()          Session handling                         │   │
│  │  Agent::turn()         Single synchronous turn                  │   │
│  │  Agent::turn_stream()  Streaming turn with tool events          │   │
│  └──────────────────────────────┬──────────────────────────────────┘   │
│                                 │                                      │
│  ┌──────────────────────────────┼──────────────────────────────────┐   │
│  │                              ▼                                  │   │
│  │  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐          │   │
│  │  │   Agentic   │───▶│   Context   │───▶│    Tool     │          │   │
│  │  │    Loop     │    │   Builder   │    │  Executor   │          │   │
│  │  │             │    │             │    │             │          │   │
│  │  │ Max turns,  │    │ System msg, │    │ Shell,file, │          │   │
│  │  │ stop conds  │    │ memory,MCP  │    │ memory,web  │          │   │
│  │  └─────────────┘    └─────────────┘    └──────┬──────┘          │   │
│  │                                               │                 │   │
│  │                           ┌───────────────────┤                 │   │
│  │                           ▼                   ▼                 │   │
│  │                   ┌─────────────┐    ┌─────────────┐            │   │
│  │                   │   Output    │    │   Stream    │            │   │
│  │                   │  Processor  │    │   Chunks    │            │   │
│  │                   │             │    │             │            │   │
│  │                   │ Truncation, │    │ Text,Tool   │            │   │
│  │                   │ validation  │    │ Start/End   │            │   │
│  │                   └─────────────┘    └─────────────┘            │   │
│  │                                                                 │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                        │
│  External Dependencies:                                                │
│    → arawn-llm (LLM inference)                                         │
│    → arawn-memory (fact storage, recall)                               │
│    → arawn-mcp (MCP tool discovery)                                    │
│    → arawn-plugin (custom tools)                                       │
└────────────────────────────────────────────────────────────────────────┘
```

### 3.2 HTTP Server (arawn-server)

```
┌─────────────────────────────────────────────────────────────────────────┐
│ arawn-server                                                            │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                       Route Modules                             │    │
│  │                                                                 │    │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌──────────┐ ┌─────────┐   │    │
│  │  │  chat   │ │sessions │ │ memory  │ │workstream│ │  mcp    │   │    │
│  │  │         │ │         │ │         │ │          │ │         │   │    │
│  │  │POST/chat│ │CRUD     │ │search,  │ │CRUD,     │ │servers, │   │    │
│  │  │streaming│ │list,get │ │notes    │ │messages  │ │tools    │   │    │
│  │  └─────────┘ └─────────┘ └─────────┘ └──────────┘ └─────────┘   │    │
│  │                                                                 │    │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐                │    │
│  │  │ agents  │ │  tasks  │ │ config  │ │ health  │                │    │
│  │  │         │ │         │ │         │ │         │                │    │
│  │  │list,get │ │list,    │ │get      │ │ready,   │                │    │
│  │  │         │ │cancel   │ │config   │ │live     │                │    │
│  │  └─────────┘ └─────────┘ └─────────┘ └─────────┘                │    │
│  └─────────────────────────────────────────────────────────────────┘    │ 
│                                                                         │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │                     WebSocket Module (ws/)                       │   │
│  │                                                                  │   │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐                  │   │
│  │  │  protocol  │  │ connection │  │  handlers  │                  │   │
│  │  │            │  │            │  │            │                  │   │
│  │  │ Client/    │  │ State,idle │  │ auth,chat, │                  │   │
│  │  │ Server Msg │  │ timeout    │  │ subscribe  │                  │   │
│  │  └────────────┘  └────────────┘  └────────────┘                  │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                      Infrastructure                             │    │
│  │                                                                 │    │
│  │  ┌───────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐        │    │
│  │  │  error    │  │   auth   │  │  state   │  │ ratelimit│        │    │
│  │  │           │  │          │  │          │  │          │        │    │
│  │  │ServerError│  │Identity, │  │AppState, │  │ Governor │        │    │
│  │  │From impls │  │middleware│  │session   │  │per-IP    │        │    │
│  │  └───────────┘  └──────────┘  └──────────┘  └──────────┘        │    │
│  └─────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────┘
```

### 3.3 TUI (arawn-tui)

```
┌─────────────────────────────────────────────────────────────────────────┐
│ arawn-tui                                                               │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                        Application Core                         │    │
│  │                                                                 │    │
│  │  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐          │    │
│  │  │     App     │───▶│   Focus     │───▶│   Events    │          │    │
│  │  │             │    │  Manager    │    │   Handler   │          │    │
│  │  │ State,      │    │             │    │             │          │    │
│  │  │ lifecycle   │    │ Overlays,   │    │ Key,tick,   │          │    │
│  │  │             │    │ cycling     │    │ resize      │          │    │
│  │  └─────────────┘    └─────────────┘    └─────────────┘          │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                         UI Components                           │    │
│  │                                                                 │    │
│  │  ┌─────────┐ ┌─────────┐ ┌──────────┐ ┌─────────┐ ┌─────────┐   │    │
│  │  │  chat   │ │  input  │ │ sidebar  │ │ sessions│ │ palette │   │    │
│  │  │         │ │         │ │          │ │         │ │         │   │    │
│  │  │Messages,│ │Multi-   │ │Workstream│ │Session  │ │Command  │   │    │
│  │  │streaming│ │line,    │ │tree,     │ │list,    │ │search,  │   │    │
│  │  │         │ │history  │ │filter    │ │switch   │ │actions  │   │    │
│  │  └─────────┘ └─────────┘ └──────────┘ └─────────┘ └─────────┘   │    │
│  │                                                                 │    │
│  │  ┌─────────┐ ┌─────────┐                                        │    │
│  │  │  logs   │ │ tools   │                                        │    │
│  │  │         │ │         │                                        │    │
│  │  │Debug    │ │Execution│                                        │    │
│  │  │output   │ │status   │                                        │    │
│  │  └─────────┘ └─────────┘                                        │    │ 
│  └─────────────────────────────────────────────────────────────────┘    │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                        Infrastructure                           │    │
│  │                                                                 │    │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌───────────┐        │    │
│  │  │ bounded  │  │  client  │  │ protocol │  │  focus    │        │    │
│  │  │          │  │          │  │          │  │           │        │    │
│  │  │BoundedVec│  │WebSocket │  │Client/   │  │FocusMgr,  │        │    │ 
│  │  │eviction  │  │+ HTTP    │  │Server Msg│  │FocusTarget│        │    │
│  │  └──────────┘  └──────────┘  └──────────┘  └───────────┘        │    │
│  └─────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Level 4: Data Flow Diagrams

### 4.1 Chat Message Flow

```
┌──────┐  message   ┌─────────┐  HTTP/WS  ┌──────────┐
│ User │───────────▶│   TUI   │──────────▶│  Server  │
└──────┘            │         │           │          │
                    └─────────┘           └────┬─────┘
                                               │
                         ┌─────────────────────┤
                         ▼                     ▼
                   ┌───────────┐       ┌───────────┐
                   │ Session   │       │ Workstream│
                   │  Cache    │       │  Manager  │
                   │           │       │           │
                   │ Get/Create│       │ Store user│
                   │ session   │       │ message   │
                   └─────┬─────┘       └───────────┘
                         │
                         ▼
                   ┌───────────┐
                   │   Agent   │
                   │           │
                   │ Build     │
                   │ context   │
                   └─────┬─────┘
                         │
              ┌──────────┼──────────┐
              ▼          ▼          ▼
        ┌─────────┐ ┌─────────┐ ┌─────────┐
        │ Memory  │ │   MCP   │ │  LLM    │
        │ Recall  │ │  Tools  │ │ Backend │
        └─────────┘ └─────────┘ └────┬────┘
                                     │
                         ┌───────────┴───────────┐
                         ▼                       ▼
                   ┌───────────┐           ┌───────────┐
                   │ Stream    │           │  Tool     │
                   │ Chunks    │           │ Execution │
                   │           │           │           │
                   │ Text,Done │           │ Shell,file│
                   └─────┬─────┘           └─────┬─────┘
                         │                       │
                         └───────────┬───────────┘
                                     ▼
                               ┌───────────┐
                               │ Workstream│
                               │  Manager  │
                               │           │
                               │ Store turn│
                               │ (JSONL)   │
                               └─────┬─────┘
                                     │
                                     ▼
                               ┌───────────┐
                               │   TUI     │
                               │           │
                               │ Render    │
                               │ response  │
                               └───────────┘
```

### 4.2 Session Lifecycle

```
                    ┌──────────────────────────────────────────┐
                    │           Session Lifecycle              │
                    └──────────────────────────────────────────┘

  New Connection                                      Reconnection
       │                                                   │
       ▼                                                   ▼
┌─────────────┐                                    ┌─────────────┐
│ Subscribe   │                                    │   Lookup    │
│ (WebSocket) │                                    │  by UUID    │
└──────┬──────┘                                    └──────┬──────┘
       │                                                  │
       └──────────────────────┬───────────────────────────┘
                              ▼
                    ┌─────────────────┐
                    │  Session Cache  │
                    │  (arawn-session)│
                    │                 │
                    │  LRU + TTL      │
                    │  Max 10K items  │
                    │  30min TTL      │
                    └────────┬────────┘
                             │
              ┌──────────────┼──────────────┐
              │              │              │
              ▼              ▼              ▼
        ┌──────────┐  ┌──────────┐  ┌──────────┐
        │ Hit:     │  │ Miss:    │  │ Evict:   │
        │ Return   │  │ Load from│  │ Persist  │
        │ cached   │  │workstream│  │ to       │
        │          │  │          │  │workstream│
        └──────────┘  └──────────┘  └──────────┘
                             │
                             ▼
                    ┌─────────────────┐
                    │   Workstream    │
                    │    Manager      │
                    │                 │
                    │ sessions.db     │
                    │ *.jsonl history │
                    └─────────────────┘
```

---

## Crate Dependency Graph

```
                              ┌─────────────┐
                              │    arawn    │ (CLI binary)
                              └──────┬──────┘
                                     │
              ┌──────────────────────┼──────────────────────┐
              │                      │                      │
              ▼                      ▼                      ▼
      ┌─────────────┐        ┌─────────────┐        ┌─────────────┐
      │arawn-server │        │  arawn-tui  │        │arawn-config │
      └──────┬──────┘        └──────┬──────┘        └─────────────┘
             │                      │
             │               ┌──────┴──────┐
             │               │             │
             │               ▼             │
             │        ┌─────────────┐      │
             │        │arawn-client │      │
             │        └─────────────┘      │
             │                             │
             └──────────────┬──────────────┘
                            │
              ┌─────────────┼─────────────┐
              │             │             │
              ▼             ▼             ▼
      ┌─────────────┐ ┌───────────┐ ┌───────────┐
      │arawn-session│ │arawn-agent│ │arawn-     │
      └─────────────┘ └─────┬─────┘ │workstream │
                            │       └───────────┘
              ┌─────────────┼─────────────┐
              │             │             │
              ▼             ▼             ▼
      ┌─────────────┐ ┌───────────┐ ┌───────────┐
      │  arawn-llm  │ │arawn-mcp  │ │arawn-memory│
      └─────────────┘ └───────────┘ └───────────┘
              │             │             │
              └─────────────┼─────────────┘
                            │
                            ▼
                    ┌─────────────┐
                    │arawn-types  │
                    └─────────────┘
```

---

## Key Architectural Decisions

| Decision | Rationale |
|----------|-----------|
| **SQLite for state** | Single-file, embedded, no external dependencies |
| **JSONL for history** | Append-only, human-readable, easy backup |
| **LRU+TTL session cache** | Bounded memory, automatic cleanup |
| **Streaming responses** | Real-time feedback, better UX |
| **MCP protocol** | Standard tool integration, ecosystem compatibility |
| **Modular crates** | Clear boundaries, independent testing |
| **WebSocket + REST** | Real-time streaming + standard API access |
| **BoundedVec collections** | Prevent unbounded memory growth |
| **Focus manager pattern** | Clean UI state management |
