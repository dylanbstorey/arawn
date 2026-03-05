---
id: split-large-files-tool-rs-types-rs
level: task
title: "Split large files: tool.rs, types.rs, directory.rs"
short_code: "ARAWN-T-0256"
created_at: 2026-03-04T13:24:00.590884+00:00
updated_at: 2026-03-04T23:39:07.400742+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#tech-debt"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
---

# Split large files: tool.rs, types.rs, directory.rs

## Objective

Break apart oversized source files to improve navigability and reduce merge conflicts. Target files: `tool.rs` (3155 lines), `types.rs` (2948 lines), `directory.rs` (2625 lines).

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P2 - Medium (nice to have)

### Technical Debt Impact
- **Current Problems**: Large files are hard to navigate, cause frequent merge conflicts, and make it difficult to understand module boundaries
- **Benefits of Fixing**: Clearer separation of concerns, easier code review, better IDE navigation
- **Risk Assessment**: Low risk if done as pure refactoring with no behavior changes

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `tool.rs` split into: `tool/registry.rs`, `tool/context.rs`, `tool/execution.rs`, `tool/config.rs`
- [ ] `types.rs` split by domain (e.g., `types/session.rs`, `types/message.rs`, `types/tool.rs`)
- [ ] `directory.rs` split into: `directory/manager.rs`, `directory/paths.rs`, `directory/operations.rs`
- [ ] No files over 800 lines after split
- [ ] All existing tests pass unchanged
- [ ] Public API surface unchanged (re-exports from parent modules)

## Status Updates

### Session Progress
- Read and analyzed tool.rs (3603 lines) and directory.rs (2693 lines)
- types.rs is only 751 lines — already under 800, skipping

#### tool.rs split (COMPLETE)
Split into 9 files in `crates/arawn-agent/src/tool/`:
- `mod.rs` (63 lines) — module declarations + re-exports
- `validation.rs` (334 lines) — ParameterValidationError, ParamExt
- `params.rs` (622 lines) — all typed *Params structs
- `output.rs` (335 lines) — OutputConfig, sanitization
- `context.rs` (484 lines) — Tool trait, ToolContext, ToolResult
- `registry.rs` (498 lines) — ToolRegistry struct + MockTool
- `execution.rs` (323 lines) — execute methods + secret resolution
- `gate.rs` (714 lines) — filesystem gate enforcement
- `command_validator.rs` (375 lines) — CommandValidator

#### directory.rs split (COMPLETE)
Split into 6 files in `crates/arawn-workstream/src/directory/`:
- `mod.rs` (198 lines) — error types, result structs, constants
- `manager.rs` (598 lines) — DirectoryManager struct, paths, validation, CRUD
- `operations.rs` (482 lines) — promote, resolve_conflict, export
- `clone.rs` (314 lines) — clone_repo + helpers
- `session.rs` (297 lines) — attach_session + copy_dir_recursive
- `usage.rs` (730 lines) — get_usage, cleanup_work, utility fns

#### Verification (COMPLETE)
- All files under 800 lines (largest: usage.rs at 730, gate.rs at 714)
- `angreal check all` passes (clippy + fmt clean)
- `angreal test unit` passes — 0 failures across entire workspace
- Public API surface unchanged (re-exports from mod.rs files)
- No changes to lib.rs files needed (Rust resolves `pub mod tool;` to `tool/mod.rs` automatically)