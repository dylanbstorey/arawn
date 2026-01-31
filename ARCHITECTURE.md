# Arawn Architecture

Personal Research Agent for Edge Computing — written in Rust.

---

## C4 Model

### Level 1: System Context

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

**Users**: Developers interacting via CLI (`arawn chat`, `arawn ask`) or HTTP API.

**Arawn**: An agentic platform that uses LLMs with tool-calling to perform research tasks, retaining knowledge across sessions via a persistent memory system.

**External LLM Providers**: Arawn calls cloud LLM APIs (Groq, OpenAI, Anthropic) or local models (Ollama) for inference. API keys stored in OS keychain.

**Local Storage**: SQLite for memories/sessions, graphqlite for knowledge graph, ONNX Runtime for local embeddings and optional NER.

---

### Level 2: Container Diagram

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

---

### Level 3: Component Diagram — Agent

```
┌─────────────────────────────────────────────────────────────────┐
│ arawn-agent                                                      │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ Agent                                                     │   │
│  │                                                           │   │
│  │  backend: SharedBackend        config: AgentConfig        │   │
│  │  tools: Arc<ToolRegistry>      memory_store: Option       │   │
│  │  embedder: Option<SharedEmbedder>                         │   │
│  │  recall_config: RecallConfig                              │   │
│  │                                                           │   │
│  │  turn(session, message) ─────────────────────────────┐    │   │
│  │  turn_stream(session, message) ──────────────────┐   │    │   │
│  └──────────────────────────────────────────────────┼───┼────┘   │
│                                                     │   │        │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┼───┼────┐  │
│  │ SystemPrompt │  │ Context      │  │ Tool Registry│   │    │  │
│  │ Builder      │  │ Builder      │  │              │   │    │  │
│  │              │  │              │  │ shell        │   │    │  │
│  │ bootstrap,   │  │ history,     │  │ file_read    │   │    │  │
│  │ tool docs,   │  │ recall,      │  │ file_write   │   │    │  │
│  │ workspace    │  │ notes        │  │ glob, grep   │   │    │  │
│  └──────────────┘  └──────────────┘  │ web_fetch    │   │    │  │
│                                      │ web_search   │   │    │  │
│  ┌───────────────────────────────┐   │ note, think  │   │    │  │
│  │ Session Indexer               │   │ workflow     │   │    │  │
│  │                               │   │ catalog      │   │    │  │
│  │ ┌───────────┐ ┌────────────┐  │   └──────────────┘   │    │  │
│  │ │ Extraction│ │Summarize   │  │                       │    │  │
│  │ │ (LLM/NER) │ │(LLM)      │  │                       │    │  │
│  │ └─────┬─────┘ └─────┬──────┘  │                       │    │  │
│  │       │             │         │                       │    │  │
│  │ ┌─────┴─────────────┴──────┐  │                       │    │  │
│  │ │ Store: entities, facts,  │  │                       │    │  │
│  │ │ relationships, summary   │  │                       │    │  │
│  │ └─────────────────────────┘  │                       │    │  │
│  └───────────────────────────────┘                       │    │  │
│                                                          │    │  │
│  ┌───────────────────────────────┐                       │    │  │
│  │ Streaming (AgentStream)       │◀──────────────────────┘    │  │
│  │ Text, ToolStart, ToolEnd,Done │                            │  │
│  └───────────────────────────────┘                            │  │
└───────────────────────────────────────────────────────────────┘
```

---

### Level 3: Component Diagram — Memory

```
┌──────────────────────────────────────────────────────────────┐
│ arawn-memory                                                  │
│                                                               │
│  ┌────────────────────────────────────────────────────────┐  │
│  │ MemoryStore                                             │  │
│  │                                                         │  │
│  │  conn: Mutex<Connection>   ── SQLite (memory.db)        │  │
│  │  graph: Option<GraphStore> ── graphqlite (graph.db)     │  │
│  │  vectors_initialized: bool ── sqlite-vec (vec0 table)   │  │
│  └────────┬──────────┬──────────┬──────────┬───────────────┘  │
│           │          │          │          │                   │
│  ┌────────┴───┐ ┌────┴─────┐ ┌─┴────────┐ ┌┴─────────────┐  │
│  │ Unified    │ │ Recall   │ │ Vector   │ │ Graph Ops    │  │
│  │ Ops        │ │          │ │ Ops      │ │              │  │
│  │            │ │ hybrid   │ │          │ │ add_entity   │  │
│  │ store()    │ │ search:  │ │ init()   │ │ add_relation │  │
│  │ store_fact │ │ vector + │ │ store()  │ │ get_neighbors│  │
│  │  └─detect  │ │ graph +  │ │ search() │ │ query()      │  │
│  │   contradict│ │ confidence│ │ reindex()│ │              │  │
│  │  └─reinforce│ │          │ │          │ │              │  │
│  │  └─supersede│ │          │ │          │ │              │  │
│  └────────────┘ └──────────┘ └──────────┘ └──────────────┘  │
│                                                               │
│  ┌──────────────────────────────────┐                         │
│  │ Confidence Scoring               │                         │
│  │                                  │                         │
│  │ score = base × reinforcement     │                         │
│  │                × staleness       │                         │
│  │                                  │                         │
│  │ base: stated=1.0, system=0.9,    │                         │
│  │       observed=0.7, inferred=0.5 │                         │
│  │ reinforcement: min(1+0.1n, 1.5)  │                         │
│  │ staleness: linear decay to 0.3   │                         │
│  │   over 365 days (fresh < 30d)    │                         │
│  │ superseded: always 0.0           │                         │
│  └──────────────────────────────────┘                         │
└──────────────────────────────────────────────────────────────┘
```

---

## Sequence Diagrams

### 1. Chat Request (Synchronous)

```
Client            Server           AppState          Agent             LLM Backend
  │                 │                 │                 │                 │
  │ POST /api/v1/chat                │                 │                 │
  │ {message, session_id?}           │                 │                 │
  │────────────────▶│                │                 │                 │
  │                 │                │                 │                 │
  │                 │ get_or_create_ │                 │                 │
  │                 │ session(id)    │                 │                 │
  │                 │───────────────▶│                 │                 │
  │                 │◀───────────────│                 │                 │
  │                 │ session_id     │                 │                 │
  │                 │                │                 │                 │
  │                 │ agent.turn(session, msg)         │                 │
  │                 │────────────────────────────────▶│                 │
  │                 │                │                 │                 │
  │                 │                │    ┌────────────┤                 │
  │                 │                │    │ Build      │                 │
  │                 │                │    │ context:   │                 │
  │                 │                │    │ system     │                 │
  │                 │                │    │ prompt +   │                 │
  │                 │                │    │ history +  │                 │
  │                 │                │    │ recall     │                 │
  │                 │                │    └────────────┤                 │
  │                 │                │                 │                 │
  │                 │                │                 │ TOOL LOOP       │
  │                 │                │                 │─────────────────│
  │                 │                │                 │                 │
  │                 │                │                 │ complete(req)   │
  │                 │                │                 │────────────────▶│
  │                 │                │                 │                 │
  │                 │                │                 │◀────────────────│
  │                 │                │                 │ response        │
  │                 │                │                 │                 │
  │                 │                │                 │ [if tool_use]:  │
  │                 │                │                 │  execute tools  │
  │                 │                │                 │  append results │
  │                 │                │                 │  → loop again   │
  │                 │                │                 │                 │
  │                 │                │                 │ [if text only]: │
  │                 │                │                 │  complete turn  │
  │                 │                │                 │                 │
  │                 │◀────────────────────────────────│                 │
  │                 │ AgentResponse                    │                 │
  │                 │                │                 │                 │
  │◀────────────────│                │                 │                 │
  │ {session_id,    │                │                 │                 │
  │  response,      │                │                 │                 │
  │  tool_calls,    │                │                 │                 │
  │  usage}         │                │                 │                 │
```

### 2. Agentic Tool Loop (Detail)

```
Agent                    LLM Backend              ToolRegistry            Tool
  │                         │                         │                    │
  │ ── iteration 1 ────────────────────────────────────────────────────── │
  │                         │                         │                    │
  │ complete(messages +     │                         │                    │
  │   tools + system)       │                         │                    │
  │────────────────────────▶│                         │                    │
  │                         │                         │                    │
  │◀────────────────────────│                         │                    │
  │ response: tool_use[     │                         │                    │
  │   {id:"t1",             │                         │                    │
  │    name:"shell",        │                         │                    │
  │    input:{cmd:"ls"}}]   │                         │                    │
  │                         │                         │                    │
  │ execute("shell",        │                         │                    │
  │   {cmd:"ls"}, ctx)      │                         │                    │
  │──────────────────────────────────────────────────▶│                    │
  │                         │                         │ execute(params,ctx)│
  │                         │                         │───────────────────▶│
  │                         │                         │                    │
  │                         │                         │◀───────────────────│
  │                         │                         │ ToolResult::Text   │
  │◀──────────────────────────────────────────────────│                    │
  │ ToolResultRecord        │                         │                    │
  │                         │                         │                    │
  │ append to messages:     │                         │                    │
  │  assistant(tool_use)    │                         │                    │
  │  user(tool_result)      │                         │                    │
  │                         │                         │                    │
  │ ── iteration 2 ────────────────────────────────────────────────────── │
  │                         │                         │                    │
  │ complete(messages +     │                         │                    │
  │   tools + system)       │                         │                    │
  │────────────────────────▶│                         │                    │
  │                         │                         │                    │
  │◀────────────────────────│                         │                    │
  │ response: text only     │                         │                    │
  │ "Here are the files..." │                         │                    │
  │                         │                         │                    │
  │ ── done ──── return AgentResponse ─────────────────────────────────── │
```

### 3. Session Close & Indexing Pipeline

```
Client          Server         AppState        SessionIndexer      LLM          MemoryStore
  │                │               │                │                │              │
  │ DELETE         │               │                │                │              │
  │ /sessions/{id} │               │                │                │              │
  │───────────────▶│               │                │                │              │
  │                │               │                │                │              │
  │                │ close_session  │                │                │              │
  │                │──────────────▶│                │                │              │
  │                │               │                │                │              │
  │                │               │ remove session │                │              │
  │                │               │ from store     │                │              │
  │                │               │                │                │              │
  │                │               │ tokio::spawn ──────────────────────────────── │
  │                │               │ (background)   │                │              │
  │                │               │                │                │              │
  │◀───────────────│               │                │                │              │
  │ 204 No Content │               │                │                │              │
  │                │               │                │                │              │
  │ ═══ background task ═══════════════════════════════════════════════════════════ │
  │                │               │                │                │              │
  │                │               │    index_session(sid, messages) │              │
  │                │               │───────────────▶│                │              │
  │                │               │                │                │              │
  │                │               │                │ STAGE 1: EXTRACT              │
  │                │               │                │                │              │
  │                │               │                │ [if NER engine]:              │
  │                │               │                │  ner.extract() │              │
  │                │               │                │  → entities,   │              │
  │                │               │                │    relations   │              │
  │                │               │                │                │              │
  │                │               │                │  LLM facts_only│              │
  │                │               │                │───────────────▶│              │
  │                │               │                │◀───────────────│              │
  │                │               │                │                │              │
  │                │               │                │ [else LLM-only]:              │
  │                │               │                │  LLM full      │              │
  │                │               │                │  extraction    │              │
  │                │               │                │───────────────▶│              │
  │                │               │                │◀───────────────│              │
  │                │               │                │ entities,facts,│              │
  │                │               │                │ relationships  │              │
  │                │               │                │                │              │
  │                │               │                │ STAGE 2: STORE ENTITIES       │
  │                │               │                │──────────────────────────────▶│
  │                │               │                │              graph.add_entity │
  │                │               │                │                │              │
  │                │               │                │ STAGE 3: STORE FACTS          │
  │                │               │                │                │              │
  │                │               │                │ for each fact: │              │
  │                │               │                │  embed(content)│              │
  │                │               │                │  store_fact()──────────────▶ │
  │                │               │                │                │  detect      │
  │                │               │                │                │  contradict  │
  │                │               │                │                │  / reinforce │
  │                │               │                │                │  / insert    │
  │                │               │                │                │              │
  │                │               │                │ STAGE 4: STORE RELATIONSHIPS  │
  │                │               │                │──────────────────────────────▶│
  │                │               │                │           graph.add_relation  │
  │                │               │                │                │              │
  │                │               │                │ STAGE 5: SUMMARIZE            │
  │                │               │                │───────────────▶│              │
  │                │               │                │◀───────────────│              │
  │                │               │                │ summary text   │              │
  │                │               │                │                │              │
  │                │               │                │ embed + store ────────────▶  │
  │                │               │                │                │              │
  │                │               │◀───────────────│                │              │
  │                │               │ IndexReport    │                │              │
  │                │               │ {entities: 11, │                │              │
  │                │               │  facts: 7,     │                │              │
  │                │               │  rels: 8,      │                │              │
  │                │               │  summary: true} │               │              │
```

### 4. Active Recall During Agent Turn

```
Agent                  Embedder            MemoryStore           GraphStore
  │                       │                    │                     │
  │ user_message          │                    │                     │
  │                       │                    │                     │
  │ embed(user_message)   │                    │                     │
  │──────────────────────▶│                    │                     │
  │◀──────────────────────│                    │                     │
  │ query_embedding       │                    │                     │
  │                       │                    │                     │
  │ recall(RecallQuery{   │                    │                     │
  │   embedding,          │                    │                     │
  │   limit: 5,           │                    │                     │
  │   threshold: 0.6})    │                    │                     │
  │───────────────────────────────────────────▶│                     │
  │                       │                    │                     │
  │                       │    search_similar  │                     │
  │                       │    (vec0 query)    │                     │
  │                       │                    │                     │
  │                       │                    │ get_neighbors(id)   │
  │                       │                    │────────────────────▶│
  │                       │                    │◀────────────────────│
  │                       │                    │ graph entities      │
  │                       │                    │                     │
  │                       │    SCORING:        │                     │
  │                       │    sim*0.4 +       │                     │
  │                       │    graph*0.3 +     │                     │
  │                       │    confidence*0.3  │                     │
  │                       │                    │                     │
  │◀──────────────────────────────────────────│                     │
  │ RecallResult{matches} │                    │                     │
  │                       │                    │                     │
  │ inject as context     │                    │                     │
  │ message before        │                    │                     │
  │ LLM call              │                    │                     │
```

### 5. Fact Storage with Contradiction Detection

```
SessionIndexer              MemoryStore                    Embedder
  │                             │                             │
  │ store_fact(Memory{          │                             │
  │   content: "arawn.lang      │                             │
  │            is Rust",        │                             │
  │   metadata: {session_id,    │                             │
  │     subject: "arawn.lang",  │                             │
  │     predicate: "is"}})      │                             │
  │                             │                             │
  │ embed("arawn.lang is Rust") │                             │
  │────────────────────────────────────────────────────────▶ │
  │◀────────────────────────────────────────────────────────│
  │ embedding [f32; 384]        │                             │
  │                             │                             │
  │ store_fact(memory, opts)    │                             │
  │────────────────────────────▶│                             │
  │                             │                             │
  │                             │ find_contradictions(        │
  │                             │   subject="arawn.lang",     │
  │                             │   predicate="is")           │
  │                             │                             │
  │                             │ SELECT * FROM memories      │
  │                             │ WHERE metadata LIKE         │
  │                             │   '%arawn.lang%'            │
  │                             │ AND superseded = 0          │
  │                             │                             │
  │                             │ ┌─────────────────────────┐ │
  │                             │ │ CASE 1: No match        │ │
  │                             │ │ → INSERT new memory     │ │
  │                             │ │ → store embedding       │ │
  │                             │ │ → return Inserted       │ │
  │                             │ │                         │ │
  │                             │ │ CASE 2: Same content    │ │
  │                             │ │ → reinforcement_count++ │ │
  │                             │ │ → return Reinforced     │ │
  │                             │ │                         │ │
  │                             │ │ CASE 3: Diff content    │ │
  │                             │ │ → old.superseded = true │ │
  │                             │ │ → old.superseded_by = id│ │
  │                             │ │ → INSERT new memory     │ │
  │                             │ │ → return Superseded     │ │
  │                             │ └─────────────────────────┘ │
  │                             │                             │
  │◀────────────────────────────│                             │
  │ StoreFactResult             │                             │
```

---

## Crate Dependency Graph

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

---

## Data Flow Summary

| Flow | Path | Trigger |
|------|------|---------|
| **Chat** | Client → Server → Agent → LLM → Tools → Agent → Server → Client | POST /api/v1/chat |
| **Stream** | Client → Server → Agent → LLM (SSE) → Client (chunked) | POST /api/v1/chat/stream |
| **Recall** | Agent turn → Embedder → MemoryStore (vec0 + graph) → Context | Every agent turn (if enabled) |
| **Indexing** | Session close → SessionIndexer → LLM extract → MemoryStore + GraphStore | DELETE /sessions/{id} or WS disconnect |
| **Contradiction** | Indexer → store_fact → find same subject+predicate → supersede or reinforce | During indexing |
| **Config** | TOML files → cascading merge → secret resolution (keyring→env→file) | Server startup |
| **Pipeline** | Workflow TOML → Cloacina engine → WASM sandbox → ScriptExecutor | Via workflow tool |
| **Delegation** | Agent → DelegateTool → SubagentSpawner → Subagent → Result truncation → Parent | Via delegate tool |

---

## Subagent Delegation

Arawn supports spawning specialized subagents to handle specific tasks. See [docs/subagent-delegation.md](docs/subagent-delegation.md) for detailed documentation.

### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│ Parent Agent                                                     │
│                                                                  │
│  ┌──────────────┐   ┌─────────────────┐   ┌──────────────────┐ │
│  │ Tool         │   │ Delegate Tool    │   │ Other Tools      │ │
│  │ Registry     │──▶│                  │◀──│ (shell, file,    │ │
│  │              │   │ agent: string    │   │  web, etc.)      │ │
│  └──────────────┘   │ task: string     │   └──────────────────┘ │
│                     │ context: string  │                        │
│                     │ mode: blocking/bg│                        │
│                     └────────┬─────────┘                        │
│                              │                                  │
└──────────────────────────────┼──────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────┐
│ PluginSubagentSpawner (arawn-plugin)                            │
│                                                                  │
│  agent_configs: HashMap<String, PluginAgentConfig>              │
│  agent_sources: HashMap<String, String>  (plugin name)          │
│  hook_dispatcher: Option<SharedHookDispatcher>                  │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ AgentSpawner                                              │   │
│  │  parent_tools: Arc<ToolRegistry>                         │   │
│  │  backend: SharedBackend                                   │   │
│  │                                                           │   │
│  │  spawn(config) → Agent with constrained tools             │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
                               │
             ┌─────────────────┴─────────────────┐
             ▼                                   ▼
┌─────────────────────────┐       ┌─────────────────────────┐
│ Blocking Execution       │       │ Background Execution     │
│                          │       │                          │
│ 1. Create Session        │       │ 1. Create Session        │
│ 2. Set context preamble  │       │ 2. Set context preamble  │
│ 3. agent.turn()          │       │ 3. tokio::spawn          │
│ 4. Truncate result       │       │ 4. Fire SubagentStarted  │
│ 5. Return to parent      │       │ 5. agent.turn()          │
│                          │       │ 6. Fire SubagentCompleted│
└─────────────────────────┘       └─────────────────────────┘
```

### Key Components

| Component | Location | Role |
|-----------|----------|------|
| `DelegateTool` | arawn-agent/src/tools/delegate.rs | Tool interface for delegation |
| `SubagentSpawner` | arawn-types/src/delegation.rs | Trait for spawning subagents |
| `PluginSubagentSpawner` | arawn-plugin/src/agent_spawner.rs | Implementation using plugin configs |
| `SubagentResult` | arawn-types/src/delegation.rs | Result type with truncation metadata |

### Context Injection

Parent context is injected into subagent's system prompt, not conversation history:

```
Session::set_context_preamble() → Agent::build_request() → System prompt prefix
```

Context is truncated at 4000 characters to prevent bloat.

### Result Truncation

Long subagent responses are truncated to prevent context overflow:

```
truncate_result(text, 8000) → TruncatedResult {
    text: "Beginning...omitted...End",
    truncated: true,
    original_len: Some(15000)
}
```

Preserves beginning (65%) and end (35%) with truncation notice.

---

## MCP Integration

Model Context Protocol (MCP) enables Arawn to bridge external tool servers into its agent loop.

### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│ arawn-mcp                                                        │
│                                                                  │
│  ┌───────────────────────┐    ┌───────────────────────────────┐ │
│  │ McpManager            │    │ McpClient                      │ │
│  │                       │    │                                │ │
│  │ servers: HashMap<     │    │ transport: StdioTransport      │ │
│  │   String, McpClient>  │───▶│ capabilities: ServerCaps      │ │
│  │                       │    │                                │ │
│  │ list_tools() ────────▶│    │ list_tools() → Vec<Tool>      │ │
│  │ call_tool()  ────────▶│    │ call_tool(name, params)       │ │
│  └───────────────────────┘    └───────────────────────────────┘ │
│                                                                  │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │ McpToolAdapter                                             │  │
│  │                                                            │  │
│  │ Wraps MCP tools as Arawn Tool trait:                      │  │
│  │ • name() → "mcp__{server}__{tool}"                        │  │
│  │ • parameters() → JSON Schema from MCP                     │  │
│  │ • execute() → McpClient.call_tool()                       │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

### Transport Types

| Transport | Config Key | Use Case |
|-----------|------------|----------|
| **stdio** | `command`, `args` | Local CLI tools (e.g., `sqlite-mcp`) |
| **sse** | `url` | HTTP Server-Sent Events |

### Configuration

```toml
[mcp]
enabled = true

[mcp.servers.sqlite]
command = "sqlite-mcp"
args = ["--db", "memory.db"]
```

### Tool Namespacing

MCP tools are namespaced to avoid collisions:
- Pattern: `mcp__{server_name}__{tool_name}`
- Example: `mcp__sqlite__query` for the `query` tool from `sqlite` server

---

## Plugin System

Arawn supports Claude Code-compatible plugins for extensibility.

### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│ arawn-plugin                                                     │
│                                                                  │
│  ┌──────────────────────┐  ┌──────────────────────────────────┐ │
│  │ PluginManager        │  │ Plugin                            │ │
│  │                      │  │                                   │ │
│  │ plugins: Vec<Plugin> │──│ name: String                     │ │
│  │                      │  │ root: PathBuf                    │ │
│  │ load_directory()     │  │ skills: Vec<Skill>               │ │
│  │ get_skill()          │  │ hooks: Vec<HookConfig>           │ │
│  │ get_agents()         │  │ agents: Vec<PluginAgentConfig>   │ │
│  │                      │  │ cli_tools: Vec<CliTool>          │ │
│  └──────────────────────┘  └──────────────────────────────────┘ │
│                                                                  │
│  ┌──────────────────────┐  ┌──────────────────────────────────┐ │
│  │ HookDispatcher       │  │ PluginSubagentSpawner            │ │
│  │                      │  │                                   │ │
│  │ matchers: Vec<Hook>  │  │ agent_configs: HashMap           │ │
│  │                      │  │ agent_sources: HashMap           │ │
│  │ Events:              │  │                                   │ │
│  │ • PreToolUse         │  │ spawn(config) → SubagentResult   │ │
│  │ • PostToolUse        │  │                                   │ │
│  │ • SessionStart       │  │ Tool constraints from            │ │
│  │ • SessionEnd         │  │ PluginAgentConfig.allowed_tools  │ │
│  │ • SubagentStarted    │  │                                   │ │
│  │ • SubagentCompleted  │  └──────────────────────────────────┘ │
│  └──────────────────────┘                                        │
└─────────────────────────────────────────────────────────────────┘
```

### Plugin Components

| Component | File Pattern | Purpose |
|-----------|--------------|---------|
| **Skills** | `skills/*.md` | Reusable prompts with YAML frontmatter |
| **Hooks** | `hooks/*.{sh,js,json}` | Event-driven extensibility |
| **Agents** | `agents/*.md` | Subagent definitions with tool constraints |
| **CLI Tools** | `cli-tools/*.{sh,js}` | External tools exposed via JSON stdin/stdout |

### Plugin Directory Structure

```
~/.arawn/plugins/
└── my-plugin/
    ├── plugin.json          # Manifest
    ├── skills/
    │   └── research.md      # Skill with frontmatter
    ├── agents/
    │   └── code-review.md   # Subagent definition
    ├── hooks/
    │   └── pre-shell.sh     # Hook script
    └── cli-tools/
        └── format.sh        # CLI tool wrapper
```

---

## External Integrations

### LLM Providers

| Provider | Backend | Auth | Capabilities |
|----------|---------|------|--------------|
| **Anthropic** | `anthropic` | API Key (keyring/env) | Tool calling, streaming, vision |
| **OpenAI** | `openai` | API Key (keyring/env) | Tool calling, function_call, embeddings |
| **Groq** | `groq` | API Key (keyring/env) | Fast inference, OpenAI-compatible |
| **Ollama** | `ollama` | None (localhost) | Local LLMs, no rate limits |

### Storage Systems

| System | Purpose | Integration |
|--------|---------|-------------|
| **SQLite** | Memories, sessions, notes | rusqlite, memory.db |
| **sqlite-vec** | Vector similarity search | vec0 virtual table |
| **graphqlite** | Knowledge graph | entities + relationships tables |
| **ONNX Runtime** | Local embeddings, NER | Vendored orp crate |

### Secret Management

| Source | Priority | Example |
|--------|----------|---------|
| OS Keyring | 1 (highest) | `keyring::Entry::new("arawn", "anthropic_api_key")` |
| Environment | 2 | `ANTHROPIC_API_KEY=sk-...` |
| Config file | 3 (lowest) | `api_key = "sk-..."` (not recommended) |

---

## Key Traits & Interfaces

### Core Traits

```rust
// LLM Backend abstraction (arawn-llm)
pub trait LlmBackend: Send + Sync {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse>;
    async fn complete_stream(&self, request: CompletionRequest) -> Result<ResponseStream>;
}

// Tool abstraction (arawn-agent)
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> Value;  // JSON schema
    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult>;
}

// Embedder abstraction (arawn-llm)
pub trait Embedder: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>;
}

// Subagent spawning (arawn-types)
pub trait SubagentSpawner: Send + Sync {
    async fn spawn(&self, config: SpawnConfig) -> Result<SubagentResult>;
}

// Named Entity Recognition (arawn-agent)
pub trait NerEngine: Send + Sync {
    fn extract(&self, text: &str, labels: &[&str]) -> Result<Vec<Entity>>;
}
```

### Key Types

| Type | Location | Purpose |
|------|----------|---------|
| `CompletionRequest` | arawn-llm | LLM request with messages, tools, system |
| `CompletionResponse` | arawn-llm | LLM response with content blocks, usage |
| `Message` | arawn-types | Conversation turn (user/assistant/system) |
| `Session` | arawn-types | Conversation state with history |
| `Memory` | arawn-memory | Stored fact with embedding, confidence |
| `RecallQuery` | arawn-memory | Vector search parameters |
| `RecallResult` | arawn-memory | Ranked memory matches |

---

## Configuration System

### Resolution Order

```
1. CLI flags (--config, --verbose, etc.)
   ↓
2. Project config (.arawn/arawn.toml)
   ↓
3. User config (~/.arawn/arawn.toml)
   ↓
4. XDG config ($XDG_CONFIG_HOME/arawn/arawn.toml)
   ↓
5. Built-in defaults
```

### Config Sections

```toml
# LLM backend selection
[llm]
backend = "groq"           # anthropic, openai, groq, ollama
model = "llama-3.3-70b"    # Model name

# Memory and indexing
[memory]
database = "memory.db"     # SQLite path (relative to data dir)

[memory.indexing]
enabled = true             # Enable session indexing
backend = "default"        # LLM backend for extraction
model = "gpt-4o-mini"      # Model for extraction/summarization

# MCP servers
[mcp]
enabled = true

[mcp.servers.sqlite]
command = "sqlite-mcp"
args = ["--db", "data.db"]

# HTTP server
[server]
port = 8080
bind = "127.0.0.1"

# Multiple LLM backends
[backends.anthropic]
api_key = "$keyring:anthropic_api_key"  # Keyring reference
model = "claude-sonnet-4-20250514"

[backends.groq]
api_key = "$env:GROQ_API_KEY"           # Environment reference
model = "llama-3.3-70b-versatile"
```

### Secret Resolution Syntax

| Syntax | Source | Example |
|--------|--------|---------|
| `$keyring:name` | OS Keyring | `$keyring:anthropic_api_key` |
| `$env:VAR` | Environment | `$env:OPENAI_API_KEY` |
| `$file:path` | File contents | `$file:~/.secrets/api_key` |
| Literal | Config value | `sk-ant-...` (not recommended) |

---

## Deployment

### Single-Binary Edge Deployment

Arawn is designed for edge computing — a single binary with embedded storage:

```
~/.arawn/
├── arawn.toml           # User configuration
├── arawn.log            # Rotating logs
└── data/
    ├── memory.db        # SQLite (memories, sessions, notes)
    ├── memory.db-wal    # Write-ahead log
    ├── workstreams.db   # Workstream cache
    └── workstreams/     # JSONL message history
```

### Resource Requirements

| Resource | Minimum | Recommended |
|----------|---------|-------------|
| RAM | 512 MB | 2 GB (with local embeddings) |
| Disk | 100 MB | 1 GB (for memories + models) |
| CPU | 2 cores | 4 cores (parallel tool execution) |

### Optional ONNX Features

Enable local inference with feature flags:

```bash
# Build with local embeddings
cargo build --release --features gliner

# Models downloaded on first use:
# - all-MiniLM-L6-v2 (embeddings, ~80MB)
# - GLiNER-small (NER, ~400MB)
```
