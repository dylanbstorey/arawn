---
id: system-prompt-instructions-for
level: task
title: "System prompt instructions for think tool"
short_code: "ARAWN-T-0094"
created_at: 2026-01-31T02:41:44.461910+00:00
updated_at: 2026-01-31T03:55:52.686949+00:00
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

# System prompt instructions for think tool

## Objective

Update the agent's bootstrap/system prompt to instruct it on when and how to use the `think` tool for multi-step reasoning, planning, and self-correction.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] System prompt includes instructions for `think` tool usage
- [ ] Instructions cover: when to think (complex questions, multi-step reasoning, planning), what to record (reasoning steps, assumptions, corrections), and that thoughts persist across sessions
- [ ] Instructions are only included when the think tool is registered (conditional on tool availability)
- [ ] Tests: prompt builder includes think section when tool registered, omits when not registered

## Implementation Notes

### Files
- `crates/arawn-agent/src/prompt/` — update bootstrap prompt content or add a conditional section in the prompt builder

### Technical Approach
- The prompt builder already supports conditional sections based on available tools/capabilities
- Add a think-specific prompt block that's appended when "think" is in the tool registry
- Keep instructions concise — the agent should understand the tool's purpose without lengthy explanation
- Example prompt text: "Use the `think` tool to record your reasoning before answering complex questions. Think through multi-step problems, note assumptions, and correct yourself. Your thoughts are stored permanently and may be recalled in future sessions."

### Dependencies
- ARAWN-T-0091 (ThinkTool must exist to register)

## Status Updates

### Session 1
- Added `think_enabled` bool field to `SystemPromptBuilder`
- Auto-detected from tool list in `with_tools()` and `with_tool_summaries()` — checks for tool named "think"
- Added `build_think_section()` returning guidance on when/how to use the think tool
- Section included in `build()` gated by `mode.include_memory_hints() && think_enabled` (Full mode only)
- Added 3 tests: included when registered, omitted without think tool, omitted in Minimal mode
- All checks and tests pass