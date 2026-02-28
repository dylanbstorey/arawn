---
id: wire-memorysearchtool-to-arawn
level: task
title: "Wire MemorySearchTool to arawn-memory backend"
short_code: "ARAWN-T-0233"
created_at: 2026-02-27T00:28:11.191751+00:00
updated_at: 2026-02-28T19:03:12.644860+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#feature"
  - "#phase/completed"


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

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] `MemorySearchTool` accepts an `Arc<MemoryStore>` (or equivalent) instead of a `connected: bool` flag
- [x] `execute()` searches the memory store via text-based `search_memories_in_range()` (embedding not needed for text search)
- [x] Results include relevance scores and metadata (confidence score, content_type, session_id, created_at)
- [x] Vestigial `MemoryType`/`MemoryResult` removed — tool uses `arawn_memory` types directly
- [x] Tool registered with real memory backend in `start.rs` when memory is configured
- [x] Falls back gracefully when memory is not configured (current "disconnected" behavior)
- [x] Existing tests updated, new tests with in-memory backend (7 test cases)

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

### Session 2026-02-28

**Tool rewrite complete** (`crates/arawn-agent/src/tools/memory.rs`):
- Replaced `connected: bool` with `store: Option<Arc<MemoryStore>>`
- Added `with_store(store: Arc<MemoryStore>)` constructor
- `execute()` performs real search via `search_memories_in_range()` (text-based, no embedding needed)
- Content type filtering: conversation→UserMessage+AssistantMessage, fact→Fact, preference→Note, research→WebContent+FileContent
- Supplements results with notes search when capacity remains
- Removed vestigial `MemoryType` and `MemoryResult` types (tool now uses `arawn_memory` types directly)
- 7 comprehensive tests: disconnected, with_store, time_range, empty_results, missing_query, parse helpers

**Wiring in start.rs complete**:
- Moved `MemoryStore::open()` + init earlier in startup (before tool registry), decoupling it from `indexing.enabled`
- Registered `MemorySearchTool::with_store(store)` when store available, `MemorySearchTool::new()` as disconnected fallback
- Refactored indexer section to reuse the early-created `memory_store` Arc instead of re-opening
- Memory search now works even when session indexing is disabled (as long as DB can be opened)

**Verification**: `angreal check all` clean, `angreal test unit` all 46 suites pass, 14 memory-specific tests pass