# Subagent Delegation

Spawn specialized child agents to handle specific tasks.

## Overview

Subagents are specialized agents defined in plugins. They:

- Have focused system prompts for specific tasks
- Access only the tools they need (principle of least privilege)
- Execute independently and return results to the parent agent
- Can run in blocking or background mode

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│ Parent Agent                                                     │
│                                                                  │
│  ┌──────────────┐   ┌─────────────────┐   ┌──────────────────┐ │
│  │ Tool         │   │ Delegate Tool    │   │ Other Tools      │ │
│  │ Registry     │──▶│                  │◀──│ (shell, file,    │ │
│  │              │   │ agent: string    │   │  web, etc.)      │ │
│  └──────────────┘   │ task: string     │   └──────────────────┘ │
│                     │ context: string  │                        │
│                     │ mode: blocking/bg│                        │
│                     └────────┬─────────┘                        │
│                              │                                  │
└──────────────────────────────┼──────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────┐
│ PluginSubagentSpawner                                            │
│                                                                  │
│  agent_configs: HashMap<String, PluginAgentConfig>              │
│  agent_sources: HashMap<String, String>  (plugin name)          │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ AgentSpawner                                              │   │
│  │  parent_tools: Arc<ToolRegistry>                         │   │
│  │  backend: SharedBackend                                   │   │
│  │                                                           │   │
│  │  spawn(config) → Agent with constrained tools             │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

## When to Use Subagents

**Good use cases:**
- Research tasks requiring web searching
- Code analysis across large codebases
- Summarizing long documents
- Tasks requiring specialized expertise
- Parallel execution of independent subtasks

**Not ideal for:**
- Simple, single-tool operations
- Tasks that need continuous user interaction
- Operations where parent context is critical throughout

## Defining Subagents

Subagents are markdown files in a plugin's `agents/` directory:

```markdown
---
name: researcher
description: Web research specialist
model: sonnet
tools: ["web_fetch", "web_search", "think"]
max_iterations: 10
---

You are a research assistant specialized in finding accurate information.

## Your Process
1. Understand the research question
2. Search for relevant sources
3. Synthesize findings
4. Cite your sources

Always verify information across multiple sources when possible.
```

### Frontmatter Fields

| Field | Required | Description |
|-------|----------|-------------|
| `name` | Yes | Unique identifier |
| `description` | Yes | Human-readable description |
| `model` | No | Model override (sonnet, haiku, opus) |
| `tools` | No | Allowed tools from parent |
| `max_iterations` | No | Maximum turns before stopping |

### Tool Filtering

The `tools` list specifies which parent tools the subagent can access:

```yaml
tools: ["file_read", "glob", "grep"]
# Can read files, search for files, search contents
# No shell access - can't execute commands
# No file_write - can't modify files
```

**Security:** Always use the minimum set of tools needed.

## Using the Delegate Tool

The parent agent uses `delegate` to spawn subagents:

```json
{
  "name": "delegate",
  "parameters": {
    "agent": "researcher",
    "task": "Find recent papers on transformer architectures",
    "context": "User is building a RAG system",
    "mode": "blocking"
  }
}
```

### Parameters

| Parameter | Required | Description |
|-----------|----------|-------------|
| `agent` | Yes | Subagent name |
| `task` | Yes | Task description |
| `context` | No | Context from parent session |
| `mode` | No | `blocking` (default) or `background` |
| `max_turns` | No | Override max iterations |

## Context Injection

Context is injected into the subagent's system prompt:

```
## Context from parent session

User is working on a Rust project with async/await.
They're using tokio for the async runtime.
```

Context is truncated at 4000 characters to avoid bloat.

## Execution Modes

### Blocking Mode (default)

- Parent waits for completion
- Result returned immediately
- Best for quick tasks

### Background Mode

- Parent continues immediately
- Subagent runs asynchronously
- Use hooks to handle results

## Result Handling

Long responses are automatically truncated:
- Default limit: 8000 characters
- Preserves beginning (65%) and end (35%)
- Includes truncation notice

### Result Metadata

```rust
pub struct SubagentResult {
    pub text: String,
    pub success: bool,
    pub turns: usize,
    pub duration_ms: u64,
    pub truncated: bool,
    pub original_len: Option<usize>,
}
```

## CLI Commands

```bash
# List available agents
arawn agent list

# Agent details
arawn agent info web-researcher
```

## Best Practices

1. **Design Focused Agents** — Each agent should do one thing well
2. **Use Minimal Tool Sets** — Grant only needed tools
3. **Provide Clear Context** — Help subagents understand the situation
4. **Chain Agents** — Use one agent's output as another's input
5. **Set Appropriate Limits** — Prevent runaway execution

## Troubleshooting

| Issue | Solution |
|-------|----------|
| "Unknown agent" | Check `arawn agent list`, verify plugin loaded |
| Timeout | Break task into subtasks, increase `max_turns` |
| Missing tool access | Add tool to agent's `tools` list |
| Empty results | Check `truncated` field, ask for focused output |
