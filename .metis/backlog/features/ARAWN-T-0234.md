---
id: complete-notes-cli-wire-remaining
level: task
title: "Complete notes CLI — wire remaining CRUD operations to server API"
short_code: "ARAWN-T-0234"
created_at: 2026-02-27T01:01:23.926633+00:00
updated_at: 2026-02-27T01:01:23.926633+00:00
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

# Complete notes CLI — wire remaining CRUD operations to server API

## Objective

The server has full CRUD for notes (`POST/GET/PUT/DELETE /api/v1/notes` + `GET /api/v1/notes/:id`), all implemented and tested. The CLI client (`arawn/src/client/mod.rs`) only exposes `create_note` and `list_notes`. The CLI command (`notes.rs`) has `search`, `show`, and `delete` subcommands that print "not yet implemented" because the client methods don't exist. Wire the remaining operations through.

## Backlog Item Details

### Type
- [x] Feature - New functionality or enhancement

### Priority
- [x] P2 - Medium (nice to have)

### Business Justification
- **User Value**: Users can manage notes from the CLI — view, search, delete individual notes
- **Business Value**: Completes the notes CLI surface; eliminates "not yet implemented" messages
- **Effort Estimate**: S — the server endpoints exist and are tested, this is just client + CLI wiring

## Acceptance Criteria

- [ ] `Client::get_note(id)` method calls `GET /api/v1/notes/:id`
- [ ] `Client::delete_note(id)` method calls `DELETE /api/v1/notes/:id`
- [ ] `Client::search_notes(query, limit)` method calls `GET /api/v1/notes?q=...` (or memory search with note filter)
- [ ] `arawn notes show <id>` displays note content, tags, timestamps
- [ ] `arawn notes delete <id>` deletes the note with confirmation
- [ ] `arawn notes search <query>` returns matching notes
- [ ] JSON output mode works for all three subcommands
- [ ] No "not yet implemented" messages remain in notes.rs

## Implementation Notes

### Current State
- **Server**: Full CRUD at `arawn-server/src/routes/memory.rs` — `create_note_handler`, `list_notes_handler`, `get_note_handler`, `update_note_handler`, `delete_note_handler`. All tested.
- **Client**: `arawn/src/client/mod.rs` — only `create_note()` and `list_notes()` exist
- **CLI**: `arawn/src/commands/notes.rs` — `Add` and `List` work, `Search`/`Show`/`Delete` are stubs (lines 119-142)
- **Note type**: `arawn-memory::types::Note` — id, title, content, tags, created_at, updated_at

### Key Files
- `crates/arawn/src/client/mod.rs` — add `get_note`, `delete_note`, `search_notes` methods
- `crates/arawn/src/commands/notes.rs` — wire the three stub match arms
- `crates/arawn-server/src/routes/memory.rs` — server endpoints (reference, don't modify)

### Notes vs Memory distinction
Notes are discrete user-created records (title, content, tags). Memory is the cognitive system (embeddings, knowledge graph, confidence scores, cache invalidation, ontologies). Notes surface in memory search as supplementary results but are a separate CRUD resource. `search_notes` in MemoryStore does text matching on note content — distinct from `search_memories` which does vector/semantic search.

## Status Updates

*To be added during implementation*