---
id: handle-symlink-escape-in-fsgate
level: task
title: "Handle symlink escape in FsGate path validation"
short_code: "ARAWN-T-0252"
created_at: 2026-03-04T13:23:53.197583+00:00
updated_at: 2026-03-04T17:50:54.905567+00:00
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

# Handle symlink escape in FsGate path validation

## Objective

Harden FsGate path validation against symlink-based sandbox escape. A symlink inside the allowed directory could point outside it, bypassing path validation that only checks the logical path.

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing

### Priority
- [x] P2 - Medium (nice to have)

### Impact Assessment
- **Severity**: MEDIUM - Requires ability to create symlinks inside the workstream directory, which limits practical exploitability
- **Expected vs Actual**: Path validation should resolve symlinks before checking against allowed paths; currently may only check the logical path

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `validate_read()` and `validate_write()` resolve symlinks (canonicalize) before boundary check
- [ ] Symlink pointing outside workstream directory is rejected
- [ ] Symlink pointing within workstream directory is allowed
- [ ] Unit tests for symlink escape attempts

## Implementation Notes

### Key Files
- `crates/arawn-workstream/src/fs_gate.rs`
- `crates/arawn-workstream/src/path_validator.rs`

## Status Updates

### Session — 2026-03-04

**Finding**: `validate()` (read path) already had full symlink protection — canonicalizes paths before boundary checks and has explicit `SymlinkEscape` detection. Tests existed for both symlink-within-allowed and symlink-escape scenarios.

**Gap found**: `validate_write()` only checked the parent directory. If the target file already existed and was a symlink pointing outside the sandbox, `validate_write()` would approve the path and a subsequent write would follow the symlink to overwrite an external file.

**Fix**: Added symlink-aware checking to `validate_write()`:
1. If target path exists, canonicalize the full path (resolves symlinks)
2. Check canonical path against denied paths
3. If it's a symlink, verify the target is under an allowed directory
4. Check canonical path against allowed paths
5. For new files (don't exist yet), fall through to the existing parent-based check

**Tests added** (3 new):
- `test_validate_write_symlink_escape_rejected` — symlink inside sandbox pointing to file outside → rejected
- `test_validate_write_symlink_within_allowed_succeeds` — symlink within sandbox pointing to file inside → allowed
- `test_validate_write_symlink_dir_escape_rejected` — parent directory is a symlink to outside → rejected

**Files modified**:
- `crates/arawn-workstream/src/path_validator.rs` — hardened `validate_write()`, added 3 tests

**Verification**: All 20 path_validator tests pass, `angreal check all` clean.