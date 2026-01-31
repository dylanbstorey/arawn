---
id: agent-core-conversation-and-task
level: initiative
title: "Agent Core: Conversation and Task Execution"
short_code: "ARAWN-I-0004"
created_at: 2026-01-28T01:37:28.013078+00:00
updated_at: 2026-01-28T03:56:21.797405+00:00
parent: ARAWN-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: L
strategy_id: NULL
initiative_id: agent-core-conversation-and-task
---

# Agent Core: Conversation and Task Execution

## Context

The agent core is the brain of the system. It handles the conversation loop, decides when to use tools, plans multi-step tasks, and orchestrates long-running workflows via cloacina.

This is where the "personal AI agent" behavior emerges from the combination of LLM, memory, and tools.

## Goals & Non-Goals

**Goals:**
- Conversation loop with context management
- Tool definition and execution framework
- Task planning and decomposition
- Long-running workflow orchestration (cloacina integration)
- Interruptible/resumable execution
- Context window management (summarization, pruning)

**Non-Goals:**
- Transport layer (handled by Server initiative)
- Specific integrations (separate initiatives)
- Sandbox implementation (separate initiative)

## Detailed Design

### Agent Loop

```rust
pub struct Agent {
    llm: LlmClient,
    memory: MemoryStore,
    tools: ToolRegistry,
    workflows: WorkflowEngine,  // cloacina
}

impl Agent {
    /// Single turn: user message -> agent response
    pub async fn turn(&self, session: &mut Session, message: &str) -> Result<Response> {
        // 1. Load context from memory
        let context = self.build_context(session, message).await?;
        
        // 2. Call LLM with tools
        let llm_response = self.llm.complete(context).await?;
        
        // 3. Execute tool calls if any
        let response = self.execute_tools(session, llm_response).await?;
        
        // 4. Store in memory
        self.memory.store_turn(session, message, &response).await?;
        
        response
    }
    
    /// Multi-turn conversation with streaming
    pub async fn converse(&self, session: &mut Session, message: &str) -> impl Stream<Item = Chunk>;
    
    /// Submit long-running task (returns immediately)
    pub async fn submit_task(&self, task: TaskRequest) -> Result<TaskId>;
    
    /// Check task status
    pub async fn task_status(&self, task_id: TaskId) -> Result<TaskStatus>;
}
```

### Tool Framework

```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> JsonSchema;
    
    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult>;
}

pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

// Built-in tools (MVP)
pub struct WebSearchTool;      // Search the web
pub struct WebFetchTool;       // Fetch URL content
pub struct FileReadTool;       // Read local files
pub struct FileWriteTool;      // Write local files
pub struct ShellTool;          // Execute commands (sandboxed)
pub struct MemorySearchTool;   // Query agent's memory
pub struct NoteTool;           // Create/update notes
```

### Task Planning

```rust
pub struct Planner {
    llm: LlmClient,
}

impl Planner {
    /// Decompose a high-level goal into steps
    pub async fn plan(&self, goal: &str, context: &Context) -> Result<Plan>;
    
    /// Re-plan based on execution results
    pub async fn replan(&self, plan: &Plan, results: &[StepResult]) -> Result<Plan>;
}

pub struct Plan {
    goal: String,
    steps: Vec<PlanStep>,
    dependencies: HashMap<StepId, Vec<StepId>>,
}

pub struct PlanStep {
    id: StepId,
    description: String,
    tool_hint: Option<String>,
    estimated_complexity: Complexity,
}
```

### Workflow Integration (cloacina)

```rust
use cloacina::{Workflow, Task, Context as CloaContext};

// Define research workflow
#[workflow(name = "research")]
pub struct ResearchWorkflow;

#[task(id = "decompose", dependencies = [])]
async fn decompose_question(ctx: &mut CloaContext<ResearchState>) -> Result<()> {
    // Break question into sub-questions
}

#[task(id = "search", dependencies = ["decompose"])]
async fn search_sources(ctx: &mut CloaContext<ResearchState>) -> Result<()> {
    // Search for each sub-question
}

#[task(id = "analyze", dependencies = ["search"])]
async fn analyze_findings(ctx: &mut CloaContext<ResearchState>) -> Result<()> {
    // Synthesize findings
}

#[task(id = "report", dependencies = ["analyze"])]
async fn generate_report(ctx: &mut CloaContext<ResearchState>) -> Result<()> {
    // Create final report
}
```

### Context Management

```rust
pub struct ContextBuilder {
    max_tokens: usize,
    memory: Arc<MemoryStore>,
}

impl ContextBuilder {
    pub async fn build(&self, session: &Session, query: &str) -> Result<Context> {
        // 1. Get recent conversation history
        let history = session.recent_messages(self.max_history)?;
        
        // 2. Retrieve relevant memories
        let embedding = self.embed(query).await?;
        let memories = self.memory.search_similar(&embedding, 10)?;
        
        // 3. Get relevant graph context
        let graph_context = self.memory.query_related_entities(query)?;
        
        // 4. Fit within token budget
        self.fit_to_budget(history, memories, graph_context)
    }
}
```

### Session State

```rust
pub struct Session {
    id: SessionId,
    messages: Vec<Message>,
    active_task: Option<TaskId>,
    metadata: SessionMetadata,
}

pub struct Message {
    role: Role,
    content: String,
    tool_calls: Vec<ToolCall>,
    tool_results: Vec<ToolResult>,
    timestamp: DateTime<Utc>,
}
```

### Dependencies

```toml
[dependencies]
arawn-memory = { path = "../arawn-memory" }
arawn-llm = { path = "../arawn-llm" }
cloacina = "0.1"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
async-trait = "0.1"
futures = "0.3"
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
```

## Alternatives Considered

- **No planning, just react**: Rejected - multi-step research requires planning
- **External orchestrator**: Rejected - cloacina provides embedded orchestration
- **Stateless agent**: Rejected - sessions and memory are core to the value prop

## Implementation Plan

1. Basic agent loop (single turn, no tools)
2. Tool framework and registry
3. Built-in tools (web search, file ops)
4. Context builder with memory integration
5. Task planner
6. cloacina workflow integration
7. Long-running task management
8. Streaming responses