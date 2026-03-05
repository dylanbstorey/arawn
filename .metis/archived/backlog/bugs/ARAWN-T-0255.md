---
id: validate-workstream-ids-before
level: task
title: "Validate workstream IDs before path construction"
short_code: "ARAWN-T-0255"
created_at: 2026-03-04T13:23:59.234138+00:00
updated_at: 2026-03-04T19:28:23.557341+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#bug"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
---

# Validate workstream IDs before path construction

## Objective

Validate workstream IDs against a strict pattern before using them in filesystem path construction. Malicious workstream IDs (e.g., containing `../`) could be used to construct paths outside the workspace.

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing

### Priority
- [x] P2 - Medium (nice to have)

### Impact Assessment
- **Severity**: MEDIUM - Workstream IDs come from API requests and are used in `DirectoryManager` path construction
- **Expected vs Actual**: IDs should be alphanumeric/dash/underscore only; currently no validation is enforced

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Workstream IDs validated against `^[a-zA-Z0-9_-]+$` pattern (existing `is_valid_name()` + new `validate_workstream_id()`)
- [x] Invalid IDs rejected with clear error before any filesystem operation (400 Bad Request at API boundary)
- [x] Validation applied at API boundary (route handler level) — 13 REST handlers + WebSocket + session update
- [x] Unit tests for valid and invalid IDs (4 new tests including traversal patterns)

### Key Files
- `crates/arawn-workstream/src/directory.rs`
- `crates/arawn-server/src/routes/`

## Status Updates

### Implementation Complete

**Layers of defense added:**

**1. API Boundary Validation (arawn-server)**
- Added `validate_id()` helper in `routes/workstreams.rs` using `DirectoryManager::validate_workstream_id()`
- Applied to all 13 route handlers that accept workstream IDs from URL path parameters
- Added validation in WebSocket `handle_chat` handler (`routes/ws/handlers.rs`)
- Added validation in session update handler (`routes/sessions.rs`) for workstream reassignment
- Returns 400 Bad Request with clear error message for invalid IDs

**2. DirectoryManager Hardening (arawn-workstream)**
- Added `validate_workstream_id()` and `validate_session_id()` public methods that return `DirectoryResult`
- Added `debug_assert!` guards in `workstream_path()` and `scratch_session_path()` — catches bugs in dev, no overhead in release
- Existing `is_valid_name()` pattern: `^[a-zA-Z0-9_-]+$` (no leading `-` or `.`)

**3. Tests Added (4 new)**
- `test_is_valid_name_rejects_traversal` — `../etc`, `../../passwd`, `foo/../bar`, `..`
- `test_validate_workstream_id` — valid and invalid IDs including traversal
- `test_validate_session_id` — valid and invalid session IDs

**Files modified:**
- `crates/arawn-workstream/src/directory.rs` — `validate_workstream_id()`, `validate_session_id()`, debug assertions, 4 tests
- `crates/arawn-server/src/routes/workstreams.rs` — `validate_id()` helper, applied to 13 handlers
- `crates/arawn-server/src/routes/ws/handlers.rs` — validation in WebSocket chat handler
- `crates/arawn-server/src/routes/sessions.rs` — validation in session update handler

**Verification:**
- `angreal check all` — clippy + fmt clean
- `angreal test unit` — all tests pass (0 failures)