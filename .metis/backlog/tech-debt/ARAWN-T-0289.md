---
id: enable-and-fix-sandbox-execution
level: task
title: "Enable and fix sandbox execution tests (currently #[ignore])"
short_code: "ARAWN-T-0289"
created_at: 2026-03-08T03:17:34.015303+00:00
updated_at: 2026-03-08T03:17:34.015303+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#tech-debt"


exit_criteria_met: false
initiative_id: NULL
---

# Enable and fix sandbox execution tests (currently #[ignore])

## Objective

`arawn-sandbox` has 17 tests, but the 3 most important ones — actual sandboxed command execution — are marked `#[ignore]`. These are the tests that verify the sandbox actually works: commands run, file writes are restricted, read access is controlled. Fix and enable them so they run in `angreal test integration` and optionally in CI.

### Priority
- [x] P2 - Medium (sandbox not yet wired up via ARAWN-T-0278, but needed once it is)
- **Size**: S

### Current Problems
- 3 tests in `manager.rs` are `#[ignore]`:
  - `test_sandboxed_echo` — runs `echo hello` in sandbox
  - `test_sandboxed_write_file` — writes a file inside sandbox boundaries
  - `test_sandboxed_restricted_read` — attempts to read outside sandbox (should be denied)
- These tests likely fail because they require:
  - A compiled WASM shell runtime
  - `rustup` with `wasm32-wasip1` target installed
  - Platform-specific sandbox support (macOS sandbox-exec or Linux landlock)
- 4 tests in `arawn-pipeline/src/loader.rs` are `#[ignore]` due to filesystem timing (debounce)
- 2 tests in `arawn-plugin/src/subscription.rs` are `#[ignore]` — require hook dispatcher
- `angreal test integration` runs `cargo test -- --ignored --test-threads=1` but it's unclear if these pass

## Acceptance Criteria

- [ ] All 3 sandbox execution tests pass when run with `angreal test integration`
- [ ] Tests gracefully skip (not fail) when platform doesn't support sandboxing
- [ ] Tests gracefully skip when WASM target not installed
- [ ] Add a `can_sandbox()` helper function that checks prerequisites
- [ ] Pipeline loader ignored tests investigated and either fixed or documented why they must remain ignored
- [ ] Plugin subscription ignored tests investigated and either fixed or documented
- [ ] `angreal test integration` in CI runs the non-platform-specific ignored tests

## Implementation Notes

### Sandbox test prerequisites

```rust
fn can_sandbox() -> bool {
    // Check 1: Platform supports sandboxing
    if !SandboxManager::is_available() { return false; }
    // Check 2: WASM target installed
    // Check 3: Shell runtime compiled or compilable
    true
}

#[test]
fn test_sandboxed_echo() {
    if !can_sandbox() {
        eprintln!("Skipping: sandbox prerequisites not met");
        return;
    }
    // ... actual test
}
```

### Approach

1. Run the ignored tests locally to see what actually fails
2. Fix any compilation or runtime issues
3. Replace `#[ignore]` with conditional skip based on `can_sandbox()`
4. Add a CI step that installs `wasm32-wasip1` target and runs integration tests

### Key files
- `crates/arawn-sandbox/src/manager.rs` — 3 ignored tests
- `crates/arawn-pipeline/src/loader.rs` — 4 ignored tests (filesystem timing)
- `crates/arawn-plugin/src/subscription.rs` — 2 ignored tests (hook dispatcher)
- `.github/workflows/ci.yml` — Add integration test step

### Dependencies
- Related to ARAWN-T-0278 (FsGate wiring) — sandbox tests validate what the gate system depends on

## Status Updates

*To be added during implementation*