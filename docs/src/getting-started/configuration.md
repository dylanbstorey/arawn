# Configuration Basics

Arawn uses TOML configuration with cascading resolution.

## Configuration Files

Configuration is loaded from multiple sources in order:

```
1. CLI flags (--config, --verbose, etc.)
   ↓
2. Project config (.arawn/arawn.toml)
   ↓
3. User config (~/.arawn/arawn.toml)
   ↓
4. XDG config ($XDG_CONFIG_HOME/arawn/arawn.toml)
   ↓
5. Built-in defaults
```

## Minimal Configuration

Create `~/.arawn/arawn.toml`:

```toml
[llm]
backend = "anthropic"
model = "claude-sonnet-4-20250514"
```

## Common Settings

### LLM Backend

```toml
[llm]
backend = "groq"           # anthropic, openai, groq, ollama
model = "llama-3.3-70b"    # Model name for the backend
```

### Memory

```toml
[memory]
database = "memory.db"     # SQLite path (relative to data dir)

[memory.indexing]
enabled = true             # Enable session indexing
backend = "default"        # LLM backend for extraction
model = "gpt-4o-mini"      # Model for extraction/summarization
```

### HTTP Server

```toml
[server]
port = 8080
bind = "127.0.0.1"
```

## Secret Management

Arawn supports multiple ways to provide API keys:

| Syntax | Source | Example |
|--------|--------|---------|
| `$keyring:name` | OS Keyring | `$keyring:anthropic_api_key` |
| `$env:VAR` | Environment | `$env:OPENAI_API_KEY` |
| `$file:path` | File contents | `$file:~/.secrets/api_key` |
| Literal | Config value | `sk-ant-...` (not recommended) |

Example using keyring:

```toml
[backends.anthropic]
api_key = "$keyring:anthropic_api_key"
model = "claude-sonnet-4-20250514"
```

## View Current Configuration

```bash
arawn config show
```

## Next Steps

- [Full Configuration Reference](../configuration/reference.md)
- [Secret Management](../configuration/secrets.md)
- [LLM Backends](../configuration/backends.md)
