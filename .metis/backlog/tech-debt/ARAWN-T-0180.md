---
id: error-type-consolidation
level: task
title: "Error Type Consolidation"
short_code: "ARAWN-T-0180"
created_at: 2026-02-13T16:39:55.355236+00:00
updated_at: 2026-02-13T16:39:55.355236+00:00
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

# Error Type Consolidation

## Objective

Consolidate error handling across all crates to use a consistent error chain pattern.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P1 - High (important for user experience)

### Technical Debt Impact
- **Current Problems**: Mixed error handling approaches:
  - `anyhow::Error` for general errors
  - `ServerError` enum for HTTP responses
  - Raw `serde_json::Value` in some API responses
  - Inconsistent error codes and messages
- **Benefits of Fixing**: Consistent API error responses, better debugging, cleaner error propagation.
- **Risk Assessment**: MEDIUM - Inconsistent errors can confuse API clients and make debugging harder.

## Acceptance Criteria

- [ ] Define consistent error hierarchy across crates
- [ ] Create `arawn-error` crate or add to `arawn-core`
- [ ] All HTTP endpoints return structured `ServerError` responses
- [ ] Remove raw `serde_json::Value` error returns
- [ ] Add error codes for programmatic handling
- [ ] Update API documentation with error responses
- [ ] All tests pass with new error types

## Implementation Notes

### Technical Approach

**Proposed Error Hierarchy:**
```rust
// Base error trait
pub trait ArawnError: std::error::Error {
    fn error_code(&self) -> &'static str;
    fn http_status(&self) -> StatusCode;
}

// Server errors (HTTP layer)
#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    #[error("Unauthorized")]
    Unauthorized,
    
    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

// Workstream errors
#[derive(Debug, thiserror::Error)]
pub enum WorkstreamError {
    #[error("Workstream not found: {0}")]
    NotFound(String),
    
    #[error("Storage error: {0}")]
    Storage(#[from] rusqlite::Error),
}
```

### Files to Audit
- `crates/arawn-server/src/error.rs`
- `crates/arawn-server/src/routes/*.rs` - Find raw JSON error returns
- `crates/arawn-workstream/src/lib.rs`
- `crates/arawn-config/src/error.rs`

### Migration Strategy
1. Create unified error types
2. Update one route file at a time
3. Run tests after each change
4. Update API docs last

## Status Updates

*To be added during implementation*