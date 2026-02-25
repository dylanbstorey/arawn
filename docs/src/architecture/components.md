# Component Diagrams

Detailed component breakdowns for the Agent and Memory subsystems.

## Agent Component (arawn-agent)

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
│  │  interaction_logger: Option<Arc<InteractionLogger>>       │   │
│  │  hook_dispatcher: Option<SharedHookDispatcher>            │   │
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

### Agent Responsibilities

| Component | Purpose |
|-----------|---------|
| **Agent** | Main entry point, coordinates turns |
| **SystemPrompt Builder** | Constructs system prompt with context |
| **Context Builder** | Assembles history, recall, notes |
| **Tool Registry** | Manages available tools |
| **Session Indexer** | Extracts knowledge after session ends |
| **AgentStream** | Streaming response events |

## Memory Component (arawn-memory)

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
│  ┌──────────────────────────────┐                             │
│  │ Confidence Scoring           │                             │
│  │                              │                             │
│  │ score = base × reinforcement │                             │
│  │                × staleness   │                             │
│  │                              │                             │
│  │ base: stated=1.0, system=0.9,│                             │
│  │       observed=0.7, inferred=0.5                           │
│  │ reinforcement: min(1+0.1n, 1.5)                            │
│  │ staleness: linear decay to 0.3                             │
│  │   over 365 days (fresh < 30d)│                             │
│  │ superseded: always 0.0       │                             │
│  └──────────────────────────────┘                             │
└──────────────────────────────────────────────────────────────┘
```

### Memory Responsibilities

| Component | Purpose |
|-----------|---------|
| **MemoryStore** | Central store coordinating SQLite, vectors, graph |
| **Unified Ops** | Store facts with contradiction detection |
| **Recall** | Hybrid search combining vector + graph + confidence |
| **Vector Ops** | sqlite-vec for embedding similarity |
| **Graph Ops** | graphqlite for entity relationships |
| **Confidence Scoring** | Dynamic scoring based on source, age, reinforcement |

## Key Traits

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

## Key Types

| Type | Location | Purpose |
|------|----------|---------|
| `CompletionRequest` | arawn-llm | LLM request with messages, tools, system |
| `CompletionResponse` | arawn-llm | LLM response with content blocks, usage |
| `Message` | arawn-types | Conversation turn (user/assistant/system) |
| `Session` | arawn-types | Conversation state with history |
| `Memory` | arawn-memory | Stored fact with embedding, confidence |
| `RecallQuery` | arawn-memory | Vector search parameters |
| `RecallResult` | arawn-memory | Ranked memory matches |
