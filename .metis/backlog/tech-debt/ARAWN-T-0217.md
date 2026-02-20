---
id: documentation-lock-ordering-api
level: task
title: "Documentation: Lock Ordering, API Versioning, OpenAPI Improvements"
short_code: "ARAWN-T-0217"
created_at: 2026-02-20T14:06:03.086992+00:00
updated_at: 2026-02-20T14:06:03.086992+00:00
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

# Documentation: Lock Ordering, API Versioning, OpenAPI Improvements

## Objective

Add critical documentation: lock ordering invariants to prevent deadlocks, API versioning strategy for breaking changes, and improved OpenAPI descriptions for conditional logic.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P3 - Low (when time permits)

### Technical Debt Impact
- **Current Problems**:
  - No documented lock ordering - risk of deadlocks as codebase grows
  - No API versioning strategy - unclear how to handle breaking changes
  - OpenAPI descriptions vague for conditional logic (e.g., MCP transport types)
  - API version tied to package version (should be independent)
  - No deprecation headers or policy
- **Benefits of Fixing**: Maintainability, clear contracts for clients, safer concurrency
- **Risk Assessment**: LOW - documentation improvements, no code changes required initially

## Acceptance Criteria

- [ ] Lock ordering documented in code comments at `arawn-server/src/state.rs`
- [ ] Lock ordering documented in `docs/architecture/concurrency.md`
- [ ] API versioning strategy documented in `docs/api/versioning.md`
- [ ] OpenAPI descriptions improved for all conditional fields
- [ ] Deprecation policy documented
- [ ] API version decoupled from package version in `/config` response

## Implementation Notes

### Issue 1: Lock Ordering Documentation

**Current locks** (need documented ordering):
- `pending_reconnects: Arc<RwLock<HashMap<SessionId, PendingReconnect>>>`
- `session_owners: Arc<RwLock<HashMap<SessionId, ConnectionId>>>`
- `mcp_manager: Arc<RwLock<McpManager>>`
- `session_cache.inner: Arc<RwLock<LruCache<...>>>`
- `tasks: Arc<RwLock<HashMap<String, TrackedTask>>>`

**Correct ordering** (already followed, needs documentation):
```
pending_reconnects < session_owners < mcp_manager < tasks
```

**Add to `arawn-server/src/state.rs`**:
```rust
/// # Lock Ordering
/// 
/// To prevent deadlocks, locks must be acquired in this order:
/// 1. `pending_reconnects` (first)
/// 2. `session_owners`
/// 3. `session_cache`
/// 4. `mcp_manager`
/// 5. `tasks` (last)
/// 
/// Never hold a higher-numbered lock while acquiring a lower-numbered one.
/// Always release locks before spawning tasks that acquire locks.
```

**Create `docs/architecture/concurrency.md`**:
- Lock ordering diagram
- Guidelines for adding new locks
- Common patterns (release before spawn, etc.)

### Issue 2: API Versioning Strategy

**Current state**:
- Single version: `/api/v1/`
- Version in `/config` response is package version
- No deprecation mechanism

**Create `docs/api/versioning.md`**:
```markdown
# API Versioning Strategy

## Version Format
- API version: `/api/v{major}/`
- Independent of package version

## Breaking Change Policy
- Breaking changes require new major version
- Old versions supported for 6 months after deprecation
- Deprecation announced via `Deprecation` header

## Deprecation Headers
- `Deprecation: true` - endpoint is deprecated
- `Sunset: <date>` - when endpoint will be removed
- `Link: <new-endpoint>; rel="successor"` - replacement

## Non-Breaking Changes
These do NOT require version bump:
- Adding optional fields to responses
- Adding new endpoints
- Adding optional query parameters
```

**Code change**: Separate API version from package version:
```rust
pub struct ConfigResponse {
    pub package_version: String,  // env!("CARGO_PKG_VERSION")
    pub api_version: String,      // "1.0"
    // ...
}
```

### Issue 3: OpenAPI Description Improvements

**Conditional fields lacking documentation**:

1. **MCP `AddServerRequest`** - transport-dependent fields:
```rust
/// Request to add an MCP server.
/// 
/// ## Transport Types
/// 
/// ### stdio
/// Requires: `command`, optional `args` and `env`
/// 
/// ### sse
/// Requires: `url`
#[derive(ToSchema)]
pub struct AddServerRequest {
    /// Server name (unique identifier)
    pub name: String,
    
    /// Transport type: "stdio" or "sse"
    pub transport: String,
    
    /// Command to execute (required for stdio transport)
    #[schema(example = "/usr/bin/mcp-server")]
    pub command: Option<String>,
    
    // ...
}
```

2. **SendMessageRequest.role** - enum values:
```rust
/// Message role.
/// 
/// Valid values: "user", "assistant", "system"
#[schema(example = "user")]
pub role: Option<String>,
```

3. **Error responses** - when each occurs:
```rust
#[utoipa::path(
    // ...
    responses(
        (status = 201, description = "Server added successfully"),
        (status = 400, description = "Invalid request - missing required fields for transport type"),
        (status = 409, description = "Server with this name already exists"),
        (status = 500, description = "MCP not enabled in server configuration"),
    ),
)]
```

### Deliverables

| Document | Location | Purpose |
|----------|----------|---------|
| Lock ordering comments | `arawn-server/src/state.rs` | In-code reference |
| Concurrency guide | `docs/architecture/concurrency.md` | Developer guide |
| API versioning | `docs/api/versioning.md` | Client/contributor guide |
| OpenAPI improvements | Various `routes/*.rs` | API documentation |

## Status Updates

*To be added during implementation*