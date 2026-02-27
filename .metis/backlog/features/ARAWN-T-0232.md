---
id: wire-workstream-session-compressor
level: task
title: "Wire workstream session compressor into server"
short_code: "ARAWN-T-0232"
created_at: 2026-02-27T00:14:15.788585+00:00
updated_at: 2026-02-27T00:14:15.788585+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#feature"


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

- [ ] When a session ends, the compressor auto-summarizes it (if message count/size warrants it)
- [ ] Workstream summary updated after session compression (reduce step)
- [ ] `CompressorConfig` wired from `arawn.toml` (model, max tokens, threshold)
- [ ] Compression runs asynchronously (doesn't block session end response)
- [ ] Summaries stored in SQLite via existing `update_session_summary` / `update_workstream` methods
- [ ] API endpoint or CLI command to trigger manual compression

## Implementation Notes

### Existing Code (fully implemented + tested)
- `arawn-workstream/src/compression.rs` — `Compressor`, `CompressorConfig`, `compress_session()`, `compress_workstream()`, `needs_compression()`
- Tests cover: below/above threshold, session compression, workstream reduce, active session rejection

### Integration Points
- **Session end hook**: When `end_session()` is called, spawn a background task to compress
- **`start.rs`**: Create `Compressor` with an LLM backend and wire config
- **Config**: Add `[workstream.compression]` section to `arawn.toml` with `model`, `max_summary_tokens`, `token_threshold_chars`

## Status Updates

*To be added during implementation*