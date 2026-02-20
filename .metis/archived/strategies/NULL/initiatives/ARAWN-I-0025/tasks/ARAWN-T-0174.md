---
id: fix-medium-priority-workstream
level: task
title: "Fix Medium Priority Workstream Issues (Phase 4)"
short_code: "ARAWN-T-0174"
created_at: 2026-02-13T14:00:24.252166+00:00
updated_at: 2026-02-13T14:05:24.518909+00:00
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

# Fix Medium Priority Workstream Issues (Phase 4)

Fix 5 medium-priority workstream issues from the codebase audit.

## Parent Initiative

[[ARAWN-I-0025]]

## Objective

Improve workstream crate error handling, logging, and data integrity.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] #24: parse_dt silently returns now - Added warning log with input and error
- [x] #25: Empty tool_call_id on legacy format - Added debug log for correlation issues
- [x] #26: Char count != token count - Added documentation explaining approximation
- [x] #27: Non-atomic scratch promotion - Added TOCTOU handling for directory rename
- [x] #28: Silent metadata deserialization failure - Added warning logs for both ToolUse and ToolResult

## Issues Detail

### #24: parse_dt silently returns now (store.rs:455-459)
**Impact**: Invalid dates become current time silently, corrupting timestamps.
**Fix**: Return Result or at minimum log warning when parsing fails.

### #25: Empty tool_call_id on legacy format (session_loader.rs:175-184)
**Impact**: Legacy sessions have empty tool_call_ids, breaking correlation.
**Fix**: Log warning when tool_call_id is empty for debugging purposes.

### #26: Char count != token count (compression.rs:147-149)
**Impact**: Character count underestimates token usage, affecting compression.
**Fix**: Document limitation clearly or integrate tokenizer for accuracy.

### #27: Non-atomic scratch promotion (scratch.rs:70-76)
**Impact**: Directory operations could fail partially, leaving inconsistent state.
**Fix**: Handle ENOENT gracefully, add recovery logic.

### #28: Silent metadata deserialization failure (session_loader.rs:162-170)
**Impact**: Malformed metadata silently ignored, data loss goes unnoticed.
**Fix**: Log warning when metadata fails to deserialize.

## Status Updates

### Completed 2026-02-13

All 5 workstream medium-priority issues fixed:

**#24 parse_dt warning (store.rs)**
- Added `tracing::warn!` with input value and error when datetime parsing fails
- Logs help debug data corruption without breaking existing behavior

**#25 Empty tool_call_id logging (session_loader.rs)**
- Added `tracing::debug!` when tool_call_id is empty in ToolResult messages
- Helps identify legacy format messages that may have correlation issues

**#26 Char/token documentation (compression.rs)**
- Added doc comment explaining the ~4 chars/token approximation
- Documents that threshold is set conservatively (~8k tokens ≈ 32k chars)

**#27 TOCTOU handling (scratch.rs)**
- Added match arms for fs::rename errors during scratch promotion
- Handles ENOENT gracefully with warning log for race condition
- Other errors still propagate with detailed error logging

**#28 Metadata deserialization warnings (session_loader.rs)**
- Added `tracing::warn!` for failed ToolUseMetadata parsing
- Added `tracing::warn!` for missing ToolUse metadata
- Added `tracing::debug!` for legacy ToolResult format without metadata

All 45 arawn-workstream tests pass.