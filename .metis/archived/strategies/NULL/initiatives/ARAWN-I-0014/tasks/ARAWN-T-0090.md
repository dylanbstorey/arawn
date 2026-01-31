---
id: add-contenttype-thought-and
level: task
title: "Add ContentType::Thought and RecallQuery::with_min_score()"
short_code: "ARAWN-T-0090"
created_at: 2026-01-31T02:41:42.232288+00:00
updated_at: 2026-01-31T03:55:51.915859+00:00
parent: ARAWN-I-0014
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0014
---

# Add ContentType::Thought and RecallQuery::with_min_score()

## Objective

Extend `arawn-memory` with a new `ContentType::Thought` variant for storing agent reasoning, and add `with_min_score(f32)` to the `RecallQuery` builder to enable threshold-based filtering of recall results.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `ContentType::Thought` variant added with `as_str()` returning `"thought"` and `from_str("thought")` returning `Some(Self::Thought)`
- [ ] `RecallQuery` has `min_score: Option<f32>` field and `with_min_score(f32)` builder method
- [ ] `recall()` filters out matches with `score < min_score` before returning results
- [ ] Default `min_score` is `None` (no filtering, preserves existing behavior)
- [ ] All existing tests still pass
- [ ] New tests: content type roundtrip, score threshold filtering, default no-filter behavior

## Implementation Notes

### Files
- `crates/arawn-memory/src/types.rs` — add `Thought` to `ContentType` enum, update `as_str()` and `from_str()`
- `crates/arawn-memory/src/store.rs` — add `min_score` field to `RecallQuery`, add `with_min_score()` builder, apply filter in `recall()` before returning `RecallResult`

### Technical Approach
- `ContentType::Thought` is a new variant alongside the existing 7 (UserMessage, AssistantMessage, ToolUse, FileContent, Note, Fact, WebContent)
- `min_score` filtering happens post-query: after vector search and score calculation, filter `matches.retain(|m| m.score >= min_score)` before building `RecallResult`
- No schema migration needed — `ContentType` is stored as a string column

## Status Updates

### Session 2
- Added `ContentType::Thought` variant to `types.rs` (enum, as_str, from_str, test)
- Added `min_score: Option<f32>` field to `RecallQuery` in `store.rs`
- Added `with_min_score()` builder method with `clamp(0.0, 1.0)`
- Added `matches.retain()` filter in `recall()` before sort step
- Fixed angreal `task_check.py` argument naming (`name="check_only"` not `name="--check-only"`)
- `angreal check all` passes, `angreal test unit` passes (125+ tests)

*To be added during implementation*