# Quick Start

Get up and running with Arawn in minutes.

## First Run

### Configure an LLM Backend

Arawn needs access to an LLM. The simplest setup uses environment variables:

```bash
# For Anthropic (Claude)
export ANTHROPIC_API_KEY="sk-ant-..."

# Or for OpenAI
export OPENAI_API_KEY="sk-..."

# Or for Groq (fast inference)
export GROQ_API_KEY="gsk_..."
```

### Interactive Chat

Start a chat session:

```bash
arawn chat
```

This opens an interactive prompt where you can converse with the agent.

### Single Question

For one-off questions:

```bash
arawn ask "What are the main components of a Rust project?"
```

## HTTP Server Mode

Run Arawn as a server for API access:

```bash
# Start the server
arawn start

# Server listens on http://127.0.0.1:8080
```

### Chat via API

```bash
curl -X POST http://localhost:8080/api/v1/chat \
  -H "Content-Type: application/json" \
  -d '{"message": "Hello, Arawn!"}'
```

### Streaming Response

```bash
curl -X POST http://localhost:8080/api/v1/chat/stream \
  -H "Content-Type: application/json" \
  -d '{"message": "Explain async/await in Rust"}'
```

## Key Concepts

### Sessions

Each conversation happens in a **session**. Sessions track:
- Conversation history
- Context state
- Tool executions

### Memory

Arawn can remember facts across sessions. When a session ends, important information is automatically extracted and stored:

- **Entities** — People, projects, concepts
- **Facts** — Specific pieces of information
- **Relationships** — How entities connect

### Tools

The agent has access to tools for:
- File operations (read, write, search)
- Shell commands
- Web fetching and searching
- Notes and memory operations

## Common Commands

```bash
# List available commands
arawn help

# Show configuration
arawn config show

# Search memory
arawn memory search "rust project"

# List sessions
arawn session list

# Start server with custom port
arawn start --port 9000
```

## Next Steps

- [Configuration](configuration.md) — Customize Arawn's behavior
- [Agent Behavior](../reference/behavior.md) — Understand tool usage patterns
- [Memory Guidelines](../reference/memory-guidelines.md) — Best practices for memory
