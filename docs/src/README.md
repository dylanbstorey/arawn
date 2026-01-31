# Arawn Technical Reference

**Personal Research Agent for Edge Computing — written in Rust.**

Arawn is an agentic platform that uses LLMs with tool-calling to perform research tasks, retaining knowledge across sessions via a persistent memory system.

## What is Arawn?

Arawn provides:

- **Agentic Tool Loop** — LLM-driven reasoning with automatic tool execution
- **Persistent Memory** — Vector + graph storage for facts and relationships
- **Session Indexing** — Automatic extraction of entities, facts, and summaries
- **Plugin System** — Claude Code-compatible plugins for extensibility
- **MCP Integration** — Bridge external tool servers into the agent loop
- **Edge Deployment** — Single binary with embedded SQLite storage

## Who is this for?

- **Developers** building personal research assistants
- **Users** wanting a local-first AI agent with memory
- **Contributors** extending Arawn's capabilities

## Quick Links

- [Installation](getting-started/installation.md) — Build and run Arawn
- [Quick Start](getting-started/quickstart.md) — Your first conversation
- [Architecture](architecture/README.md) — System design and components
- [Configuration](configuration/reference.md) — Full configuration reference

## Design Philosophy

1. **Edge-First** — Runs on modest hardware, no cloud dependencies required
2. **Memory-Centric** — Persistent context across sessions, not just chat history
3. **Tool-Native** — LLM as orchestrator, tools as capabilities
4. **Extensible** — Plugins, hooks, and subagents for customization

## Getting Help

- Check the [Architecture](architecture/README.md) for system understanding
- See [Agent Behavior](reference/behavior.md) for operational guidelines
- Review [Configuration](configuration/reference.md) for setup options
