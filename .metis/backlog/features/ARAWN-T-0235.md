---
id: complete-memory-cli-add-recent-and
level: task
title: "Complete memory CLI — add recent and export subcommands"
short_code: "ARAWN-T-0235"
created_at: 2026-02-27T01:01:24.945568+00:00
updated_at: 2026-02-28T19:56:15.736506+00:00
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

# Complete memory CLI — add recent and export subcommands

## Objective

The `arawn memory` CLI has `recent` and `export` subcommands that print "not yet implemented". Unlike the notes stubs (which just need client wiring), these require **new server endpoints** — there is no `GET /api/v1/memory/recent` or `GET /api/v1/memory/export` on the server today.

## Backlog Item Details

### Type
- [x] Feature - New functionality or enhancement

### Priority
- [x] P3 - Low (when time permits)

### Business Justification
- **User Value**: `recent` shows what the memory system has learned recently (useful for debugging/inspection). `export` enables backup and portability of the memory store.
- **Business Value**: Completes the memory CLI surface; makes the memory system more transparent and debuggable
- **Effort Estimate**: M — requires new server endpoints + client methods + CLI wiring

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] `arawn memory recent [--limit N]` shows the N most recently created memories, sorted by time
- [x] `arawn memory export [path]` dumps all memories + notes as JSON (to file or stdout)
- [x] No new server endpoints needed — follows existing pattern of direct local store access (like `stats` and `reindex`)
- [x] JSON output mode works for both subcommands
- [x] No "not yet implemented" messages remain in memory.rs

## Implementation Notes

### Current State
- **Server**: `arawn-server/src/routes/memory.rs` has `memory_search_handler`, `store_memory_handler`, `delete_memory_handler`. No recent/export endpoints.
- **Client**: `arawn/src/client/mod.rs` has `memory_search()` only
- **CLI**: `arawn/src/commands/memory.rs` — `search`, `stats`, `reembed` work. `recent` (line 116) and `export` (lines 261-265) are stubs.
- **MemoryStore**: `arawn-memory/src/store/` — may need `list_recent_memories(limit)` and `export_all()` methods

### Approach
1. Add `list_recent(limit)` to `MemoryStore` — query memories table ordered by `created_at DESC`
2. Add `export_all()` to `MemoryStore` — iterate all memories + notes
3. Add server endpoints
4. Add client methods
5. Wire CLI stubs

### Key Files
- `crates/arawn-memory/src/store/memory_ops.rs` — add store methods
- `crates/arawn-server/src/routes/memory.rs` — add endpoints
- `crates/arawn/src/client/mod.rs` — add client methods
- `crates/arawn/src/commands/memory.rs` — wire stubs

## Status Updates

### Session 2026-02-28

**Key decision**: `stats` and `reindex` already use direct local store access via `open_memory_store()`, not server endpoints. Following this pattern for `recent` and `export` — no new server endpoints or client methods needed since `list_memories(None, limit, 0)` already returns memories ordered by `created_at DESC` (which IS "recent").

**Changes** (`crates/arawn/src/commands/memory.rs`):
- `cmd_recent`: Uses `open_memory_store()` + `list_memories(None, limit, 0)`. Displays content_type label, truncated content, and timestamp. JSON output mode supported.
- `cmd_export`: Uses `open_memory_store()` + `list_memories` + `list_notes`. Exports JSON with `memories`, `notes`, `exported_at`, `counts`. Writes to file or stdout. Summary confirmation in non-JSON mode.
- All "not yet implemented" messages removed.

**Verification**: `angreal check all` clean, `angreal test unit` all suites pass