---
id: server-api-routing-profile-field
level: task
title: "Server API: routing_profile Field and Multi-Backend Integration Tests"
short_code: "ARAWN-T-0057"
created_at: 2026-01-29T02:40:18.146486+00:00
updated_at: 2026-01-29T02:40:18.146486+00:00
parent: ARAWN-I-0011
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0011
---

# Server API: routing_profile Field and Multi-Backend Integration Tests

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[ARAWN-I-0011]]

## Objective

Add `routing_profile` field to the server `ChatRequest` so API callers can override routing, and write integration tests verifying end-to-end routing with mock backends.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `ChatRequest` gains optional `routing_profile: Option<String>` field
- [ ] Chat route handler passes `routing_profile` into `CompletionRequest.metadata` for the router to read
- [ ] Integration test: mock "fast" and "capable" backends, send simple request → routed to fast
- [ ] Integration test: send request with `routing_profile: "capable"` → caller hint overrides heuristic
- [ ] Integration test: send complex request (long message + tools) → routed to capable
- [ ] Integration test: single-backend config → all requests go to default, no error
- [ ] Verify interaction JSONL files written during integration tests contain correct routing metadata
- [ ] `cargo test` passes

## Implementation Notes

### Technical Approach
- Add `routing_profile` to `ChatRequest` in `arawn-server/src/routes/chat.rs`
- In chat handler: insert into `CompletionRequest.metadata` before passing to agent
- Integration tests: create `MockBackend` implementing `LlmBackend` that records which requests it received. Create two mock backends, wrap in `RoutingBackend`, verify correct mock received each request.
- Use `tempdir` for interaction log output in tests

### Dependencies
- ARAWN-T-0052 (InteractionLogger)
- ARAWN-T-0054 (RoutingPolicy)
- ARAWN-T-0055 (RoutingBackend)
- ARAWN-T-0056 (config)

## Status Updates **[REQUIRED]**

*To be added during implementation*