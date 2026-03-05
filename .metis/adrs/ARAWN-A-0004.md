---
id: 001-claude-code-compatible-plugin
level: adr
title: "Claude Code-Compatible Plugin System"
number: 1
short_code: "ARAWN-A-0004"
created_at: 2026-03-05T05:15:59.694650+00:00
updated_at: 2026-03-05T05:18:59.685500+00:00
decision_date: 
decision_maker: 
parent: 
archived: false

tags:
  - "#adr"
  - "#phase/decided"


exit_criteria_met: false
initiative_id: NULL
---

# ADR-4: Claude Code-Compatible Plugin System

## Context

Arawn needs an extension mechanism for users and third parties to add capabilities — custom skills (prompt templates), lifecycle hooks (pre/post tool use, session events), subagent definitions, and MCP server integrations — without modifying core code.

Two distinct extension needs exist:

1. **Prompt-level extensions**: Skills, hooks, and agents that inject prompts, intercept tool calls, or spawn subagents. These are inherently prompt-driven and don't require code execution sandboxing.
2. **Code-level extensions**: Arbitrary scripts that run as workflow task actions (covered by ADR-2's WASM sandbox).

Claude Code has established a plugin format that a growing community of users already understands: JSON manifests, markdown skill files with YAML frontmatter, hook configuration files, and agent definitions. Adopting this format means existing plugins work with Arawn out of the box.

## Decision

Adopt **Claude Code's plugin format** for prompt-level extensions (skills, hooks, agents) while keeping WASM sandboxing (ADR-2) for code-level script execution in workflows.

### Plugin Structure

```
my-plugin/
  .claude-plugin/
    plugin.json          # manifest: name, version, capabilities
  skills/
    my-skill/
      SKILL.md           # prompt template with YAML frontmatter
  hooks/
    hooks.json           # lifecycle hook definitions
  agents/
    my-agent.md          # agent definition with system prompt
```

### Component Types

- **Skills**: Markdown prompt templates invoked via `/skill-name`. YAML frontmatter defines arguments, description, and trigger conditions.
- **Hooks**: JSON-configured event handlers (PreToolUse, PostToolUse, Stop, SessionStart, etc.) that can block, modify, or augment tool calls. Support both bash command hooks and prompt-based hooks.
- **Agents**: Markdown files defining subagent configurations — system prompt, available tools, constraints, and trigger conditions.
- **MCP Servers**: Standard `.mcp.json` configuration for external tool integrations.

### Plugin Discovery

Plugins are loaded from:
1. Project-local: `.claude/plugins/` in the working directory
2. User-global: `~/.claude/plugins/`
3. Subscribed: Git-hosted plugins synced via `arawn plugin add <url>`

### Runtime Model

Plugins operate at the **prompt layer**, not the code execution layer:
- Skills inject prompt text into agent context
- Hooks intercept events and return allow/deny/modify decisions
- Agents spawn as subagents with constrained tool sets
- No plugin code runs outside of hook bash commands (which are subject to FsGate sandboxing)

## Alternatives Analysis

| Option | Pros | Cons | Risk Level |
|--------|------|------|------------|
| **Claude Code format (chosen)** | Existing ecosystem, familiar format, community plugins work | Tied to Claude Code's design decisions | Low |
| **Custom WASM plugin API** | Strongest isolation, arbitrary code | Complex API surface, no existing ecosystem, high implementation cost | Medium |
| **Lua scripting** | Fast, embeddable, proven in game engines | No existing AI-tool ecosystem, another language to learn | Medium |
| **Python plugin API** | Familiar to most developers | Requires Python runtime, sandboxing is hard, dependency management | High |
| **Pure MCP** | Standard protocol, tool-focused | Too narrow — can't express skills, hooks, or agent definitions | Low |

## Rationale

- **Ecosystem leverage**: Claude Code plugins already exist for many common tasks. Compatibility means Arawn gets these for free.
- **Right abstraction level**: Most agent extensions are prompt-level (inject instructions, intercept tool calls, define subagents). These don't need code sandboxing — they operate on text.
- **Separation of concerns**: Prompt-level extensions (this ADR) and code-level extensions (ADR-2 WASM sandbox) serve different needs. Conflating them adds unnecessary complexity.
- **Low barrier to entry**: Writing a plugin requires only markdown and JSON — no compilation, no SDK, no special tooling.
- **Hooks provide sufficient power**: PreToolUse/PostToolUse hooks can block dangerous operations, modify parameters, or inject guardrails. Prompt-based hooks let the LLM itself make policy decisions.

## Consequences

### Positive
- Existing Claude Code plugins work with Arawn without modification
- Plugin authoring requires only markdown and JSON knowledge
- Hot-reloading via filesystem watcher — edit a skill file, changes take effect immediately
- Clear security boundary — plugins inject prompts, not arbitrary code
- Git-based plugin distribution enables sharing and versioning

### Negative
- Tied to Claude Code's format evolution — breaking changes upstream require adaptation
- Prompt-based hooks rely on LLM judgment, which can be inconsistent
- Bash command hooks in hooks.json run as subprocesses (mitigated by FsGate sandboxing)
- Plugin validation must handle malformed markdown/JSON gracefully

### Neutral
- WASM script execution (ADR-2) remains the path for arbitrary code in workflows
- MCP servers provide a complementary extension mechanism for external tools
- Plugin format may diverge from Claude Code over time as Arawn-specific needs emerge