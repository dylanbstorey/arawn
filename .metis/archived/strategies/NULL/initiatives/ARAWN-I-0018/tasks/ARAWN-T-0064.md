---
id: server-api-workstream-rest
level: task
title: "Server API: Workstream REST Endpoints"
short_code: "ARAWN-T-0064"
created_at: 2026-01-29T03:51:31.396374+00:00
updated_at: 2026-01-29T04:15:21.634151+00:00
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

# Server API: Workstream REST Endpoints

## Parent Initiative

[[ARAWN-I-0018]]

## Objective

Expose workstream operations via the arawn-server HTTP API so clients (SMS gateway, web UI, CLI) can create workstreams, send messages, list history, and manage sessions over REST.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `POST /api/v1/workstreams` — create a new workstream (title, default_model, tags)
- [ ] `GET /api/v1/workstreams` — list all workstreams (id, title, status, last_active)
- [ ] `GET /api/v1/workstreams/:id` — get workstream details + active session info
- [ ] `DELETE /api/v1/workstreams/:id` — archive a workstream
- [ ] `POST /api/v1/workstreams/:id/messages` — send a message (routes through agent, returns response)
- [ ] `GET /api/v1/workstreams/:id/messages` — list message history (with optional `since` query param)
- [ ] `POST /api/v1/workstreams/:id/promote` — promote scratch to named workstream
- [ ] Existing `POST /api/v1/chat` updated to accept optional `workstream_id` field, defaults to scratch
- [ ] Error responses for missing workstream, invalid operations
- [ ] Integration test: create workstream → send message → get history → verify

## Implementation Notes

### Technical Approach

New route module `crates/arawn-server/src/routes/workstreams.rs`. Mount under `/api/v1/workstreams`. All routes delegate to `WorkstreamManager` (injected as Axum state). The existing `/api/v1/chat` route gets an optional `workstream_id` field — if present, messages go to that workstream; if absent, goes to scratch. Message send endpoint triggers the agent loop and returns the assistant response.

### Dependencies

- ARAWN-T-0062 (WorkstreamManager must be complete)

## Status Updates

*To be added during implementation*