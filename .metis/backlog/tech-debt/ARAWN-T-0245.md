---
id: centralized-filesystem-access-gate
level: task
title: "Centralized Filesystem Access Gate (FsGate)"
short_code: "ARAWN-T-0245"
created_at: 2026-03-02T13:39:50.270831+00:00
updated_at: 2026-03-02T16:31:48.112563+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#tech-debt"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Centralized Filesystem Access Gate (FsGate)

## Objective

Centralize filesystem access control so that **no agent tool can read, write, or execute outside its workstream sandbox**. Currently tools are registered without path constraints — this is a security hole. The fix adds a single enforcement point in `ToolRegistry::execute_with_config()` that gates every filesystem-touching tool through the existing `PathValidator` and `SandboxManager`.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P1 - High (important for user experience)

### Technical Debt Impact
- **Current Problems**: Agent tools (FileReadTool, FileWriteTool, GlobTool, GrepTool, ShellTool) have unrestricted filesystem access. Registered without `base_dir` in `crates/arawn/src/commands/start.rs:369-374`. Tools can read/write anywhere the process user can.
- **Benefits of Fixing**: Workstream sandbox becomes truly enforced. Named workstreams scope tools to `production/` + `work/` (shared across sessions). Scratch sessions are isolated per-session. Shell commands run in OS-level sandbox.
- **Risk Assessment**: Without this, any agent hallucination or prompt injection can access arbitrary files on the host.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] `FsGate` trait defined in `arawn-types` with `validate_read`, `validate_write`, `working_dir`, `sandbox_execute`
- [x] `WorkstreamFsGate` impl in `arawn-workstream` wrapping `PathValidator` + `SandboxManager`
- [x] `ToolContext` carries `Option<Arc<dyn FsGate>>`
- [x] `execute_with_config()` enforces gate for gated tools (file_read, file_write, glob, grep, shell)
- [x] Deny-by-default: gated tool + no gate = error (not silent pass-through)
- [x] Named workstreams: tools access full workstream (`production/` + `work/`), not just session dir
- [x] Scratch workstreams: tools isolated to `scratch/sessions/<id>/work/`
- [x] ShellTool routed through `SandboxManager` for OS-level containment
- [x] `workstream_id` threaded through `turn()`/`turn_stream()` signatures
- [x] All call sites updated, existing tests pass
- [ ] Startup wiring: `FsGateResolver` closure built in `start.rs` (follow-on task — depends on DirectoryManager/SandboxManager construction)

## Implementation Notes

### Architecture

```
ToolContext.fs_gate → ToolRegistry::execute_with_config()
  ├─ file_read/glob/grep → gate.validate_read(path)
  ├─ file_write → gate.validate_write(path)
  └─ shell → gate.sandbox_execute(command)
```

Single chokepoint — individual tools remain unchanged.

### Key Files

| File | Action |
|------|--------|
| `crates/arawn-agent/src/fs_gate.rs` | **Create** — `FsGate` trait + `FsGateError` |
| `crates/arawn-agent/src/lib.rs` | **Edit** — export fs_gate module |
| `crates/arawn-workstream/src/fs_gate.rs` | **Create** — `WorkstreamFsGate` impl |
| `crates/arawn-workstream/src/lib.rs` | **Edit** — export fs_gate module |
| `crates/arawn-agent/src/tool.rs` | **Edit** — add `fs_gate` to `ToolContext`, enforce in `execute_with_config()` |
| `crates/arawn-agent/src/agent.rs` | **Edit** — add `FsGateResolver` type, thread `workstream_id` through `turn()`/`turn_stream()` |
| `crates/arawn/src/commands/start.rs` | **Edit** — build resolver closure, pass to agent |
| `crates/arawn-server/src/routes/chat.rs` | **Edit** — pass `workstream_id` to `turn()` |
| `crates/arawn/src/commands/{chat,run,agent}.rs` | **Edit** — pass `workstream_id` (or `None`) |
| `crates/arawn-agent/src/rlm/spawner.rs` | **Edit** — pass `workstream_id` |

### Sandbox Boundaries

- **Named workstream** (`allowed_paths` returns `[production/, work/]`): all sessions share the full workstream
- **Scratch** (`allowed_paths` returns `[scratch/sessions/<id>/work/]`): per-session isolation
- **Denied paths**: `.ssh`, `.gnupg`, `.aws`, `/etc`, `/usr`, `/System` — always rejected

### Dependencies
- Existing: `PathValidator` (`arawn-workstream`), `SandboxManager` (`arawn-sandbox`), `DirectoryManager` (`arawn-workstream`)
- `FsGate` trait in `arawn-agent` to avoid circular deps (agent → workstream, not vice versa)

### Detailed Plan
See `/Users/dstorey/.claude/plans/crystalline-whistling-feather.md` for full design with code sketches.

## Pre-Implementation Review Findings

### Gap 1: `stream.rs` is a second tool execution path

`turn_stream()` → `create_turn_stream()` in `stream.rs:278` calls `tools.execute()` directly — not through `execute_tools()` in `agent.rs`. Both ultimately hit `execute_with_config()`, so the chokepoint enforcement still works. But `ToolContext` is constructed independently in `stream.rs:261-265`. Must:
- Add `fs_gate: Option<Arc<dyn FsGate>>` to `StreamState`
- Pass it through `create_turn_stream()`
- Set it on the `ToolContext` constructed in `stream.rs`

**File**: `crates/arawn-agent/src/stream.rs` (not in original plan)

### Gap 2: `ChatService` in `arawn-domain` is an intermediary

`arawn-domain/src/services/chat.rs:107` calls `self.agent.turn(session, message)`. The plan lists server routes but not the domain service layer between them. `ChatService::turn()` also needs `workstream_id` threaded through its signature.

**File**: `crates/arawn-domain/src/services/chat.rs` (not in original plan)

### Gap 3: `execute_raw()` is a potential bypass

`ToolRegistry::execute_raw()` at `tool.rs:1179` calls `tool.execute()` directly, skipping `execute_with_config()`. Currently only used in tests, but it's a public API. Should either apply gate enforcement there too, or restrict to `#[cfg(test)]`.

### Full call site inventory

Production callers of `turn()` / `turn_stream()`:
| Caller | File | Notes |
|--------|------|-------|
| `chat` route (sync) | `arawn-server/src/routes/chat.rs:150` | Via `state.agent().turn()` directly |
| `chat_stream` route | `arawn-server/src/routes/chat.rs:267` | Via `state.agent().turn_stream()` |
| `ChatService::turn()` | `arawn-domain/src/services/chat.rs:107` | Domain service intermediary |
| `Orchestrator` | `arawn-agent/src/orchestrator.rs:176` | Multi-turn orchestration loop |
| `AgentSpawner` (2 sites) | `arawn-plugin/src/agent_spawner.rs:481,613` | Plugin-spawned agents |
| `ws handler` | `arawn-server/src/routes/ws/handlers.rs:375` | WebSocket streaming |
| Tests (~15) | `arawn-agent/src/agent.rs` | Pass `None` for workstream_id |

All handled by `workstream_id: Option<&str>` — CLI/tests pass `None`, server routes pass the real ID.

### Updated file list

Added to original plan:
| File | Action |
|------|--------|
| `crates/arawn-agent/src/stream.rs` | **Edit** — add `fs_gate` to `StreamState`, pass to `ToolContext` |
| `crates/arawn-domain/src/services/chat.rs` | **Edit** — thread `workstream_id` through `ChatService::turn()` |
| `crates/arawn-server/src/routes/ws/handlers.rs` | **Edit** — pass `workstream_id` to `turn_stream()` |

## Status Updates

### Session 2 — Implementation Complete

**All core infrastructure is implemented and passing:**

1. **`FsGate` trait** — Created in `crates/arawn-types/src/fs_gate.rs` (moved from arawn-agent to avoid circular dependency)
   - `validate_read()`, `validate_write()`, `working_dir()`, `sandbox_execute()`
   - `FsGateError` enum, `SharedFsGate` type alias, `FsGateResolver` type
   - `GATED_TOOLS` constant + `is_gated_tool()` helper

2. **`WorkstreamFsGate` impl** — Created in `crates/arawn-workstream/src/fs_gate.rs`
   - Wraps `PathValidator` + `SandboxManager`
   - Constructor takes `DirectoryManager`, `SandboxManager`, workstream_id, session_id
   - Unit tests for named workstream access, scratch isolation, denied paths

3. **`ToolContext.fs_gate`** — Added to `crates/arawn-agent/src/tool.rs`
   - Manual Debug impl (SharedFsGate doesn't derive Debug)
   - `with_fs_gate()` builder method

4. **Gate enforcement** — `execute_with_config()` and `execute_raw()` in `tool.rs`
   - Shell tools → `execute_shell_sandboxed()` via `gate.sandbox_execute()`
   - File/search tools → `validate_tool_paths()` via `gate.validate_read()`/`validate_write()`
   - Deny-by-default: gated tool + no gate = error

5. **Agent wiring** — `crates/arawn-agent/src/agent.rs`
   - `FsGateResolver` on `Agent` + `AgentBuilder`
   - `workstream_id: Option<&str>` on `turn()`, `turn_stream()`, `execute_tools()`
   - Gate resolved from resolver in both sync and stream paths

6. **Stream path** — `crates/arawn-agent/src/stream.rs`
   - `fs_gate: Option<SharedFsGate>` on `StreamState` + `create_turn_stream()`
   - ToolContext in streaming loop sets `ctx.fs_gate`

7. **All call sites updated:**
   - `orchestrator.rs:176` → `turn(..., None)`
   - `arawn-domain/services/chat.rs` → `turn(..., workstream_id)` (signature updated)
   - `arawn-server/routes/chat.rs` → sync and stream routes pass `None`
   - `arawn-server/routes/ws/handlers.rs` → passes `workstream_id.as_deref()`
   - `arawn-plugin/agent_spawner.rs` → both delegate and background pass `None`
   - ~15 tests in `agent.rs` → all pass `None`

8. **Verification:**
   - `angreal check all` — clean (0 warnings, 0 errors)
   - `angreal test unit` — all tests pass (0 failures)

### Remaining: Startup wiring (separate task recommended)

The `FsGateResolver` closure needs to be constructed at startup in `start.rs`, which requires:
- Constructing `DirectoryManager` and `SandboxManager` before the `Agent`
- Building the closure: `move |session_id, ws_id| Some(Arc::new(WorkstreamFsGate::new(&dm, sandbox.clone(), ws_id, session_id)))`
- Passing it via `builder.with_fs_gate_resolver(resolver)`

Currently neither `DirectoryManager` nor `SandboxManager` is constructed in `start.rs`. This is infrastructure setup work that should be a follow-on task — it depends on decisions about how these managers are configured at startup.