---
id: complete-memory-cli-add-recent-and
level: task
title: "Complete memory CLI — add recent and export subcommands"
short_code: "ARAWN-T-0235"
created_at: 2026-02-27T01:01:24.945568+00:00
updated_at: 2026-02-27T01:01:24.945568+00:00
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

- [ ] `arawn memory recent [--limit N]` shows the N most recently created memories, sorted by time
- [ ] `arawn memory export [path]` dumps all memories as JSON (to file or stdout)
- [ ] Server endpoints exist: `GET /api/v1/memory/recent` and `GET /api/v1/memory/export`
- [ ] JSON output mode works for both subcommands
- [ ] No "not yet implemented" messages remain in memory.rs

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

*To be added during implementation*