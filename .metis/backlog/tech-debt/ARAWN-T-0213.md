---
id: security-hardening-websocket
level: task
title: "Security Hardening: WebSocket Origin, Token Comparison, Size Limits"
short_code: "ARAWN-T-0213"
created_at: 2026-02-20T13:41:42.561874+00:00
updated_at: 2026-02-21T21:56:30.117380+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#tech-debt"
  - "#phase/active"


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

## Acceptance Criteria

## Acceptance Criteria

- [x] WebSocket upgrade validates Origin header against allowed origins list
- [x] Auth token comparison uses constant-time comparison (`subtle` crate or similar)
- [x] WebSocket messages have configurable max size (reject oversized)
- [x] REST request bodies have configurable max size
- [x] WebSocket connections have rate limiting (connections/minute per IP)
- [x] All security changes have unit tests (14 new tests added)

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

### Session 1 - 2026-02-21

**Initial Analysis**:
- `auth.rs:138` - Token comparison uses `==` (timing vulnerable)
- `ws/mod.rs:34-36` - No Origin header validation
- `ws/connection.rs` - No message size limits
- `config.rs` - Has `cors_origins` but not `allowed_origins` for WebSocket
- No DefaultBodyLimit layer in lib.rs

**Implementation Plan**:
1. Add `subtle` crate for constant-time comparison
2. Add security config fields: `max_message_size`, `max_body_size`, `ws_allowed_origins`, `ws_rate_limit`
3. Fix token comparison to use constant-time
4. Add Origin validation to WebSocket handler
5. Add message size limits
6. Add body size limits via DefaultBodyLimit layer
7. Add WebSocket connection rate limiting

**Completed**:

1. **Timing-Safe Token Comparison** (`auth.rs`)
   - Added `subtle` crate dependency (v2.5)
   - Implemented `constant_time_eq()` function using `subtle::ConstantTimeEq`
   - Updated token validation to use constant-time comparison
   - Added 4 unit tests for constant-time comparison

2. **Security Configuration** (`config.rs`)
   - Added `max_ws_message_size` (default: 1 MB)
   - Added `max_body_size` (default: 10 MB)
   - Added `ws_allowed_origins` (empty = all allowed in dev mode)
   - Added `ws_connections_per_minute` (default: 30)
   - Added builder methods for all new config fields

3. **WebSocket Origin Validation** (`routes/ws/mod.rs`)
   - Added `validate_origin()` function to check Origin header
   - Supports exact matches and wildcard subdomains (e.g., `*.example.com`)
   - Rejects connections without Origin header when origins are configured
   - Added 6 unit tests for origin validation

4. **WebSocket Message Size Limits** (`routes/ws/mod.rs`)
   - Configured `WebSocketUpgrade::max_message_size()` from config
   - Messages exceeding limit are automatically rejected by Axum

5. **REST Body Size Limits** (`lib.rs`)
   - Added `DefaultBodyLimit::max()` layer to router
   - Requests exceeding limit receive 413 Payload Too Large

6. **WebSocket Connection Rate Limiting** (`state.rs`, `routes/ws/mod.rs`)
   - Added `WsConnectionTracker` type with sliding window rate limiting
   - Tracks connections per IP address within 60-second window
   - Added `check_ws_connection_rate()` method to AppState
   - Updated `ws_handler` to check rate before upgrade
   - Updated server to provide `ConnectInfo<SocketAddr>` for IP extraction
   - Added 4 unit tests for rate limiting

**Files Modified**:
- `crates/arawn-server/Cargo.toml` - Added `subtle = "2.5"`
- `crates/arawn-server/src/config.rs` - Security config fields
- `crates/arawn-server/src/auth.rs` - Constant-time comparison
- `crates/arawn-server/src/lib.rs` - Body size limits, ConnectInfo
- `crates/arawn-server/src/state.rs` - WsConnectionTracker
- `crates/arawn-server/src/routes/ws/mod.rs` - Origin validation, rate limiting
- `crates/arawn-server/src/routes/ws/connection.rs` - Updated signature

**Test Summary**: 141 unit tests + 92 integration tests + 57 validation tests = 290 total, all passing