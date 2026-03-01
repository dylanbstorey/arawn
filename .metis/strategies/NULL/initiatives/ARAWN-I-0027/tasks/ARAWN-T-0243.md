---
id: add-rlm-configuration-section
level: task
title: "Add RLM configuration section"
short_code: "ARAWN-T-0243"
created_at: 2026-03-01T16:27:48.806569+00:00
updated_at: 2026-03-01T19:54:48.244888+00:00
parent: ARAWN-I-0027
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0027
---

# Add RLM configuration section

## Parent Initiative

[[ARAWN-I-0027]] — RLM Exploration Agent

## Objective

Add an `[rlm]` configuration section to `arawn-config` so RLM behavior is configurable via `arawn.toml`. Wire the config through to `RlmSpawner` creation in `start.rs`.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `RlmConfig` struct in `arawn-config/src/types.rs` with fields: model, max_iterations, max_tokens, compaction_threshold, compaction_model
- [ ] Sensible defaults: model="default", max_iterations=25, max_tokens=50000, compaction_threshold=0.7, compaction_model="default"
- [ ] Deserializable from TOML `[rlm]` section
- [ ] `ArawnConfig` has an `rlm` field (with Default)
- [ ] Wired in `start.rs`: config values passed to `RlmSpawner` construction
- [ ] Test: TOML deserialization round-trip with custom values
- [ ] Test: default values are correct
- [ ] `angreal check all` passes

## Implementation Notes

### Files
- `crates/arawn-config/src/types.rs` (add `RlmConfig` struct, add field to `ArawnConfig`)
- `crates/arawn/src/commands/start.rs` (wire config to RlmSpawner)

### Example TOML
```toml
[rlm]
model = "claude-haiku-4-5-20251001"
max_iterations = 25
max_tokens = 50000
compaction_threshold = 0.7
compaction_model = "claude-haiku-4-5-20251001"
```

### Scope
Small — config struct, defaults, deserialization, wiring. Can be done in parallel with T-0241/T-0242.

## Status Updates

### Session 1
- Added `RlmTomlConfig` struct to `arawn-config/src/types.rs` with 7 optional fields: model, max_turns, max_context_tokens, compaction_threshold, max_compactions, max_total_tokens, compaction_model
- All fields are `Option<T>` — absent fields fall through to agent-side `RlmConfig` defaults
- Added `rlm: Option<RlmTomlConfig>` to `ArawnConfig`, `RawConfig`, both `From` impls, and `merge()`
- Added `Clone` derive to `ToolRegistry` (needed for cloning registry to pass to `RlmSpawner`)
- Wired config in `start.rs`: reads `[rlm]` section, applies overrides to `RlmConfig`, creates `RlmSpawner`, registers `ExploreTool`
- 5 tests: deserialization with all fields, defaults (all None), partial config, absent section, merge behavior
- `angreal check all` passes clean
- `angreal test unit` passes: 136 config tests (5 new), 1711 total, 0 failures