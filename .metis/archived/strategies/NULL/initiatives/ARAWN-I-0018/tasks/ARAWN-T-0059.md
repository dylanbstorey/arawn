---
id: workstream-data-model-and-jsonl
level: task
title: "Workstream Data Model and JSONL Message Store"
short_code: "ARAWN-T-0059"
created_at: 2026-01-29T03:51:28.195295+00:00
updated_at: 2026-01-29T04:04:00.905898+00:00
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

# Workstream Data Model and JSONL Message Store

## Parent Initiative

[[ARAWN-I-0018]]

## Objective

Define the in-memory Rust types for workstreams and messages, and implement the JSONL message store that serves as the source of truth for all conversation history.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `Workstream` struct with id, title, summary, default_model, status, created_at, updated_at
- [ ] `WorkstreamMessage` struct with id, workstream_id, session_id, role (User, Assistant, System, AgentPush), content, timestamp, metadata (JSON string)
- [ ] `MessageRole` enum: User, Assistant, System, ToolResult, AgentPush
- [ ] `MessageStore` that appends messages as newline-delimited JSON to per-workstream JSONL files
- [ ] `MessageStore::append(workstream_id, message)` writes to `{data_dir}/workstreams/{id}/messages.jsonl`
- [ ] `MessageStore::read_all(workstream_id)` reads and deserializes full JSONL history
- [ ] `MessageStore::read_range(workstream_id, since: DateTime)` reads messages after a timestamp
- [ ] File-level locking or append-only safety for concurrent writes
- [ ] Unit tests: roundtrip write/read, multi-message append, range query, missing file returns empty vec

## Implementation Notes

### Technical Approach

Types live in `crates/arawn-workstream/src/types.rs`. MessageStore lives in `crates/arawn-workstream/src/message_store.rs`. JSONL files are organized as `{data_dir}/workstreams/{workstream_id}/messages.jsonl`. Each line is a self-contained JSON object (one `WorkstreamMessage`). Append uses `OpenOptions::new().create(true).append(true)` for atomic appends. `read_range` uses a linear scan with timestamp filter — acceptable for now, optimize later if needed.

### Dependencies

- ARAWN-T-0058 (crate scaffold and schema must exist first)

## Status Updates

*To be added during implementation*