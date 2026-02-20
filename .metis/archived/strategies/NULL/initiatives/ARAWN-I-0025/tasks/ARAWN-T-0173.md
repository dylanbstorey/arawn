---
id: fix-medium-priority-issues-phase-4
level: task
title: "Fix Medium Priority Issues (Phase 4)"
short_code: "ARAWN-T-0173"
created_at: 2026-02-13T13:51:14.943381+00:00
updated_at: 2026-02-13T13:57:40.189739+00:00
parent: ARAWN-I-0025
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0025
---

# Fix Medium Priority Issues (Phase 4) - Server

Fix 7 medium-priority server issues from the codebase audit.

## Parent Initiative

[[ARAWN-I-0025]]

## Objective

Improve server code quality, error handling, and API consistency.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] #15: Tool calls/results length mismatch - Add assertion or warning
- [x] #16: Memory search fallback - Add degraded flag to response
- [x] #17: Task list pagination - Return total count before filtering
- [x] #18: Auth error distinction - Already correct (401 for both missing/invalid)
- [x] #19: Unused Identity extension - Now used for audit logging
- [x] #20: Workstream errors - Use ServerError instead of raw JSON
- [x] #21: Session cache not found - Already warns; behavior is intentional for load-or-create

## Issues Detail

### #15: Tool calls/results length mismatch (chat.rs:118-127)
**Impact**: Silent data corruption if tool_calls and tool_results don't match.
**Fix**: Add debug_assert or log warning when lengths differ.

### #16: Memory search falls back silently (memory.rs:325-356)
**Impact**: User doesn't know search is degraded when embedding fails.
**Fix**: Add `degraded: bool` field to response indicating fallback mode.

### #17: Task list pagination incorrect (tasks.rs:148-174)
**Impact**: `total` count is after filtering, breaking pagination UI.
**Fix**: Return total before applying limit/offset.

### #18: Auth doesn't distinguish missing vs invalid (auth.rs:132-145)
**Impact**: Information leakage about valid vs invalid tokens.
**Fix**: Always return same 401 error regardless of missing/invalid.

### #19: Unused Identity extension (chat.rs:87)
**Impact**: Dead code, confusing to readers.
**Fix**: Either use for audit logging or remove the extraction.

### #20: Workstream errors use raw JSON (workstreams.rs:113-135)
**Impact**: Inconsistent error format with other endpoints.
**Fix**: Convert to ServerError variants.

### #21: Session cache creates empty on not found (session_cache.rs:62-139)
**Impact**: "Session disappeared" errors when session doesn't exist.
**Fix**: Return Option or Result, don't auto-create.

## Status Updates

### 2026-02-13: Phase 4 Server Issues Complete

All 7 server medium-priority issues addressed:

**#15: Tool calls/results length mismatch (chat.rs)**
- Added warning log when `tool_calls.len() != tool_results.len()`
- Helps debug data corruption issues

**#16: Memory search degraded flag (memory.rs)**
- Added `degraded: bool` field to `MemorySearchResponse`
- Set to `true` when MemoryStore search fails and falls back to notes
- Uses `skip_serializing_if` to omit when false

**#17: Task list pagination (tasks.rs)**
- Moved `total` calculation before `truncate()`
- Now returns correct total count for pagination UI

**#18: Auth error distinction (auth.rs)**
- Reviewed: Already returns 401 for both MissingToken and InvalidToken
- InvalidFormat returns 400 (correct - malformed request, not auth failure)
- No information leakage about token validity

**#19: Unused Identity extension (chat.rs)**
- Changed `_identity` to `identity` in both handlers
- Added audit logging with identity type and session info
- Logs at debug level for both sync and streaming handlers

**#20: Workstream errors (workstreams.rs)**
- Replaced `(StatusCode, Json<serde_json::Value>)` with `ServerError`
- Updated all 10 handlers to use `Result<..., ServerError>`
- Converted `workstream_error_response()` to `workstream_error()`
- Consistent error format across all API endpoints

**#21: Session cache not found (session_cache.rs)**
- Reviewed: Already warns when updating non-cached session
- Load-or-create behavior is intentional for `get_or_load()`
- LRU eviction from Phase 1 mitigates "session disappeared" race

**Files Modified:**
- `crates/arawn-server/src/routes/chat.rs`
- `crates/arawn-server/src/routes/memory.rs`
- `crates/arawn-server/src/routes/tasks.rs`
- `crates/arawn-server/src/routes/workstreams.rs`

**Verification:**
- `cargo check -p arawn-server` passes without warnings
- All 89 unit tests pass
- All integration tests pass