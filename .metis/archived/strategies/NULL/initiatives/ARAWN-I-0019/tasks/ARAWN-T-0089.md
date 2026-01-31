---
id: auto-compile-and-register-built-in
level: task
title: "Auto-compile and register built-in WASM runtimes at server startup"
short_code: "ARAWN-T-0089"
created_at: 2026-01-30T04:36:29.954339+00:00
updated_at: 2026-01-30T04:42:31.934822+00:00
parent: ARAWN-I-0019
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0019
---

# Auto-compile and register built-in WASM runtimes at server startup

## Parent Initiative

[[ARAWN-I-0019]] — WASM Script Runtime System

## Objective

At server startup, automatically compile each `runtimes/*` crate to `wasm32-wasip1` and register the resulting `.wasm` in the `RuntimeCatalog` as builtin entries. Skip compilation if already registered. This ensures runtimes (passthrough, http, file_read, file_write, shell, transform) are available without manual build steps.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `ScriptExecutor` has a `compile_crate()` method that runs `cargo build --target wasm32-wasip1 --release` on a crate directory
- [ ] On startup, each `runtimes/*` subdirectory is compiled and registered as a builtin in the catalog
- [ ] Already-registered runtimes are skipped (no recompilation on subsequent starts)
- [ ] Runtime source directory discovered via `env!("CARGO_MANIFEST_DIR")` (dev builds); gracefully skipped if not found (release/installed builds)
- [ ] `catalog list` via agent returns all 6 built-in runtimes after fresh start
- [ ] `cargo check --workspace` passes

## Implementation Notes

### Technical Approach
1. Add `compile_crate(crate_dir: &Path) -> Result<PathBuf>` to `ScriptExecutor` — runs `cargo build --target wasm32-wasip1 --release`, finds `.wasm` in target dir
2. Add `register_builtin_runtimes()` async fn in `start.rs` — iterates `runtimes/` subdirs, skips already-registered, compiles, copies `.wasm` to `builtin/`, registers in catalog
3. Discover runtimes source dir via `env!("CARGO_MANIFEST_DIR")` → workspace root → `runtimes/`

### Dependencies
- ARAWN-T-0081 (catalog), ARAWN-T-0083 (ScriptExecutor), ARAWN-T-0087 (runtime crates)

### Files to Modify
- `crates/arawn-pipeline/src/sandbox.rs` — add `compile_crate()`
- `crates/arawn/src/commands/start.rs` — add `register_builtin_runtimes()` + call at startup

## Status Updates

*No updates yet.*