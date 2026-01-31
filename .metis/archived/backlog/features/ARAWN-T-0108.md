---
id: make-session-id-a-first-class
level: task
title: "Make session_id a first-class column on memories"
short_code: "ARAWN-T-0108"
created_at: 2026-02-01T03:31:12.500104+00:00
updated_at: 2026-02-01T04:53:26.194203+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#feature"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Make session_id a first-class column on memories

## Objective

Promote `session_id` from a metadata JSON field to a required first-class column on the `memories` table with a foreign key to `sessions`. Memories are always tied to a specific session — a user might prefer Rust for one project, C for another, Go for another, and the session context is what disambiguates.

## Scope

- Add `session_id TEXT NOT NULL` column to `memories` schema (with migration for existing rows)
- Add foreign key constraint to `sessions` table
- Update `MemoryStore` write paths to require `session_id`
- Update recall/search queries to optionally filter by session
- Update indexer to pass session_id through the store API (currently only in metadata JSON)

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `session_id` is a column on `memories`, not just in metadata JSON
- [ ] All indexed memories have a valid session_id
- [ ] Recall queries can scope to a specific session or search across all
- [ ] Existing memories are migrated (backfill from metadata JSON)