# Codebase Audit Report & Remediation Plan

**Date:** 2026-02-12
**Auditor:** Claude Code (Opus 4.5)

## Executive Summary

**Total Issues Found: 73**
| Severity | arawn-server | arawn-workstream | arawn-tui/client | Total |
|----------|--------------|------------------|------------------|-------|
| CRITICAL | 4 | 3 | 5 | **12** |
| HIGH | 4 | 3 | 8 | **15** |
| MEDIUM | 11 | 5 | 13 | **29** |
| LOW | 7 | 6 | 4 | **17** |

---

## CRITICAL Issues (Fix Immediately)

### 1. Memory Leak via Box::leak (arawn-workstream)
**File:** `crates/arawn-workstream/src/store.rs:486-489`
```rust
fn leak_set(col: &str, idx: u32) -> &'static str {
    let s = format!("{col} = ?{idx}");
    Box::leak(s.into_boxed_str())  // LEAKS MEMORY
}
```
**Impact:** Each `update_workstream()` leaks 4-5 strings. Unbounded memory growth.
**Fix:** Use string interning or build SQL dynamically without leaking.

### 2. Missing fsync on Message Writes (arawn-workstream)
**File:** `crates/arawn-workstream/src/message_store.rs:48-52`
**Impact:** Process crash = data loss. Messages in OS buffer not persisted.
**Fix:** Add `file.sync_all()?` after `write_all()`.

### 3. Session Cache Without TTL/Eviction (arawn-server)
**File:** `crates/arawn-server/src/session_cache.rs:43-48`
**Impact:** HashMap grows indefinitely → OOM on long-running servers.
**Fix:** Implement LRU eviction with configurable max size (e.g., 10,000 sessions).

### 4. Mutex Panic on Poisoning (arawn-workstream + arawn-tui)
**Files:**
- `crates/arawn-workstream/src/store.rs:83`
- `crates/arawn-tui/src/logs.rs:67,72,77,82,87`
**Impact:** If any operation panics holding lock, all subsequent operations panic.
**Fix:** Use `parking_lot::Mutex` or handle poisoning gracefully.

### 5. Unbounded Message/Tool Vectors (arawn-tui)
**File:** `crates/arawn-tui/src/app.rs:124,126`
**Impact:** Memory leak as conversation grows.
**Fix:** Implement ring buffer with MAX_MESSAGES constant.

### 6. API Client Build Panic (arawn-tui)
**File:** `crates/arawn-tui/src/app.rs:163`
**Impact:** Invalid URL crashes app on startup.
**Fix:** Propagate error instead of `expect()`.

---

## HIGH Issues (Fix This Sprint)

### 7. Chat Handler Doesn't Persist Turns (arawn-server)
**File:** `crates/arawn-server/src/routes/chat.rs:85-139`
**Impact:** REST API conversations not saved to workstream storage.
**Fix:** Call `session_cache.save_turn()` after agent response.

### 8. WebSocket Session State Not Captured After Stream (arawn-server)
**File:** `crates/arawn-server/src/routes/ws.rs:387-399`
**Impact:** Tool results and state changes during streaming not persisted.
**Fix:** Capture final session state after stream completes.

### 9. Race Condition: Session Disappearance (arawn-server)
**File:** `crates/arawn-server/src/routes/chat.rs:101-105`
**Impact:** "Session disappeared" errors during legitimate operations.
**Fix:** Implement retry mechanism or hold session lock through operation.

### 10. TOCTOU Race in Scratch Promotion (arawn-workstream)
**File:** `crates/arawn-workstream/src/scratch.rs:61-67`
**Impact:** Directory could disappear between check and rename.
**Fix:** Wrap in transaction, handle ENOENT gracefully.

### 11. Unbounded Message Loading (arawn-workstream)
**File:** `crates/arawn-workstream/src/compression.rs:80-87`
**Impact:** Loading all messages can exhaust RAM.
**Fix:** Add pagination/streaming to message store.

### 12. Workstream Rename ID Lookup Bug (arawn-tui)
**File:** `crates/arawn-tui/src/app.rs:328`
**Impact:** After rename, workstream not found in sidebar.
**Fix:** Look up by `ws.id` not `ws.name`.

### 13. Session Creation on New Workstream Missing (arawn-tui)
**File:** `crates/arawn-tui/src/app.rs:296-304`
**Impact:** Messages sent to old session after workstream switch.
**Fix:** Clear session_id before switching workstreams.

### 14. Global Rate Limiter (Not Per-IP) (arawn-server)
**File:** `crates/arawn-server/src/ratelimit.rs:76-129`
**Impact:** One bad actor blocks all users.
**Fix:** Implement per-IP rate limiting with concurrent HashMap.

---

## MEDIUM Issues (Fix Next Sprint)

### Server Issues
| # | Issue | File | Fix |
|---|-------|------|-----|
| 15 | Tool calls/results length mismatch silent | chat.rs:118-127 | Add assertion |
| 16 | Memory search falls back silently | memory.rs:325-356 | Add degraded flag |
| 17 | Task list pagination incorrect | tasks.rs:148-174 | Return total before filter |
| 18 | Auth doesn't distinguish missing vs invalid | auth.rs:132-145 | Always return 401 |
| 19 | Unused Identity extension | chat.rs:87 | Remove or implement |
| 20 | Workstream errors use raw JSON | workstreams.rs:113-135 | Use ServerError |
| 21 | Session cache creates empty on not found | session_cache.rs:62-139 | Return error |
| 22 | No WebSocket idle timeout | ws.rs:189-261 | Add 5-minute timeout |
| 23 | No chat message length validation | chat.rs:85-139 | Add max 100KB limit |

### Workstream Issues
| # | Issue | File | Fix |
|---|-------|------|-----|
| 24 | parse_dt silently returns now | store.rs:455-459 | Return Result |
| 25 | Empty tool_call_id on legacy format | session_loader.rs:175-184 | Log warning |
| 26 | Char count != token count | compression.rs:147-149 | Use tokenizer |
| 27 | Non-atomic scratch promotion | scratch.rs:70-76 | Wrap in transaction |
| 28 | Silent metadata deserialization failure | session_loader.rs:162-170 | Log failures |

### TUI Issues
| # | Issue | File | Fix |
|---|-------|------|-----|
| 29 | Race in message rendering | ui/chat.rs:40 | Separate tool rendering |
| 30 | Event channel silent termination | events.rs:51 | Log reason |
| 31 | Sidebar filter not implemented | app.rs:1280 | Implement or remove |
| 32 | Tool pane scrolling TODO | app.rs:1093 | Implement |
| 33 | Cancel doesn't notify server | app.rs:661 | Send cancel message |
| 34 | Cursor position can panic | input.rs:63-66 | Add bounds check |
| 35 | Session state loss on switch | app.rs:1025-1029 | Subscribe before clear |
| 36 | Auto-scroll overrides manual | app.rs:468,870 | Check scroll state |
| 37 | Pending actions not deduplicated | app.rs:247-280 | Dedupe queue |

---

## LOW Issues (Backlog)

- Note store global singleton breaks test isolation
- Streaming response unbounded buffer
- No cascade delete in schema
- Health check always returns ready
- WebSocket binary frame handling inconsistent
- Config response hardcodes limit
- History draft memory in input
- Tool output truncation no indicator

---

## Refactoring Recommendations

### 1. Extract Session Cache to Dedicated Crate
The session cache logic spans server and workstream. Create `arawn-session` crate with:
- LRU eviction policy
- TTL support
- Proper persistence hooks

### 2. Bounded Collections Library
Create utility types:
```rust
pub struct BoundedVec<T, const N: usize> { ... }
pub struct RingBuffer<T, const N: usize> { ... }
```

### 3. TUI Focus Management
Extract `FocusManager` from app.rs to handle:
- Focus transitions
- Input routing
- Overlay state

### 4. Error Type Consolidation
Currently using mix of:
- `anyhow::Error`
- `ServerError`
- Raw `serde_json::Value`
Consolidate to consistent error chain.

### 5. WebSocket Protocol Module
Split ws.rs message handling into:
- `protocol.rs` - Message types
- `connection.rs` - Connection lifecycle
- `handlers.rs` - Message handlers

---

## Implementation Priority

### Phase 1: Critical Fixes (Blocking)
1. Fix Box::leak memory leak
2. Add fsync to message writes
3. Implement session cache eviction
4. Handle mutex poisoning
5. Bound message vectors in TUI
6. Fix API client panic

### Phase 2: Data Integrity (High Priority)
7. Persist REST API turns
8. Fix WebSocket session state capture
9. Fix workstream rename bug
10. Clear session on workstream switch

### Phase 3: Security & Stability
11. Per-IP rate limiting
12. Message length validation
13. WebSocket idle timeout
14. Retry mechanisms

### Phase 4: Polish
15. Remaining medium issues
16. Refactoring items
17. Low priority cleanup

---

## Verification

After fixes:
1. `cargo test --workspace` - All tests pass
2. `cargo clippy --workspace` - No warnings
3. Memory test: Run server 24h with load, check RSS growth
4. Integration test: Full TUI workflow with workstream/session operations
