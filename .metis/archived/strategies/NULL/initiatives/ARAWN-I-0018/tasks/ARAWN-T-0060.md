---
id: session-lifecycle-start-end
level: task
title: "Session Lifecycle: Start, End, Timeout, and Summary Records"
short_code: "ARAWN-T-0060"
created_at: 2026-01-29T03:51:28.831671+00:00
updated_at: 2026-01-29T04:05:44.813563+00:00
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

# Session Lifecycle: Start, End, Timeout, and Summary Records

## Parent Initiative

[[ARAWN-I-0018]]

## Objective

Implement session lifecycle management — starting, ending, and timing out sessions within workstreams. Sessions are "turn batches" that group a period of activity. On session end, a summary record is written to SQLite for later compression.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `Session` struct with id, workstream_id, started_at, ended_at, status (active/ended/timed_out), message_count, summary (optional)
- [ ] `SessionManager::start_session(workstream_id)` creates a new session row in SQLite, returns session_id
- [ ] `SessionManager::end_session(session_id)` marks session ended, writes message_count
- [ ] `SessionManager::timeout_check()` ends sessions idle beyond configurable threshold (e.g. 30 min)
- [ ] Session summary field populated on end (placeholder — actual compression in T-0065)
- [ ] `WorkstreamStore` methods: `create_session`, `end_session`, `get_active_session`, `list_sessions(workstream_id)`
- [ ] Only one active session per workstream at a time
- [ ] Unit tests: start/end lifecycle, timeout detection, one-active constraint, list sessions

## Implementation Notes

### Technical Approach

`SessionManager` in `crates/arawn-workstream/src/session.rs`. Uses `WorkstreamStore` for all SQLite operations. `timeout_check` scans for sessions where `started_at + threshold < now` and `ended_at IS NULL`. On session end, count messages from JSONL between session start and end timestamps. Summary field is left NULL for now — T-0065 will populate it via compression.

### Dependencies

- ARAWN-T-0058 (schema with sessions table)
- ARAWN-T-0059 (message store for counting messages)

## Status Updates

*To be added during implementation*