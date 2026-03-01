---
id: rlm-exploration-agent
level: initiative
title: "RLM Exploration Agent"
short_code: "ARAWN-I-0027"
created_at: 2026-02-16T16:55:11.109128+00:00
updated_at: 2026-03-01T15:29:37.145330+00:00
parent: ARAWN-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/design"


exit_criteria_met: false
estimated_complexity: M
strategy_id: NULL
initiative_id: rlm-exploration-agent
---

# RLM Exploration Agent Initiative

## Context

When agents explore large information spaces — codebases, documentation, web content, research — raw tool outputs consume significant context window space. A single `grep` returning 50 files can add 15k+ tokens to context, most of which is noise for the actual question. The same applies to fetching multiple web pages, reading long documents, or searching across doc sites.

The **Recursive Language Model (RLM)** pattern (inspired by [Muninn](https://github.com/colliery-io/muninn)) addresses this by:
1. Running exploration in an **isolated context** (doesn't pollute main agent)
2. Using potentially cheaper/faster models for the exploration legwork
3. Returning **compressed natural language findings** instead of raw data

This keeps the main agent's context clean and efficient while still enabling deep exploration across any information source: code, documentation, web content, or research materials.

**Relationship to memory**: The RLM is intentionally decoupled from memory/indexing in this phase. Cache invalidation for explored facts is unsolved - we'll learn from RLM usage patterns before designing persistence.

## Goals & Non-Goals

**Goals (Phase 1):**
- Isolated exploration context that doesn't pollute main agent history
- Explicit `explore()` tool for agent-initiated exploration
- Configurable model (falls back to default LLM)
- Read-only tool access: code tools (glob, grep, file_read), documentation tools (web_fetch, web_search), and MCP tools where available
- Budget enforcement (iteration + token limits, configurable) — generic mechanism reusable by other agent types
- Natural language output for injection into main conversation

**Phase 2 (Deferred):**
- Streaming exploration progress to client
- Large-result-intercept with user prompt ("compress with RLM?")
- Server→client prompt protocol

**Non-Goals:**
- Memory/knowledge graph persistence (cache invalidation unsolved)
- Automatic fact extraction
- Query intent classification (understand vs modify)
- Cross-session knowledge sharing

## Use Cases

### Use Case 1: Codebase Exploration
- **Actor**: Main agent processing user question "how does auth work?"
- **Scenario**: 
  1. Agent decides exploration needed
  2. Calls `explore(query: "how does authentication work in this codebase")`
  3. RLM spawns with isolated context
  4. RLM does glob/grep/read cycles (up to 25 iterations)
  5. RLM synthesizes findings into natural language
  6. Main agent receives compressed summary
- **Expected Outcome**: Main context gains ~500 tokens of findings instead of ~15k of raw files

### Use Case 2: Documentation Research
- **Actor**: Main agent needs to understand an external API or library
- **Scenario**:
  1. Agent needs to integrate with a library it doesn't know well
  2. Calls `explore(query: "how does the utoipa crate handle custom security schemes")`
  3. RLM fetches docs, reads examples, searches for patterns
  4. RLM returns summary with relevant API surface, code examples, and caveats
- **Expected Outcome**: Agent gets actionable knowledge without loading entire doc sites into context

### Use Case 3: Multi-Source Research
- **Actor**: Developer asking "what are the current best practices for WebSocket auth?"
- **Scenario**:
  1. Main agent recognizes this needs broad research
  2. Spawns RLM with query
  3. RLM searches web, reads relevant articles/docs, compares approaches
  4. RLM returns synthesized findings with sources cited
- **Expected Outcome**: Comprehensive research summary without context explosion

### Use Case 4: Large Tool Result Compression (Phase 2)
- **Actor**: Main agent after grep returns massive results
- **Scenario**:
  1. Agent calls `grep("TODO")` 
  2. Result exceeds threshold (e.g., 8k chars)
  3. System prompts user: "Large result (50 files). Compress with RLM?"
  4. User confirms
  5. RLM compresses result to key findings
  6. Main agent receives summary
- **Expected Outcome**: User controls when compression happens, context stays manageable

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
    /// Sources touched during exploration (file paths, URLs, MCP resources)
    sources_explored: Vec<String>,
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
    "description": "Explore and research to answer a question. Returns compressed findings.",
    "parameters": {
        "query": "string - What to explore/find/understand"
    }
}
// Note: Scope hints come from query context, not a separate parameter
```

### RlmSpawner

Factory that creates an `Agent` configured for exploration:

```rust
pub struct RlmSpawner {
    /// LLM backend for RLM (may differ from main agent)
    backend: SharedBackend,
    /// Read-only tool registry (filtered from main agent's tools)
    tools: Arc<ToolRegistry>,
    /// Default budget configuration
    default_budget: ExplorationBudget,
}

impl RlmSpawner {
    /// Create an Agent configured for exploration, run it, return summary.
    pub async fn explore(
        &self,
        query: &str,
    ) -> Result<ExplorationResult>;
}
```

Internally, `explore()` creates an `Agent` with the RLM system prompt, the read-only tool registry, and budget enforcement, then runs its turn loop to completion.

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
    pub sources_explored: Vec<String>,  // file paths, URLs, MCP resource IDs
    pub tokens_used: usize,
    pub model_used: String,
}
```

### Large Result Interceptor (Phase 2)

Deferred. Will prompt user when tool output exceeds threshold and offer RLM compression.

### RLM System Prompt

The RLM gets a specialized system prompt:

```
You are an exploration agent. Your task is to research and explore information sources to answer a question.

Instructions:
- Use the tools available to you to find relevant information
  - Code: glob, grep, file_read for local codebases
  - Documentation: web_fetch, web_search for online resources
  - MCP tools for specialized data sources
- Build understanding incrementally - don't try to consume everything at once
- Focus on answering the specific query
- When you have enough information, synthesize your findings

Your final response should be a natural language summary that:
- Directly answers the query
- Cites specific sources (files with line numbers, URLs, doc sections)
- Highlights key findings, patterns, or trade-offs discovered
- Is concise (aim for 200-500 words)

Do NOT include raw file contents or full page text in your final response - summarize and cite.
```

### Configuration

```toml
[rlm]
# Model for exploration (defaults to main agent model)
model = "default"

# Budget constraints (safety valve)
max_iterations = 25
max_tokens = 50000

# Compaction
compaction_threshold = 0.7  # compact when context reaches 70% of max_tokens
compaction_model = "default" # can use cheaper model (e.g., haiku)
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

### 4. Read-Only Tool Access (Accepted with nuance)
Restrict RLM to read-only tools plus internal scratch space.
- **Accepted**: Exploration is about gathering information, not changing state. If RLM writes externally, main agent doesn't know (isolation breaks). Cheaper models are riskier for writes.
- **Allowed**: glob, grep, file_read, web_fetch, web_search, MCP tools (read-only), internal scratch/think tool
- **Disallowed**: file_write, shell (initially), external modifications
- **Revisit**: Shell for read-only commands (jq, etc.) if usage patterns show need

### 5. Coupled Memory Persistence (Deferred)
Extract facts during exploration and persist to knowledge graph.
- **Deferred**: Cache invalidation unsolved. Learn from RLM usage first, design persistence later.

## Implementation Plan (Phase 1)

### Prerequisites

None for Phase 1. The existing `Agent` struct, `ToolRegistry`, and LLM backend provide the foundation.

### Task Breakdown

1. **Iterative compaction mechanism + compaction agent** (arawn-agent)
   - This is the core RLM capability and the most significant piece of work
   - **Outer loop orchestrator**: manages the explore→compact→continue cycle
   - **Compaction agent**: separate agent (no tools, potentially cheaper model) that receives conversation history and produces a compressed summary. Configurable compaction prompt — default generic prompt on Agent, overridable per agent type
   - When context grows beyond a threshold, orchestrator:
     1. Pauses exploration agent
     2. Feeds conversation to compaction agent
     3. Replaces history with: original query + compacted summary
     4. Resumes exploration agent with fresh context
   - "Done" signal: exploration agent stops calling tools (primary termination)
   - Budget (iteration + token limits) is a safety valve, not the primary stop condition
   - Generic mechanism — useful for any long-running agent, not just RLM

2. **Read-only ToolRegistry filtering** (arawn-agent)
   - Method on `ToolRegistry` to produce a filtered clone with only specified tools
   - Allowlist: glob, grep, file_read, web_fetch, web_search, plus MCP read-only tools
   - Simple `filtered_by_names()` for Phase 1 — clones Arc refs, cheap

3. **RLM module — RlmSpawner + ExplorationContext** (arawn-agent)
   - `arawn-agent/src/rlm/` module
   - `RlmSpawner`: creates an `Agent` with RLM system prompt, filtered tools, compaction config
   - `ExplorationContext`: query, sources explored, metadata tracking
   - `ExplorationResult`: summary string + metadata (iterations, compactions, sources, tokens, model)

4. **ExploreTool** (arawn-agent)
   - Tool definition and handler
   - Wiring to `RlmSpawner`
   - Result formatting (summary injected into main context, metadata logged)

5. **Configuration** (arawn-config)
   - `[rlm]` config section: model, max_iterations, max_tokens, compaction_threshold

6. **Integration Testing**
   - End-to-end exploration test (mock LLM + real tools)
   - Compaction cycle test (verify context is compressed and exploration continues)
   - Budget enforcement test (safety valve iteration/token limits)
   - Cancellation with partial synthesis test
   - Tool filtering test (write tools excluded)

## Design Decisions

1. **Reuse Agent**: The RLM reuses the existing `Agent` struct with different configuration (system prompt, filtered tool registry, budget). No custom loop — the existing agent machinery handles tool execution, error handling, and turn management. Lives as a module inside `arawn-agent`, not a separate crate.

2. **Iterative compaction as core mechanism**: The RLM's power comes from compacting findings and continuing exploration — not just running until a budget is exhausted. When context grows large, a separate **compaction agent** (potentially cheaper/faster model) summarizes findings, history is replaced with the compacted summary, and the exploration agent resumes with fresh context. This allows exploration far beyond the context window. Budget (iterations + tokens) is a safety valve. The compaction prompt is agent-configurable — default generic prompt, overridable per agent type. This is generic infrastructure useful for any long-running agent.

3. **Model selection**: Configurable per-exploration. Falls back to default LLM (same pattern as other tools/agents).

4. **Tool access**: Read-only plus internal scratch. See "Alternatives Considered" for full breakdown.

5. **Phase 1 scope**: Explicit `explore()` tool only. Large-result-intercept and streaming progress deferred to phase 2.

6. **Client prompts**: Not needed for phase 1. The intercept flow ("compress with RLM?") requires server→client prompts, but that's phase 2.

7. **Cancellation**: Yes - user can cancel mid-exploration. RLM synthesizes partial findings.

8. **Scope hints**: Deferred. Context for "where to look" comes from the query and injected prompt, not a separate parameter. Simplifies tool interface.

9. **Recursive exploration**: Max 1 level. RLM cannot spawn sub-RLMs.

## Open Questions

1. **Compaction strategy**: Compaction is handled by a **separate compaction agent**, not the exploration agent itself.
   - Compaction agent: no tools, receives the conversation to summarize, returns compressed summary
   - Can use a cheaper/faster model (e.g., Haiku-class) — configurable independently
   - Compaction prompt is agent-configurable: default generic prompt on Agent, overridable per agent type (RLM uses research-focused prompt, coding agents use code-focused prompt, etc.)
   - This keeps the exploration agent focused on exploring and makes compaction reusable across any long-running agent type

2. **Compaction threshold**: What triggers compaction? Options:
   - Token count threshold (e.g., 70% of `max_tokens`)
   - Number of tool results accumulated (e.g., every 10 tool calls)
   - Both (whichever hits first)
   - Need to experiment — the right threshold depends on how much raw data tools typically return.

3. **"Done" signal**: How does the RLM signal it's finished exploring?
   - Option A: LLM stops calling tools (same as current agent termination). The system prompt instructs it to produce a final synthesis when it has enough.
   - Option B: Explicit "done" tool the RLM calls with its summary.
   - Leaning toward A — simpler, reuses existing loop termination.

4. **Compaction and the existing Agent loop**: The existing `Agent::turn()` manages messages within a single turn. Compaction replaces the conversation history mid-turn. Does this fit cleanly into the current loop, or does the RLM need to manage its own outer loop that calls `turn()` repeatedly with compacted state between calls?

## Future Extensions (Post Phase 1)

- Memory integration with intent classification
- Transparent proxy mode (auto-route exploration queries)
- Cross-exploration caching (remember what was explored recently)
- Structured output mode (JSON findings alongside NL summary)