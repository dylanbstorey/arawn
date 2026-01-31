# Session Indexing

Automatic knowledge extraction when sessions close.

## Overview

When a session ends, the indexing pipeline extracts:
- **Entities** — People, projects, concepts mentioned
- **Facts** — Specific pieces of information
- **Relationships** — How entities connect
- **Summary** — High-level session summary

## Pipeline Stages

```
Session Close
     │
     ▼
┌─────────────────────────────────────────┐
│ STAGE 1: EXTRACTION                      │
│                                          │
│ [if NER engine available]:               │
│   • Run local NER for entities/relations │
│   • LLM call for facts only              │
│                                          │
│ [else LLM-only]:                         │
│   • Single LLM call for all extraction   │
└─────────────────────────────────────────┘
     │
     ▼
┌─────────────────────────────────────────┐
│ STAGE 2: STORE ENTITIES                  │
│                                          │
│ • Add to graph database                  │
│ • Merge with existing entities           │
└─────────────────────────────────────────┘
     │
     ▼
┌─────────────────────────────────────────┐
│ STAGE 3: STORE FACTS                     │
│                                          │
│ For each fact:                           │
│   • Generate embedding                   │
│   • Check for contradictions             │
│   • Insert/reinforce/supersede           │
└─────────────────────────────────────────┘
     │
     ▼
┌─────────────────────────────────────────┐
│ STAGE 4: STORE RELATIONSHIPS             │
│                                          │
│ • Add edges to graph                     │
│ • Link entities discovered               │
└─────────────────────────────────────────┘
     │
     ▼
┌─────────────────────────────────────────┐
│ STAGE 5: SUMMARIZE                       │
│                                          │
│ • LLM generates session summary          │
│ • Embed and store as special memory      │
└─────────────────────────────────────────┘
     │
     ▼
  IndexReport
```

## Extraction Methods

### LLM-Only Extraction

Default method using a single LLM call:

```json
{
  "entities": [
    {"name": "Arawn", "type": "project"},
    {"name": "David", "type": "person"}
  ],
  "facts": [
    {"content": "Arawn is written in Rust", "subject": "Arawn", "predicate": "written_in"},
    {"content": "David created Arawn", "subject": "David", "predicate": "created"}
  ],
  "relationships": [
    {"source": "David", "target": "Arawn", "type": "created"}
  ]
}
```

### Hybrid NER + LLM

When GLiNER is enabled, extraction is split:

1. **Local NER** — Fast entity/relationship extraction
2. **LLM** — Fact extraction and validation

This reduces LLM costs for entity-heavy sessions.

## Configuration

```toml
[memory.indexing]
enabled = true             # Enable session indexing
backend = "default"        # LLM backend for extraction
model = "gpt-4o-mini"      # Model for extraction/summarization
```

### Disabling Indexing

For sessions where indexing isn't desired:

```bash
arawn chat --no-index
```

Or via API:

```json
{
  "message": "Hello",
  "session_options": {
    "skip_indexing": true
  }
}
```

## Index Report

After indexing completes, a report is generated:

```rust
pub struct IndexReport {
    pub session_id: String,
    pub entities_added: usize,
    pub facts_added: usize,
    pub facts_reinforced: usize,
    pub facts_superseded: usize,
    pub relationships_added: usize,
    pub summary_stored: bool,
    pub duration_ms: u64,
}
```

## Background Processing

Indexing runs asynchronously after session close:

1. Client receives `204 No Content` immediately
2. Indexing spawns as background task
3. Results logged but not returned to client

### Monitoring

Check indexing status via logs:

```bash
tail -f ~/.arawn/arawn.log | grep "index"
```

Or via the memory API:

```bash
curl http://localhost:8080/api/v1/memory/stats
```

## Extraction Prompts

### Entity/Fact Extraction

```
Analyze this conversation and extract:
1. Named entities (people, projects, technologies, concepts)
2. Factual statements about these entities
3. Relationships between entities

Focus on information worth remembering long-term.
Ignore session-specific or trivial details.
```

### Summarization

```
Summarize this conversation in 2-3 sentences.
Focus on:
- Main topics discussed
- Key decisions made
- Important outcomes
```

## Error Handling

| Error | Behavior |
|-------|----------|
| LLM API failure | Log error, skip indexing |
| Parse failure | Log warning, continue with partial results |
| Storage failure | Log error, retry once |
| Timeout | Cancel and log warning |
