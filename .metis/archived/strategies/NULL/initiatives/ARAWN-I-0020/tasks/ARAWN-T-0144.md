---
id: subagent-delegation-documentation
level: task
title: "Subagent delegation documentation and examples"
short_code: "ARAWN-T-0144"
created_at: 2026-02-06T03:47:53.617181+00:00
updated_at: 2026-02-07T13:04:17.872927+00:00
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

# Subagent delegation documentation and examples

## Parent Initiative

[[ARAWN-I-0020]] - Subagent Delegation

## Objective

Create documentation and example configurations for subagent delegation, helping users understand how to define and use subagents.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] README section on subagent delegation (added to ARCHITECTURE.md)
- [x] Example plugin with 2-3 subagent definitions (examples/plugins/research)
- [x] Agent definition format documented (AGENT.md structure)
- [x] Tool filtering syntax documented
- [x] Use cases and best practices section
- [x] Troubleshooting common issues

## Documentation Sections

### User Guide Content

#### Subagent Delegation Overview
- What subagents are and when to use them
- Blocking vs background execution
- Security considerations (tool constraints)

#### Defining Subagents in Plugins

Example `agents/researcher/AGENT.md`:
```markdown
---
name: researcher
description: Web research specialist for finding information
tools:
  - web_fetch
  - web_search
  - memory
model: claude-sonnet-4-20250514
---

You are a research assistant. Your job is to find and summarize information from the web.

When given a research task:
1. Search for relevant sources
2. Fetch and read the content
3. Synthesize findings into a clear summary

Always cite your sources.
```

#### Using the Delegate Tool

```
User: Find recent papers on RAG architectures
Agent: I'll delegate this to the researcher subagent.
[Uses delegate tool with agent="researcher", task="Find recent papers..."]
[Subagent executes, returns results]
Agent: The researcher found 5 relevant papers...
```

#### Best Practices
- Keep subagent prompts focused and specific
- Use minimal tool sets for security
- Prefer blocking for short tasks, background for long research
- Pass relevant context from parent conversation

### Troubleshooting

- "Unknown agent" - Check agent name matches AGENT.md filename
- Subagent timeout - Increase max_turns or break into smaller tasks
- Missing tools - Verify tool names in agent config match registry

### Files to Create

- `docs/subagent-delegation.md` - Main documentation
- `examples/plugins/research/` - Example research plugin with agents
- Update main README with link to delegation docs

### Dependencies

- All other tasks should be complete first

## Status Updates

### 2026-02-06: Implementation Complete

**Documentation created:**

1. **`docs/subagent-delegation.md`** - Comprehensive user guide
   - Overview of what subagents are and when to use them
   - Agent definition format (frontmatter fields, system prompt)
   - Tool filtering syntax and security considerations
   - Using the delegate tool (parameters, modes)
   - Context injection and result handling
   - CLI commands (list, info, --json)
   - Best practices for focused agents, minimal tools, clear context
   - Troubleshooting section for common issues

2. **`ARCHITECTURE.md`** - Added subagent delegation section
   - Architecture diagram showing spawner flow
   - Key component table
   - Context injection and result truncation docs
   - Updated data flow summary table

**Example plugin created:**

3. **`examples/plugins/research/`** - Complete example with 3 agents
   - `web-researcher.md` - Web research specialist
   - `code-analyzer.md` - Codebase exploration specialist
   - `summarizer.md` - Content summarization specialist
   - `plugin.json` - Plugin manifest
   - `README.md` - Plugin documentation with examples

**Files created:**
- `docs/subagent-delegation.md` (new, ~350 lines)
- `examples/plugins/research/.claude-plugin/plugin.json` (new)
- `examples/plugins/research/agents/web-researcher.md` (new)
- `examples/plugins/research/agents/code-analyzer.md` (new)
- `examples/plugins/research/agents/summarizer.md` (new)
- `examples/plugins/research/README.md` (new)
- `ARCHITECTURE.md` (updated with delegation section)