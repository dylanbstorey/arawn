# Core Systems

The foundational systems that power Arawn's agentic capabilities.

## Overview

Arawn's core systems work together to provide intelligent, memory-enhanced agent behavior:

- **[Agent Loop](agent-loop.md)** — The agentic reasoning cycle with tool execution
- **[Memory](memory.md)** — Persistent storage, recall, and confidence scoring
- **[Session Indexing](indexing.md)** — Automatic knowledge extraction from conversations
- **[Workstreams](workstreams.md)** — Persistent conversation contexts

## System Interactions

```
┌─────────────────────────────────────────────────────────────────┐
│                        Agent Loop                                │
│                                                                  │
│  1. Receive message                                              │
│  2. Build context (history + recall)  ◀────── Memory System     │
│  3. Call LLM                                                     │
│  4. Execute tools                                                │
│  5. Loop until complete                                          │
│  6. Close session  ─────────────────────────▶ Session Indexer   │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                     Workstream Manager                           │
│                                                                  │
│  • Persistent contexts across sessions                           │
│  • JSONL message history                                         │
│  • SQLite metadata cache                                         │
└─────────────────────────────────────────────────────────────────┘
```

## Design Principles

1. **Memory-First** — Context from memory enhances every turn
2. **Automatic Indexing** — Knowledge extracted without user intervention
3. **Confidence-Aware** — Facts scored and ranked by reliability
4. **Workstream Continuity** — Long-running contexts preserved
