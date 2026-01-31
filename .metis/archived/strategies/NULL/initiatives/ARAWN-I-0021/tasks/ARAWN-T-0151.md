---
id: interface-validation-integration
level: task
title: "Interface Validation Integration Tests"
short_code: "ARAWN-T-0151"
created_at: 2026-02-07T16:46:16.969611+00:00
updated_at: 2026-02-07T19:40:20.674103+00:00
parent: ARAWN-I-0021
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0021
---

# Interface Validation Integration Tests

## Parent Initiative

[[ARAWN-I-0021]] - Interface Enforcement and Defensive Validation

## Objective

Create integration tests that verify validation works correctly at interface boundaries. Test malformed inputs across plugin, tool, LLM, and memory interfaces to ensure graceful error handling.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Integration test suite for interface validation
- [x] Tests for malformed plugin manifests (rejected at load time)
- [x] Tests for invalid tool parameters (rejected before execution)
- [x] Tests for malformed LLM responses (handled gracefully)
- [x] Tests for invalid memory operations (proper errors returned)
- [x] Tests verify error messages are actionable
- [x] Tests run in CI pipeline (part of `angreal test unit`)
- [x] Document test patterns for future validation additions

## Test Cases

### Test Case 1: Malformed Plugin Manifest
- **Test ID**: TC-001
- **Preconditions**: Plugin loader available
- **Steps**: 
  1. Create plugin with missing required field
  2. Attempt to load plugin
  3. Verify specific error returned
- **Expected Results**: `ManifestValidationError::MissingField` with field name

### Test Case 2: Invalid Tool Parameters
- **Test ID**: TC-002
- **Preconditions**: Tool registry with shell tool
- **Steps**: 
  1. Call shell tool with empty command
  2. Call shell tool with negative timeout
- **Expected Results**: `ParameterValidationError` before execution

### Test Case 3: Malformed LLM Response
- **Test ID**: TC-003
- **Preconditions**: Mock LLM backend
- **Steps**: 
  1. Configure mock to return invalid tool_use (missing id)
  2. Execute agent turn
- **Expected Results**: `ResponseValidationError`, agent continues gracefully

### Test Case 4: Invalid Memory Content
- **Test ID**: TC-004
- **Preconditions**: Memory store available
- **Steps**: 
  1. Attempt to insert memory with empty content
  2. Attempt to insert memory with invalid confidence
- **Expected Results**: `ValidationError` with specific reason

## Implementation Notes

### Technical Approach

1. Create `tests/integration/validation.rs`
2. Use test fixtures for malformed inputs:
   ```rust
   mod fixtures {
       pub fn invalid_plugin_manifest() -> serde_json::Value { ... }
       pub fn malformed_llm_response() -> CompletionResponse { ... }
   }
   ```
3. Test error propagation through layers
4. Verify error messages contain actionable information

### Test Organization

```
tests/
  integration/
    validation/
      mod.rs
      plugin_tests.rs
      tool_tests.rs
      llm_tests.rs
      memory_tests.rs
```

### Dependencies

- Depends on: T-0146, T-0147, T-0148, T-0149, T-0150 (validation implementations)
- Should be done after other tasks complete

## Status Updates

### Session 2026-02-07 - Implementation Complete

#### Created Integration Test Suite

Created `crates/arawn-server/tests/validation_integration.rs` with 57 tests across 6 test modules:

**Test Modules:**

| Module | Tests | Coverage |
|--------|-------|----------|
| `plugin_tests` | 7 | Malformed manifests, name/version validation, path/capability checks |
| `tool_tests` | 17 | Missing params, empty values, out-of-range, type errors |
| `llm_tests` | 6 | Critical vs non-critical errors, error aggregation, message quality |
| `memory_tests` | 11 | Content, confidence, embedding, session ID validation |
| `output_tests` | 10 | Truncation, binary detection, control chars, JSON depth |
| `integration_tests` | 3 | Error chain propagation, server startup |

**Test Fixtures (`fixtures` module):**
- `plugin_manifest_missing_name()` - Missing required field
- `plugin_manifest_invalid_name()` - Non-kebab-case name
- `plugin_manifest_invalid_version()` - Bad semver format
- `shell_params_*` - Various invalid shell parameters
- `memory_store_*` - Invalid memory operations
- `web_search_*` / `file_read_*` - Edge cases

**Key Test Patterns Documented:**
- Each test verifies specific error type returned
- Tests check error messages are actionable (contain relevant info)
- Tests verify error chain propagation through layers
- Tests cover boundary conditions and edge cases

#### Files Modified

- `crates/arawn-server/tests/validation_integration.rs` (new - 57 tests)
- `crates/arawn-server/Cargo.toml` (added arawn-agent, arawn-plugin dev deps)

#### Verification

All 57 integration tests pass. Full unit test suite passes.