---
id: command-rest-api
level: task
title: "Command REST API"
short_code: "ARAWN-T-0187"
created_at: 2026-02-16T18:54:50.699638+00:00
updated_at: 2026-02-17T01:39:55.869376+00:00
parent: ARAWN-I-0026
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


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

## Acceptance Criteria

## Acceptance Criteria

- [x] `CommandHandler` trait with `name()`, `description()`, `execute()`
- [x] `CommandRegistry` for registering and looking up handlers
- [x] `GET /api/v1/commands` - list available commands
- [x] `POST /api/v1/commands/compact` - execute compact command
- [x] SSE streaming for progress updates (`POST /api/v1/commands/compact/stream`)
- [x] CompactCommand handler wired to SessionCompactor
- [x] Error handling with proper HTTP status codes

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

### 2026-02-17: Implementation Complete

**Files Created/Modified:**
- `crates/arawn-server/src/routes/commands.rs` (new) - 550+ lines
- `crates/arawn-server/src/routes/mod.rs` - added commands module and exports
- `crates/arawn-server/src/lib.rs` - wired command routes
- `crates/arawn-server/Cargo.toml` - added `async-trait = "0.1"`

**Implementation Details:**

1. **CommandHandler trait** - async trait with `name()`, `description()`, `execute()` methods
2. **CommandRegistry** - thread-safe registry using `Arc<RwLock<HashMap>>` for handler lookup
3. **SharedCommandRegistry** - type alias for `Arc<CommandRegistry>`
4. **CompactCommand** - handler that uses SessionCompactor from ARAWN-T-0186
5. **CommandError** - dedicated error type with `not_found`, `invalid_params`, `execution_failed`

**REST Endpoints:**
- `GET /api/v1/commands` → `ListCommandsResponse` with command info
- `POST /api/v1/commands/compact` → `CompactResponse` (sync execution)
- `POST /api/v1/commands/compact/stream` → SSE stream with `CompactEvent` progress

**SSE Events:**
- `CompactEvent::Started` - compaction initiated
- `CompactEvent::Progress` - intermediate updates
- `CompactEvent::Completed` - final result with `CompactionResult`
- `CompactEvent::Error` - failure with error message

**Tests:** 13 unit tests covering registry, handler, and error handling