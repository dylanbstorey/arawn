# Configuration Reference

Complete reference for Arawn configuration options.

## LLM Configuration

### Primary Backend

```toml
[llm]
backend = "anthropic"      # anthropic, openai, groq, ollama
model = "claude-sonnet-4-20250514"    # Model name for the backend
retry_max = 3              # Max retry attempts for failed requests
retry_backoff_ms = 500     # Initial backoff between retries (ms)
```

### Multiple Backends

Define multiple backends for different purposes:

```toml
[backends.anthropic]
api_key = "$keyring:anthropic_api_key"
model = "claude-sonnet-4-20250514"

[backends.groq]
api_key = "$env:GROQ_API_KEY"
model = "llama-3.3-70b-versatile"

[backends.ollama]
base_url = "http://localhost:11434"
model = "llama3"
```

## Memory Configuration

```toml
[memory]
database = "memory.db"     # SQLite path (relative to data dir)

[memory.indexing]
enabled = true             # Enable session indexing
backend = "default"        # LLM backend for extraction
model = "gpt-4o-mini"      # Model for extraction/summarization

[memory.recall]
limit = 5                  # Max memories to recall
threshold = 0.6            # Min confidence score
include_graph = true       # Use graph expansion
```

## Server Configuration

```toml
[server]
port = 8080                # HTTP port
bind = "127.0.0.1"         # Bind address
rate_limiting = true       # Enable per-IP rate limiting
api_rpm = 120              # Requests per minute per IP
request_logging = true     # Enable request logging
```

### Authentication

```toml
[server.auth]
enabled = true
token = "$env:ARAWN_AUTH_TOKEN"
```

### Rate Limiting

```toml
[server.rate_limit]
requests_per_minute = 60
burst = 10
```

## MCP Configuration

```toml
[mcp]
enabled = true

[mcp.servers.sqlite]
command = "sqlite-mcp"
args = ["--db", "data.db"]

[mcp.servers.remote]
url = "http://localhost:3000/mcp"
headers = { "Authorization" = "Bearer $env:MCP_TOKEN" }
```

## Agent Configuration

```toml
[agent.default]
max_iterations = 25        # Max tool loop iterations (default for all agents)
llm = "claude"             # LLM profile to use

[agent.summarizer]         # Named agent override
max_iterations = 10
llm = "fast"

[agent.recall]
enabled = true
limit = 5
threshold = 0.6
```

## Session Cache Configuration

```toml
[session]
max_sessions = 10000           # Max sessions in cache before LRU eviction
cleanup_interval_secs = 60     # Interval between cleanup runs
```

## Tool Configuration

```toml
[tools.output]
max_size_bytes = 102400    # Max tool output size (100KB default)

[tools.shell]
timeout_secs = 30          # Shell command timeout

[tools.web]
timeout_secs = 30          # Web request timeout
```

## Workstream Configuration

```toml
[workstreams]
enabled = true
storage_path = "~/.arawn/data/workstreams"
max_history = 1000         # Max messages before summarization
context_window = 50        # Recent messages to keep verbatim
```

## Logging Configuration

```toml
[logging]
level = "info"             # trace, debug, info, warn, error
file = "~/.arawn/arawn.log"
max_size = "10MB"
max_files = 5
```

## Plugin Configuration

```toml
[plugins]
directories = [
  "~/.arawn/plugins",
  ".arawn/plugins"
]
```

## Full Example

```toml
# Primary LLM
[llm]
backend = "groq"
model = "llama-3.3-70b"

# Backend definitions
[backends.anthropic]
api_key = "$keyring:anthropic_api_key"
model = "claude-sonnet-4-20250514"

[backends.groq]
api_key = "$env:GROQ_API_KEY"
model = "llama-3.3-70b-versatile"

[backends.ollama]
base_url = "http://localhost:11434"
model = "llama3"

# Memory system
[memory]
database = "memory.db"

[memory.indexing]
enabled = true
backend = "anthropic"
model = "claude-haiku-3"

[memory.recall]
limit = 5
threshold = 0.6

# HTTP server
[server]
port = 8080
bind = "127.0.0.1"

[server.auth]
enabled = false

# MCP servers
[mcp]
enabled = true

[mcp.servers.sqlite]
command = "sqlite-mcp"
args = ["--db", "~/.arawn/data/memory.db"]

# Agent settings
[agent]
max_iterations = 25
tool_timeout = 30

# Workstreams
[workstreams]
enabled = true
max_history = 1000

# Logging
[logging]
level = "info"
file = "~/.arawn/arawn.log"
```

## Environment Variables

Some settings can be overridden via environment:

| Variable | Purpose |
|----------|---------|
| `ARAWN_CONFIG` | Config file path |
| `ARAWN_LOG_LEVEL` | Logging level |
| `ARAWN_PORT` | Server port |
| `ARAWN_DATA_DIR` | Data directory |

## CLI Overrides

```bash
# Override config file
arawn --config /path/to/config.toml

# Override port
arawn start --port 9000

# Override log level
arawn --verbose chat

# Override backend
arawn --backend anthropic chat
```
