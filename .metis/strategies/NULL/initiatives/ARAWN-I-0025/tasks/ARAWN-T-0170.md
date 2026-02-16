---
id: fix-critical-issues-from-codebase
level: task
title: "Fix Critical Issues from Codebase Audit (Phase 1)"
short_code: "ARAWN-T-0170"
created_at: 2026-02-13T04:04:11.205233+00:00
updated_at: 2026-02-13T04:13:06.374593+00:00
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

# Fix Critical Issues from Codebase Audit (Phase 1)

Addresses the 6 critical issues identified in AUDIT_REPORT.md that could cause data loss, memory exhaustion, or crashes.

## Parent Initiative

[[ARAWN-I-0025]]

## Objective

Fix all CRITICAL severity issues from the codebase audit to ensure production stability.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Fix Box::leak memory leak in store.rs (build SQL dynamically without leaking)
- [x] Add fsync to message writes in message_store.rs (ensure data persistence)
- [ ] Implement session cache LRU eviction with configurable max size
- [x] Handle mutex poisoning gracefully (use parking_lot::Mutex)
- [x] Bound message/tool vectors in TUI app.rs (ring buffer or max limit)
- [x] Fix API client panic (propagate error instead of expect())

## Issues Detail

### 1. Memory Leak via Box::leak (store.rs:486-489)
**Impact**: Each `update_workstream()` leaks 4-5 strings. Unbounded memory growth.
**Fix**: Build SQL string directly without static lifetime requirement.

### 2. Missing fsync on Message Writes (message_store.rs:48-52)
**Impact**: Process crash = data loss. Messages in OS buffer not persisted.
**Fix**: Add `file.sync_all()?` after `write_all()`.

### 3. Session Cache Without TTL/Eviction (session_cache.rs:43-48)
**Impact**: HashMap grows indefinitely → OOM on long-running servers.
**Fix**: Implement LRU eviction with configurable max size (default 10,000 sessions).

### 4. Mutex Panic on Poisoning (store.rs:83, logs.rs:67,72,77,82,87)
**Impact**: If any operation panics holding lock, all subsequent operations panic.
**Fix**: Use `parking_lot::Mutex` which doesn't poison.

### 5. Unbounded Message/Tool Vectors (app.rs:124,126)
**Impact**: Memory leak as conversation grows.
**Fix**: Implement max limit with oldest message removal.

### 6. API Client Build Panic (app.rs:163)
**Impact**: Invalid URL crashes app on startup.
**Fix**: Propagate error from `new()` function.

## Implementation Notes

### Technical Approach

1. **Box::leak fix**: Refactor `update_workstream` to use a local String for the SQL query instead of collecting &'static str slices.

2. **fsync fix**: Simple one-liner after write_all().

3. **LRU cache**: Add `lru` crate dependency and wrap HashMap with size limit.

4. **parking_lot**: Add dependency and replace std::sync::Mutex.

5. **Bounded vectors**: Add MAX_MESSAGES const (e.g., 10000) and truncate on insert.

6. **Error propagation**: Change App::new to return Result<Self>.

## Status Updates

### 2026-02-13: All 6 Critical Issues Fixed

**Completed:**

1. **Box::leak memory leak** - Refactored `update_workstream()` in `store.rs` to use owned `String` vector instead of leaking static references. Removed the `leak_set` function entirely.

2. **Missing fsync** - Added `file.sync_all()?` after `write_all()` in `message_store.rs` to ensure data is persisted to disk before returning success.

3. **Session cache LRU eviction** - Replaced `HashMap` with `lru::LruCache` in `session_cache.rs`. Added configurable capacity (default 10,000 sessions). Added `len()`, `is_empty()`, `with_capacity()` methods. Added tests for LRU eviction behavior.

4. **Mutex poisoning** - Added `parking_lot` dependency to workspace. Replaced `std::sync::Mutex` with `parking_lot::Mutex` in both `store.rs` and `logs.rs`. parking_lot doesn't poison on panic, preventing cascading failures.

5. **Unbounded vectors in TUI** - Added `MAX_MESSAGES` (10,000) and `MAX_TOOLS` (1,000) constants. Added `push_message()` and `push_tool()` helper methods that drain oldest 10% when at capacity.

6. **API client panic** - Changed `App::new()` to return `Result<Self>` and propagate errors from `ArawnClient::build()`. Updated `lib.rs` call site to handle the Result.

**Files Modified:**
- `Cargo.toml` (workspace) - Added `parking_lot = "0.12"`
- `crates/arawn-workstream/Cargo.toml` - Added parking_lot dependency
- `crates/arawn-workstream/src/store.rs` - Fixed Box::leak, switched to parking_lot::Mutex
- `crates/arawn-workstream/src/message_store.rs` - Added fsync
- `crates/arawn-server/Cargo.toml` - Added lru dependency
- `crates/arawn-server/src/session_cache.rs` - Implemented LRU cache
- `crates/arawn-tui/Cargo.toml` - Added parking_lot dependency
- `crates/arawn-tui/src/logs.rs` - Switched to parking_lot::Mutex
- `crates/arawn-tui/src/app.rs` - Bounded vectors, error propagation
- `crates/arawn-tui/src/lib.rs` - Handle App::new() Result

**All tests pass**: `angreal test unit` completes successfully