---
id: sessioncompactor-implementation
level: task
title: "SessionCompactor implementation"
short_code: "ARAWN-T-0186"
created_at: 2026-02-16T18:54:49.937584+00:00
updated_at: 2026-02-17T01:40:04.940465+00:00
parent: ARAWN-I-0026
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


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

## Acceptance Criteria

## Acceptance Criteria

- [x] `SessionCompactor` struct wrapping existing Compressor
- [x] `compact(session)` method that summarizes old turns
- [x] Preserves last N turns verbatim (configurable, default: 3)
- [x] Returns `CompactionResult` with stats (turns compacted, tokens freed)
- [x] Progress callback for streaming status to clients
- [x] Handles partial compaction on cancellation
- [x] Unit tests for turn preservation logic

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

### Session 1 (2026-02-16)
- Created `crates/arawn-agent/src/compaction.rs` with:
  - `CompactorConfig` - configuration for model, max tokens, preserve count
  - `CompactionResult` - stats with turns_compacted, tokens_before/after, summary
  - `CompactionProgress` enum - Started, Summarizing, Completed, Cancelled
  - `CancellationToken` - atomic bool for cooperative cancellation
  - `SessionCompactor` - main compactor struct
- Implemented key methods:
  - `compact()` - simple compaction
  - `compact_with_progress()` - with progress callback
  - `compact_with_options()` - full options with cancellation support
  - `needs_compaction()` - check if session exceeds threshold
- Cancellation support:
  - Checks for cancellation at start and between stages
  - Reports cancellation via progress callback
  - Returns `AgentError::Cancelled` on cancellation
- Added 17 unit tests covering:
  - Config defaults, result stats, compression ratio
  - Turn preservation logic
  - Progress callback invocation
  - Cancellation handling
- Exported all types from arawn-agent crate

**All acceptance criteria complete.**