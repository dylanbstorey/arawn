---
id: implement-confidence-scoring-logic
level: task
title: "Implement confidence scoring logic"
short_code: "ARAWN-T-0096"
created_at: 2026-01-31T04:09:06.057275+00:00
updated_at: 2026-02-01T03:51:11.588330+00:00
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

# Implement confidence scoring logic

## Objective

Implement the `MemoryConfidence::compute_score()` method that calculates a composite confidence score from source type, reinforcement count, staleness, and supersession status. Add configurable parameters for staleness decay and reinforcement cap.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `MemoryConfidence::compute_score()` method returns f32 in range [0.0, 1.0]
- [ ] Base scores: Stated=1.0, System=0.9, Observed=0.7, Inferred=0.5
- [ ] Reinforcement boost: `min(1.0 + 0.1 * count, cap)` where cap defaults to 1.5
- [ ] Staleness penalty: 1.0 for <30 days, linear decay to floor at staleness_days
- [ ] Superseded memories always return 0.0
- [ ] `ConfidenceParams` struct for configurable staleness_days, staleness_floor, reinforcement_cap
- [ ] Tests: each scoring component in isolation, combined scoring, edge cases (zero reinforcement, very old memories, superseded)

## Implementation Notes

### Files
- `crates/arawn-memory/src/types.rs` — add `compute_score()` and `ConfidenceParams`

### Dependencies
- ARAWN-T-0095 (confidence types must exist)

## Status Updates

### Session — 2026-01-31
- Added `ConfidenceSource::base_score()` method: Stated=1.0, System=0.9, Observed=0.7, Inferred=0.5
- Added `MemoryConfidence::compute_score(&self, params: &ConfidenceParams) -> f32`
  - Superseded → 0.0
  - base * reinforcement_boost * staleness, clamped to [0.0, 1.0]
  - Reinforcement: `min(1.0 + 0.1*count, cap)`
  - Staleness: 1.0 within fresh_days, linear decay to floor at staleness_days
- Added `ConfidenceParams` struct (fresh_days=30, staleness_days=365, floor=0.3, cap=1.5)
- Added re-export of `ConfidenceParams` in lib.rs
- 10 new tests covering: base scores, fresh/stale/half-stale, reinforcement boost/cap, superseded, clamp, params default
- All checks + tests pass