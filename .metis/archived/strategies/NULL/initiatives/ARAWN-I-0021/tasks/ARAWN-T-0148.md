---
id: llm-response-validation
level: task
title: "LLM Response Validation"
short_code: "ARAWN-T-0148"
created_at: 2026-02-07T16:46:14.038809+00:00
updated_at: 2026-02-07T18:43:57.832127+00:00
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

# LLM Response Validation

## Parent Initiative

[[ARAWN-I-0021]] - Interface Enforcement and Defensive Validation

## Objective

Add structural validation for LLM backend responses in `arawn-llm` crate. Validate response fields, tool call formats, and streaming chunks before they're processed by the agent loop.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Define `ResponseValidationError` with specific failure modes
- [x] Validate `CompletionResponse` structure (required fields present)
- [x] Validate tool_use blocks have valid id, name, and input
- [x] Validate streaming chunks have expected structure
- [x] Token count sanity checks (not negative, within limits)
- [x] Graceful handling of unexpected response formats
- [ ] Tracing spans for validation failures (deferred - not critical for validation layer)
- [x] Unit tests with malformed response fixtures

## Implementation Notes

### Technical Approach

1. Add validation layer in `arawn-llm/src/response.rs`
2. Define error types:
   ```rust
   pub enum ResponseValidationError {
       MissingField { field: &'static str },
       InvalidToolUse { id: String, reason: String },
       InvalidTokenCount { value: i64 },
       MalformedContent { reason: String },
   }
   ```
3. Add `CompletionResponse::validate(&self) -> Result<(), ResponseValidationError>`
4. Validate tool_use blocks:
   - `id` is non-empty
   - `name` matches registered tool
   - `input` is valid JSON object
5. Add validation to streaming chunk parsing

### What to Validate

| Field | Validation |
|-------|------------|
| `id` | Non-empty string |
| `content` | Array of valid content blocks |
| `stop_reason` | One of expected enum values |
| `usage.input_tokens` | >= 0 |
| `usage.output_tokens` | >= 0 |
| Tool `id` | Non-empty, unique within response |
| Tool `name` | Non-empty, valid identifier format |
| Tool `input` | Valid JSON object |

### Files to Modify

- `crates/arawn-llm/src/response.rs` - Add validation
- `crates/arawn-llm/src/stream.rs` - Validate stream chunks
- `crates/arawn-llm/src/error.rs` - Add validation errors

### Dependencies

None - can run in parallel with other validation tasks

## Status Updates

### Session 2026-02-07 - Implementation Complete

**Implemented `ResponseValidationError` in `crates/arawn-llm/src/error.rs`:**
- `MissingField { field: &'static str }` - Required field missing
- `InvalidToolUse { id: String, reason: String }` - Tool use block malformed
- `InvalidTokenCount { field, value, constraint }` - Token count sanity check failed
- `MalformedContent { index, reason }` - Content block malformed
- `InvalidStopReason { reason }` - Unexpected stop reason
- `InvalidStreamEvent { reason }` - Stream chunk validation failed
- `Multiple(Vec<ResponseValidationError>)` - Aggregated errors
- Added `is_critical()` method to distinguish fatal vs warning errors
- Implemented `From<ResponseValidationError> for LlmError`

**Added validation to `CompletionResponse` in `crates/arawn-llm/src/types.rs`:**
- `validate(&self) -> Result<(), ResponseValidationError>` - Full response validation
- `validate_content_block()` - Per-block validation with duplicate ID detection
- `validated(self) -> Result<Self, ResponseValidationError>` - Convenience wrapper
- Validates: id/model non-empty, tool_use blocks have valid id/name/input (object), no duplicate tool IDs, stop_reason consistency

**Added validation to `StreamEvent` in `crates/arawn-llm/src/backend.rs`:**
- `validate(&self) -> Result<(), ResponseValidationError>` - Event validation
- `is_error(&self) -> bool` - Check if error event
- `is_terminal(&self) -> bool` - Check if terminal event (MessageStop/Error)
- Validates MessageStart (id/model), ContentBlockStart (known type), Error (message present)

**Exported in `crates/arawn-llm/src/lib.rs`:**
- `pub use error::{LlmError, ResponseValidationError, Result};`

**Tests added:**
- 15+ unit tests for response validation scenarios
- Tests for stream event validation
- Tests for error type behavior and criticality

**Verification:**
- All 89 arawn-llm tests pass
- All 324 workspace unit tests pass
- Workspace compiles cleanly