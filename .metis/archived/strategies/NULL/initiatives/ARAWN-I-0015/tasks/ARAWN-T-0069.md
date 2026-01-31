---
id: dimension-migration-detection-and
level: task
title: "Dimension Migration Detection and Reindex Workflow"
short_code: "ARAWN-T-0069"
created_at: 2026-01-29T04:43:47.203324+00:00
updated_at: 2026-01-29T15:01:47.448033+00:00
parent: ARAWN-I-0015
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0015
---

# Dimension Migration Detection and Reindex Workflow

## Parent Initiative

[[ARAWN-I-0015]]

## Objective

Detect when the configured embedding dimensions differ from the existing sqlite-vec table, warn the user, and provide a programmatic reindex workflow that drops the old vector table and re-embeds all memories using the current provider.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `MemoryStore` stores current embedding dimensions + provider name in a metadata table (or SQLite pragma)
- [ ] On `init_vectors()`, compare configured dimensions with stored dimensions
- [ ] If mismatch: log a `tracing::warn!` with clear message and disable vector search (return empty results)
- [ ] `reindex()` method on `MemoryStore`: drops `memory_embeddings` table, recreates with new dimensions, re-embeds all memories
- [ ] `reindex()` accepts a `SharedEmbedder` and `BatchEmbedder` for the actual embedding work
- [ ] Dry-run mode: returns count of memories and estimated API cost without doing work
- [ ] Unit tests for mismatch detection and reindex flow
- [ ] All existing tests pass

## Implementation Notes

### Technical Approach
- Add `embedding_metadata` table to arawn-memory schema: `(key TEXT PRIMARY KEY, value TEXT)` storing `dimensions`, `provider`, `model`
- On `init_vectors(dims)`: check stored dims vs requested dims. If different, set `vectors_stale = true` flag
- `reindex(embedder, batch_embedder)`: read all memories, batch-embed, drop+recreate vector table, insert new embeddings, update metadata
- Dry-run: just count memories and estimate tokens (content.len() / 4 as rough token estimate)

### Dependencies
- T-0067 (BatchEmbedder) for the actual batch processing
- T-0068 (Config) for knowing current provider/dimensions

## Status Updates

*To be added during implementation*