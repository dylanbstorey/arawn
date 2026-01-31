# Data Flow

Sequence diagrams for key operations in Arawn.

## Chat Request (Synchronous)

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

## Agentic Tool Loop (Detail)

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

## Session Close & Indexing Pipeline

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

## Active Recall During Agent Turn

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

## Fact Storage with Contradiction Detection

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
