---
id: api-consistency-pagination
level: task
title: "API Consistency: Pagination, Response Wrapping, Status Codes"
short_code: "ARAWN-T-0214"
created_at: 2026-02-20T13:41:43.517518+00:00
updated_at: 2026-02-20T13:41:43.517518+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#tech-debt"


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

- [ ] All list endpoints support `limit` and `offset` query parameters
- [ ] All list responses include `total` field for pagination
- [ ] Response wrapping is consistent (decide: always wrap OR never wrap)
- [ ] All POST create endpoints return 201 Created
- [ ] Path variables use consistent naming (`{id}` everywhere)
- [ ] OpenAPI documentation updated to reflect changes
- [ ] Breaking changes documented in CHANGELOG

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

*To be added during implementation*