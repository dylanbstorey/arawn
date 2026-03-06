---
id: bug-tool-arguments-and-results
level: task
title: "Bug: Tool arguments and results stored as null/empty in session log"
short_code: "ARAWN-T-0270"
created_at: 2026-03-06T03:14:15.589393+00:00
updated_at: 2026-03-06T03:14:15.589393+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#bug"


exit_criteria_met: false
initiative_id: NULL
---

# Bug: Tool arguments and results stored as null/empty in session log

## Objective

Tool call arguments are stored as `null` and tool results have empty content in the workstream messages.jsonl. This means session history loses all tool interaction detail, making replay and debugging impossible.

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing

### Priority
- [x] P1 - High (important for user experience)

### Impact Assessment
- **Affected Users**: All users — tool history is lost for every session
- **Reproduction Steps**:
  1. Start the server and send a message that triggers tool use
  2. Inspect `~/.config/arawn/workstreams/workstreams/<id>/messages.jsonl`
  3. Look at `tool_use` and `tool_result` entries
- **Expected vs Actual**:
  - Expected: `tool_use` entries contain arguments JSON; `tool_result` entries contain output content
  - Actual: Arguments are `null`, result content is empty `""`

### Evidence

From workstream `76ea4fdc`:
```json
// tool_use — arguments null
{"role":"tool_use","content":"","metadata":"{\"tool_id\":\"fc_10f20ee1...\",\"name\":\"shell\",\"arguments\":null}"}

// tool_result — content empty
{"role":"tool_result","content":"","metadata":"{\"tool_call_id\":\"fc_10f20ee1...\",\"success\":false}"}
```

The `shell` tool was called with null arguments (causing failure), and `web_fetch` results returned `success:true` but with no content stored.

## Acceptance Criteria

- [ ] `tool_use` entries in messages.jsonl include the full arguments JSON
- [ ] `tool_result` entries include the tool output content
- [ ] Existing session replay works with populated fields

## Implementation Notes

### Likely Areas
- `crates/arawn-workstream/` — message serialization when persisting tool calls
- `crates/arawn-agent/src/agent.rs` — how tool calls/results are passed to the workstream store
- The `metadata` field is a stringified JSON — arguments may be lost during serialization

## Status Updates

*To be added during implementation*