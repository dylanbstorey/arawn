# Configuration Reference

Complete reference for Arawn configuration options. All config is TOML format,
loaded with cascading resolution (see [Configuration Overview](README.md)).

---

## LLM Configuration

### Primary Backend

The bare `[llm]` section defines the default LLM for all operations.

```toml
[llm]
backend = "anthropic"          # anthropic, openai, groq, ollama, custom, claude-oauth
model = "claude-sonnet-4-20250514"
base_url = "https://..."       # Optional: custom API base URL (for proxies)
retry_max = 3                  # Max retry attempts for failed requests
retry_backoff_ms = 500         # Backoff delay between retries (ms)
max_context_tokens = 200000    # Max context window size in tokens
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `backend` | string | *(required)* | LLM provider: `anthropic`, `openai`, `groq`, `ollama`, `custom`, `claude-oauth` |
| `model` | string | *(required)* | Model identifier (e.g., `claude-sonnet-4-20250514`) |
| `base_url` | string | *(provider default)* | Custom API base URL for proxies or self-hosted |
| `api_key` | string | *(not recommended)* | API key — use keyring or env vars instead |
| `retry_max` | u32 | — | Max retry attempts for transient failures |
| `retry_backoff_ms` | u64 | — | Millisecond delay between retries |
| `max_context_tokens` | usize | — | Max context window size in tokens |

> **Warning:** Setting `api_key` in the config file is insecure. Use `arawn config set-secret`
> or environment variables instead. See [Secret Management](secrets.md).

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

Each named profile accepts the same fields as the primary `[llm]` section.

---

## Agent Configuration

Per-agent settings. The `default` key applies to all agents unless overridden by
a named agent section.

```toml
[agent.default]
llm = "claude"                 # LLM profile name (references [llm.claude])
max_iterations = 25            # Max tool loop iterations
max_tokens = 8192              # Max tokens per LLM response
system_prompt = "..."          # Optional system prompt override

[agent.summarizer]
llm = "fast"                   # Use the fast profile for summarization
max_iterations = 10
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `llm` | string | — | LLM profile name (references `[llm.<name>]`) |
| `system_prompt` | string | — | System prompt override |
| `max_iterations` | u32 | — | Max tool loop iterations |
| `max_tokens` | u32 | — | Max tokens per LLM response |

---

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
ws_allowed_origins = ["..."]   # WebSocket allowed origins (empty = all)
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `port` | u16 | `8080` | HTTP listen port |
| `bind` | string | `"127.0.0.1"` | Bind address |
| `rate_limiting` | bool | `true` | Enable per-IP rate limiting |
| `api_rpm` | u32 | `120` | Requests per minute per IP |
| `request_logging` | bool | `true` | Enable HTTP request logging |
| `bootstrap_dir` | path | — | Bootstrap files directory |
| `workspace` | path | — | Server working directory |
| `ws_allowed_origins` | string[] | `[]` | WebSocket allowed origins (empty = allow all) |

Authentication is configured via `ARAWN_AUTH_TOKEN` or the keyring — not in the
config file. See [Secret Management](secrets.md).

### Runtime Server Settings

These settings are set programmatically or via environment (not in the TOML file):

| Setting | Default | Description |
|---------|---------|-------------|
| `cors_origins` | *(empty)* | CORS allowed origins |
| `tailscale_users` | *(none)* | Allowed Tailscale users |
| `reconnect_grace_period` | 30s | Grace period for WebSocket session reconnect |
| `max_ws_message_size` | 1 MB | Max WebSocket message size |
| `max_body_size` | 10 MB | Max REST request body size |
| `ws_connections_per_minute` | 30 | Max WS connections per minute per IP |

---

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
backend = "openai"             # LLM backend for extraction
model = "gpt-4o-mini"          # Model for extraction/summarization
ner_model_path = "..."         # Optional: GLiNER ONNX model for local NER
ner_tokenizer_path = "..."     # Optional: GLiNER tokenizer JSON
ner_threshold = 0.5            # NER confidence threshold (0.0–1.0)
ner_model_url = "..."          # Download URL for GLiNER model (auto-fetched)
ner_tokenizer_url = "..."      # Download URL for GLiNER tokenizer (auto-fetched)

[memory.confidence]
fresh_days = 30.0              # Days before staleness decay begins
staleness_days = 365.0         # Days at which staleness reaches floor
staleness_floor = 0.3          # Minimum staleness multiplier
reinforcement_cap = 1.5        # Maximum reinforcement multiplier
```

| Section | Field | Type | Default | Description |
|---------|-------|------|---------|-------------|
| `memory` | `database` | path | — | SQLite database path |
| `recall` | `enabled` | bool | `true` | Enable active recall |
| `recall` | `limit` | usize | `5` | Max memories per turn |
| `recall` | `threshold` | f32 | `0.6` | Min similarity score (0.0–1.0) |
| `indexing` | `enabled` | bool | `true` | Enable session indexing |
| `indexing` | `backend` | string | `"openai"` | LLM backend for extraction |
| `indexing` | `model` | string | `"gpt-4o-mini"` | Extraction model |
| `indexing` | `ner_model_path` | path | — | GLiNER ONNX model path |
| `indexing` | `ner_tokenizer_path` | path | — | GLiNER tokenizer path |
| `indexing` | `ner_threshold` | f32 | `0.5` | NER confidence threshold |
| `indexing` | `ner_model_url` | string | — | Auto-download URL for NER model |
| `indexing` | `ner_tokenizer_url` | string | — | Auto-download URL for NER tokenizer |
| `confidence` | `fresh_days` | f32 | `30.0` | Days before staleness starts |
| `confidence` | `staleness_days` | f32 | `365.0` | Days to reach staleness floor |
| `confidence` | `staleness_floor` | f32 | `0.3` | Min staleness multiplier |
| `confidence` | `reinforcement_cap` | f32 | `1.5` | Max reinforcement multiplier |

---

## Embedding Configuration

```toml
[embedding]
provider = "local"             # "local" (ONNX), "openai", or "mock"
dimensions = 384               # Output dimensions (default depends on provider)

[embedding.openai]
model = "text-embedding-3-small"
dimensions = 1536              # Optional dimension override
base_url = "..."               # Optional custom endpoint
api_key = "..."                # Optional (prefer env var OPENAI_API_KEY)

[embedding.local]
model_path = "..."             # Custom ONNX model path
tokenizer_path = "..."         # Custom tokenizer path
model_url = "..."              # Auto-download URL for ONNX model
tokenizer_url = "..."          # Auto-download URL for tokenizer
```

| Section | Field | Type | Default | Description |
|---------|-------|------|---------|-------------|
| `embedding` | `provider` | string | `"local"` | `local`, `openai`, or `mock` |
| `embedding` | `dimensions` | usize | *(provider default)* | Output embedding dimensions |
| `openai` | `model` | string | `"text-embedding-3-small"` | OpenAI model name |
| `openai` | `dimensions` | usize | — | Override embedding dimensions |
| `openai` | `base_url` | string | — | Custom endpoint URL |
| `openai` | `api_key` | string | — | API key (prefer `OPENAI_API_KEY` env) |
| `local` | `model_path` | path | — | Custom ONNX model path |
| `local` | `tokenizer_path` | path | — | Custom tokenizer.json path |
| `local` | `model_url` | string | — | URL to auto-download ONNX model |
| `local` | `tokenizer_url` | string | — | URL to auto-download tokenizer |

---

## Tool Configuration

```toml
[tools.output]
max_size_bytes = 102400        # Default max tool output size (100KB)
shell = 102400                 # Override for shell tool (default: 100KB)
file_read = 512000             # Override for file_read tool (default: 500KB)
web_fetch = 204800             # Override for web_fetch tool (default: 200KB)
search = 51200                 # Override for search/grep/glob (default: 50KB)

[tools.shell]
timeout_secs = 30              # Shell command timeout

[tools.web]
timeout_secs = 30              # Web request timeout
```

| Section | Field | Type | Default | Description |
|---------|-------|------|---------|-------------|
| `output` | `max_size_bytes` | usize | `102400` | Default max output size (bytes) |
| `output` | `shell` | usize | `102400` | Max output for shell tool |
| `output` | `file_read` | usize | `512000` | Max output for file_read tool |
| `output` | `web_fetch` | usize | `204800` | Max output for web_fetch tool |
| `output` | `search` | usize | `51200` | Max output for search/grep/glob |
| `shell` | `timeout_secs` | u64 | `30` | Shell command timeout (seconds) |
| `web` | `timeout_secs` | u64 | `30` | Web request timeout (seconds) |

---

## Workstream Configuration

```toml
[workstream]
database = "workstreams.db"        # SQLite path (relative to data dir)
data_dir = "workstreams"           # JSONL message history directory
session_timeout_minutes = 60       # Session timeout in minutes

[workstream.compression]
enabled = false                    # Enable LLM-based session compression
backend = "default"                # LLM profile for summarization
model = "claude-sonnet"            # Model for compression
max_summary_tokens = 1024          # Max tokens in generated summary
token_threshold_chars = 32000      # Char threshold to trigger compression (~8k tokens)
```

| Section | Field | Type | Default | Description |
|---------|-------|------|---------|-------------|
| `workstream` | `database` | path | — | SQLite database path |
| `workstream` | `data_dir` | path | — | JSONL message file directory |
| `workstream` | `session_timeout_minutes` | i64 | `60` | Session timeout |
| `compression` | `enabled` | bool | `false` | Enable auto-compression |
| `compression` | `backend` | string | `"default"` | LLM profile for compression |
| `compression` | `model` | string | `"claude-sonnet"` | Summarization model |
| `compression` | `max_summary_tokens` | u32 | `1024` | Max summary tokens |
| `compression` | `token_threshold_chars` | usize | `32000` | Char threshold to trigger |

---

## Session Cache Configuration

```toml
[session]
max_sessions = 10000           # Max sessions in cache before LRU eviction
cleanup_interval_secs = 60     # Seconds between cleanup runs
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `max_sessions` | usize | `10000` | LRU eviction threshold |
| `cleanup_interval_secs` | u64 | `60` | Cache cleanup interval |

---

## Delegation Configuration

Controls subagent delegation and result compaction.

```toml
[delegation]
max_result_len = 8000          # Max subagent result length before compaction

[delegation.compaction]
enabled = false                # Enable LLM-based result compaction
threshold = 8000               # Min length to trigger compaction (chars)
backend = "default"            # LLM profile for compaction
model = "gpt-4o-mini"          # Model for compaction
target_len = 4000              # Target length for compacted output
```

| Section | Field | Type | Default | Description |
|---------|-------|------|---------|-------------|
| `delegation` | `max_result_len` | usize | `8000` | Max result length before compaction |
| `compaction` | `enabled` | bool | `false` | Enable LLM-based compaction |
| `compaction` | `threshold` | usize | `8000` | Min length to trigger |
| `compaction` | `backend` | string | `"default"` | LLM profile name |
| `compaction` | `model` | string | `"gpt-4o-mini"` | Compaction model |
| `compaction` | `target_len` | usize | `4000` | Target compacted length |

---

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

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | bool | `true` | Enable MCP globally |
| **Per server:** | | | |
| `name` | string | *(required)* | Unique server name |
| `transport` | string | `"stdio"` | `stdio` or `http` |
| `command` | string | — | Command to spawn (stdio) |
| `url` | string | — | Server URL (http) |
| `args` | string[] | `[]` | Command arguments (stdio) |
| `env` | [key, value][] | `[]` | Environment variables |
| `headers` | [key, value][] | `[]` | HTTP headers (http) |
| `timeout_secs` | u64 | `30` | Request timeout (http) |
| `retries` | u32 | `3` | Retry count (http) |
| `enabled` | bool | `true` | Enable this server |

---

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

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | bool | `true` | Enable pipeline engine |
| `database` | path | — | SQLite state database |
| `workflow_dir` | path | — | Workflow TOML definitions directory |
| `max_concurrent_tasks` | usize | `4` | Concurrent task limit |
| `task_timeout_secs` | u64 | `300` | Per-task timeout |
| `pipeline_timeout_secs` | u64 | `600` | Per-pipeline timeout |
| `cron_enabled` | bool | `true` | Enable cron scheduling |
| `triggers_enabled` | bool | `true` | Enable event triggers |

---

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

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | bool | `true` | Enable plugin system |
| `dirs` | path[] | `[]` | Additional plugin directories |
| `hot_reload` | bool | `true` | File-watching hot reload |
| `auto_update` | bool | `true` | Auto-update on startup |
| **Per subscription:** | | | |
| `source` | string | — | `github`, `url`, or `local` |
| `repo` | string | — | GitHub `owner/repo` (github source) |
| `url` | string | — | Git clone URL (url source) |
| `path` | path | — | Local filesystem path (local source) |
| `ref` | string | `"main"` | Git ref (branch/tag/commit) |
| `enabled` | bool | `true` | Enable this subscription |

---

## RLM Configuration

Controls the Recursive Learning Machine (exploration/research agent).

```toml
[rlm]
model = "claude-sonnet-4-20250514"  # Exploration model (inherits from backend if absent)
max_turns = 50                      # Max agent turns before stopping
max_context_tokens = 150000         # Max context tokens before compaction
compaction_threshold = 0.8          # Fraction of max_context_tokens to trigger (0.0–1.0)
max_compactions = 5                 # Max compaction cycles before stopping
max_total_tokens = 500000           # Cumulative token budget for entire exploration
compaction_model = "gpt-4o-mini"    # Cheaper model for compaction summaries
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `model` | string | *(inherit from backend)* | Exploration model |
| `max_turns` | u32 | — | Max agent turns (safety valve) |
| `max_context_tokens` | usize | — | Max context before compaction |
| `compaction_threshold` | f32 | — | Fraction of max to trigger compaction (0.0–1.0) |
| `max_compactions` | u32 | — | Max compaction cycles |
| `max_total_tokens` | usize | — | Total token budget |
| `compaction_model` | string | — | Separate cheaper model for compaction |

---

## Logging Configuration

```toml
[logging.interactions]
enabled = true                 # Enable structured interaction logging
path = "~/.arawn/logs"         # Directory for JSONL log files
retention_days = 90            # Days to retain log files
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | bool | `true` | Enable interaction logging |
| `path` | path | — | JSONL log directory |
| `retention_days` | u32 | `90` | Log retention period |

---

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

| Section | Field | Type | Default | Description |
|---------|-------|------|---------|-------------|
| `paths` | `base_path` | path | `~/.arawn` | Base data directory |
| `usage` | `total_warning_gb` | u64 | `10` | Total disk usage warning |
| `usage` | `workstream_warning_gb` | u64 | `1` | Per-workstream warning |
| `usage` | `session_warning_mb` | u64 | `200` | Per-session warning |
| `cleanup` | `scratch_cleanup_days` | u32 | `7` | Scratch cleanup threshold |
| `cleanup` | `dry_run` | bool | `false` | Dry-run mode |
| `monitoring` | `enabled` | bool | `true` | Enable filesystem monitoring |
| `monitoring` | `debounce_ms` | u64 | `500` | Event debounce interval |
| `monitoring` | `polling_interval_secs` | u64 | `30` | Polling fallback interval |

---

## OAuth Configuration

Overrides for the OAuth PKCE flow used by the `claude-oauth` backend. All fields
are optional — unset fields fall through to environment variables
(`ARAWN_OAUTH_*`) and then built-in defaults.

```toml
[oauth]
client_id = "9d1c250a-e61b-44d9-88ed-5944d1962f5e"
authorize_url = "https://claude.ai/oauth/authorize"
token_url = "https://console.anthropic.com/v1/oauth/token"
redirect_uri = "https://console.anthropic.com/oauth/code/callback"
scope = "org:create_api_key user:profile user:inference"
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `client_id` | string | *(built-in)* | OAuth client ID |
| `authorize_url` | string | *(built-in)* | Authorization endpoint URL |
| `token_url` | string | *(built-in)* | Token exchange endpoint URL |
| `redirect_uri` | string | *(built-in)* | OAuth redirect URI |
| `scope` | string | *(built-in)* | OAuth scopes (space-separated) |

Resolution order: `[oauth]` config → `ARAWN_OAUTH_*` env vars → built-in defaults.

---

## Environment Variables

| Variable | Purpose |
|----------|---------|
| `ARAWN_CONFIG` | Config file path override |
| `ARAWN_SERVER_URL` | Server URL for CLI commands |
| `ARAWN_BASE_PATH` | Override base data path |
| `ARAWN_MONITORING_ENABLED` | Enable/disable filesystem monitoring (`true`/`false`) |
| `ARAWN_AUTH_TOKEN` | Server authentication token |
| `ANTHROPIC_API_KEY` | Anthropic API key |
| `OPENAI_API_KEY` | OpenAI API key |
| `GROQ_API_KEY` | Groq API key |
| `OLLAMA_API_KEY` | Ollama API key |
| `ARAWN_OAUTH_CLIENT_ID` | OAuth client ID override |
| `ARAWN_OAUTH_AUTHORIZE_URL` | OAuth authorization endpoint override |
| `ARAWN_OAUTH_TOKEN_URL` | OAuth token endpoint override |
| `ARAWN_OAUTH_REDIRECT_URI` | OAuth redirect URI override |
| `ARAWN_OAUTH_SCOPE` | OAuth scopes override |

## CLI Overrides

```bash
# Override config file
arawn --config /path/to/config.toml

# Override server URL
arawn --server http://remote:8080 status

# Override port at startup
arawn start --port 9000

# Enable verbose logging
arawn --verbose chat
```
