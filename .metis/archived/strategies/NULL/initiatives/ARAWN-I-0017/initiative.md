---
id: session-intelligence-indexing
level: initiative
title: "Session Intelligence: Indexing, Summarization, and Memory Confidence"
short_code: "ARAWN-I-0017"
created_at: 2026-01-29T02:28:40.299210+00:00
updated_at: 2026-02-01T03:51:20.848669+00:00
parent: ARAWN-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: L
strategy_id: NULL
initiative_id: session-intelligence-indexing
---

# Session Intelligence: Indexing, Summarization, and Memory Confidence Initiative

## Context

ARAWN-I-0014 covers the turn-level memory capabilities (think tool, active recall). This initiative covers the background intelligence layer that runs after sessions end and manages memory quality over time.

Completed sessions currently sit inert — no facts extracted, no entities identified, no summary generated. The knowledge graph gets populated only through explicit `note` tool usage. Meanwhile, every conversation contains implicit facts, preferences, decisions, and entity references that should be captured automatically.

More critically, Arawn has no model for memory confidence. All memories are treated equally regardless of how many times they've been reinforced, whether they've been contradicted, or how stale they are. When a user says "my API key is X" and later "my API key is now Y," both memories persist with equal weight. The system needs cache invalidation — a mechanism to detect contradictions and supersede stale facts.

The LLM backend used for entity/fact extraction is configurable via `llm.services` in `config.toml`, so users can choose the cheapest backend appropriate for extraction tasks (e.g., Groq, local Ollama, or a dedicated extraction model).

## Goals & Non-Goals

**Goals:**
- Post-session entity and fact extraction using a configurable LLM backend
- Session summarization on close (stored as searchable memory)
- Memory confidence model based on reinforcement, contradiction, and staleness
- Cache invalidation: new facts automatically supersede contradictory old facts
- Configurable indexing: enable/disable, choose LLM backend and model
- Knowledge graph population from extracted entities and relationships

**Non-Goals:**
- Turn-level memory (think tool, active recall) — that's ARAWN-I-0014
- Changing the embedding model/provider — that's ARAWN-I-0015
- Multi-user memory isolation
- Long-term memory consolidation/forgetting strategies (future work)
- RAG over external documents (future work)

## Detailed Design

### 1. Post-Session Indexing

When a session ends (explicit close or timeout), a background task runs:

```rust
pub struct SessionIndexer {
    memory: Arc<MemoryStore>,
    llm: Arc<dyn LlmBackend>,  // configured via [memory.indexing] section
}

impl SessionIndexer {
    pub async fn index_session(&self, session_id: SessionId) -> Result<IndexReport> {
        let history = self.memory.get_session_history(session_id).await?;
        
        // 1. Extract structured data via LLM
        let extraction = self.extract(&history).await?;
        
        // 2. Store entities in knowledge graph
        for entity in &extraction.entities {
            self.memory.graph().add_node(entity).await?;
        }
        
        // 3. Store facts with confidence metadata
        for fact in &extraction.facts {
            self.store_fact_with_confidence(fact).await?;
        }
        
        // 4. Add relationships
        for rel in &extraction.relationships {
            self.memory.graph().add_edge(rel).await?;
        }
        
        // 5. Generate and store summary
        let summary = self.summarize(&history).await?;
        self.memory.store_summary(session_id, &summary).await?;
        
        Ok(report)
    }
}
```

The extraction prompt asks the LLM to output structured JSON:
```json
{
  "entities": [
    {"name": "Groq", "type": "service", "context": "LLM API provider"},
    {"name": "arawn-agent", "type": "crate", "context": "core agent crate"}
  ],
  "facts": [
    {"subject": "user", "predicate": "prefers", "object": "Groq for simple tasks", "confidence": "stated"},
    {"subject": "arawn", "predicate": "has_tool_count", "object": "7", "confidence": "observed"}
  ],
  "relationships": [
    {"from": "arawn-agent", "relation": "depends_on", "to": "arawn-llm"}
  ]
}
```

### 2. Memory Confidence Model

Each memory gets a confidence score that determines its weight in recall:

```rust
pub struct MemoryConfidence {
    /// How the fact was established
    source: ConfidenceSource,
    /// Number of times this fact was reinforced (referenced/confirmed)
    reinforcement_count: u32,
    /// Whether this fact has been contradicted by newer information
    superseded: bool,
    /// ID of the memory that superseded this one (if any)
    superseded_by: Option<MemoryId>,
    /// Last time this memory was accessed/referenced
    last_accessed: DateTime<Utc>,
    /// Computed confidence score (0.0 - 1.0)
    score: f32,
}

pub enum ConfidenceSource {
    Stated,      // user explicitly said it ("my key is X")
    Observed,    // derived from behavior (user always uses Groq)
    Inferred,    // extracted by indexing pipeline
    System,      // set by the system (e.g., config values)
}
```

Confidence scoring:
```
score = base_score(source) 
      * reinforcement_boost(reinforcement_count)
      * staleness_penalty(last_accessed)
      * (if superseded: 0.0 else: 1.0)

base_score: Stated=1.0, System=0.9, Observed=0.7, Inferred=0.5
reinforcement_boost: min(1.0 + 0.1 * count, 1.5)
staleness_penalty: 1.0 for <30 days, linear decay to 0.5 at 365 days
```

Superseded memories get score 0.0 — they're effectively invalidated but kept for audit trail.

### 3. Cache Invalidation (Contradiction Detection)

When a new fact is stored, check for contradictions:

```rust
impl MemoryStore {
    async fn store_fact(&self, new_fact: &Fact) -> Result<()> {
        // 1. Find existing facts with same subject+predicate
        let existing = self.find_facts(
            &new_fact.subject, 
            &new_fact.predicate
        ).await?;
        
        // 2. If object differs, supersede the old fact
        for old in existing {
            if old.object != new_fact.object {
                self.supersede(&old.id, &new_fact.id).await?;
                tracing::info!(
                    old = %old.object, new = %new_fact.object,
                    "Memory superseded: {}.{}", new_fact.subject, new_fact.predicate
                );
            }
        }
        
        // 3. If object matches, reinforce
        for old in existing {
            if old.object == new_fact.object {
                self.reinforce(&old.id).await?;
            }
        }
        
        // 4. Store the new fact
        self.insert_fact(new_fact).await?;
    }
}
```

This handles the "my API key changed" case: new fact supersedes old, old gets score 0.0, recall only surfaces the current value.

### 4. Session Summarization

On session close, generate a concise summary:

```rust
async fn summarize(&self, history: &[Message]) -> Result<String> {
    let prompt = format!(
        "Summarize this conversation in 2-3 sentences. Focus on:\n\
         - What was accomplished\n\
         - Key decisions made\n\
         - Open questions or next steps\n\n\
         Conversation:\n{}",
        format_history(history)
    );
    let response = self.llm.complete(&prompt).await?;
    Ok(response.text)
}
```

Summaries are stored as `ContentType::Summary` memories with the session ID as metadata. This enables "what did we work on yesterday?" queries.

### 5. Configuration

```toml
[memory.indexing]
enabled = true
backend = "groq"                    # references [llm.services.groq]
model = "llama-3.3-70b-versatile"   # model for extraction/summarization

[memory.confidence]
staleness_days = 365        # days before staleness penalty reaches minimum
staleness_floor = 0.5       # minimum staleness multiplier
reinforcement_cap = 1.5     # max reinforcement boost
```

### 6. Recall Integration

The confidence score feeds into the existing `recall()` scoring:

```
final_score = vector_similarity * 0.4
            + graph_relevance * 0.3
            + confidence.score * 0.3
```

Superseded memories (score 0.0) are automatically excluded from recall results.

## Alternatives Considered

- **Local NER only (no LLM)**: Could use spaCy-style NER models for entity extraction. Misses nuanced facts like preferences and decisions. The configurable LLM backend lets users choose the cost/quality tradeoff.
- **Simple temporal decay instead of confidence model**: `exp(-decay * days)` is simpler but doesn't handle contradictions, reinforcement, or the difference between stated and inferred facts. The confidence model is more work but solves real problems (cache invalidation).
- **Automatic reindexing of all sessions**: Could retroactively index old sessions when the extraction model improves. Deferred — expensive and unclear value. Focus on new sessions first.
- **User-confirmed facts only**: Only store facts the user explicitly confirms. Too conservative — many useful facts are implicit in conversation. The confidence model's `Inferred` source type handles this by giving inferred facts lower base scores.

## Implementation Plan

1. Add `MemoryConfidence` struct and `ConfidenceSource` enum to `arawn-memory`
2. Add confidence columns to memory SQLite schema
3. Implement contradiction detection and supersession logic
4. Implement reinforcement tracking
5. Update `recall()` scoring to incorporate confidence
6. Implement `SessionIndexer` with LLM-based extraction
7. Add extraction prompt and structured JSON parsing
8. Implement session summarization
9. Wire indexing as background task on session close
10. Add `[memory.indexing]` and `[memory.confidence]` config sections
11. Integration tests for contradiction detection and indexing pipeline