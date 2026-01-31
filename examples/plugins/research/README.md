# Research Plugin

This plugin provides specialized subagents for research tasks: web research, code analysis, and content summarization.

## Agents

### web-researcher

A web research specialist that can search the internet and synthesize findings.

**Tools**: `web_fetch`, `web_search`, `think`

**Use for**:
- Finding information on topics
- Researching technologies or libraries
- Gathering data from multiple sources

**Example**:
```
delegate agent="web-researcher" task="Find the latest best practices for Rust error handling"
```

### code-analyzer

A code analysis specialist that can explore and explain codebases.

**Tools**: `file_read`, `glob`, `grep`, `think`

**Use for**:
- Understanding how a feature works
- Tracing data flow
- Finding where specific logic lives
- Architecture analysis

**Example**:
```
delegate agent="code-analyzer" task="Explain how authentication works in this codebase"
```

### summarizer

A summarization specialist that condenses long content.

**Tools**: `file_read`, `web_fetch`, `think`

**Use for**:
- Summarizing long documents
- Creating executive summaries
- Distilling research findings

**Example**:
```
delegate agent="summarizer" task="Summarize the README and key architecture decisions in this project"
```

## Installation

### Local Development

Copy this plugin to your plugins directory:
```bash
cp -r examples/plugins/research ~/.config/arawn/plugins/
```

### Via Subscription

If published to a git repository:
```bash
arawn plugin add your-username/research-plugin
```

## Customization

### Modifying Agents

Edit the markdown files in `agents/` to customize:
- System prompts
- Tool access
- Model selection
- Max iterations

### Adding New Agents

Create a new markdown file in `agents/`:
```markdown
---
name: my-agent
description: What this agent does
tools: ["tool1", "tool2"]
---

Your system prompt here...
```

## Best Practices

1. **Match agent to task**: Use web-researcher for internet queries, code-analyzer for codebase exploration
2. **Provide context**: Pass relevant context from your session to help the agent understand the situation
3. **Be specific**: Clear, specific tasks get better results than vague requests
4. **Chain agents**: Use one agent's output as input to another for complex workflows
