---
id: implement-thinktool
level: task
title: "Implement ThinkTool"
short_code: "ARAWN-T-0091"
created_at: 2026-01-31T02:41:42.802727+00:00
updated_at: 2026-01-31T03:55:52.112601+00:00
parent: ARAWN-I-0014
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0014
---

# Implement ThinkTool

## Objective

Create a new `ThinkTool` that allows the agent to persist internal reasoning as `ContentType::Thought` memories. Thoughts are visible to the agent in subsequent turns but not shown to the user.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `ThinkTool` struct in `crates/arawn-agent/src/tools/think.rs` implementing the `Tool` trait
- [ ] Accepts `thought` (required string) parameter
- [ ] Stores thought in MemoryStore with `ContentType::Thought` and current session_id metadata
- [ ] Returns confirmation text to the agent (e.g., "Thought recorded.")
- [ ] Registered in `tools/mod.rs` exports
- [ ] Tests: tool metadata/parameters schema, successful thought storage, missing thought param error

## Implementation Notes

### Files
- `crates/arawn-agent/src/tools/think.rs` — new file
- `crates/arawn-agent/src/tools/mod.rs` — add `pub mod think;` and export

### Technical Approach
- Follow the same pattern as `NoteTool` — takes an `Arc<MemoryStore>`, stores content
- The `think` tool's JSON schema: `{ "thought": { "type": "string", "description": "Your internal reasoning..." } }`
- Metadata should include `session_id` from `ToolContext` so thoughts are scoped to sessions
- The tool result text is only seen by the agent, never the user — this is inherent to tool results in the agent loop

### Dependencies
- ARAWN-T-0090 (ContentType::Thought must exist)

## Status Updates

### Session 1
- Created `crates/arawn-agent/src/tools/think.rs` implementing `Tool` trait
- Accepts `thought` (required string) parameter, stores as `ContentType::Thought`
- Sets `session_id` in metadata from `ToolContext`
- Returns "Thought recorded." confirmation
- Registered in `tools/mod.rs` as `pub use think::ThinkTool`
- Added `Send + Sync` unsafe impls and `Debug` impl for `MemoryStore` (needed for `Arc<MemoryStore>` in async tool context)
- Tests: metadata/schema validation, successful thought storage with content verification, missing param error
- `angreal check all` passes, `angreal test unit` passes (657 tests, 0 failures)