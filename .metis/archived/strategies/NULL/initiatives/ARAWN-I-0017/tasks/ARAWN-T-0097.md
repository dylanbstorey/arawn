---
id: implement-contradiction-detection
level: task
title: "Implement contradiction detection and memory supersession"
short_code: "ARAWN-T-0097"
created_at: 2026-01-31T04:09:06.275847+00:00
updated_at: 2026-02-01T03:51:12.270330+00:00
parent: ARAWN-I-0017
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0017
---

# Implement contradiction detection and memory supersession

## Objective

When storing a new fact/memory, detect existing memories with the same subject+predicate but different object values. Supersede the old memory (set `superseded=true`, `superseded_by=new_id`, `confidence_score=0.0`) and log the contradiction. This is the "cache invalidation" mechanism for memory.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `MemoryStore::find_contradictions(subject, predicate)` query returns existing memories matching subject+predicate
- [ ] `MemoryStore::supersede(old_id, new_id)` marks old memory as superseded and updates confidence to 0.0
- [ ] `store_fact()` method that wraps insert + contradiction check in a transaction
- [ ] Superseded memories excluded from `recall()` results (score 0.0 filtered by min_score)
- [ ] Tests: storing contradictory fact supersedes old, storing matching fact does NOT supersede, superseded memory has score 0.0

## Implementation Notes

### Files
- `crates/arawn-memory/src/store/memory_ops.rs` — `find_contradictions()`, `supersede()`
- `crates/arawn-memory/src/store/unified_ops.rs` — `store_fact()` with contradiction check

### Technical Approach
Contradiction detection uses metadata fields. Memories need `subject` and `predicate` metadata keys (set by the indexing pipeline). The query matches on `metadata->>'subject'` and `metadata->>'predicate'` using SQLite JSON functions.

### Dependencies
- ARAWN-T-0095 (confidence schema)

## Status Updates

### Session — 2026-01-31
- Added `subject` and `predicate` optional fields to `Metadata` struct in types.rs
- Added `find_contradictions(subject, predicate)` to memory_ops.rs — queries via `json_extract()`, excludes superseded
- Added `supersede(old_id, new_id)` to memory_ops.rs — sets superseded=true, superseded_by, score=0.0
- Added `store_fact(memory, options)` to unified_ops.rs — contradiction check + store in one call, returns superseded IDs
- Tests in memory_ops.rs: find_contradictions (match/no-match/different-predicate), supersede (fields updated, excluded from future queries), supersede not-found error
- Tests in unified_ops.rs: store_fact supersedes contradiction, no contradiction on different predicate, skips check without subject/predicate, chain supersession (m1→m2→m3)
- All checks + tests pass (689+ tests, 0 failures)