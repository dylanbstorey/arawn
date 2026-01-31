---
id: core-configuration-infrastructure
level: initiative
title: "Core Configuration Infrastructure: Layered Config, Named LLM Providers, Secrets Management"
short_code: "ARAWN-I-0009"
created_at: 2026-01-28T16:45:38.095500+00:00
updated_at: 2026-01-28T17:39:42.646267+00:00
parent: ARAWN-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
strategy_id: NULL
initiative_id: core-configuration-infrastructure
---

# Core Configuration Infrastructure

Architectural decisions captured in **ARAWN-A-0001**.

## Context

Currently Arawn relies on CLI arguments and environment variables for configuration (API keys, backend selection, model choice). This is cumbersome for regular use and doesn't support the multi-model architecture we want.

We need a file-based configuration system that:
- Reduces reliance on CLI flags and env vars
- Supports named LLM configurations with cascading defaults
- Securely handles API keys via system keyring
- Follows XDG conventions with project-local overrides

## Goals & Non-Goals

**Goals:**
- TOML-based configuration files
- Named LLM configurations (`[llm]`, `[llm.claude]`, `[llm.fast]`, etc.)
- Cascading resolution: agent-specific → agent.default → global default
- System keyring integration for API keys (macOS Keychain, Linux secret-service)
- XDG config location (`~/.config/arawn/config.toml`) with `./arawn.toml` override
- Warn (don't block) if secrets are stored in plaintext

**Non-Goals:**
- Complex encryption schemes (age, SOPS, GPG) - keyring is sufficient for v1
- Remote config fetching
- Config file encryption

## Detailed Design

### Config File Format

```toml
# ~/.config/arawn/config.toml

# Default LLM - used when nothing else specified
[llm]
backend = "groq"
model = "llama-3.1-70b-versatile"

# Named configurations
[llm.claude]
backend = "anthropic"
model = "claude-sonnet-4-20250514"

[llm.fast]
backend = "groq"
model = "llama-3.1-8b-instant"

[llm.local]
backend = "ollama"
model = "llama3.2"
base_url = "http://localhost:11434/v1"

# Agent defaults
[agent.default]
llm = "claude"  # all agents use claude unless overridden

[agent.summarizer]
llm = "fast"    # this agent uses the fast config

# Server settings
[server]
port = 8080
bind = "127.0.0.1"
```

### LLM Resolution Order

1. **Agent-specific**: `agent.<name>.llm` → lookup in `llm.<value>`
2. **Agent default**: `agent.default.llm` → lookup in `llm.<value>`
3. **Global default**: `[llm]` section directly

### API Key Resolution

1. System keyring (preferred) - stored as `arawn/<backend>_api_key`
2. Environment variable fallback (`ANTHROPIC_API_KEY`, `GROQ_API_KEY`, etc.)
3. Config file (with warning) - `api_key` field in LLM config

### Config File Resolution

1. `./arawn.toml` (project-local, highest priority)
2. `~/.config/arawn/config.toml` (XDG user config)
3. CLI args override any config file values

### CLI Integration

```bash
# Set a secret in keyring
arawn config set-secret anthropic
# Prompts for API key, stores in system keyring

# Show resolved config
arawn config show

# Show where config is loaded from
arawn config which
```

## Alternatives Considered

**Environment variables only**: Current approach. Rejected because it's cumbersome and doesn't support named configs.

**YAML format**: Considered but TOML is more idiomatic for Rust projects and has cleaner syntax for this use case.

**Encrypted config files (age/SOPS)**: Overkill for single-user CLI tool. System keyring provides adequate security with better UX.

**Single model config**: Rejected in favor of named configs to support multi-model routing and per-agent customization.

## Implementation Plan

1. Create `arawn-config` crate with TOML parsing and config structs
2. Implement config file discovery (XDG + project-local)
3. Add keyring integration for API key storage
4. Update CLI to load config and merge with CLI args
5. Wire named LLM configs into agent builder
6. Add `arawn config` subcommands (show, set-secret, which)