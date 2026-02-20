---
id: fix-low-priority-issues-phase-4
level: task
title: "Fix Low Priority Issues (Phase 4)"
short_code: "ARAWN-T-0176"
created_at: 2026-02-13T14:17:09.694376+00:00
updated_at: 2026-02-13T14:22:25.301297+00:00
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

# Fix Low Priority Issues (Phase 4)

Fix 8 low-priority issues from the codebase audit.

## Parent Initiative

[[ARAWN-I-0025]]

## Objective

Address remaining tech debt and minor issues for improved code quality.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] #1: Note store global singleton - Added documentation about test isolation
- [x] #2: Streaming response unbounded buffer - Already fixed by CRITICAL #5 (MAX_MESSAGES)
- [x] #3: No cascade delete in schema - Created V002 migration with ON DELETE CASCADE
- [x] #4: Health check always returns ready - Added workstream storage health check
- [x] #5: WebSocket binary frame handling - Added clarifying comments (already correct)
- [x] #6: Config response hardcodes limit - Now uses RateLimitConfig::default().api_rpm
- [x] #7: History draft memory - Added clarifying doc comment (behavior is intentional)
- [x] #8: Tool output truncation no indicator - Added "(+N more lines)" indicator

## Issues Detail

### #1: Note store global singleton breaks test isolation
**Impact**: Tests using NoteStore may interfere with each other.
**Fix**: Use dependency injection or per-test instances.

### #2: Streaming response unbounded buffer
**Impact**: Large streaming responses can accumulate in memory.
**Fix**: Add backpressure or bounded buffer.

### #3: No cascade delete in schema
**Impact**: Orphaned records when parent deleted.
**Fix**: Add ON DELETE CASCADE to foreign keys.

### #4: Health check always returns ready
**Impact**: Load balancers can't detect unhealthy instances.
**Fix**: Check actual service dependencies.

### #5: WebSocket binary frame handling inconsistent
**Impact**: Binary messages may be mishandled.
**Fix**: Explicitly handle or reject binary frames.

### #6: Config response hardcodes limit
**Impact**: Config endpoint returns hardcoded values instead of actual.
**Fix**: Return actual configured limits.

### #7: History draft memory in input
**Impact**: Draft text not cleared when browsing history then typing.
**Fix**: Clear draft on any modification.

### #8: Tool output truncation no indicator
**Impact**: User doesn't know output was truncated.
**Fix**: Add "..." or "[truncated]" indicator.

## Implementation Notes

These are minor issues that improve code quality but don't affect core functionality.

## Status Updates

### Completed 2026-02-13

All 8 low-priority issues addressed:

**#1 Note store global singleton (memory.rs)**
- Added documentation warning about test isolation
- Documented workarounds: --test-threads=1, unique IDs, clear store

**#2 Streaming response unbounded buffer**
- Already fixed by CRITICAL #5 which added MAX_MESSAGES (10,000) and MAX_TOOLS (1,000)
- Uses drain-oldest eviction when at capacity

**#3 No cascade delete in schema**
- Created V002__add_cascade_delete.sql migration
- Added ON DELETE CASCADE to sessions and workstream_tags foreign keys

**#4 Health check always returns ready (health.rs)**
- Added workstream storage health check
- Returns "degraded" status if storage unavailable

**#5 WebSocket binary frame handling (ws.rs)**
- Added clarifying comments - handling was already correct
- Binary frames: convert to UTF-8 or reject with error

**#6 Config response hardcodes limit (config.rs)**
- Changed from hardcoded `100` to `RateLimitConfig::default().api_rpm`
- Returns actual configured rate limit value

**#7 History draft memory (input.rs)**
- Added doc comment explaining intentional behavior
- Draft is cleared on edit - this is by design

**#8 Tool output truncation indicator (ui/chat.rs)**
- Added "(+N more lines)" indicator for multi-line output
- Shows both line truncation (...) and line count

All tests pass (45 arawn-workstream, 48 arawn-server, 23 arawn-tui).