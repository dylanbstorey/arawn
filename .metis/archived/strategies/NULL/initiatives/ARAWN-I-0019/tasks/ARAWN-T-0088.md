---
id: end-to-end-test-multi-step
level: task
title: "End-to-end test: multi-step workflow and agent self-extension"
short_code: "ARAWN-T-0088"
created_at: 2026-01-30T03:41:27.317807+00:00
updated_at: 2026-01-30T04:28:58.732391+00:00
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

# End-to-end test: multi-step workflow and agent self-extension

## Parent Initiative

[[ARAWN-I-0019]] — WASM Script Runtime System

## Objective

Create integration tests proving the full WASM runtime pipeline works end-to-end. Test 1: create and run a multi-step workflow (passthrough -> transform -> file_write) where context flows between tasks in dependency order. Test 2: simulate agent self-extension by writing a custom runtime in Rust source, compiling it to wasm32-wasip1, registering it in the catalog, and executing it in a workflow.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Multi-step workflow test creates a workflow with three tasks (passthrough, transform, file_write) with explicit dependency edges
- [ ] Tasks execute in correct topological order respecting dependencies
- [ ] Context propagates correctly — transform receives passthrough output, file_write receives transform output
- [ ] Final file_write produces an output file with the expected transformed content
- [ ] Agent self-extension test writes Rust source for a custom runtime, compiles to `.wasm`, registers via `catalog_register`, creates a workflow using it, and executes successfully
- [ ] Both tests pass in CI (cargo test --test integration)

## Implementation Notes

### Dependencies
- All prior tasks: ARAWN-T-0080 (protocol), T-0081 (catalog), T-0082 (passthrough/http), T-0083 (execute_runtime), T-0084 (factory), T-0085 (wiring), T-0086 (catalog tool actions), T-0087 (file/shell/transform runtimes)

### Approach
Tests live in `tests/integration/` as Rust integration tests. Test 1 uses the MCP tool interface to `action_create` a workflow, then `action_run` it, and asserts on the output file. Test 2 uses `std::process::Command` to invoke `cargo build --target wasm32-wasip1` on a temporary crate, then calls `catalog_register` to add the artifact, creates a workflow referencing the custom runtime name, runs it, and verifies output. Both tests initialize a fresh `RuntimeCatalog` in a temp directory to avoid polluting the real catalog.

## Status Updates

*No updates yet.*