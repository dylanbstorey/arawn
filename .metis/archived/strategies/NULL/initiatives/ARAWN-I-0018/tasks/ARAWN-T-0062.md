---
id: workstreammanager-unified-api-and
level: task
title: "WorkstreamManager: Unified API and Agent Message Push"
short_code: "ARAWN-T-0062"
created_at: 2026-01-29T03:51:30.102134+00:00
updated_at: 2026-01-29T04:09:16.634013+00:00
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

# WorkstreamManager: Unified API and Agent Message Push

## Parent Initiative

[[ARAWN-I-0018]]

## Objective

Build `WorkstreamManager` — the high-level facade that coordinates the message store, session manager, workstream store, and scratch logic into a single API. Also implement agent message push (the `AgentPush` role) so background processes can write into workstreams.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `WorkstreamManager` struct holding `WorkstreamStore`, `MessageStore`, `SessionManager`
- [ ] `WorkstreamManager::new(config)` initializes SQLite, runs migrations, sets up data dirs
- [ ] `send_message(workstream_id, role, content)` → appends to JSONL, ensures active session exists, returns message
- [ ] `push_agent_message(workstream_id, content)` → sends a message with `AgentPush` role (for background tasks, cron jobs)
- [ ] `create_workstream(title, default_model, tags)` → creates workstream in SQLite + JSONL dir
- [ ] `list_workstreams()` → returns all non-archived workstreams
- [ ] `get_workstream(id)` → returns workstream metadata + active session info
- [ ] `delete_workstream(id)` → archives workstream (does not delete JSONL)
- [ ] Scratch workstream auto-created on first `send_message` without explicit workstream_id
- [ ] Unit tests: full send/receive cycle, agent push, workstream CRUD, scratch auto-create

## Implementation Notes

### Technical Approach

`WorkstreamManager` in `crates/arawn-workstream/src/manager.rs`. This is the only public entry point other crates need to use. It owns the store, message store, and session manager. `send_message` flow: resolve workstream (scratch if none) → get or start session → append message to JSONL → return. `push_agent_message` is the same flow but with `AgentPush` role and no session requirement (creates one if needed). The manager is `Clone + Send + Sync` (wraps internals in `Arc`).

### Dependencies

- ARAWN-T-0058 (store)
- ARAWN-T-0059 (message store)
- ARAWN-T-0060 (session manager)
- ARAWN-T-0061 (scratch logic)

## Status Updates

*To be added during implementation*