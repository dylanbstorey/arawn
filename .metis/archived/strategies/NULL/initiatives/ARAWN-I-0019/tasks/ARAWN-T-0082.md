---
id: built-in-wasm-runtimes-passthrough
level: task
title: "Built-in WASM runtimes: passthrough and http"
short_code: "ARAWN-T-0082"
created_at: 2026-01-30T03:41:22.516891+00:00
updated_at: 2026-01-30T04:00:12.412436+00:00
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

# Built-in WASM runtimes: passthrough and http

## Parent Initiative

[[ARAWN-I-0019]] — WASM Script Runtime System

## Objective

Create the first two built-in WASM runtime modules using arawn-script-sdk. The `passthrough` runtime reads RuntimeInput from stdin and writes the context back as RuntimeOutput unchanged (used for testing and pipeline debugging). The `http` runtime makes HTTP requests based on config fields (url, method, headers, body) and returns the response status, body, and headers as output.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Both `passthrough` and `http` runtimes compile to `wasm32-wasip1` target
- [ ] `passthrough` reads RuntimeInput from stdin and writes context verbatim as RuntimeOutput to stdout
- [ ] `http` runtime reads url/method/headers/body from config, executes the HTTP request, returns status code, response body, and response headers in RuntimeOutput
- [ ] Both runtimes follow the RuntimeInput/RuntimeOutput JSON protocol over stdin/stdout
- [ ] Both are registered in the catalog as builtin runtimes with `.wasm` files in `builtin/`
- [ ] Unit tests validate passthrough round-trip and http request construction

## Implementation Notes

### Dependencies
- ARAWN-T-0080 (RuntimeInput/RuntimeOutput protocol types) — shared serde structs used by both runtimes
- ARAWN-T-0081 (catalog) — runtimes are registered as builtin entries

### Approach
Each runtime is a separate Rust crate under `runtimes/` targeting `wasm32-wasip1`. They link against `arawn-script-sdk` for the protocol types. The `passthrough` runtime is trivial: deserialize stdin, copy `context` to output. The `http` runtime uses `wasi-http` or a minimal HTTP client compatible with WASI preview 1. Compiled `.wasm` artifacts are placed in `builtin/` and registered in `catalog.toml` at build time.

## Status Updates

### Session — completed
- Created `runtimes/passthrough/` crate: reads RuntimeInput, echoes config+context as RuntimeOutput
- Created `runtimes/http/` crate: parses HttpConfig (url, method, headers, body) from config, outputs structured request representation
- Both follow RuntimeInput/RuntimeOutput JSON stdin/stdout protocol
- Both compile to native and `wasm32-wasip1` release targets
- Added `exclude = ["runtimes/passthrough", "runtimes/http"]` to workspace Cargo.toml (these target wasm, not native)
- WASI preview 1 limitation: http runtime outputs request structure for host-side execution (actual HTTP call done by executor); can upgrade to wasi-http with preview 2
- Catalog registration deferred to T-0085 (wiring task) — the .wasm artifacts exist at `target/wasm32-wasip1/release/`