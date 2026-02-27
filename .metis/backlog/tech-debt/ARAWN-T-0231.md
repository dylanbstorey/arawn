---
id: finish-server-session-cache
level: task
title: "Finish server session cache migration to arawn-session crate"
short_code: "ARAWN-T-0231"
created_at: 2026-02-27T00:10:52.286989+00:00
updated_at: 2026-02-27T18:20:14.070168+00:00
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

# Finish server session cache migration to arawn-session crate

## Objective

The `arawn-session` crate was built as a generic, decoupled session cache with LRU eviction, TTL support, and persistence hooks. The server was supposed to migrate to it but only adopted `TtlTracker` — the rest of `arawn-server/src/session_cache.rs` is a hand-rolled cache with workstream-specific logic baked in.

Finish the migration: refactor the server's `SessionCache` to use `arawn_session::SessionCache` with a custom `PersistenceHook` implementation for workstream storage.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P2 - Medium (nice to have)

### Technical Debt Impact
- **Current Problems**: Two session cache implementations — `arawn_session::SessionCache` (unused, generic) and `arawn_server::session_cache::SessionCache` (in use, server-specific). Only `TtlTracker` is consumed from the crate. The crate's `CacheConfig`, `PersistenceHook`, `SessionData`, `CacheEntry` exports are all dead code externally.
- **Benefits of Fixing**: Single session cache implementation. Server gets a cleaner separation — workstream persistence becomes a `PersistenceHook` impl rather than inline logic. The `arawn-session` crate justifies its existence.
- **Risk Assessment**: Medium — session cache is on the hot path. Needs careful testing to ensure no behavioral regressions (LRU eviction order, TTL expiry, workstream load-on-miss).

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Server's `session_cache.rs` uses `arawn_session::SessionCache` instead of hand-rolled LRU + TtlTracker combo
- [x] Workstream persistence implemented as a `PersistenceHook` (load session from JSONL on cache miss, save turns on write)
- [x] All existing session-related tests pass
- [x] `arawn_session` crate exports are all used (no dead public API)
- [x] No behavioral changes from user perspective (sessions load, persist, expire, evict as before)

## Implementation Notes

### Current State
- **`arawn-session/src/cache.rs`**: Generic `SessionCache<T>` with LRU + TTL + persistence hooks
- **`arawn-session/src/persistence.rs`**: `PersistenceHook` trait with `load`/`save`/`delete` async methods
- **`arawn-server/src/session_cache.rs`**: Custom `SessionCache` wrapping `lru::LruCache` + `arawn_session::TtlTracker`, with inline workstream load/save logic

### Technical Approach
1. Implement `PersistenceHook` for workstream storage (loads session from JSONL, saves turn data)
2. Replace server's `SessionCache` with `arawn_session::SessionCache` parameterized with the workstream hook
3. May need to extend `arawn_session::SessionCache` if the server's version has capabilities the generic one lacks
4. Verify the generic cache's LRU eviction and TTL behavior matches current server behavior

## Status Updates

### Session 1 — Complete

**Core problem:** `arawn_session::SessionCache` stored `SessionData` (generic bytes), but the server stores `Session` objects (rich domain type). Serializing `Session` to bytes for every cache access would be wasteful.

**Solution: Associated value type on PersistenceHook**

Made `PersistenceHook` generic via associated type `Value`. Cache stores `V` directly — no serialization overhead. `NoPersistence` defaults `Value = SessionData` for backwards compat. Server defines `WorkstreamPersistence` with `Value = Session`.

**Phase 1 — Extended `arawn-session` crate:** DONE
- `persistence.rs`: Added `type Value: Clone + Send + Sync + 'static` to `PersistenceHook`; updated `load`/`save`/`delete` signatures to take `session_id, context_id` separately
- `cache.rs`: Made `CacheEntry<V>` generic with `context_id` field; `CacheInner<P>` uses `P::Value`; added `peek_context_id()`, `peek_entry()`, `with_mut()`, `with_ref()`, `for_each()` helper methods
- `lib.rs`: Updated exports to include `CacheStats`, `NoPersistence`
- All 18 arawn-session tests pass

**Phase 2 — Server migration:** DONE
- Created `WorkstreamPersistence` implementing `PersistenceHook` with `Value = Session`
- Server `SessionCache` now wraps `arawn_session::SessionCache<WorkstreamPersistence>`
- Same public API preserved — no changes to routes/state code
- Removed direct `lru = "0.12"` dependency from server Cargo.toml

**Verification:**
- `angreal check all` — clean (fmt, clippy, cargo check)
- `angreal test unit` — all tests pass, 0 failures across all crates
- No behavioral changes — same session lifecycle (load, persist, expire, evict)