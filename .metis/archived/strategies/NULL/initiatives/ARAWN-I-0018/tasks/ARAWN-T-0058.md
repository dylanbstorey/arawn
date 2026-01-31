---
id: crate-scaffold-schema-migrations
level: task
title: "Crate Scaffold, Schema, Migrations, and WorkstreamStore"
short_code: "ARAWN-T-0058"
created_at: 2026-01-29T03:51:27.556144+00:00
updated_at: 2026-01-29T04:02:50.226963+00:00
parent: ARAWN-I-0018
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0018
---

# Crate Scaffold, Schema, Migrations, and WorkstreamStore

## Parent Initiative

[[ARAWN-I-0018]]

## Objective

Create the `arawn-workstream` crate, set up refinery migrations with rusqlite, define the initial schema (workstreams + sessions + tags tables), and implement `WorkstreamStore` with thin repository CRUD methods.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `arawn-workstream` crate created in workspace with dependencies: `rusqlite`, `refinery`, `serde`, `serde_json`, `chrono`, `uuid`, `thiserror`
- [ ] Refinery migration runner configured with rusqlite driver
- [ ] V001 migration: `workstreams` table (id, title, summary, is_scratch, state, default_model, created_at, updated_at)
- [ ] V001 migration: `sessions` table (id, workstream_id FK, started_at, ended_at, turn_count, summary, compressed)
- [ ] V001 migration: `workstream_tags` junction table (workstream_id, tag, PRIMARY KEY (workstream_id, tag))
- [ ] `WorkstreamStore` struct wrapping a rusqlite `Connection`
- [ ] `WorkstreamStore::open(path)` — opens/creates DB, runs pending migrations
- [ ] `WorkstreamStore::create_workstream()` — insert workstream row
- [ ] `WorkstreamStore::get_workstream(id)` — fetch by ID
- [ ] `WorkstreamStore::list_workstreams(state_filter)` — list with optional state filter
- [ ] `WorkstreamStore::update_workstream()` — update title, summary, state, default_model, updated_at
- [ ] `WorkstreamStore::set_tags(workstream_id, tags)` — replace tags for a workstream
- [ ] `WorkstreamStore::get_tags(workstream_id)` — fetch tags
- [ ] Session CRUD: `create_session()`, `get_session()`, `end_session()`, `list_sessions(workstream_id)`
- [ ] Well-known scratch workstream: `WorkstreamStore::ensure_scratch()` creates scratch row if missing (id = "scratch")
- [ ] Unit tests: CRUD roundtrips, migration runs cleanly, scratch auto-creation
- [ ] `cargo test` passes

## Implementation Notes

### Technical Approach
- Refinery migrations stored as SQL files in `crates/arawn-workstream/migrations/`
- `WorkstreamStore` methods take/return typed Rust structs (defined in sibling task T-0059), not raw rows
- rusqlite `params!` macro for parameterized queries
- All timestamps stored as RFC 3339 strings in SQLite TEXT columns
- State stored as TEXT enum ("active", "paused", "archived")

### Dependencies
- None (foundational task)

## Status Updates

*To be added during implementation*