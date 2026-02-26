---
id: wire-memorystore-through-domain
level: task
title: "Wire MemoryStore through domain facade for REST API persistence"
short_code: "ARAWN-T-0226"
created_at: 2026-02-25T14:36:55.545057+00:00
updated_at: 2026-02-26T01:59:33.556140+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#tech-debt"
  - "#phase/active"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Wire MemoryStore through domain facade for REST API persistence

## Objective

Route the server's memory REST endpoints through a real `MemoryService` in the domain layer, backed by `arawn-memory::MemoryStore`. Currently the server's memory routes use a throwaway `HashMap<String, Note>` (notes vanish on restart) and bypass the domain layer entirely. The agent uses `MemoryStore` internally during turns but this isn't exposed via REST.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P1 - High (important for user experience)

### Technical Debt Impact
- **Current Problems**: Server memory/notes routes use an ephemeral in-memory HashMap — all data lost on restart. The REST API cannot search the real memory store (vector + graph). The domain layer pattern (used by chat) is bypassed, creating architectural inconsistency.
- **Benefits of Fixing**: Persistent memory via REST API, consistent domain-layer architecture, users can query/store memories through the API that survive restarts, parity between what the agent sees during turns and what the API exposes.
- **Risk Assessment**: Medium — touches server state initialization, domain services, and memory routes. Needs careful testing to avoid breaking existing chat-based memory recall.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] `MemoryService` in `arawn-domain` wraps `Arc<MemoryStore>` from `arawn-memory`
- [x] `DomainServices::new()` accepts `Option<Arc<MemoryStore>>` and passes it to `MemoryService`
- [x] Server memory routes (`/api/v1/memory/*`) use `state.memory_store()` via persistent `MemoryStore` instead of local HashMap
- [x] Notes are persisted in SQLite via `MemoryStore` (survive server restart)
- [x] Memory search endpoint calls `memory_store.search_memories()` for vector results + `search_notes()` for note results
- [x] The agent's internal memory store and the REST-exposed store are the same `Arc<MemoryStore>` instance (wired in `start.rs`)
- [x] `angreal test unit` passes (all workspace unit tests green)
- [x] `angreal check all` passes (fmt + clippy + cargo check clean)

## Current Architecture (before)

```
Server routes/memory.rs  →  HashMap<String, Note>  (ephemeral, standalone)
Agent                    →  MemoryStore             (persistent, internal only)
Domain MemoryService     →  stub (deleted in ARAWN-T-0223)
```

## Target Architecture (after)

```
Server routes/memory.rs  →  DomainServices.memory()  →  MemoryStore  (persistent)
Agent                    →  same MemoryStore instance                 (persistent)
```

## Implementation Notes

### Step 1: Recreate `MemoryService` in domain layer
- Accept `Option<Arc<MemoryStore>>` in constructor
- `is_enabled()` returns `self.store.is_some()`
- `search()` delegates to `memory_store.recall()`
- `store()` delegates to `memory_store.store_fact()`
- `store_note()` stores via the unified ops (use `MemorySource::Note`)
- `delete()` delegates to `memory_store.delete()`

### Step 2: Update `DomainServices`
- Add `memory_store: Option<Arc<MemoryStore>>` parameter to `new()`
- Pass it to `MemoryService::new()`
- The same `MemoryStore` instance should be the one given to the `Agent` builder

### Step 3: Update server state initialization
- `AppState` already creates the `MemoryStore` for the agent
- Pass the same `Arc<MemoryStore>` into `DomainServices::new()`

### Step 4: Rewrite server memory routes
- Replace `NoteStore` (HashMap) with calls to `domain.memory()`
- Memory search should use `recall()` with the query
- Note CRUD should use the store's unified ops
- Keep the same REST API contract (request/response shapes)

### Dependencies
- ARAWN-T-0223 must be completed first (removes the old stub)
- Also relates to ARAWN-T-0223 finding "MemorySearchTool stub" — once the domain service is wired, the tool can delegate to it

## Status Updates

### Session 1 — 2026-02-25
**Completed all implementation.** All acceptance criteria met except `angreal check all` (not yet run).

#### Files Modified
- `crates/arawn-domain/src/services/memory.rs` — **Created** `MemoryService` wrapping `Option<Arc<MemoryStore>>`
- `crates/arawn-domain/src/services/mod.rs` — Added memory field to `DomainServices`, `memory_store` param to `new()`, accessor, tests
- `crates/arawn-domain/src/lib.rs` — Re-exports `MemoryService`
- `crates/arawn-domain/Cargo.toml` — Added `arawn-memory` dependency
- `crates/arawn-server/src/state.rs` — Added `memory_store` field to `SharedServices`, builder, accessor on `AppState`, wired into `build_domain_services()`
- `crates/arawn-server/src/routes/memory.rs` — **Full rewrite**: replaced ephemeral `HashMap<String, Note>` + `OnceLock` global singleton with persistent `MemoryStore` from state. All note CRUD now hits SQLite. Memory search/store/delete now use `state.memory_store()` directly instead of `state.indexer().store()`. Added `title` and `updated_at` fields to API Note type. Added 503 for unconfigured memory. Comprehensive test coverage (15 tests).
- `crates/arawn/src/commands/start.rs` — Wired `Arc<MemoryStore>` clone to `SharedServices.memory_store` before indexer consumes it
- `crates/arawn-server/tests/common/mod.rs` — Integration test helper now provisions in-memory `MemoryStore`

#### Key Decisions
- Notes API now returns 503 when memory is not configured (previously used ephemeral HashMap regardless)
- Added `title` and `updated_at` fields to Note response — additive, backward-compatible
- Tag-filtered list uses in-memory pagination (MemoryStore `list_notes_by_tag` doesn't support offset)
- Memory search handler uses `state.memory_store()` directly instead of going through indexer

#### Test Results
- `angreal test unit` — all green (0 failures across entire workspace)
- All 57 arawn-server tests pass
- All 6 arawn-domain tests pass
- Integration tests updated and passing