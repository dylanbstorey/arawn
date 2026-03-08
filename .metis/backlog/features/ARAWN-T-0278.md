---
id: wire-up-fsgateresolver
level: task
title: "Wire up FsGateResolver, DirectoryManager, and SandboxManager in server startup"
short_code: "ARAWN-T-0278"
created_at: 2026-03-08T03:11:19.758828+00:00
updated_at: 2026-03-08T14:51:12.353744+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#feature"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
---

# Wire up FsGateResolver, DirectoryManager, and SandboxManager in server startup

## Objective

Wire up the filesystem gate system in `crates/arawn/src/commands/start.rs` so that gated tools (file_read, file_write, glob, grep, shell) actually work for agents. Currently all gated tools are permanently denied because no `FsGateResolver` is configured on the agent builder.

## Problem

The `FsGateResolver`, `DirectoryManager`, and `SandboxManager` exist in crate code but are never integrated into the server startup path:

- `WorkstreamFsGate` in `crates/arawn-workstream/src/fs_gate.rs` — fully implemented
- `DirectoryManager` in `crates/arawn-workstream/src/directory.rs` — fully implemented
- `SandboxManager` in `crates/arawn-sandbox/` — exists but shell WASM runtime fails to compile under launchd (no `rustup` in PATH)
- `Agent::builder().with_fs_gate_resolver()` — never called in `start.rs`

**Impact**: Every gated tool call returns `"Tool 'X' requires a filesystem gate but none is configured"`. The agent loops trying gated tools, gets denied, tries another, denied again — burning tokens and producing unhelpful responses.

### Priority
- [x] P1 - High (blocks core agent functionality)

### Effort Estimate
- **Size**: M

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] `DirectoryManager` created and wired into `WorkstreamManager` during startup
- [x] `SandboxManager` created (gracefully handling missing WASM runtime)
- [x] `FsGateResolver` closure constructed and passed to `Agent::builder().with_fs_gate_resolver()`
- [x] `glob` and `grep` work in agent sessions (read-only, path-validated)
- [x] `file_read` and `file_write` work within workstream boundaries
- [x] `shell` works when WASM runtime is available; clear error when not
- [x] launchd wrapper script updated to include `~/.cargo/bin` in PATH so `rustup` is available for shell runtime compilation
- [x] Gate denial logged at WARN level (already done in `execution.rs`)

## Implementation Notes

### Technical Approach

1. **In `start.rs`** (agent construction section ~line 1100):
   - Create `DirectoryManager` from config data dir
   - Create `SandboxManager` (wrap in `Arc`)
   - Build `FsGateResolver` closure that creates `WorkstreamFsGate` instances:
     ```rust
     let dm = Arc::new(DirectoryManager::new(&data_dir));
     let sandbox = Arc::new(SandboxManager::new(/* runtime */));
     let resolver: FsGateResolver = Arc::new(move |session_id, workstream_id| {
         Some(Arc::new(WorkstreamFsGate::new(&dm, sandbox.clone(), workstream_id, session_id)))
     });
     builder = builder.with_fs_gate_resolver(resolver);
     ```

2. **In `arawn-wrapper.sh`**:
   - Add `export PATH="$HOME/.cargo/bin:$PATH"` so `rustup` is available for WASM compilation

3. **Graceful degradation**: If `SandboxManager` can't initialize (no WASM runtime), still create the gate resolver but have `shell` tool return a clear error. Other gated tools (glob, grep, file_read, file_write) should still work with path validation only.

### Key Files
- `crates/arawn/src/commands/start.rs` — agent construction (~line 1100)
- `crates/arawn-workstream/src/fs_gate.rs` — `WorkstreamFsGate`
- `crates/arawn-workstream/src/directory.rs` — `DirectoryManager`
- `crates/arawn-sandbox/src/lib.rs` — `SandboxManager`
- `crates/arawn-agent/src/tool/execution.rs` — gate enforcement (logging already added)
- `~/.config/arawn/arawn-wrapper.sh` — launchd PATH fix

### Dependencies
- `arawn-sandbox` crate must be a dependency of the `arawn` binary crate
- Shell WASM runtime compilation requires `rustup` + `wasm32-wasip1` target

## Status Updates

### Completed

**WorkstreamFsGate refactored**: Made `sandbox_manager` field `Option<Arc<SandboxManager>>` to support path-only mode. Added `path_only()` constructor that enables file tools without requiring sandbox. Shell commands return a clear error when sandbox is unavailable.

**FsGateResolver wired in start.rs**: Added ~50 lines after secret resolver wiring (line ~1158). Creates `DirectoryManager` from workstream data dir, tries `SandboxManager::new().await` with graceful fallback, builds closure that creates `WorkstreamFsGate` per session/workstream.

**Dependency added**: `arawn-sandbox` added to `crates/arawn/Cargo.toml`.

**Launchd wrapper updated**: `scripts/service/arawn-wrapper.sh` now exports `$HOME/.cargo/bin` in PATH so `rustup` is available for WASM compilation.

**Files modified**:
- `crates/arawn/Cargo.toml` — added arawn-sandbox dependency
- `crates/arawn/src/commands/start.rs` — FsGateResolver wiring + imports
- `crates/arawn-workstream/src/fs_gate.rs` — Optional sandbox, path_only constructor
- `scripts/service/arawn-wrapper.sh` — PATH fix for launchd