---
id: wire-api-v1-memory-search-to
level: task
title: "Wire /api/v1/memory/search to actual MemoryStore"
short_code: "ARAWN-T-0109"
created_at: 2026-02-01T03:32:01.021850+00:00
updated_at: 2026-02-02T01:26:30.067564+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#bug"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Wire /api/v1/memory/search to actual MemoryStore

## Bug

The `/api/v1/memory/search` endpoint (`crates/arawn-server/src/routes/memory.rs:187`) is a stub that only searches in-memory notes via substring match. It does not query the actual `MemoryStore` SQLite database where the indexer writes facts, summaries, and entities.

The indexer successfully extracts and stores memories on session close, but there is no way to retrieve them through the API.

## Current Behavior

`GET /api/v1/memory/search?q=Rust` returns empty results even though the `memories` table has matching facts.

## Expected Behavior

The endpoint should query the `MemoryStore` (semantic vector search + text match), returning indexed facts, summaries, and entities with relevance scores.

## Fix

- Add `MemoryStore` (or `Arc<MemoryStore>`) to `AppState` (the indexer already holds one, but the search handler needs access)
- Replace the stub note-search logic in `memory_search_handler` with actual `MemoryStore::search()` or equivalent
- Return scored results from the real store

## Files

- `crates/arawn-server/src/routes/memory.rs` — stub handler
- `crates/arawn-server/src/state.rs` — needs shared MemoryStore reference