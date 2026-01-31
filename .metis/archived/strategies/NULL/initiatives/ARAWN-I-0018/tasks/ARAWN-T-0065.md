---
id: context-compression-map-reduce
level: task
title: "Context Compression: Map-Reduce Session and Workstream Summaries"
short_code: "ARAWN-T-0065"
created_at: 2026-01-29T03:51:32.031552+00:00
updated_at: 2026-01-29T04:20:47.272713+00:00
parent: ARAWN-I-0018
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0018
---

# Context Compression: Map-Reduce Session and Workstream Summaries

## Parent Initiative

[[ARAWN-I-0018]]

## Objective

Implement map-reduce context compression. When a session ends, summarize it (map). When the workstream needs a fresh summary, reduce all session summaries into a single workstream summary. Triggers: session end, token threshold mid-session, startup, manual API call.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `Compressor` struct that uses an LLM backend to generate summaries
- [ ] `compress_session(session_id)` — reads session messages from JSONL, sends to LLM with summarization prompt, writes summary to session record in SQLite
- [ ] `compress_workstream(workstream_id)` — reads all session summaries, reduces into a single workstream summary, writes to workstream record
- [ ] Summarization prompts focus on: objective of the workstream, current state, key decisions, open questions
- [ ] Token threshold trigger: if current session messages exceed configurable token count, compress older messages in-session
- [ ] Session end trigger: automatically compress session on `end_session`
- [ ] Startup trigger: on workstream resume, check if workstream summary is stale and recompress
- [ ] Manual trigger: `POST /api/v1/workstreams/:id/compress` endpoint
- [ ] Compressed summaries stored in SQLite (session.summary, workstream.summary columns)
- [ ] Unit tests: session compression produces summary, workstream reduce merges session summaries, threshold detection

## Implementation Notes

### Technical Approach

`Compressor` in `crates/arawn-workstream/src/compression.rs`. Takes a `SharedBackend` for LLM calls. Session compression prompt: "Summarize this conversation session. Focus on: what was discussed, decisions made, action items, and current state." Workstream compression prompt: "Given these session summaries for workstream '{title}', produce a unified summary covering: the workstream's purpose, current progress, key decisions, and open items." The compressor is async since it makes LLM calls. For batch API support in the future, the interface should return a `Future` that can be dispatched to a batch queue.

### Dependencies

- ARAWN-T-0060 (session lifecycle — need session end hook)
- ARAWN-T-0062 (WorkstreamManager for coordinating compression triggers)

## Status Updates

*To be added during implementation*