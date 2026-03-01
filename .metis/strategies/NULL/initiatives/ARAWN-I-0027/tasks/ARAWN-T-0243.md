---
id: add-rlm-configuration-section
level: task
title: "Add RLM configuration section"
short_code: "ARAWN-T-0243"
created_at: 2026-03-01T16:27:48.806569+00:00
updated_at: 2026-03-01T16:27:48.806569+00:00
parent: ARAWN-I-0027
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


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

*To be added during implementation*