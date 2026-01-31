---
id: add-recall-config-to-arawn-config
level: task
title: "Add recall config to arawn-config"
short_code: "ARAWN-T-0092"
created_at: 2026-01-31T02:41:43.345581+00:00
updated_at: 2026-01-31T03:55:52.298884+00:00
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

# Add recall config to arawn-config

## Objective

Add a `[memory.recall]` config section to `arawn-config` with `enabled`, `threshold`, and `limit` fields to control active recall behavior.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `RecallConfig` struct with `enabled: bool`, `threshold: f32`, `limit: usize`
- [ ] Sensible defaults: `enabled: true`, `threshold: 0.6`, `limit: 5`
- [ ] Nested under existing `MemoryConfig` as `recall: RecallConfig`
- [ ] Parseable from TOML `[memory.recall]` section
- [ ] Merges correctly with layered config (file + env + CLI)
- [ ] Tests: default values, TOML parsing, override behavior

## Implementation Notes

### Files
- `crates/arawn-config/src/types.rs` — add `RecallConfig` struct, nest in `MemoryConfig`

### Technical Approach
- Add `RecallConfig` with `#[derive(Debug, Clone, Serialize, Deserialize)]` and manual `Default`
- Add `pub recall: RecallConfig` field to the existing `MemoryConfig` struct
- Use `#[serde(default)]` so missing TOML sections get defaults
- No dependencies on other tasks — config is independent

## Status Updates

### Session 1
- Added `RecallConfig` struct: `enabled: bool`, `threshold: f32`, `limit: usize` with defaults (true, 0.6, 5)
- Added `MemoryConfig` struct containing `recall: RecallConfig`
- Added `memory: Option<MemoryConfig>` to `ArawnConfig`, `RawConfig`, `From` conversions, and `merge()`
- Parseable from `[memory.recall]` TOML section, `#[serde(default)]` for missing sections
- Tests: default values, TOML parsing, missing section defaults, merge override behavior
- `angreal check all` + `angreal test unit` pass (661 tests, 0 failures)