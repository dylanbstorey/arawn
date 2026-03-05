---
id: use-constant-time-comparison-for
level: task
title: "Use constant-time comparison for auth token validation"
short_code: "ARAWN-T-0251"
created_at: 2026-03-04T13:23:51.860501+00:00
updated_at: 2026-03-04T17:46:59.674750+00:00
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

# Use constant-time comparison for auth token validation

## Objective

Replace `==` string comparison for auth tokens with constant-time comparison to prevent timing side-channel attacks.

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing

### Priority
- [x] P2 - Medium (nice to have)

### Impact Assessment
- **Severity**: MEDIUM - Timing attacks are difficult to exploit in practice over network, but this is a security best practice
- **Expected vs Actual**: Token comparison should use constant-time equality; currently uses standard `==` which leaks timing information

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Auth token comparison uses `subtle::ConstantTimeEq` or equivalent
- [ ] Add `subtle` crate dependency (or use a manual constant-time compare)
- [ ] All token/secret comparison points use the new method
- [ ] Unit test verifying the comparison works correctly

## Implementation Notes

### Key Files
- Auth middleware in `crates/arawn-server/src/`
- Any other location where bearer tokens or API keys are compared

## Status Updates

### Session — 2026-03-04

**Finding**: The HTTP auth middleware (`auth.rs`) already uses `subtle::ConstantTimeEq` with a well-implemented `constant_time_eq()` helper and 4 unit tests. The `subtle` crate is already a dependency.

**Bug found**: The WebSocket auth path in `handlers.rs:65` used standard `==` comparison (`token == *expected`), bypassing the constant-time helper.

**Fix**: Updated `handlers.rs` to use `subtle::ConstantTimeEq` directly:
```rust
let a = token.as_bytes();
let b = expected.as_bytes();
a.len() == b.len() && a.ct_eq(b).into()
```

**Files modified**:
- `crates/arawn-server/src/routes/ws/handlers.rs` — replaced `==` with constant-time comparison

**Verification**: All 57 arawn-server tests pass, clippy clean.