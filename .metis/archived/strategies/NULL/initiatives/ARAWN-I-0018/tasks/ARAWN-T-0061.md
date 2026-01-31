---
id: scratch-workstream-and-promotion
level: task
title: "Scratch Workstream and Promotion Flow"
short_code: "ARAWN-T-0061"
created_at: 2026-01-29T03:51:29.468105+00:00
updated_at: 2026-01-29T04:07:02.954385+00:00
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

# Scratch Workstream and Promotion Flow

## Parent Initiative

[[ARAWN-I-0018]]

## Objective

Implement the "scratch" workstream — a well-known default workstream that receives messages when no explicit workstream_id is provided. Implement promotion flow so the agent can suggest converting a maturing scratch conversation into a named workstream.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Well-known scratch workstream with constant ID (e.g. `"scratch"`) auto-created on first use
- [ ] `WorkstreamStore::get_or_create_scratch()` ensures scratch exists, returns it
- [ ] `promote_scratch(new_title, new_tags)` creates a new named workstream, migrates JSONL history and SQLite records from scratch, resets scratch to empty
- [ ] After promotion, old scratch messages live under the new workstream_id (JSONL file renamed/moved)
- [ ] SQLite session and tag records updated to point to new workstream_id
- [ ] Scratch workstream cannot be deleted or archived
- [ ] Unit tests: auto-creation, promotion moves messages, scratch resets after promotion, double-promote safety

## Implementation Notes

### Technical Approach

Scratch logic in `crates/arawn-workstream/src/scratch.rs`. The scratch workstream is a normal `Workstream` row with `id = "scratch"` and `status = "active"`. Promotion: create new workstream row → rename `{data_dir}/workstreams/scratch/messages.jsonl` to `{data_dir}/workstreams/{new_id}/messages.jsonl` → update session rows `SET workstream_id = new_id WHERE workstream_id = 'scratch'` → update tag rows similarly. The agent integration (prompting user "want to name this?") will be handled in T-0062 or T-0063.

### Dependencies

- ARAWN-T-0058 (store and schema)
- ARAWN-T-0059 (message store for JSONL operations)

## Status Updates

*To be added during implementation*