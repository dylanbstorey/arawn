---
id: fix-file-write-path-traversal-via
level: task
title: "Fix file write path traversal via relative sequences"
short_code: "ARAWN-T-0254"
created_at: 2026-03-04T13:23:58.278061+00:00
updated_at: 2026-03-04T18:29:21.848476+00:00
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

# Fix file write path traversal via relative sequences

## Objective

Ensure FileWriteTool rejects paths containing `../` sequences that could escape the workstream sandbox, even after FsGate validation.

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing

### Priority
- [x] P2 - Medium (nice to have)

### Impact Assessment
- **Severity**: MEDIUM - Defense-in-depth; FsGate should catch this, but the tool should also validate
- **Expected vs Actual**: Relative path sequences should be resolved and checked; raw `../` in paths could bypass naive prefix checks

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] FileWriteTool canonicalizes paths before writing (added `reject_traversal()` + `normalize_path()`)
- [x] Paths containing `../` that resolve outside allowed dirs are rejected (both with and without `base_dir`)
- [x] Unit tests for traversal attempts (7 new tests)

### Key Files
- `crates/arawn-agent/src/tools/file_write.rs`
- `crates/arawn-workstream/src/fs_gate.rs`

## Status Updates

### Implementation Complete

**Vulnerabilities found and fixed in `crates/arawn-agent/src/tools/file.rs`:**

1. **`FileWriteTool::resolve_path()` without `base_dir`** (line 220-222): Returned raw path with zero validation. Fixed by adding `reject_traversal()` check that rejects any path containing `..` components.

2. **`FileWriteTool::resolve_path()` with `base_dir` when parent doesn't exist** (line 212-219): Used `starts_with()` on non-canonicalized path — e.g., `/base/nonexistent/../../etc/passwd` passed the prefix check. Fixed by applying `normalize_path()` which lexically resolves `..` and `.` before the prefix check.

3. **`FileReadTool::resolve_path()` without `base_dir`**: Same as #1 — returned raw path. Fixed with `reject_traversal()`.

**New utilities added:**
- `reject_traversal(path)` — rejects paths containing `Component::ParentDir` 
- `normalize_path(path)` — lexically resolves `..` and `.` without filesystem access

**Tests added (6 new, all passing):**
- `test_reject_traversal_blocks_dotdot` — verifies `../` sequences are rejected
- `test_reject_traversal_allows_normal_paths` — verifies clean paths pass
- `test_normalize_path_resolves_dotdot` — verifies lexical normalization
- `test_file_write_traversal_rejected_no_base` — write with `../` and no base_dir
- `test_file_write_traversal_rejected_with_base` — write with `../` and base_dir
- `test_file_read_traversal_rejected` — read with `../`
- `test_file_write_base_dir_traversal_nonexistent_parent` — traversal via nonexistent intermediate dir

**Verification:**
- `angreal check all` — clippy + fmt clean
- `angreal test unit` — all tests pass (0 failures)