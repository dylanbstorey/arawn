---
id: wasmtime-sandbox-integration-for
level: task
title: "Wasmtime sandbox integration for script execution"
short_code: "ARAWN-T-0076"
created_at: 2026-01-29T18:34:45.231840+00:00
updated_at: 2026-01-30T02:28:30.536605+00:00
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

# Wasmtime sandbox integration for script execution

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[ARAWN-I-0012]]

## Objective

Integrate Wasmtime as the sandbox runtime for executing agent-generated Rust scripts within workflow tasks. This implements the `script` action type per ARAWN-A-0002. Agent-generated Rust code is compiled to `wasm32-wasip1` and executed in a capability-scoped WASI sandbox.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `wasmtime` crate added as dependency to `arawn-pipeline`
- [ ] `ScriptExecutor` struct that manages compilation and execution
- [ ] `ScriptExecutor::compile(source: &str)` — invokes `rustc` targeting `wasm32-wasip1`, returns compiled WASM module bytes
- [ ] `ScriptExecutor::execute(module, context, capabilities)` — runs WASM module in Wasmtime with scoped WASI permissions
- [ ] Capability grants: filesystem paths (read/write scoped), network (on/off), memory limits, execution timeout
- [ ] Context passed in via WASI stdin (JSON), output captured from WASI stdout (JSON)
- [ ] Compile error capture — rustc stderr returned as structured error for agent self-correction
- [ ] WASM module caching — identical source content hashes to skip recompilation
- [ ] `wasm32-wasip1` target must be installed (detect and provide helpful error if missing)
- [ ] Tests: compile simple Rust script, execute and capture output, capability restriction (no network access when denied), compile error returned cleanly, cache hit on identical source
- [ ] Tests pass

## Implementation Notes

### Technical Approach
- Shell out to `rustc --target wasm32-wasip1` for compilation (or `cargo` with a minimal manifest)
- Use `wasmtime` crate API: `Engine`, `Module`, `Store`, `Linker` with `wasmtime_wasi` for WASI integration
- `WasiCtx` builder for capability scoping: `dir()` for filesystem, stdin/stdout for context I/O
- Cache compiled `.wasm` files keyed by SHA256 of source content
- Pre-compiled SDK (T-0077) linked via `--extern` flag during compilation

### Dependencies
- ARAWN-T-0072 (PipelineEngine provides the execution context)
- ARAWN-T-0077 (script SDK — can be developed in parallel, but scripts need it for useful operations)

### Risk Considerations
- `wasm32-wasip1` target not installed by default — need clear error messaging
- WASI networking proposals still maturing — initial version may be filesystem + stdin/stdout only
- Compile times: 3-8s cold, mitigated by caching

## Status Updates

### Completed
- `sandbox.rs` with `ScriptExecutor` struct managing compilation and sandboxed execution
- `compile()` — invokes `rustc --target wasm32-wasip1`, SHA-256 cache (disk + in-memory)
- `execute()` — runs WASM in Wasmtime with WASI P1, context via stdin, output from stdout
- `compile_and_execute()` — convenience combo method
- Capability scoping: preopened filesystem dirs via `WasiCtxBuilder::preopened_dir()`
- Fuel-based execution limits for timeout enforcement
- `check_wasm_target()` — detects missing target with actionable error message
- `MemoryInputPipe`/`MemoryOutputPipe` for stdin/stdout capture
- `I32Exit` handling for WASI proc_exit codes
- Added `wasmtime = "41"`, `wasmtime-wasi = "41"`, `sha2`, `hex` dependencies
- Added `CompilationFailed` and `ScriptFailed` error variants
- 12 tests all passing (compile, cache hit, compile error, execute, stdin context, exit code, cache clear, nonexistent hash)
- Workspace compiles clean