---
id: command-rest-api
level: task
title: "Command REST API"
short_code: "ARAWN-T-0187"
created_at: 2026-02-16T18:54:50.699638+00:00
updated_at: 2026-02-16T18:54:50.699638+00:00
parent: ARAWN-I-0026
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0026
---

# Command REST API

## Parent Initiative

[[ARAWN-I-0026]] - Context Management and Auto-Compaction

## Objective

Implement server-side command infrastructure with REST API endpoints. Commands are a server concept; `/` syntax is client presentation.

## Acceptance Criteria

- [ ] `CommandHandler` trait with `name()`, `description()`, `execute()`
- [ ] `CommandRegistry` for registering and looking up handlers
- [ ] `GET /api/v1/commands` - list available commands
- [ ] `POST /api/v1/commands/compact` - execute compact command
- [ ] SSE streaming for progress updates
- [ ] CompactCommand handler wired to SessionCompactor
- [ ] Error handling with proper HTTP status codes

## Implementation Notes

### Files to Modify
- `crates/arawn-server/src/routes/commands.rs` (new file)
- `crates/arawn-server/src/routes/mod.rs` - add routes

### Endpoints

```
GET  /api/v1/commands
     → { "commands": [{ "name": "compact", "description": "..." }] }

POST /api/v1/commands/compact
     Body: { "session_id": "...", "force": false }
     Response: SSE stream of progress, then final result
```

### Dependencies
- ARAWN-T-0186 (SessionCompactor)

## Tests

### Unit Tests
- `test_command_registry_register_and_lookup` - register handler, find by name
- `test_command_registry_list` - lists all registered commands
- `test_compact_command_handler` - CompactCommand executes correctly

### API Tests
- `test_get_commands_endpoint` - GET /api/v1/commands returns list
- `test_post_compact_success` - POST compact with valid session returns 200
- `test_post_compact_invalid_session` - POST compact with bad session returns 404
- `test_post_compact_sse_streaming` - verify SSE events in response

### Test File
- `crates/arawn-server/src/routes/commands.rs` (inline `#[cfg(test)]` module)

## Status Updates

*To be added during implementation*