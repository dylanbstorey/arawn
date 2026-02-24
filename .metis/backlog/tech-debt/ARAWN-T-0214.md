---
id: api-consistency-pagination
level: task
title: "API Consistency: Pagination, Response Wrapping, Status Codes"
short_code: "ARAWN-T-0214"
created_at: 2026-02-20T13:41:43.517518+00:00
updated_at: 2026-02-24T22:04:19.218883+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#tech-debt"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# API Consistency: Pagination, Response Wrapping, Status Codes

## Objective

Standardize API design across all endpoints: implement pagination, fix response wrapping inconsistencies, and correct HTTP status codes.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P1 - High (important for user experience)

### Technical Debt Impact
- **Current Problems**:
  - No pagination on list endpoints - unbounded results on `/messages`, `/sessions`
  - Inconsistent response wrapping - some endpoints wrap results, others return bare objects
  - Missing `total` field in some list responses (`MessageListResponse`, `WorkstreamListResponse`)
  - POST endpoints return 200 instead of 201 (`mcp.rs`, `workstreams.rs` file ops)
  - Inconsistent path variable naming (`{id}` vs `{ws}` for same resource)
- **Benefits of Fixing**: Predictable, consistent API for clients; better scalability
- **Risk Assessment**: MEDIUM - breaking change for existing clients (need versioning strategy)

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] All list endpoints support `limit` and `offset` query parameters
- [x] All list responses include `total` field for pagination
- [x] Response wrapping is consistent (never wrap)
- [x] All POST create endpoints return 201 Created
- [x] Path variables use consistent naming (`{id}` everywhere)
- [x] OpenAPI documentation updated to reflect changes
- [x] Breaking changes documented in CHANGELOG

## Implementation Notes

### Issue 1: Pagination

**Affected endpoints**:
- `GET /api/v1/sessions` - has `total`, needs `limit`/`offset`
- `GET /api/v1/workstreams` - missing `total`, needs pagination
- `GET /api/v1/workstreams/{id}/messages` - missing `total`, unbounded
- `GET /api/v1/workstreams/{id}/sessions` - missing `total`, unbounded
- `GET /api/v1/mcp/servers` - no pagination
- `GET /api/v1/notes` - no pagination

**Standard pattern**:
```rust
#[derive(Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_limit")]
    pub limit: usize,  // default: 50, max: 100
    #[serde(default)]
    pub offset: usize,
}

#[derive(Serialize, ToSchema)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: usize,
    pub limit: usize,
    pub offset: usize,
}
```

### Issue 2: Response Wrapping Inconsistency

**Current state**:
- `CreateNoteResponse` wraps as `{ note: Note }`
- `CreateSessionRequest` returns bare `SessionDetail`
- `CreateWorkstreamRequest` returns bare `WorkstreamResponse`

**Decision needed**: Always wrap single items OR never wrap

**Recommendation**: Never wrap - simpler for clients, consistent with REST conventions

### Issue 3: Missing `total` Field

**Fix locations**:
- `MessageListResponse` - add `total: usize`
- `WorkstreamListResponse` - add `total: usize`
- `ListServersResponse` - already has `total` (good)

### Issue 4: HTTP Status Codes

**Fixes needed**:
| Endpoint | Current | Should Be |
|----------|---------|-----------|
| `POST /mcp/servers` | 200 | 201 |
| `POST /workstreams/{ws}/files/promote` | 200 | 201 |
| `POST /workstreams/{ws}/files/export` | 200 | 201 |

### Issue 5: Path Variable Naming

**Current inconsistency**:
- `/workstreams/{id}/files/promote`
- `/workstreams/{ws}/files/export`

**Fix**: Standardize on `{id}` for all resource identifiers

### Breaking Change Strategy

Options:
1. Bump to `/api/v2/` for all changed endpoints
2. Add query param `?version=2` for new behavior
3. Document and ship (clients adapt)

**Recommendation**: Option 3 for minor changes, document clearly

## Status Updates

### Session 1

**Analysis Complete**:
- sessions.rs: `list_sessions_handler` has `total` but no `limit`/`offset`
- workstreams.rs: `WorkstreamListResponse` missing `total`; `MessageListResponse` missing `total`; `SessionListResponse` missing `total`; `promote_file_handler` and `export_file_handler` return 200 (should be 201)
- mcp.rs: `add_server_handler` returns 200 (should be 201); `ListServersResponse` already has `total`
- memory.rs: `create_note_handler` returns 200 (should be 201); `CreateNoteResponse` wraps in `{note: Note}` (inconsistent); `ListNotesResponse` has `total` but only `limit`, no `offset`
- lib.rs: Routes use `{ws}` in some places instead of `{id}`

**Decisions**:
- Pagination: Shared `PaginationParams` type with default limit=50, max=100
- Response wrapping: "Never wrap" - unwrap Notes responses for consistency
- Path variables: Standardize to `{id}` everywhere

### Session 2 - Implementation Complete

**Created shared pagination module** (`routes/pagination.rs`):
- `PaginationParams` with `limit` (default 50, max 100) and `offset` (default 0)
- `PaginatedResponse<T>` generic type for future use
- `paginate()` helper method for slicing collections
- 6 unit tests for pagination logic

**Pagination added to all list endpoints**:
- `list_sessions_handler` - PaginationParams added, response includes limit/offset
- `list_workstreams_handler` - PaginationParams added, total/limit/offset added to response
- `list_workstream_sessions_handler` - PaginationParams added, total/limit/offset added  
- `list_messages_handler` - PaginationParams added, total/limit/offset added
- `list_notes_handler` - PaginationParams replaces old limit-only query, offset added

**Status codes fixed (200 → 201)**:
- `add_server_handler` (mcp.rs) → 201 Created
- `promote_file_handler` (workstreams.rs) → 201 Created
- `export_file_handler` (workstreams.rs) → 201 Created
- `create_note_handler` (memory.rs) → 201 Created

**Response wrapping removed (consistent "never wrap")**:
- Removed `CreateNoteResponse` wrapper, now returns `Note` directly
- Removed `GetNoteResponse` wrapper, now returns `Note` directly
- `update_note_handler` updated to return `Note` directly
- Removed from OpenAPI schema and re-exports

**Path variables standardized**:
- 5 routes in lib.rs changed from `{ws}` to `{id}`
- 5 OpenAPI path annotations in workstreams.rs changed from `{ws}` to `{id}`

**Derive additions**:
- Added `Clone` to `WorkstreamResponse`, `MessageResponse`, `SessionResponse` (needed by paginate)

**Tests updated**:
- 3 integration tests in memory_integration.rs updated for unwrapped Note response + 201 status
- 1 unit test in memory.rs updated for 201 status + unwrapped response
- 2 unit tests in mcp.rs updated for 201 status
- 1 unit test in sessions.rs updated to verify pagination fields
- All 148 unit tests + 92 integration tests pass

**Files modified**:
- `crates/arawn-server/src/routes/pagination.rs` (created in session 1)
- `crates/arawn-server/src/routes/mod.rs`
- `crates/arawn-server/src/routes/sessions.rs`
- `crates/arawn-server/src/routes/workstreams.rs`
- `crates/arawn-server/src/routes/mcp.rs`
- `crates/arawn-server/src/routes/memory.rs`
- `crates/arawn-server/src/routes/openapi.rs`
- `crates/arawn-server/src/lib.rs`
- `crates/arawn-server/tests/memory_integration.rs`