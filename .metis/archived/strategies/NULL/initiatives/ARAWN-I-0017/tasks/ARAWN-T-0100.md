---
id: add-indexing-and-confidence-config
level: task
title: "Add indexing and confidence config sections to arawn-config"
short_code: "ARAWN-T-0100"
created_at: 2026-01-31T04:09:06.924706+00:00
updated_at: 2026-02-01T03:51:14.370110+00:00
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

# Add indexing and confidence config sections to arawn-config

## Objective

Add `[memory.indexing]` and `[memory.confidence]` config sections to `arawn-config`. These control the LLM backend for extraction/summarization, and the confidence scoring parameters.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `IndexingConfig` struct: `enabled: bool`, `backend: String`, `model: String` with sane defaults
- [ ] `ConfidenceConfig` struct: `staleness_days: u32`, `staleness_floor: f32`, `reinforcement_cap: f32` with defaults from initiative spec
- [ ] `MemoryConfig` extended with `indexing: IndexingConfig` and `confidence: ConfidenceConfig` fields
- [ ] TOML parsing: `[memory.indexing]` and `[memory.confidence]` sections parsed correctly
- [ ] Missing sections default gracefully
- [ ] Merge logic handles overrides correctly
- [ ] Tests: parse full config, defaults, merge overrides

## Implementation Notes

### Files
- `crates/arawn-config/src/types.rs` — new structs, extend `MemoryConfig`

### Dependencies
- None (extends existing MemoryConfig from T-0092, already merged)

## Status Updates

### Session — 2026-01-31
- Added `IndexingConfig` struct: enabled, backend, model (defaults: true, "openai", "gpt-4o-mini")
- Added `ConfidenceConfig` struct: fresh_days, staleness_days, staleness_floor, reinforcement_cap (defaults match ConfidenceParams)
- Extended `MemoryConfig` with `indexing: IndexingConfig` and `confidence: ConfidenceConfig`
- All structs use `#[serde(default)]` for graceful missing-section handling
- Merge logic works via existing Option-level replacement on MemoryConfig
- 6 new tests: defaults for indexing/confidence, overrides, partial sections, merge with indexing
- All checks + tests pass (716+ tests, 0 failures)