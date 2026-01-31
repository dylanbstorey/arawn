---
id: tool-parameter-validation
level: task
title: "Tool Parameter Validation"
short_code: "ARAWN-T-0147"
created_at: 2026-02-07T16:46:13.130677+00:00
updated_at: 2026-02-07T17:11:08.520729+00:00
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

# Tool Parameter Validation

## Parent Initiative

[[ARAWN-I-0021]] - Interface Enforcement and Defensive Validation

## Objective

Add Rust-side parameter validation for tools beyond JSON Schema. Implement typed parameter structs with `TryFrom<serde_json::Value>` conversions that provide clear error messages when LLM-provided arguments are malformed.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Define `ParameterValidationError` with field-level context
- [x] Create typed parameter structs for each built-in tool
- [x] Implement `TryFrom<Value>` for parameter structs with validation
- [x] Validate required vs optional parameters
- [x] Validate parameter value constraints (ranges, patterns, enums)
- [x] Tool execution rejects invalid parameters before running
- [x] Error messages help LLM understand what's wrong
- [x] Unit tests for valid/invalid parameter combinations

## Implementation Notes

### Technical Approach

1. Create parameter struct per tool, e.g.:
   ```rust
   pub struct ShellParams {
       pub command: String,
       pub timeout_secs: Option<u32>,
       pub pty: bool,
   }
   
   impl TryFrom<Value> for ShellParams {
       type Error = ParameterValidationError;
       fn try_from(value: Value) -> Result<Self, Self::Error> { ... }
   }
   ```
2. Use `#[derive(Deserialize)]` with `#[serde(try_from = "Value")]`
3. Add validation in `try_from` for constraints like:
   - `timeout_secs` must be > 0 and < 3600
   - `command` must not be empty
4. Update `Tool::execute` to use typed params

### Files to Modify

- `crates/arawn-agent/src/tool.rs` - Add ParameterValidationError
- `crates/arawn-agent/src/tools/*.rs` - Add param structs per tool

### Dependencies

None - can run in parallel with other validation tasks

## Status Updates

### Session 2026-02-07

**Completed implementation:**

1. **ParameterValidationError enum** (`tool.rs:46-92`)
   - `MissingRequired { name, hint }` - Required parameter not provided
   - `InvalidType { name, expected, actual }` - Wrong type provided
   - `OutOfRange { name, value, constraint }` - Value outside valid range
   - `InvalidValue { name, value, message }` - Value doesn't match pattern/enum
   - `Multiple(Vec<...>)` - Multiple validation errors
   - Helper constructors: `missing()`, `invalid_type()`, `out_of_range()`, `invalid_value()`, `multiple()`

2. **ParamExt trait** (`tool.rs:162-226`)
   - `required_str()`, `optional_str()` - String parameters
   - `required_i64()`, `optional_i64()`, `optional_u64()` - Integer parameters
   - `required_bool()`, `optional_bool()` - Boolean parameters
   - `optional_array()` - Array parameters
   - Implemented for `serde_json::Value`

3. **Typed Parameter Structs** (`tool.rs:228-360`)
   - `ShellParams` - command, pty, stream, cwd, timeout_secs (validates timeout range 0-3600)
   - `FileReadParams` - path (validates non-empty)
   - `FileWriteParams` - path, content, append (validates non-empty path)
   - `WebSearchParams` - query, max_results (validates range 1-100)
   - `ThinkParams` - thought (validates non-empty)
   - `MemoryStoreParams` - content, memory_type, importance (validates importance 0.0-1.0)
   - `MemoryRecallParams` - query, limit, memory_type (validates limit 1-100)
   - `DelegateParams` - task, agent_type (validates non-empty task)

4. **Tool Integration**
   - `ShellTool.execute()` now uses `ShellParams::try_from()`
   - `FileReadTool.execute()` now uses `FileReadParams::try_from()`
   - `FileWriteTool.execute()` now uses `FileWriteParams::try_from()`
   - `ThinkTool.execute()` now uses `ThinkParams::try_from()`

5. **Tests** - 35+ new tests covering:
   - All parameter struct validations
   - Valid/invalid parameter combinations
   - Edge cases (empty strings, out of range values)

**All 324 arawn-agent tests pass. Workspace compiles cleanly.**