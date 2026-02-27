---
id: wire-memorysearchtool-to-arawn
level: task
title: "Wire MemorySearchTool to arawn-memory backend"
short_code: "ARAWN-T-0233"
created_at: 2026-02-27T00:28:11.191751+00:00
updated_at: 2026-02-27T00:28:11.191751+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#feature"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Wire MemorySearchTool to arawn-memory backend

## Objective

`MemorySearchTool` (`arawn-agent/src/tools/memory.rs`) is a fully-structured stub that always returns empty results. The `arawn-memory` crate now exists with vector store and graph store backends. Wire the tool to query the real memory backend instead of returning stubs.

## Backlog Item Details

### Type
- [x] Feature - New functionality or enhancement

### Priority
- [x] P2 - Medium (nice to have)

### Business Justification
- **User Value**: Agents can search persistent memory for past conversations, facts, and context
- **Business Value**: Core feature for long-running agent sessions and cross-session knowledge
- **Effort Estimate**: M

## Acceptance Criteria

- [ ] `MemorySearchTool` accepts an `Arc<MemoryStore>` (or equivalent) instead of a `connected: bool` flag
- [ ] `execute()` generates an embedding for the query and searches the vector store
- [ ] Results include relevance scores and metadata
- [ ] `MemoryType` enum and `MemoryResult` struct used for typed results
- [ ] Tool registered with real memory backend in `start.rs` when memory is configured
- [ ] Falls back gracefully when memory is not configured (current "disconnected" behavior)
- [ ] Existing tests updated, new integration test with in-memory backend

## Implementation Notes

### Current State
- `MemorySearchTool` at `arawn-agent/src/tools/memory.rs` — full interface with `MemoryType`, `MemoryResult`, parameter schema
- `connected: bool` flag controls stub vs "empty results" path — both return nothing
- Re-exported from `arawn-agent/src/tools/mod.rs` and `arawn-agent/src/lib.rs`
- Output config already wired for `memory_search` tool name
- `arawn/src/client/mod.rs` has `memory_search()` method on client
- `arawn/src/commands/repl.rs` calls `client.memory_search()`

### Key Files
- `crates/arawn-agent/src/tools/memory.rs` — the stub tool
- `crates/arawn-memory/src/lib.rs` — the real memory backend
- `crates/arawn/src/commands/start.rs` — wiring site
- `crates/arawn/src/commands/repl.rs` — REPL memory search command

## Status Updates

*To be added during implementation*