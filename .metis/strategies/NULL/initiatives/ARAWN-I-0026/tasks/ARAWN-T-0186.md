---
id: sessioncompactor-implementation
level: task
title: "SessionCompactor implementation"
short_code: "ARAWN-T-0186"
created_at: 2026-02-16T18:54:49.937584+00:00
updated_at: 2026-02-16T18:54:49.937584+00:00
parent: ARAWN-I-0026
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0026
---

# SessionCompactor implementation

## Parent Initiative

[[ARAWN-I-0026]] - Context Management and Auto-Compaction

## Objective

Implement SessionCompactor that summarizes older turns mid-session while preserving recent turns verbatim, reusing the existing Compressor infrastructure.

## Acceptance Criteria

- [ ] `SessionCompactor` struct wrapping existing Compressor
- [ ] `compact(session)` method that summarizes old turns
- [ ] Preserves last N turns verbatim (configurable, default: 3)
- [ ] Returns `CompactionResult` with stats (turns compacted, tokens freed)
- [ ] Progress callback for streaming status to clients
- [ ] Handles partial compaction on cancellation
- [ ] Unit tests for turn preservation logic

## Implementation Notes

### Files to Modify
- `crates/arawn-agent/src/compaction.rs` (new file)
- Reuse `crates/arawn-workstream/src/compression.rs`

### Key Types

```rust
pub struct SessionCompactor {
    compressor: Compressor,
    preserve_recent: usize,
}

pub struct CompactionResult {
    pub turns_compacted: usize,
    pub tokens_before: usize,
    pub tokens_after: usize,
    pub summary: String,
}
```

### Dependencies
- ARAWN-T-0185 (ContextTracker - triggers compaction)

## Tests

### Unit Tests
- `test_compactor_preserves_recent_turns` - last N turns unchanged
- `test_compactor_summarizes_old_turns` - turns 1..N-3 become summary
- `test_compactor_result_stats` - correct turns_compacted, tokens_before/after
- `test_compactor_empty_session` - no-op for session with < N turns
- `test_compactor_configurable_preserve_count` - custom N works
- `test_compactor_progress_callback` - callback invoked with progress

### Integration Tests
- `test_compactor_with_mock_llm` - end-to-end with mocked LLM summarization

### Test File
- `crates/arawn-agent/src/compaction.rs` (inline `#[cfg(test)]` module)

## Status Updates

*To be added during implementation*