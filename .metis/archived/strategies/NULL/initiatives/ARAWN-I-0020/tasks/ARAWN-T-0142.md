---
id: subagent-result-summarization
level: task
title: "Subagent result summarization"
short_code: "ARAWN-T-0142"
created_at: 2026-02-06T03:47:51.594625+00:00
updated_at: 2026-02-07T13:04:16.790219+00:00
parent: ARAWN-I-0020
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0020
---

# Subagent result summarization

## Parent Initiative

[[ARAWN-I-0020]] - Subagent Delegation

## Objective

Truncate or summarize long subagent outputs before returning them to the parent agent, preventing context bloat.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Subagent results over threshold are truncated with indicator
- [x] Configurable max result length (default: 8000 chars)
- [x] Truncation preserves beginning and end of response
- [ ] Option for LLM-based summarization (future enhancement) - deferred
- [x] Metadata about truncation included in result
- [x] Unit tests for truncation logic

## Implementation Notes

### Basic Truncation

```rust
fn format_subagent_result(
    agent_name: &str,
    response: &AgentResponse,
    max_len: usize,
) -> String {
    let text = &response.text;
    
    if text.len() <= max_len {
        return format!("## Result from '{}'\n\n{}", agent_name, text);
    }
    
    // Keep first 60% and last 30%, with truncation notice in middle
    let first_len = (max_len as f64 * 0.6) as usize;
    let last_len = (max_len as f64 * 0.3) as usize;
    
    let first = &text[..first_len];
    let last = &text[text.len() - last_len..];
    let omitted = text.len() - first_len - last_len;
    
    format!(
        "## Result from '{}'\n\n{}\n\n[...{} characters omitted...]\n\n{}",
        agent_name, first, omitted, last
    )
}
```

### Configuration

```rust
pub struct DelegateConfig {
    pub max_result_length: usize,  // default 8000
    pub summarize_long_results: bool,  // future: use LLM to summarize
}
```

### Future: LLM Summarization

For very long results, optionally summarize:
1. If result > threshold and `summarize_long_results` enabled
2. Call LLM with "Summarize this subagent result: ..."
3. Return summary instead of truncation

This is a future enhancement, not required for MVP.

### Dependencies

- [[ARAWN-T-0138]] - Basic execution

## Status Updates

### 2026-02-06: Implementation Complete

**Changes made:**

1. **`crates/arawn-types/src/delegation.rs`** - SubagentResult struct
   - Added `truncated: bool` field (with `#[serde(default)]` for backwards compat)
   - Added `original_len: Option<usize>` field (skipped if None in serialization)

2. **`crates/arawn-plugin/src/agent_spawner.rs`** - Result truncation
   - Added `DEFAULT_MAX_RESULT_LEN = 8000` constant
   - Added `TruncatedResult` struct to hold truncation metadata
   - Added `truncate_result()` function that:
     - Preserves beginning (65%) and end (35%) of available budget
     - Respects word boundaries for clean cuts
     - Includes `[...N characters omitted...]` notice
   - Updated `delegate()` to apply truncation before returning result
   - Added 6 unit tests for result truncation

3. **`crates/arawn-agent/src/tools/delegate.rs`** - Test mock
   - Updated MockSpawner to include new `truncated` and `original_len` fields

**Tests:** 151 tests pass in arawn-plugin, all workspace tests pass.

**Deferred:** LLM-based summarization for very long results - marked as future enhancement per task notes.