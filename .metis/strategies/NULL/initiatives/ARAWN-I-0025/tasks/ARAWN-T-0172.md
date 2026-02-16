---
id: fix-security-stability-issues
level: task
title: "Fix Security & Stability Issues (Phase 3)"
short_code: "ARAWN-T-0172"
created_at: 2026-02-13T13:43:45.538915+00:00
updated_at: 2026-02-13T13:49:49.571454+00:00
parent: ARAWN-I-0025
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0025
---

# Fix Security & Stability Issues (Phase 3)

Fix 4 security and stability issues from the codebase audit.

## Parent Initiative

[[ARAWN-I-0025]]

## Objective

Improve security posture and connection stability for the server.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Per-IP rate limiting instead of global (prevent single bad actor blocking all)
- [x] Message length validation (max 100KB to prevent DoS)
- [x] WebSocket idle timeout (5 min to clean up stale connections)
- [x] Session retry mechanism for race conditions (assessed - not needed after LRU cache fix)

## Issues Detail

### 14. Global Rate Limiter (ratelimit.rs:76-129)
**Impact**: One bad actor can exhaust rate limit for all users.
**Fix**: Use keyed rate limiter by client IP. Extract IP from X-Forwarded-For or ConnectInfo.

### 22. No WebSocket Idle Timeout (ws.rs:189-261)
**Impact**: Idle connections consume resources indefinitely.
**Fix**: Add tokio::time::timeout around receiver.next() with 5-minute timeout.

### 23. No Chat Message Length Validation (chat.rs:85-139)
**Impact**: Very large messages could cause OOM or DoS.
**Fix**: Add check at start of handler, reject messages > 100KB with 400 error.

### 9. Session Disappearance Race (chat.rs:101-105)
**Status**: Assess if still needed after Phase 1 LRU cache fix.

## Implementation Notes

### Technical Approach

1. **Per-IP rate limiting**: Use `governor::Keyed` with DashMap, extract IP from headers
2. **Message length**: Simple check before JSON parsing
3. **WS timeout**: Wrap `receiver.next()` with `tokio::time::timeout(Duration::from_secs(300), ...)`

## Status Updates

### 2026-02-13: Phase 3 Complete

All security & stability fixes implemented and tested:

**1. Message Length Validation (chat.rs)**
- Added `MAX_MESSAGE_BYTES` constant (100KB)
- Validation at start of `chat_handler` and `chat_stream_handler`
- Returns 400 Bad Request with descriptive error for oversized messages

**2. WebSocket Idle Timeout (ws.rs)**
- Added `IDLE_TIMEOUT` constant (5 minutes)
- Wrapped message receive loop with `tokio::time::timeout`
- Sends `idle_timeout` error message before closing stale connections
- Logs connection closure reason

**3. Per-IP Rate Limiting (ratelimit.rs)**
- Changed from `NotKeyed` to `keyed::DefaultKeyedStateStore<IpAddr>`
- Added `extract_client_ip()` function that checks:
  1. X-Forwarded-For header (first IP for chained proxies)
  2. X-Real-IP header
  3. ConnectInfo extension from axum
  4. Falls back to 127.0.0.1
- Logs client IP in rate limit exceeded warnings

**4. Session Retry Mechanism**
- Assessed: Not needed after Phase 1 LRU cache implementation
- The LRU cache with proper eviction handles session lifecycle correctly
- Race conditions are mitigated by the atomic cache operations

**Files Modified:**
- `crates/arawn-server/src/routes/chat.rs` - Message validation
- `crates/arawn-server/src/routes/ws.rs` - Idle timeout
- `crates/arawn-server/src/ratelimit.rs` - Per-IP rate limiting

**Verification:**
- `cargo check -p arawn-server` passes without warnings
- All 89 unit tests pass
- All integration tests pass