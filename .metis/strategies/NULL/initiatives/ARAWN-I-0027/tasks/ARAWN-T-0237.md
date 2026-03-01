---
id: make-sessioncompactor-summary
level: task
title: "Make SessionCompactor summary prompt configurable"
short_code: "ARAWN-T-0237"
created_at: 2026-03-01T15:55:20.199524+00:00
updated_at: 2026-03-01T16:08:58.162725+00:00
parent: ARAWN-I-0027
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/active"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0027
---

# Make SessionCompactor summary prompt configurable

## Parent Initiative

[[ARAWN-I-0027]] — RLM Exploration Agent

## Objective

Make the `SessionCompactor` summary prompt configurable so different agent types can provide their own compaction strategy. Currently the prompt is hardcoded as `MID_SESSION_SUMMARY_PROMPT` in `compaction.rs`. The RLM exploration agent needs a research-focused compaction prompt; future agent types will need their own.

## Acceptance Criteria

## Acceptance Criteria

- [ ] `CompactorConfig` has an optional `summary_prompt` field
- [ ] When set, `summarize_turns()` uses the custom prompt instead of the hardcoded default
- [ ] When not set, falls back to the existing `MID_SESSION_SUMMARY_PROMPT` (no behavior change for current callers)
- [ ] Existing tests pass unchanged
- [ ] New test: custom prompt is used when provided
- [ ] `angreal check all` passes

## Implementation Notes

### File
`crates/arawn-agent/src/compaction.rs`

### Approach
1. Add `summary_prompt: Option<String>` to `CompactorConfig`
2. Default to `None` (preserves existing behavior)
3. In `summarize_turns()` (line 288), use `self.config.summary_prompt.as_deref().unwrap_or(MID_SESSION_SUMMARY_PROMPT)` instead of the hardcoded constant
4. Add builder method `with_summary_prompt(prompt: impl Into<String>)` on `SessionCompactor` for ergonomics

### Scope
Small change — one field, one fallback, one builder method, one test. No API changes for existing callers.

## Status Updates

### Implementation Complete
- Added `summary_prompt: Option<String>` to `CompactorConfig` with `None` default
- Added `with_summary_prompt()` builder method on `SessionCompactor`
- Updated `summarize_turns()` to use `self.config.summary_prompt.as_deref().unwrap_or(MID_SESSION_SUMMARY_PROMPT)`
- Added `test_compact_custom_summary_prompt` test — verifies custom prompt reaches MockBackend's request log
- All 18 compaction tests pass
- `angreal check all` clean
- `angreal test unit` all pass
- No behavior change for existing callers (None falls back to hardcoded default)