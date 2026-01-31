---
id: active-memory-thinking-and-recall
level: initiative
title: "Active Memory: Thinking and Recall"
short_code: "ARAWN-I-0014"
created_at: 2026-01-29T01:21:50.019588+00:00
updated_at: 2026-01-31T03:57:39.471471+00:00
parent: ARAWN-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
strategy_id: NULL
initiative_id: active-memory-thinking-and-recall
---

# Active Memory: Thinking and Recall

## Context

Arawn's `arawn-memory` crate already provides a solid foundation: SQLite persistence, 384-dim vector search (sqlite-vec), a knowledge graph (graphqlite), session management, notes, and a unified `recall()` API that combines vector similarity with graph context.

What's missing are the turn-level memory capabilities that make an agent feel intelligent:
- **Extended thinking**: The agent has no scratchpad for multi-step reasoning. It can't "think out loud" across turns without polluting the conversation.
- **Active recall**: The agent doesn't proactively surface relevant memories. It only searches when explicitly asked via the `memory_search` tool.
- **Moltbot comparison**: Moltbot has `memory_search` as a mandatory recall step before answering questions about prior work, decisions, dates, people, preferences, or todos. Arawn has no such automatic behavior.

Post-session indexing, memory confidence modeling, and cache invalidation are handled separately in ARAWN-I-0017.

## Goals & Non-Goals

**Goals:**
- Add a thinking/scratchpad tool that persists across turns within a session but doesn't get sent to the user
- Add automatic memory recall: before generating a response, query relevant memories and inject as context
- Make recall configurable: threshold, limit, enable/disable

**Non-Goals:**
- Post-session indexing and entity extraction (ARAWN-I-0017)
- Memory confidence model and cache invalidation (ARAWN-I-0017)
- Session summarization (ARAWN-I-0017)
- Changing the embedding model (ARAWN-I-0015)
- Multi-user memory isolation (single-user agent for now)
- RAG over external document corpora (future work)

## Detailed Design

### 1. Thinking/Scratchpad Tool

A new `think` tool that writes to a per-session scratchpad. Content is stored in memory but marked as `ContentType::Thought` (new variant). Thoughts are included in the agent's context window but excluded from user-visible responses.

```rust
// New content type
ContentType::Thought  // internal reasoning, not shown to user

// Think tool
pub struct ThinkTool {
    memory: Arc<MemoryStore>,
}
// execute: stores thought as memory, returns confirmation to agent
```

The system prompt instructs the agent to use `think` for multi-step reasoning, planning, and self-correction. Thoughts are:
- Visible to the agent in subsequent turns within the session
- Not included in the response to the user
- Stored in memory (can be recalled in future sessions if relevant)

### 2. Active Recall (Pre-Response Memory Injection)

Before the first LLM call in the agent turn loop, automatically query relevant memories:

```rust
// In agent.rs turn loop, before building messages:
let recall_results = memory.recall(RecallQuery::new(&user_message)
    .with_limit(config.recall.limit)          // default: 5
    .with_min_score(config.recall.threshold)   // default: 0.6
).await?;

if !recall_results.is_empty() {
    // Inject as a system message: "Relevant memories: ..."
    messages.insert(1, Message::system(format_recall_context(&recall_results)));
}
```

Key design choices:
- Runs once per user message (not per iteration in the tool-use loop)
- Only runs if the memory system has embeddings initialized
- Skipped if recall is disabled in config
- Recall results are formatted concisely to minimize context window usage

### Configuration

```toml
[memory.recall]
enabled = true
threshold = 0.6   # minimum similarity score
limit = 5         # max memories to inject
```

## Alternatives Considered

- **Always-on memory injection**: Could inject memories on every turn regardless of relevance. Rejected — wastes context window tokens on irrelevant memories. Threshold-based injection is more efficient.
- **LLM-based relevance filtering**: Have the LLM judge which recalled memories are actually relevant before injecting. Adds latency. The vector similarity threshold is a good enough filter for v1.
- **Recall on every iteration**: Could re-query memories after each tool call in case the context shifted. Rejected — one recall per user message is sufficient and avoids repeated embedding queries.

## Implementation Plan

### Task 1: Add ContentType::Thought and RecallQuery::with_min_score()
Extend `arawn-memory` types:
- Add `ContentType::Thought` variant with `as_str()`/`from_str()` support
- Add `with_min_score(f32)` to `RecallQuery` builder — filter matches below threshold
- Apply min_score filtering in `recall()` before returning results
- Tests: content type roundtrip, score threshold filtering, default behavior unchanged

**Files:** `crates/arawn-memory/src/types.rs`, `crates/arawn-memory/src/store.rs`

### Task 2: Implement ThinkTool
New tool in `arawn-agent/src/tools/think.rs`:
- Accepts `thought` (string) parameter
- Stores as memory with `ContentType::Thought`, tagged with current session_id
- Returns confirmation to agent (not shown to user)
- Register in tool registry via `start.rs`
- Tests: thought storage/retrieval, tool parameters schema, missing thought param

**Files:** `crates/arawn-agent/src/tools/think.rs`, `crates/arawn-agent/src/tools/mod.rs`

### Task 3: Add recall config to arawn-config
New `[memory.recall]` config section:
- `enabled: bool` (default: true)
- `threshold: f32` (default: 0.6, minimum similarity score)
- `limit: usize` (default: 5, max memories to inject)
- Wire into existing config discovery/merging
- Tests: config parsing, defaults, override from TOML

**Files:** `crates/arawn-config/src/types.rs`

### Task 4: Active recall injection in agent turn loop
Before the first LLM call per user message:
- Check if recall is enabled and embeddings are initialized
- Embed the user message text via the existing embedding provider
- Build `RecallQuery` with config threshold/limit and `with_min_score()`
- If matches found, inject formatted context as system message at position 1
- Only runs once per user message (not per tool-use iteration)
- Tests: recall injection with/without results, disabled config, no embeddings initialized

**Files:** `crates/arawn-agent/src/agent.rs`, `crates/arawn-agent/src/context.rs`

### Task 5: System prompt instructions for think tool
Update bootstrap prompt to instruct agent:
- Use `think` for multi-step reasoning, planning, and self-correction
- Use `think` before complex responses to organize thoughts
- Thoughts persist and can be recalled in future sessions
- Tests: prompt builder includes think instructions when think tool is registered

**Files:** `crates/arawn-agent/src/prompt/`