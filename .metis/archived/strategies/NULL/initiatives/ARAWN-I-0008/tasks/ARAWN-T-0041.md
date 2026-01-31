---
id: add-prompt-builder-tests
level: task
title: "Add Prompt Builder Tests"
short_code: "ARAWN-T-0041"
created_at: 2026-01-28T15:50:13.444020+00:00
updated_at: 2026-01-28T15:59:06.949985+00:00
parent: ARAWN-I-0008
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0008
---

# Add Prompt Builder Tests

## Parent Initiative
[[ARAWN-I-0008]]

## Objective

Add comprehensive unit tests for the prompt builder module covering all sections and edge cases.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Tests for SystemPromptBuilder basic assembly
- [ ] Tests for each section builder (identity, tools, workspace, datetime, memory)
- [ ] Tests for PromptMode variants (Full, Minimal, Identity)
- [ ] Tests for BootstrapContext loading and truncation
- [ ] Tests for truncation edge cases (exact boundary, unicode)
- [ ] All tests pass with `cargo test -p arawn-agent`

## Test Cases

### SystemPromptBuilder Tests
- `test_builder_default_empty` - New builder produces minimal output
- `test_builder_with_identity` - Identity section included
- `test_builder_with_tools` - Tools formatted correctly
- `test_builder_full_mode` - All sections included in Full mode
- `test_builder_minimal_mode` - Reduced sections in Minimal mode
- `test_builder_identity_mode` - Only identity in Identity mode
- `test_sections_joined_with_double_newline` - Proper separator

### BootstrapContext Tests
- `test_load_nonexistent_dir` - Graceful handling of missing dir
- `test_load_empty_dir` - Empty context when no files
- `test_load_soul_md` - SOUL.md loaded correctly
- `test_load_multiple_files` - All bootstrap files loaded
- `test_truncation_under_limit` - No truncation when small
- `test_truncation_over_limit` - Correct head/tail split
- `test_truncation_unicode_boundary` - Safe char boundary handling
- `test_to_prompt_section_format` - Correct markdown output

### Integration Tests
- `test_agent_with_prompt_builder` - Agent uses builder
- `test_agent_fallback_to_static_prompt` - Backward compatibility

## Implementation Notes

### Test Utilities
Create temp directories with test bootstrap files using `tempfile` crate.

### Dependencies
- Depends on ARAWN-T-0038, T-0039, T-0040 (implementation complete)

## Status Updates

### 2026-01-28
- All tests implemented as part of T-0038, T-0039, and T-0040
- Total: 31 prompt-related tests across 4 test modules

**Test counts:**
- `prompt::builder::tests` - 10 tests
- `prompt::mode::tests` - 5 tests  
- `prompt::bootstrap::tests` - 13 tests
- `agent::tests` (prompt builder integration) - 3 tests

All acceptance criteria met. All tests passing.