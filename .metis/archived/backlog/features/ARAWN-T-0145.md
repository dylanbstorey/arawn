---
id: context-compaction-llm-based
level: task
title: "Context Compaction: LLM-based Summarization for Long Subagent Responses"
short_code: "ARAWN-T-0145"
created_at: 2026-02-07T13:08:02.429993+00:00
updated_at: 2026-02-07T13:18:40.347415+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#feature"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Context Compaction: LLM-based Summarization for Long Subagent Responses

## Objective

Replace simple truncation of long subagent responses with LLM-based summarization (context compaction). Currently, when a subagent returns a response exceeding the max length (8000 chars), we truncate by preserving 65% of the beginning and 35% of the end with a notice in the middle. This loses potentially valuable information. Context compaction would use a configurable LLM to intelligently summarize the response while preserving key information.

## Backlog Item Details

### Type
- [x] Feature - New functionality or enhancement  

### Priority
- [ ] P2 - Medium (nice to have)

### Business Justification
- **User Value**: Subagent results retain more useful information instead of losing middle content to truncation
- **Business Value**: More effective agent collaboration, better task completion quality
- **Effort Estimate**: M

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] `AgentSpawner::spawn()` accepts an optional compaction backend (LLM client) - via `with_compaction()` on `PluginSubagentSpawner`
- [x] When response exceeds threshold and backend is available, call configured model to summarize
- [x] Compaction prompt preserves: key findings, code snippets, citations, actionable items - see `COMPACTION_SYSTEM_PROMPT`
- [x] `SubagentResult` includes `compacted: bool` field (in addition to `truncated`)
- [x] Fallback to truncation if compaction fails or no backend configured
- [x] Configurable via `[delegation]` config section (enable/disable, threshold, model)

## Implementation Notes

### Technical Approach

1. Add `CompactionConfig` to delegation settings (threshold, enabled, model)
2. Create compaction prompt template focused on preserving actionable information
3. In `agent_spawner.rs`, after getting raw result:
   - If len > threshold and compaction enabled and backend available → compact
   - Else if len > threshold → truncate (existing behavior)
   - Else → return as-is
4. Model is configurable - recommend fast/cheap model for speed/cost efficiency

### Key Files
- `crates/arawn-plugin/src/agent_spawner.rs` - Add compaction logic
- `crates/arawn-types/src/delegation.rs` - Add `compacted` field to `SubagentResult`
- `crates/arawn-config/src/types.rs` - Add `CompactionConfig`

### Dependencies
- Requires LLM backend access in `AgentSpawner` (may need to thread through from parent)

### Risk Considerations
- Compaction adds latency (haiku call) - mitigate with reasonable threshold
- Summarization might lose critical details - prompt engineering important
- Cost increase for long responses - make opt-in

## Status Updates

### Session 1 - 2026-02-07
**Progress:**
- Read existing implementation in `agent_spawner.rs` - has `truncate_result()` function
- Read `SubagentResult` in `delegation.rs` - already has `truncated` and `original_len` fields
- Read config types - need to add `DelegationConfig` with compaction settings
- Planning implementation approach

**Implementation Plan:**
1. Add `DelegationConfig` with `CompactionConfig` to `types.rs` ✅
2. Add `compacted: bool` field to `SubagentResult` ✅
3. Add compaction backend to `PluginSubagentSpawner` ✅
4. Implement `compact_result()` function with LLM call ✅
5. Wire compaction into `delegate()` method ✅

**Completed:**
- Added `compacted: bool` field to `SubagentResult` in `delegation.rs`
- Added `DelegationConfig` and `CompactionConfig` structs to config types
- Implemented `COMPACTION_SYSTEM_PROMPT` with preservation priorities
- Implemented async `compact_result()` function using LLM backend
- Added `with_compaction()` builder method to `PluginSubagentSpawner`
- Updated `delegate()` method to try compaction before truncation
- Added comprehensive tests for all new functionality
- All tests passing (24 agent_spawner tests, 5 delegation config tests)