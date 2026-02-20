---
id: model-context-limits-in-llm-config
level: task
title: "Model context limits in LLM config"
short_code: "ARAWN-T-0184"
created_at: 2026-02-16T18:54:48.111860+00:00
updated_at: 2026-02-16T19:33:20.484269+00:00
parent: ARAWN-I-0026
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0026
---

# Model context limits in LLM config

## Parent Initiative

[[ARAWN-I-0026]] - Context Management and Auto-Compaction

## Objective

Add `max_context_tokens` configuration to the LLM provider/model config schema, enabling ContextTracker to know the token budget for each model.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] `max_context_tokens` field added to model config schema
- [x] Config parsing handles the new field
- [x] `require_max_context_tokens()` method to get limit or error
- [x] `MissingContextLimit` error variant for validation
- [x] Unit tests for context limit functionality

**Note**: Default values for known models are configured in TOML, not hardcoded in Rust.

## Implementation Notes

### Files to Modify
- `crates/arawn-config/src/types.rs` - Add field to model config
- `crates/arawn-llm/src/backend.rs` - Expose context limit lookup

### Config Schema

```toml
[llm.providers.anthropic.models.claude-sonnet]
max_context_tokens = 200000

[llm.providers.groq.models.llama-70b]
max_context_tokens = 32000
```

### Dependencies
- None (foundational task)

## Tests

### Unit Tests
- `test_model_config_parses_max_context_tokens` - parse config with field present
- `test_model_config_default_context_limits` - known models get defaults
- `test_model_config_validation_error_missing_limit` - error when model used without context limit
- `test_model_context_lookup` - given model name, returns correct limit

### Test File
- `crates/arawn-config/src/types.rs` (inline `#[cfg(test)]` module)
- `crates/arawn-llm/src/backend.rs` (inline tests for lookup)

## Status Updates

### Session 1 (2026-02-16)
- Added `max_context_tokens` field to `LlmConfig` struct
- Added field to `RawLlmSection` for TOML parsing
- Updated From impls for RawConfig <-> ArawnConfig conversion
- Added `require_max_context_tokens()` method to get limit or error
- Added `MissingContextLimit` error variant to ConfigError
- Added 5 unit tests for context limit functionality
- All 109 arawn-config tests pass

**Simplified approach per user feedback**: No hardcoded defaults in code - all model context limits configured purely in TOML.