# Configuration Overview

Arawn uses TOML configuration with cascading resolution and secure secret management.

## Configuration Sources

Configuration is loaded in order (later overrides earlier):

```
1. Built-in defaults
   ↓
2. XDG config ($XDG_CONFIG_HOME/arawn/arawn.toml)
   ↓
3. User config (~/.arawn/arawn.toml)
   ↓
4. Project config (.arawn/arawn.toml)
   ↓
5. CLI flags (--config, --verbose, etc.)
```

## Quick Start

Create `~/.arawn/arawn.toml`:

```toml
[llm]
backend = "anthropic"
model = "claude-sonnet-4-20250514"

[backends.anthropic]
api_key = "$env:ANTHROPIC_API_KEY"
```

## Section Contents

- [Reference](reference.md) — Full configuration options
- [Secret Management](secrets.md) — API keys and credentials
- [LLM Backends](backends.md) — Provider configuration
