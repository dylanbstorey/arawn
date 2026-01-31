---
id: scriptexecutor-execute-runtime
level: task
title: "ScriptExecutor: execute_runtime with catalog lookup and caching"
short_code: "ARAWN-T-0083"
created_at: 2026-01-30T03:41:23.325977+00:00
updated_at: 2026-01-30T04:04:43.243476+00:00
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

# ScriptExecutor: execute_runtime with catalog lookup and caching

## Parent Initiative

[[ARAWN-I-0019]] — WASM Script Runtime System

## Objective

Add an `execute_runtime(name: &str, input: &RuntimeInput) -> Result<RuntimeOutput>` method to ScriptExecutor. This method looks up a runtime by name in the RuntimeCatalog, loads the corresponding `.wasm` module (using a disk + in-memory cache to avoid recompilation), serializes the RuntimeInput as JSON to the module's stdin, executes it via wasmtime, and parses stdout as RuntimeOutput.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `execute_runtime` resolves runtime name to `.wasm` path via RuntimeCatalog
- [ ] Compiled WASM modules are cached in memory (HashMap) and optionally on disk as precompiled artifacts
- [ ] RuntimeInput is serialized to JSON and passed to the WASM module's stdin
- [ ] stdout is parsed as RuntimeOutput JSON and returned
- [ ] Returns clear error for unknown runtime names, WASM execution failures, and malformed output
- [ ] Unit tests using the `passthrough` runtime to validate the full serialize-execute-parse cycle

## Implementation Notes

### Dependencies
- ARAWN-T-0080 (protocol types) — RuntimeInput/RuntimeOutput structs
- ARAWN-T-0081 (catalog) — RuntimeCatalog for name-to-path resolution
- ARAWN-T-0082 (passthrough runtime) — needed for testing

### Approach
Extend the existing `ScriptExecutor` struct to hold a `RuntimeCatalog` reference and a `HashMap<String, wasmtime::Module>` cache. On `execute_runtime`, check cache first, otherwise load bytes from the catalog path, compile with `wasmtime::Module::new()`, and cache. Use `wasmtime_wasi` to configure stdin/stdout piping. Serialize input with `serde_json::to_vec`, capture stdout bytes, deserialize output.

## Status Updates

### Session — completed
- Added `execute_runtime(name, input, catalog)` method to `ScriptExecutor`
- Resolves runtime name → .wasm path via `RuntimeCatalog`
- Loads and caches modules by `runtime:{name}` key in the existing `module_cache`
- Serializes `RuntimeInput` to JSON stdin, parses stdout as `RuntimeOutput`
- Clear errors for: unknown runtime, missing .wasm file, non-zero exit, malformed output
- 4 new tests: unknown name, missing wasm, full passthrough round-trip, module caching
- All 16 sandbox tests pass, workspace clean