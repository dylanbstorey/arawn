# Configuration Reference

Complete reference for Arawn configuration options. All config is TOML format,
loaded from `~/.arawn/arawn.toml` by default with cascading overrides.

## LLM Configuration

### Primary Backend

The bare `[llm]` section defines the default LLM for all operations.

```toml
[llm]
backend = "anthropic"          # anthropic, openai, groq, ollama, custom, claude-oauth
model = "claude-sonnet-4-20250514"
base_url = "https://..."       # Optional: custom API base URL
retry_max = 3                  # Max retry attempts for failed requests
retry_backoff_ms = 500         # Backoff delay between retries (ms)
max_context_tokens = 200000    # Max context window size in tokens
```

### Named LLM Profiles

Define multiple profiles for different purposes. Agents reference these by name.

```toml
[llm.claude]
backend = "anthropic"
model = "claude-sonnet-4-20250514"
max_context_tokens = 200000

[llm.fast]
backend = "groq"
model = "llama-3.3-70b-versatile"
max_context_tokens = 32768

[llm.local]
backend = "ollama"
base_url = "http://localhost:11434"
model = "llama3"
```

> **Note:** API keys should not be stored in config files. Use the keyring or
> environment variables instead. See [Secrets](secrets.md).

## Agent Configuration

Per-agent settings. The `default` key applies to all agents unless overridden.

```toml
[agent.default]
max_iterations = 25            # Max tool loop iterations
llm = "claude"                 # LLM profile name (references [llm.claude])
system_prompt = "..."          # Optional system prompt override

[agent.summarizer]
max_iterations = 10
llm = "fast"                   # Use the fast profile for summarization
```

## Memory Configuration

```toml
[memory]
database = "memory.db"         # SQLite path (relative to data dir)

[memory.recall]
enabled = true                 # Enable active recall
limit = 5                      # Max memories to recall per turn
threshold = 0.6                # Min similarity score (0.0–1.0)

[memory.indexing]
enabled = true                 # Enable session indexing pipeline
backend = "openai"             # LLM backend for extraction (default: "openai")
model = "gpt-4o-mini"          # Model for extraction/summarization
ner_model_path = "..."         # Optional: GLiNER ONNX model for local NER
ner_tokenizer_path = "..."     # Optional: GLiNER tokenizer JSON
ner_threshold = 0.5            # NER confidence threshold (0.0–1.0)

[memory.confidence]
fresh_days = 30.0              # Days before staleness decay begins
staleness_days = 365.0         # Days at which staleness reaches floor
staleness_floor = 0.3          # Minimum staleness multiplier
reinforcement_cap = 1.5        # Maximum reinforcement multiplier
```

## Server Configuration

```toml
[server]
port = 8080                    # HTTP port
bind = "127.0.0.1"             # Bind address
rate_limiting = true           # Enable per-IP rate limiting
api_rpm = 120                  # API requests per minute per IP
request_logging = true         # Enable request logging
bootstrap_dir = "..."          # Optional: path to bootstrap files
workspace = "..."              # Optional: working directory
```

Authentication is configured via the `ARAWN_AUTH_TOKEN` environment variable
or keyring — not in the config file. See [Secrets](secrets.md).

### Runtime Server Settings

These settings are applied at the `arawn-server` runtime layer (not in the TOML
config file, but can be set programmatically or via environment):

| Setting | Default | Description |
|---------|---------|-------------|
| `cors_origins` | *(empty)* | CORS allowed origins |
| `tailscale_users` | *(none)* | Allowed Tailscale users |
| `reconnect_grace_period` | 30s | Grace period for WebSocket session reconnect |
| `max_ws_message_size` | 1 MB | Max WebSocket message size |
| `max_body_size` | 10 MB | Max REST request body size |
| `ws_allowed_origins` | *(empty — all allowed)* | Allowed WebSocket origins |
| `ws_connections_per_minute` | 30 | Max WS connections per minute per IP |

## Session Cache Configuration

```toml
[session]
max_sessions = 10000           # Max sessions in cache before LRU eviction
cleanup_interval_secs = 60     # Seconds between cleanup runs
```

## Tool Configuration

```toml
[tools.output]
max_size_bytes = 102400        # Max tool output size (100KB)

[tools.shell]
timeout_secs = 30              # Shell command timeout

[tools.web]
timeout_secs = 30              # Web request timeout
```

## Workstream Configuration

```toml
[workstream]
database = "workstreams.db"        # SQLite path (relative to data dir)
data_dir = "workstreams"           # JSONL message history directory
session_timeout_minutes = 60       # Session timeout in minutes
```

## Delegation Configuration

Controls subagent delegation and result compaction.

```toml
[delegation]
max_result_len = 8000          # Max subagent result length before compaction

[delegation.compaction]
enabled = false                # Enable LLM-based result compaction
threshold = 8000               # Min length to trigger compaction (chars)
backend = "default"            # LLM profile for compaction
model = "gpt-4o-mini"          # Optional model override
target_len = 4000              # Target length for compacted output
```

## MCP Configuration

```toml
[mcp]
enabled = true

# Stdio transport (spawns a child process)
[[mcp.servers]]
name = "sqlite"
transport = "stdio"            # Default transport
command = "mcp-server-sqlite"
args = ["--db", "data.db"]
env = [["DEBUG", "1"]]
enabled = true

# HTTP transport (connects to a remote server)
[[mcp.servers]]
name = "remote"
transport = "http"
url = "http://localhost:3000/mcp"
headers = [["Authorization", "Bearer token"]]
timeout_secs = 30
retries = 3
```

## Embedding Configuration

```toml
[embedding]
provider = "local"             # "local" (ONNX), "openai", or "mock"
dimensions = 384               # Output dimensions (default depends on provider)

[embedding.openai]
model = "text-embedding-3-small"
dimensions = 1536              # Optional dimension override
base_url = "..."               # Optional custom endpoint

[embedding.local]
model_path = "..."             # Optional: custom ONNX model path
tokenizer_path = "..."         # Optional: custom tokenizer path
```

## Pipeline Configuration

```toml
[pipeline]
enabled = true
database = "pipeline.db"       # SQLite path for pipeline state
workflow_dir = "workflows"     # Directory containing workflow TOML definitions
max_concurrent_tasks = 4       # Max concurrent task executions
task_timeout_secs = 300        # Per-task timeout
pipeline_timeout_secs = 600    # Per-pipeline timeout
cron_enabled = true            # Enable cron-based scheduling
triggers_enabled = true        # Enable event-based triggers
```

## Plugin Configuration

```toml
[plugins]
enabled = true
dirs = ["~/.config/arawn/plugins", "./plugins"]
hot_reload = true              # Enable file-watching hot reload
auto_update = true             # Auto-update subscribed plugins on startup

# Plugin subscriptions
[[plugins.subscriptions]]
source = "github"
repo = "author/plugin-name"
ref = "main"
enabled = true

[[plugins.subscriptions]]
source = "local"
path = "/path/to/local/plugin"
```

## Logging Configuration

```toml
[logging.interactions]
enabled = true                 # Enable structured interaction logging
path = "~/.arawn/logs"         # Directory for JSONL log files
retention_days = 90            # Days to retain log files
```

## Path Configuration

Controls workstream data paths, disk usage thresholds, cleanup, and filesystem monitoring.

```toml
[paths]
base_path = "~/.arawn"         # Base path for all data (default: ~/.arawn)

[paths.usage]
total_warning_gb = 10          # Total usage warning threshold
workstream_warning_gb = 1      # Per-workstream warning threshold
session_warning_mb = 200       # Per-session warning threshold

[paths.cleanup]
scratch_cleanup_days = 7       # Days before inactive scratch cleanup
dry_run = false                # Log cleanup actions without deleting

[paths.monitoring]
enabled = true                 # Enable filesystem monitoring
debounce_ms = 500              # Event debounce interval
polling_interval_secs = 30     # Polling fallback interval
```

## Environment Variables

| Variable | Purpose |
|----------|---------|
| `ARAWN_CONFIG` | Config file path |
| `ARAWN_BASE_PATH` | Override base data path |
| `ARAWN_MONITORING_ENABLED` | Enable/disable filesystem monitoring (`true`/`false`) |
| `ANTHROPIC_API_KEY` | Anthropic API key |
| `OPENAI_API_KEY` | OpenAI API key |
| `GROQ_API_KEY` | Groq API key |
| `OLLAMA_API_KEY` | Ollama API key |

## CLI Overrides

```bash
# Override config file
arawn --config /path/to/config.toml

# Override port
arawn start --port 9000

# Override log level
arawn --verbose chat
```
