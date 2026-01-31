---
id: tool-output-sanitization
level: task
title: "Tool Output Sanitization"
short_code: "ARAWN-T-0149"
created_at: 2026-02-07T16:46:15.130501+00:00
updated_at: 2026-02-07T19:46:30.103471+00:00
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

# Tool Output Sanitization

## Parent Initiative

[[ARAWN-I-0021]] - Interface Enforcement and Defensive Validation

## Objective

Sanitize tool outputs before they're returned to the LLM. Enforce size limits, validate content structure, and add truncation indicators to prevent context overflow and malformed responses.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Configurable output size limits per tool (default 100KB)
- [x] Automatic truncation with clear indicator message
- [x] Binary content detection and rejection
- [x] JSON output structure validation (if tool declares JSON output)
- [x] Null byte and control character stripping
- [x] Graceful handling of tool execution failures
- [x] Unit tests for edge cases (huge output, binary, malformed)

## Implementation Notes

### Technical Approach

1. Add `OutputConfig` to tool configuration:
   ```rust
   pub struct OutputConfig {
       pub max_size_bytes: usize,      // default 100KB
       pub truncation_message: String, // "[truncated]"
       pub strip_control_chars: bool,  // default true
   }
   ```

2. Add `ToolResult::sanitize(&mut self, config: &OutputConfig)` method

3. Sanitization pipeline:
   - Check size, truncate if needed
   - Strip null bytes and control chars (except newlines)
   - Validate JSON if expected
   - Add truncation indicator if truncated

4. Apply sanitization in `ToolRegistry::execute` after tool returns

### Size Limit Defaults

| Tool Type | Default Limit |
|-----------|---------------|
| Shell | 100KB |
| File Read | 500KB |
| Web Fetch | 200KB |
| Search | 50KB |

### Files to Modify

- `crates/arawn-agent/src/tool.rs` - Add OutputConfig, sanitize method
- `crates/arawn-agent/src/tools/*.rs` - Configure limits per tool

### Dependencies

None - small focused task

## Status Updates

### Session 2026-02-07 - Implementation Complete

**Added `OutputConfig` struct in `crates/arawn-agent/src/tool.rs`:**
- `max_size_bytes: usize` - Maximum output size (default 100KB)
- `truncation_message: String` - Message appended when truncated
- `strip_control_chars: bool` - Strip control chars except newlines/tabs
- `strip_null_bytes: bool` - Remove null bytes
- `validate_json: bool` - Validate JSON structure

**Tool-specific configs:**
- `OutputConfig::for_shell()` - 100KB limit
- `OutputConfig::for_file_read()` - 500KB limit
- `OutputConfig::for_web_fetch()` - 200KB limit
- `OutputConfig::for_search()` - 50KB limit

**Added `OutputSanitizationError` enum:**
- `BinaryContent { null_bytes, checked_bytes }` - Binary data detected
- `MalformedJson { reason }` - Invalid JSON structure

**Added sanitization functions:**
- `sanitize_output(input, config)` - Full text sanitization pipeline
- `validate_json_output(value)` - JSON structure validation (depth check)

**Added `ToolResult` methods:**
- `sanitize(&self, config)` - Apply sanitization to result
- `sanitize_default(&self)` - Sanitize with default config
- `was_truncated(&self)` - Check if result was truncated
- `content_size(&self)` - Get content size in bytes

**Updated `ToolRegistry`:**
- `execute()` - Now applies default sanitization
- `execute_with_config()` - Apply custom sanitization config
- `execute_raw()` - Skip sanitization (for internal use)
- `output_config_for(name)` - Get recommended config for tool

**Exported in `crates/arawn-agent/src/lib.rs`:**
- `OutputConfig`, `OutputSanitizationError`, `DEFAULT_MAX_OUTPUT_SIZE`
- `sanitize_output`, `validate_json_output`

**Tests added:**
- 25+ unit tests for output sanitization
- Tests for binary detection, truncation, control char stripping
- Tests for JSON validation and deep nesting detection
- Tests for ToolResult sanitization
- Tests for registry execute with/without sanitization

**Verification:**
- All 347 arawn-agent tests pass
- All workspace unit tests pass
- Workspace compiles cleanly