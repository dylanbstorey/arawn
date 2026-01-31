---
id: system-prompt-builder-modular
level: initiative
title: "System Prompt Builder: Modular Prompt Generation"
short_code: "ARAWN-I-0008"
created_at: 2026-01-28T15:43:32.172902+00:00
updated_at: 2026-01-28T15:59:11.774592+00:00
parent: ARAWN-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
strategy_id: NULL
initiative_id: system-prompt-builder-modular
---

# System Prompt Builder: Modular Prompt Generation

## Context

The agent currently uses a simple static system prompt passed at construction. Moltbot demonstrates a sophisticated modular approach where system prompts are assembled dynamically from:
- Tool availability and descriptions
- Bootstrap context files (SOUL.md, BOOTSTRAP.md, MEMORY.md)
- Runtime environment info
- User identity and preferences
- Workspace configuration

This initiative adds a similar modular prompt builder to arawn-agent.

## Goals & Non-Goals

**Goals:**
- Create a `SystemPromptBuilder` that assembles prompts from configurable sections
- Support bootstrap context files loaded from workspace
- Generate tool summaries from registered tools
- Support different prompt modes (full vs minimal for subagents)
- Integrate with existing Agent and AgentConfig

**Non-Goals:**
- Multi-channel support (Arawn is single-user)
- Skills/SKILL.md system (future initiative)
- Heartbeat/cron functionality

## Detailed Design

### Core Components

1. **SystemPromptBuilder** - Main builder struct
   - `new()` - Create with default sections
   - `with_identity(name, description)` - Set agent identity
   - `with_tools(registry)` - Generate tool section from registry
   - `with_bootstrap_files(path)` - Load workspace context files
   - `with_workspace(path)` - Set working directory info
   - `with_datetime(timezone)` - Add current time context
   - `with_memory_hints()` - Add memory search guidance
   - `with_mode(PromptMode)` - Full vs minimal
   - `build()` - Assemble final prompt string

2. **PromptMode** enum
   - `Full` - All sections (main agent)
   - `Minimal` - Reduced sections (subagents)
   - `Identity` - Just identity line

3. **BootstrapContext** - Loaded context files
   - Load from workspace directory
   - Truncate if > max chars
   - Support: SOUL.md, BOOTSTRAP.md, MEMORY.md, IDENTITY.md

### Prompt Sections (in order)

1. Identity - "You are {name}, {description}"
2. Tools - Available tools with descriptions
3. Workspace - Working directory, repo root
4. DateTime - Current time with timezone
5. Memory - Search guidance if memory available
6. Bootstrap - Loaded context files
7. Runtime - Model info, environment

### Integration Points

- `AgentConfig` gets `workspace_path: Option<PathBuf>`
- `AgentBuilder` gets `with_prompt_builder(SystemPromptBuilder)`
- Falls back to existing `system_prompt` if no builder

## Implementation Plan

1. Create `prompt` module in arawn-agent
2. Implement `SystemPromptBuilder` with section builders
3. Add `BootstrapContext` loader
4. Integrate with `AgentBuilder`
5. Add tests for prompt generation
6. Update existing code to use builder