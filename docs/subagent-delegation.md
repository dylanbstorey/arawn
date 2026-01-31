# Subagent Delegation

Subagent delegation allows Arawn to spawn specialized child agents to handle specific tasks. Each subagent has its own system prompt, tool constraints, and execution context, enabling a divide-and-conquer approach to complex problems.

## Overview

### What Are Subagents?

Subagents are specialized agents defined in plugins. They:
- Have focused system prompts for specific tasks
- Access only the tools they need (principle of least privilege)
- Execute independently and return results to the parent agent
- Can run in blocking or background mode

### When to Use Subagents

**Good use cases**:
- Research tasks that require web searching
- Code analysis across large codebases
- Summarizing long documents
- Tasks requiring specialized expertise
- Parallel execution of independent subtasks

**Not ideal for**:
- Simple, single-tool operations
- Tasks that need continuous user interaction
- Operations where the parent context is critical throughout

## Defining Subagents

Subagents are defined as markdown files in a plugin's `agents/` directory.

### File Structure

```
my-plugin/
├── .claude-plugin/
│   └── plugin.json
└── agents/
    ├── researcher.md
    ├── code-analyzer.md
    └── summarizer.md
```

### Agent Definition Format

Each agent is a markdown file with YAML frontmatter:

```markdown
---
name: researcher
description: Web research specialist for finding information
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
| `name` | Yes | Unique identifier for the agent |
| `description` | Yes | Human-readable description shown in `arawn agent list` |
| `model` | No | Model override (e.g., `sonnet`, `haiku`, `opus`) |
| `tools` | No | List of tools this agent can use |
| `max_iterations` | No | Maximum turns before stopping |

### Tool Filtering

The `tools` list specifies which tools from the parent agent the subagent can access:

```yaml
tools: ["file_read", "glob", "grep"]
# Can read files, search for files, search file contents
# No shell access - can't execute commands
# No file_write - can't modify files
```

**Security**: Always use the minimum set of tools needed. A research agent doesn't need `shell`; a code analyzer doesn't need `web_fetch`.

### System Prompt

Everything after the frontmatter becomes the agent's system prompt. Write it as if instructing a person:

- Explain the agent's role and purpose
- Describe the process to follow
- Set expectations for output format
- Include any constraints or guidelines

## Using the Delegate Tool

The parent agent uses the `delegate` tool to spawn subagents:

```json
{
  "name": "delegate",
  "parameters": {
    "agent": "researcher",
    "task": "Find recent papers on transformer architectures published in 2024",
    "context": "User is building a RAG system and wants to understand state of the art",
    "mode": "blocking"
  }
}
```

### Parameters

| Parameter | Required | Description |
|-----------|----------|-------------|
| `agent` | Yes | Name of the subagent to use |
| `task` | Yes | Task description for the subagent |
| `context` | No | Context from parent session (included in system prompt) |
| `mode` | No | `blocking` (default) or `background` |
| `max_turns` | No | Override max iterations for this delegation |

### Context Injection

The `context` parameter passes relevant information from the parent conversation to the subagent. It's included in the system prompt, not in the conversation history:

```
## Context from parent session

User is working on a Rust project and encountered an error with async trait implementations.
They're using tokio for the async runtime.
```

Context is automatically truncated at 4000 characters to avoid bloating the subagent's context window.

### Execution Modes

**Blocking Mode** (default):
- Parent agent waits for subagent to complete
- Result returned immediately
- Best for quick tasks or when result is needed immediately

**Background Mode**:
- Parent agent continues immediately
- Subagent runs asynchronously
- Use hooks (`SubagentStarted`, `SubagentCompleted`) to handle results
- Best for long-running research or parallel tasks

## CLI Commands

### List Available Agents

```bash
arawn agent list
```

Output:
```
Available Subagents:

  NAME                 DESCRIPTION                         TOOLS                          PLUGIN
  --------------------------------------------------------------------------------------------
  web-researcher       Web research specialist            [web_fetch, web_search, think]  (research)
  code-analyzer        Code analysis specialist           [file_read, glob, grep, think]  (research)
  summarizer           Summarization specialist           [file_read, web_fetch, think]   (research)

Use 'arawn agent info <name>' for detailed information.
```

### Agent Details

```bash
arawn agent info web-researcher
```

Output:
```
Agent: web-researcher

Description: Web research specialist for finding and summarizing information from the internet
Plugin: research
Model: sonnet
Max Iterations: 10

Allowed Tools:
  - web_fetch
  - web_search
  - think

System Prompt:
  You are a web research specialist. Your job is to find accurate, relevant
  information from the internet and present it clearly...
```

### JSON Output

For scripting:
```bash
arawn agent list --json
arawn agent info web-researcher --json
```

## Result Handling

### Truncation

Long subagent responses are automatically truncated to prevent context bloat:
- Default limit: 8000 characters
- Preserves beginning (65%) and end (35%)
- Includes truncation notice with character count

### Result Metadata

`SubagentResult` includes:
- `text`: The response (possibly truncated)
- `success`: Whether execution completed normally
- `turns`: Number of iterations taken
- `duration_ms`: Execution time
- `truncated`: Whether result was truncated
- `original_len`: Original length if truncated

## Best Practices

### 1. Design Focused Agents

Each agent should do one thing well:
- ❌ "general-helper" that does everything
- ✅ "web-researcher" for internet research
- ✅ "code-analyzer" for codebase exploration
- ✅ "summarizer" for content distillation

### 2. Use Minimal Tool Sets

Grant only the tools needed:
```yaml
# Good: minimal tools for the task
tools: ["file_read", "grep"]

# Bad: everything available
tools: ["file_read", "file_write", "shell", "web_fetch", "..."]
```

### 3. Provide Clear Context

Help subagents understand the situation:
```json
{
  "task": "Analyze authentication flow",
  "context": "Looking for security issues. The app uses JWT tokens stored in cookies. Main auth logic is in src/auth/."
}
```

### 4. Chain Agents for Complex Tasks

Use one agent's output as another's input:
1. `web-researcher`: Find relevant documentation
2. `summarizer`: Condense the findings
3. Parent agent: Apply to current task

### 5. Set Appropriate Limits

Use `max_iterations` to prevent runaway execution:
- Quick lookups: 5 iterations
- Research tasks: 10-15 iterations
- Deep analysis: 15-20 iterations

## Troubleshooting

### "Unknown agent: X"

**Cause**: Agent name doesn't match any loaded agent.

**Solution**:
1. Check `arawn agent list` for available agents
2. Verify agent markdown file exists in `agents/` directory
3. Ensure plugin is loaded (`arawn plugin list`)
4. Check for typos in agent name

### Subagent Timeout

**Cause**: Task too complex or max_iterations too low.

**Solution**:
1. Break task into smaller subtasks
2. Increase `max_turns` parameter in delegate call
3. Increase `max_iterations` in agent definition
4. Simplify the task description

### Missing Tool Access

**Cause**: Tool not in agent's allowed list.

**Solution**:
1. Check agent's `tools` list in definition
2. Add needed tool to the list
3. Verify tool exists in parent agent's registry

### Empty or Truncated Results

**Cause**: Result exceeded length limit.

**Solution**:
1. Check `truncated` field in result
2. Ask for more focused output in task description
3. Break into multiple subtasks if needed

### Subagent Can't Access Files

**Cause**: Subagent running in wrong directory or lacking permissions.

**Solution**:
1. Verify `file_read` is in tools list
2. Use absolute paths in task description
3. Pass file paths in context

## Example Plugin

See `examples/plugins/research/` for a complete example with three agents:
- `web-researcher`: Internet research
- `code-analyzer`: Codebase exploration
- `summarizer`: Content summarization

Install it locally:
```bash
cp -r examples/plugins/research ~/.config/arawn/plugins/
arawn agent list  # Verify agents appear
```

## Related

- [Plugin Development](./plugins.md) - Creating plugins
- [Tool Reference](./tools.md) - Available tools
- [Configuration](./configuration.md) - Arawn configuration
