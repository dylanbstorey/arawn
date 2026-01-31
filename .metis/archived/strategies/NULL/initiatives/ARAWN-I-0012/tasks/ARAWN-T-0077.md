---
id: arawn-script-sdk-crate
level: task
title: "arawn-script-sdk crate"
short_code: "ARAWN-T-0077"
created_at: 2026-01-29T18:34:48.610292+00:00
updated_at: 2026-01-30T02:34:20.987867+00:00
parent: ARAWN-I-0012
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0012
---

# arawn-script-sdk crate

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[ARAWN-I-0012]]

## Objective

Create the `arawn-script-sdk` crate — a pre-compiled library of common utilities that agent-generated Rust scripts link against. This keeps compile times fast (no dependency resolution) and provides a consistent API surface for sandboxed scripts.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `crates/arawn-script-sdk` exists in workspace, compiles to `wasm32-wasip1`
- [ ] JSON utilities: parse stdin context, serialize stdout output, field access helpers
- [ ] String/text utilities: regex matching, string manipulation, formatting
- [ ] Data utilities: basic math, sorting, filtering collections
- [ ] Error handling: `ScriptResult<T>` type that maps to clean JSON error output
- [ ] `main()` harness: macro or function that handles stdin→context deserialization and output→stdout serialization, so agent scripts only implement a `run(context) -> Result<Value>` function
- [ ] Pre-compiled `.rlib` for `wasm32-wasip1` target, shipped alongside Arawn
- [ ] Example scripts demonstrating SDK usage (for agent training/reference)
- [ ] Tests: SDK compiles to WASM, example scripts compile and execute correctly via T-0076
- [ ] Tests pass

## Implementation Notes

### Technical Approach
- `#![no_std]`-compatible where possible to minimize WASM binary size, but pragmatically use std where needed (stdin/stdout require it)
- `serde` + `serde_json` for context serialization (these compile cleanly to WASM)
- `regex` crate compiles to WASM — include it
- HTTP client deferred until WASI networking stabilizes — document this limitation
- The SDK defines a trait/function signature that agent scripts implement; the harness handles I/O boilerplate

### Dependencies
- No ARAWN-T-* dependencies — can be developed in parallel with T-0076
- Must target `wasm32-wasip1`

## Status Updates

### Completed
- Created `crates/arawn-script-sdk` crate, added to workspace
- **context.rs**: `Context` wrapper with dot-path navigation, typed getters (`get_str`, `get_i64`, `get_f64`, `get_bool`, `get_array`, `get_object`, `get_as<T>`), array index support
- **error.rs**: `ScriptError` enum (Message/Json/Io/Regex) with `From` impls, `ScriptResult<T>` alias
- **text.rs**: `matches()`, `find_all()`, `replace_all()`, `split()`, `extract()` (named captures), `truncate()`, `word_count()`, `estimate_tokens()`
- **lib.rs**: `entry!` macro generating `main()` harness (stdin→Context, Result→stdout JSON), `prelude` module
- Dependencies: `serde`, `serde_json`, `regex` (all compile to wasm32-wasip1)
- Compiles for both native and `wasm32-wasip1` targets
- Pre-compiled `.rlib` built at `target/wasm32-wasip1/release/libarawn_script_sdk.rlib`
- 18 unit tests + 2 doc-tests passing
- Workspace compiles clean