---
id: add-server-workstream-route
level: task
title: "Add server workstream route integration tests"
short_code: "ARAWN-T-0282"
created_at: 2026-03-08T03:17:26.801568+00:00
updated_at: 2026-03-08T03:17:26.801568+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#tech-debt"


exit_criteria_met: false
initiative_id: NULL
---

# Add server workstream route integration tests

## Objective

The entire workstream REST API (`/api/v1/workstreams/*`) has **zero test coverage** in arawn-server. This is the API the TUI and CLI use for workstream CRUD, session management, and message operations. Write integration tests covering all routes.

### Priority
- [x] P1 - High (untested API surface used by TUI daily)
- **Size**: M

### Current Problems
- The session delete 404 bug we just fixed would have been caught by these tests
- No validation of request/response schemas for workstream endpoints
- No testing of workstream-session relationships
- Tags, archiving, message listing all untested

## Acceptance Criteria

- [ ] Test file: `crates/arawn-server/tests/workstream_integration.rs`
- [ ] Uses existing `TestServer` from `tests/common/mod.rs`
- [ ] Scenarios covered:
  - [ ] `POST /api/v1/workstreams` — create with title, model, tags
  - [ ] `GET /api/v1/workstreams` — list active workstreams
  - [ ] `GET /api/v1/workstreams/:id` — get single workstream
  - [ ] `PATCH /api/v1/workstreams/:id` — update title, summary, model
  - [ ] `DELETE /api/v1/workstreams/:id` — archive workstream
  - [ ] `GET /api/v1/workstreams/:id/sessions` — list sessions for workstream
  - [ ] `DELETE /api/v1/sessions/:id` — delete session (from cache + persistent store)
  - [ ] `POST /api/v1/workstreams/:id/messages` — send message to workstream
  - [ ] `GET /api/v1/workstreams/:id/messages` — list messages
  - [ ] Scratch workstream auto-creation behavior
  - [ ] Cannot archive scratch workstream (error case)
  - [ ] 404 for non-existent workstream
  - [ ] Session reassignment between workstreams

## Implementation Notes

### Test server setup

The existing `TestServer` needs workstream support. Currently it creates an in-memory server without `WorkstreamManager`. Need to add:
```rust
TestServer::builder()
    .with_workstreams()  // Enables WorkstreamManager with in-memory SQLite
    .build()
    .await
```

### Test pattern
```rust
#[tokio::test]
async fn test_create_workstream() {
    let server = TestServer::builder().with_workstreams().build().await;
    let resp = server.post("/api/v1/workstreams")
        .json(&json!({"title": "Test Project", "tags": ["rust"]}))
        .send().await.unwrap();
    assert_eq!(resp.status(), 201);
    let ws: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(ws["title"], "Test Project");
    assert!(ws["id"].is_string());
}
```

### Dependencies
- May need to extend TestServer with workstream support (could be done inline or via ARAWN-T-0279)

## Status Updates

*To be added during implementation*