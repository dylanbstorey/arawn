---
id: add-memoryconfidence-types-and
level: task
title: "Add MemoryConfidence types and schema migration"
short_code: "ARAWN-T-0095"
created_at: 2026-01-31T04:09:05.839755+00:00
updated_at: 2026-02-01T03:51:10.847755+00:00
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

# Add MemoryConfidence types and schema migration

## Objective

Add `MemoryConfidence` struct, `ConfidenceSource` enum, and `ContentType::Summary` to `arawn-memory/src/types.rs`. Add confidence columns to the memories SQLite schema (`confidence_source`, `reinforcement_count`, `superseded`, `superseded_by`, `last_accessed`, `confidence_score`). Update `row_to_memory` to read confidence fields. Add migration logic in `store/mod.rs` schema initialization.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `ConfidenceSource` enum: `Stated`, `Observed`, `Inferred`, `System` with serde + SQLite string mapping
- [ ] `MemoryConfidence` struct with all fields from the initiative design doc
- [ ] `Memory` struct gains `confidence: MemoryConfidence` field (defaults to `Inferred` source, score 1.0)
- [ ] `ContentType::Summary` variant added
- [ ] SQLite schema adds confidence columns to `memories` table (ALTER TABLE migration for existing DBs)
- [ ] `row_to_memory` reads confidence fields from DB rows
- [ ] `insert_memory` writes confidence fields
- [ ] All existing tests pass with the new fields (backward compatible defaults)
- [ ] New tests: round-trip confidence through insert/read, default confidence values

## Implementation Notes

### Files
- `crates/arawn-memory/src/types.rs` — new types + update Memory struct
- `crates/arawn-memory/src/store/mod.rs` — schema migration
- `crates/arawn-memory/src/store/memory_ops.rs` — update insert/read queries

### Dependencies
None — this is the foundation task.

## Status Updates

### Session — 2026-01-31
- Added `ConfidenceSource` enum (Stated/Observed/Inferred/System) with `as_str()`/`from_db_str()` + serde
- Added `MemoryConfidence` struct with Default impl (Inferred, score=1.0)
- Added `ContentType::Summary` variant
- Added `confidence: MemoryConfidence` field to `Memory` struct + `with_confidence()` builder
- Bumped SCHEMA_VERSION to 2, added `migrate_v2()` with 6 ALTER TABLE columns + backfill
- Updated `insert_memory`, `update_memory`, `row_to_memory` for all 13 columns
- Updated SELECT queries in session_ops.rs (get_session_history) and recall.rs (2 queries)
- Fixed Memory struct initializers in session_ops.rs to include confidence field
- Updated lib.rs re-exports with ConfidenceSource and MemoryConfidence
- All 669+ tests pass, `angreal check all` clean