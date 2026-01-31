---
id: 001-configuration-architecture-layered
level: adr
title: "Configuration Architecture: Layered Config with Named LLM Providers"
number: 1
short_code: "ARAWN-A-0001"
created_at: 2026-01-28T16:57:00.546229+00:00
updated_at: 2026-01-31T02:29:51.378879+00:00
decision_date: 
decision_maker: 
parent: 
archived: false

tags:
  - "#adr"
  - "#phase/decided"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# ADR-1: Configuration Architecture: Layered Config with Named LLM Providers

## Context

Arawn currently hardcodes backend selection in the CLI (`start.rs`), requiring CLI flags and environment variables for all configuration. This creates several problems:

- **No multi-model support**: A single backend is selected per server instance. Different agents or tasks can't use different LLMs.
- **Repetitive CLI usage**: Users must pass `--backend groq --api-key ... --model ...` every time.
- **No secret management**: API keys live in env vars or shell history.
- **No project-level config**: Every project uses the same settings.

As Arawn grows to support multiple agents with different capabilities, the configuration system becomes foundational infrastructure that everything else depends on.

## Decision

Adopt a **layered TOML configuration system** with these five pillars:

### 1. Named LLM Configurations

Define multiple LLM configs by name. A bare `[llm]` section serves as the global default.

```toml
[llm]
backend = "groq"
model = "llama-3.1-70b-versatile"

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
```

### 2. Cascading Agent-to-LLM Binding

Agents reference named configs. Resolution order:

1. `agent.<name>.llm` — agent-specific override
2. `agent.default.llm` — default for all agents
3. `[llm]` — global fallback

```toml
[agent.default]
llm = "claude"

[agent.summarizer]
llm = "fast"
```

### 3. Config File Layering

Files are discovered and merged with later sources overriding earlier:

1. `~/.config/arawn/config.toml` (XDG user config, base layer)
2. `./arawn.toml` (project-local overrides)
3. CLI arguments (highest priority, override everything)

### 4. Secrets via System Keyring

API keys are stored in the OS keyring (macOS Keychain, Linux secret-service, Windows Credential Manager). Resolution:

1. System keyring — `arawn/<backend>_api_key`
2. Environment variable — `ANTHROPIC_API_KEY`, `GROQ_API_KEY`, etc.
3. Config file — allowed with a warning

### 5. TOML Format

All config files use TOML.

## Alternatives Analysis

| Option | Pros | Cons | Risk Level |
|--------|------|------|------------|
| **Named LLM configs (chosen)** | Multi-model, reusable, clean | More complex config parsing | Low |
| **Single backend per instance** | Simple | Can't route agents to different models | Medium |
| **Per-request model selection** | Maximum flexibility | Config sprawl, hard to reason about | Medium |
| | | | |
| **TOML (chosen)** | Rust-idiomatic, clean table syntax | Less familiar to some | Low |
| **YAML** | Widely known | Indentation footguns, no Rust convention | Low |
| **JSON** | Universal | Verbose, no comments | Low |
| | | | |
| **System keyring (chosen)** | Native OS security, no extra tools | Not portable across machines | Low |
| **age encryption** | Portable, git-friendly | Requires external tool, decrypt-to-edit | Medium |
| **SOPS** | In-place encryption, git-diff friendly | Complex setup, overkill for single-user | Medium |
| **Env vars only** | Simple | No persistence, repetitive | Low |

## Rationale

- **Named configs** are the natural design for multi-agent multi-model systems. The alternative (single backend) is a dead end as soon as you want a cheap model for summaries and a capable model for code.
- **Cascading resolution** follows the principle of least surprise — specific overrides general, local overrides global.
- **TOML** is the Rust ecosystem standard. Its table syntax (`[llm.claude]`) maps naturally to named configs.
- **System keyring** provides the best UX for single-user CLI tools — no extra tools, automatic unlock on login, and tools like `gh` and `aws-cli` have proven this pattern works.
- **XDG + project-local** is standard for CLI tools and allows per-project customization without polluting global config.

## Consequences

### Positive
- Agents can use different models without code changes
- Users configure once, run many times
- API keys are stored securely by default
- Project-local overrides enable per-repo tuning
- Clean extension point for future features (tool-level routing, cost tiers, fallback chains)

### Negative
- New `arawn-config` crate adds to the dependency graph
- Keyring integration requires platform-specific dependencies
- Config merging logic adds complexity to startup path
- Users must learn the resolution order

### Neutral
- CLI arguments remain supported as overrides, so existing usage still works
- Environment variables still work as a fallback, maintaining CI/container compatibility