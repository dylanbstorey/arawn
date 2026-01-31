---
id: integrate-confidence-scores-into
level: task
title: "Integrate confidence scores into recall ranking"
short_code: "ARAWN-T-0099"
created_at: 2026-01-31T04:09:06.705246+00:00
updated_at: 2026-02-01T03:51:13.665155+00:00
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

# Integrate confidence scores into recall ranking

## Objective

Update the `recall()` function to incorporate confidence scores into the final ranking. The formula: `final_score = vector_similarity * 0.4 + graph_relevance * 0.3 + confidence_score * 0.3`. Superseded memories (score 0.0) should be excluded automatically.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `recall()` reads `confidence_score` for each candidate memory
- [ ] Final score formula applied: `similarity * 0.4 + graph * 0.3 + confidence * 0.3`
- [ ] `RecallMatch` includes `confidence_score` field alongside `similarity_score`
- [ ] Superseded memories (confidence 0.0) excluded from results when min_score > 0
- [ ] When no graph is configured, formula degrades gracefully: `similarity * 0.6 + confidence * 0.4`
- [ ] Tests: high-confidence memory ranks above low-confidence at same similarity, superseded memory excluded

## Implementation Notes

### Files
- `crates/arawn-memory/src/store/recall.rs` — update scoring in `recall()`
- `crates/arawn-memory/src/store/query.rs` — add `confidence_score` to `RecallMatch`

### Dependencies
- ARAWN-T-0095 (confidence in schema)
- ARAWN-T-0096 (scoring logic)

## Status Updates

### Session — 2026-01-31
- Added `similarity_score` and `confidence_score` fields to `RecallMatch` struct
- Updated `recall()` scoring formula:
  - With graph context: `similarity * 0.4 + graph * 0.3 + confidence * 0.3`
  - Without graph: `similarity * 0.6 + confidence * 0.4`
  - `compute_score()` called with default `ConfidenceParams`
- Superseded memories explicitly filtered out early in recall loop (`memory.confidence.superseded`)
- Updated existing `test_recall_vector_weight` threshold to reflect new blended formula
- New tests: high-confidence ranks above low at same similarity, superseded excluded by min_score, RecallMatch includes confidence_score field
- All checks + tests pass (710+ tests, 0 failures)