---
id: security-hardening-websocket
level: task
title: "Security Hardening: WebSocket Origin, Token Comparison, Size Limits"
short_code: "ARAWN-T-0213"
created_at: 2026-02-20T13:41:42.561874+00:00
updated_at: 2026-02-20T13:41:42.561874+00:00
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

# Security Hardening: WebSocket Origin, Token Comparison, Size Limits

## Objective

Address critical security vulnerabilities identified in architecture review: WebSocket CSRF protection, timing-safe authentication, and DoS prevention through message size limits.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P1 - High (important for user experience)

### Technical Debt Impact
- **Current Problems**:
  - WebSocket endpoint has NO Origin header validation - any website can connect (CSRF risk)
  - Auth token comparison uses `==` which is vulnerable to timing attacks
  - No message size limits on WebSocket or REST - potential DoS vector
  - No rate limiting on WebSocket connections (REST has limits)
- **Benefits of Fixing**: Production-ready security posture for public exposure
- **Risk Assessment**: HIGH - these are exploitable vulnerabilities if server is exposed

## Acceptance Criteria

- [ ] WebSocket upgrade validates Origin header against allowed origins list
- [ ] Auth token comparison uses constant-time comparison (`subtle` crate or similar)
- [ ] WebSocket messages have configurable max size (reject oversized)
- [ ] REST request bodies have configurable max size
- [ ] WebSocket connections have rate limiting (connections/minute per IP)
- [ ] All security changes have unit tests

## Implementation Notes

### Issue 1: WebSocket Origin Validation

**Location**: `arawn-server/src/routes/ws/mod.rs:34-36`

**Current state**: WebSocket upgrade handler doesn't validate Origin header. Axum's `WebSocketUpgrade` extractor doesn't validate by default.

**Fix options**:
1. Validate `Origin` header matches `config.allowed_origins` before upgrade
2. Require auth token in connection URL query parameter
3. Use custom WebSocket upgrade that checks Origin

**Example**:
```rust
async fn ws_handler(
    headers: HeaderMap,
    ws: WebSocketUpgrade,
    // ...
) -> impl IntoResponse {
    if let Some(origin) = headers.get("Origin") {
        if !config.allowed_origins.contains(origin.to_str()?) {
            return StatusCode::FORBIDDEN.into_response();
        }
    }
    ws.on_upgrade(...)
}
```

### Issue 2: Timing-Safe Token Comparison

**Location**: `arawn-server/src/auth.rs:138`

**Current**: `token == expected_token` - vulnerable to timing attacks

**Fix**: Use `subtle::ConstantTimeEq` or similar:
```rust
use subtle::ConstantTimeEq;
token.as_bytes().ct_eq(expected_token.as_bytes()).into()
```

**Dependency**: Add `subtle = "2.5"` to Cargo.toml

### Issue 3: Message Size Limits

**Locations**:
- WebSocket: `arawn-server/src/routes/ws/connection.rs`
- REST: Various route handlers

**Fix**:
- Add `max_message_size` to `ServerConfig` (default: 1MB)
- Configure Axum's `DefaultBodyLimit` layer
- WebSocket: Check message length before processing

### Issue 4: WebSocket Rate Limiting

**Location**: `arawn-server/src/routes/ws/`

**Current**: Rate limiting exists for REST (`config.rs:22-25`) but not WebSocket

**Fix**: Track connections per IP, reject if exceeding threshold

## Status Updates

*To be added during implementation*