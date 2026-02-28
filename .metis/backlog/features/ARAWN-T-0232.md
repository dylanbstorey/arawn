---
id: wire-workstream-session-compressor
level: task
title: "Wire workstream session compressor into server"
short_code: "ARAWN-T-0232"
created_at: 2026-02-27T00:14:15.788585+00:00
updated_at: 2026-02-28T18:48:33.669691+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#feature"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Wire workstream session compressor into server

## Objective

`arawn-workstream/src/compression.rs` has a fully implemented and tested map-reduce `Compressor` that summarizes sessions via LLM and merges summaries into workstream-level context. It's exported from the crate but never wired into the server — the feature was built but the integration was never completed.

Wire the compressor into the server so sessions get auto-compressed when they end (or exceed a token threshold), and workstream summaries are available for context retrieval.

## Backlog Item Details

### Type
- [x] Feature - New functionality or enhancement

### Priority
- [x] P3 - Low (when time permits)

### Business Justification
- **User Value**: Long-running workstreams accumulate context. Compression keeps summaries available without re-reading entire JSONL histories. Enables efficient context retrieval for new sessions in the same workstream.
- **Effort Estimate**: M

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] When a session ends, the compressor auto-summarizes it (if message count/size warrants it)
- [x] Workstream summary updated after session compression (reduce step)
- [x] `CompressorConfig` wired from `arawn.toml` (backend/model pattern matching other LLM features)
- [x] Compression runs asynchronously (doesn't block session end response)
- [x] Summaries stored in SQLite via existing `update_session_summary` / `update_workstream` methods
- [x] API endpoint to trigger manual compression (`POST /api/v1/workstreams/{id}/compress`)

## Implementation Notes

### Existing Code (fully implemented + tested)
- `arawn-workstream/src/compression.rs` — `Compressor`, `CompressorConfig`, `compress_session()`, `compress_workstream()`, `needs_compression()`
- Tests cover: below/above threshold, session compression, workstream reduce, active session rejection

### Integration Points
- **Session end hook**: When `end_session()` is called, spawn a background task to compress
- **`start.rs`**: Create `Compressor` with an LLM backend and wire config
- **Config**: Add `[workstream.compression]` section to `arawn.toml` with `model`, `max_summary_tokens`, `token_threshold_chars`

## Status Updates

### Session 1 — Complete

**Config**: Follows same pattern as other LLM-consuming features (`backend` + `model` referencing `llm_profiles`):
```toml
[workstream.compression]
enabled = true
backend = "default"
model = "claude-sonnet-4-20250514"
max_summary_tokens = 1024
token_threshold_chars = 32000
```

**Changes:**
1. `arawn-config/src/types.rs` — Added `CompressionConfig` struct to `WorkstreamConfig` with `backend`/`model` pattern matching `IndexingConfig` and `CompactionConfig`
2. `arawn-server/src/state.rs` — Added `compressor: Option<Arc<Compressor>>` to `SharedServices`, `with_compressor()` builder, convenience accessor on `AppState`
3. `arawn-server/src/state.rs` `close_session()` — Captures `workstream_id` before cache removal, spawns background compression task (session compress → workstream reduce)
4. `arawn-server/src/routes/workstreams.rs` — Added `POST /api/v1/workstreams/{id}/compress` endpoint for manual compression
5. `arawn-server/src/routes/mod.rs` — Exported new handler and response type
6. `arawn-server/src/lib.rs` — Registered compress route
7. `arawn/src/commands/start.rs` — Resolves compression backend from `backends` HashMap, creates `Compressor`, wires into `AppState`

**Verification:**
- `angreal check all` — clean
- `angreal test unit` — all pass, 0 failures