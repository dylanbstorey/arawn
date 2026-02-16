---
id: rlm-exploration-agent
level: initiative
title: "RLM Exploration Agent"
short_code: "ARAWN-I-0027"
created_at: 2026-02-16T16:55:11.109128+00:00
updated_at: 2026-02-16T16:55:11.109128+00:00
parent: ARAWN-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: L
strategy_id: NULL
initiative_id: rlm-exploration-agent
---

# RLM Exploration Agent Initiative

## Context

When agents explore codebases, raw tool outputs (grep results, file contents, directory listings) consume significant context window space. A single `grep` returning 50 files can add 15k+ tokens to context, most of which is noise for the actual question.

The **Recursive Language Model (RLM)** pattern (inspired by [Muninn](https://github.com/colliery-io/muninn)) addresses this by:
1. Running exploration in an **isolated context** (doesn't pollute main agent)
2. Using potentially cheaper/faster models for the exploration legwork
3. Returning **compressed natural language findings** instead of raw data

This keeps the main agent's context clean and efficient while still enabling deep codebase exploration.

**Relationship to memory**: The RLM is intentionally decoupled from memory/indexing in this phase. Cache invalidation for code facts is unsolved - we'll learn from RLM usage patterns before designing persistence.

## Goals & Non-Goals

**Goals:**
- Isolated exploration context that doesn't pollute main agent history
- Explicit `explore()` tool for agent-initiated exploration
- User prompt when tool results exceed threshold ("compress with RLM?")
- Configurable model (defaults to main agent's model)
- Full tool access (not restricted to read-only)
- Budget enforcement (25 iteration limit, configurable)
- Natural language output for injection into main conversation

**Non-Goals (Phase 1):**
- Memory/knowledge graph persistence (deferred - cache invalidation unsolved)
- Automatic fact extraction
- Query intent classification (understand vs modify)
- Cross-session knowledge sharing

## Use Cases

### Use Case 1: Explicit Exploration
- **Actor**: Main agent processing user question "how does auth work?"
- **Scenario**: 
  1. Agent decides exploration needed
  2. Calls `explore(query: "how does authentication work in this codebase")`
  3. RLM spawns with isolated context
  4. RLM does glob/grep/read cycles (up to 25 iterations)
  5. RLM synthesizes findings into natural language
  6. Main agent receives compressed summary
- **Expected Outcome**: Main context gains ~500 tokens of findings instead of ~15k of raw files

### Use Case 2: Large Tool Result Compression
- **Actor**: Main agent after grep returns massive results
- **Scenario**:
  1. Agent calls `grep("TODO")` 
  2. Result exceeds threshold (e.g., 8k chars)
  3. System prompts user: "Large result (50 files). Compress with RLM?"
  4. User confirms
  5. RLM compresses result to key findings
  6. Main agent receives summary
- **Expected Outcome**: User controls when compression happens, context stays manageable

### Use Case 3: Deep Codebase Question
- **Actor**: Developer asking "what tests cover the session module?"
- **Scenario**:
  1. Main agent recognizes this needs exploration
  2. Spawns RLM with query
  3. RLM searches test files, reads relevant ones, traces imports
  4. RLM returns: "Found 12 tests across 3 files: session_test.rs (unit), integration/session.rs (e2e), ..."
- **Expected Outcome**: Comprehensive answer without context explosion

## Architecture

### Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                        Main Agent                               │
│                                                                 │
│  Context: [user msg, assistant, tool results, ...]             │
│                                                                 │
└───────────────────────────┬─────────────────────────────────────┘
                            │
            ┌───────────────┴───────────────┐
            │                               │
            ▼                               ▼
   ┌─────────────────┐            ┌─────────────────┐
   │  explore() tool │            │ Large Result    │
   │                 │            │ Interceptor     │
   │ explicit call   │            │                 │
   └────────┬────────┘            │ prompts user    │
            │                     └────────┬────────┘
            │                              │
            └──────────────┬───────────────┘
                           │
                           ▼
            ┌─────────────────────────────┐
            │      RLM Exploration        │
            │                             │
            │  ExplorationContext {       │
            │    query,                   │
            │    budget: {iters, tokens}, │
            │    tools: ToolRegistry,     │
            │    model: configured,       │
            │  }                          │
            │                             │
            │  Loop:                      │
            │    LLM → tool → result      │
            │    (isolated, not shared)   │
            │                             │
            │  Synthesize → NL summary    │
            └──────────────┬──────────────┘
                           │
                           ▼
            ┌─────────────────────────────┐
            │   ExplorationResult {       │
            │     summary: String,        │
            │     iterations_used: u32,   │
            │     files_explored: Vec,    │
            │   }                         │
            └─────────────────────────────┘
```

### Key Isolation Property

The RLM's exploration context is **completely isolated**:
- Tool calls inside RLM don't appear in main agent's history
- Token usage tracked separately
- On completion, only the synthesized summary enters main context

This is different from the existing `delegate` tool which returns full conversation.

## Detailed Design

### ExplorationContext

Isolated context for RLM execution:

```rust
pub struct ExplorationContext {
    /// The exploration query/task
    query: String,
    /// Budget constraints
    budget: ExplorationBudget,
    /// Accumulated turns (isolated from main agent)
    turns: Vec<Turn>,
    /// Files touched during exploration (for metadata)
    files_explored: Vec<PathBuf>,
}

pub struct ExplorationBudget {
    /// Max tool execution iterations (default: 25)
    max_iterations: u32,
    /// Max tokens for exploration context (default: 50k)
    max_tokens: usize,
    /// Current iteration count
    iterations_used: u32,
}
```

### ExploreTool

Explicit exploration trigger for main agent:

```rust
pub struct ExploreTool {
    rlm_spawner: Arc<RlmSpawner>,
}

// Tool definition
{
    "name": "explore",
    "description": "Explore the codebase to answer a question. Returns compressed findings.",
    "parameters": {
        "query": "string - What to explore/find/understand"
    }
}
// Note: Scope hints come from query context, not a separate parameter
```

### RlmSpawner

Factory for creating RLM explorations:

```rust
pub struct RlmSpawner {
    /// LLM backend for RLM (may differ from main agent)
    backend: SharedBackend,
    /// Tool registry (full access)
    tools: Arc<ToolRegistry>,
    /// Default budget configuration
    default_budget: ExplorationBudget,
}

impl RlmSpawner {
    pub async fn explore(
        &self,
        query: &str,
    ) -> Result<ExplorationResult>;
    
    pub async fn compress(
        &self, 
        content: &str,
        context: &str,
    ) -> Result<String>;
}
```

### ExplorationResult

What the RLM returns to the main agent:

```rust
pub struct ExplorationResult {
    /// Natural language summary (injected into main context)
    pub summary: String,
    /// Metadata (not injected, for logging/debugging)
    pub metadata: ExplorationMetadata,
}

pub struct ExplorationMetadata {
    pub iterations_used: u32,
    pub files_explored: Vec<PathBuf>,
    pub tokens_used: usize,
    pub model_used: String,
}
```

### Large Result Interceptor

Prompts user when tool output is large:

```rust
pub struct LargeResultInterceptor {
    /// Threshold in characters (default: 8000)
    threshold: usize,
    /// RLM spawner for compression
    rlm_spawner: Arc<RlmSpawner>,
}

impl LargeResultInterceptor {
    /// Check result and maybe prompt for compression
    pub async fn maybe_compress(
        &self,
        tool_name: &str,
        result: &str,
        user_prompter: &dyn UserPrompter,
    ) -> Result<String>;
}
```

### RLM System Prompt

The RLM gets a specialized system prompt:

```
You are an exploration agent. Your task is to explore a codebase to answer a question.

Instructions:
- Use glob, grep, and read tools to find relevant code
- Build understanding incrementally - don't try to read everything at once
- Focus on answering the specific query
- When you have enough information, synthesize your findings

Your final response should be a natural language summary that:
- Directly answers the query
- Cites specific files and line numbers where relevant
- Highlights key code patterns or structures discovered
- Is concise (aim for 200-500 words)

Do NOT include raw file contents in your final response - summarize and cite.
```

### Configuration

```toml
[rlm]
# Model for RLM (defaults to main agent model)
model = "default"

# Budget constraints
max_iterations = 25
max_tokens = 50000

# Large result interception
intercept_threshold = 8000
```

## Alternatives Considered

### 1. Extend Existing Delegate Tool (Rejected)
Reuse `delegate` for exploration with special parameters.
- **Rejected**: Delegate returns full conversation history. RLM needs isolation - only summary should enter main context.

### 2. Proxy Architecture (Deferred)
Muninn-style transparent proxy that intercepts all LLM calls.
- **Deferred**: More invasive change. Start with explicit tool, can add transparent routing later.

### 3. Auto-compress All Large Results (Rejected)
Skip user prompt, always compress large results.
- **Rejected**: User loses control. Some contexts need raw data. Prompt gives user agency.

### 4. Read-Only Tool Access (Rejected)
Restrict RLM to glob/grep/read only.
- **Rejected**: User requested full tool access. RLM may need web search, code execution for exploration.

### 5. Coupled Memory Persistence (Deferred)
Extract facts during exploration and persist to knowledge graph.
- **Deferred**: Cache invalidation unsolved. Learn from RLM usage first, design persistence later.

## Implementation Plan

### Prerequisites

- **arawn-client bi-directional communication**: Client layer needs to support server→client prompts (for "compress with RLM?" flow). May require WebSocket message types for prompts/responses.

### Task Breakdown

1. **Client Prompt Protocol** (arawn-client, arawn-server)
   - WebSocket message type for server→client prompts
   - Response handling for user answers
   - CLI adoption of client crate (if not already using)

2. **ExplorationContext and Budget** (arawn-agent)
   - Isolated context struct
   - Budget tracking (iterations, tokens)
   - No persistence to main session

3. **RlmSpawner** (arawn-agent)
   - Factory for creating explorations
   - Backend/model configuration
   - Tool registry injection

4. **RLM Agent Loop** (arawn-agent)
   - Exploration execution loop
   - System prompt for exploration
   - Progress streaming to client
   - Synthesis step at end (or on cancellation)
   - Max 1 level (no recursive sub-RLM spawning)

5. **ExploreTool** (arawn-agent)
   - Tool definition and handler
   - Wiring to RlmSpawner
   - Result formatting

6. **LargeResultInterceptor** (arawn-agent)
   - Threshold detection
   - User prompting via client protocol
   - Compression path

7. **Configuration** (arawn-config)
   - RLM config section
   - Model, budget, threshold settings

8. **Integration Testing**
   - End-to-end exploration test
   - Large result compression test
   - Budget enforcement test
   - Cancellation with partial synthesis test

## Design Decisions

1. **User prompt UX**: Requires bi-directional communication at arawn-client layer. CLI may need to adopt client crate. TUI can use existing prompt patterns.

2. **Streaming**: Yes - stream exploration progress to user (files being examined, iterations used).

3. **Cancellation**: Yes - user can cancel mid-exploration. RLM synthesizes partial findings.

4. **Scope hints**: Deferred. Context for "where to look" comes from the query and injected prompt, not a separate parameter. Simplifies tool interface.

5. **Recursive exploration**: Max 1 level. RLM cannot spawn sub-RLMs.

## Open Questions

1. **arawn-client bi-directional comms**: What's the protocol for prompts flowing back to user?

## Future Extensions (Post Phase 1)

- Memory integration with intent classification
- Transparent proxy mode (auto-route exploration queries)
- Cross-exploration caching (remember what was explored recently)
- Structured output mode (JSON findings alongside NL summary)