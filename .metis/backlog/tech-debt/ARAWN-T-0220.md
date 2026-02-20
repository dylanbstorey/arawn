---
id: architecture-unified-configuration
level: task
title: "Architecture: Unified Configuration"
short_code: "ARAWN-T-0220"
created_at: 2026-02-20T14:47:45.809910+00:00
updated_at: 2026-02-20T14:47:45.809910+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#tech-debt"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Architecture: Unified Configuration

## Objective

Consolidate fragmented configuration structs into a single unified `ArawnConfig` TOML schema.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P3 - Low (when time permits)

### Technical Debt Impact
- **Current Problems**: Configuration fragmented across 5+ config structs in different crates:
  - `ArawnConfig` in `arawn-config`
  - `ServerConfig` in `arawn-server`
  - `TuiConfig` in `arawn-tui`
  - `WorkstreamConfig` in `arawn-workstream`
  - `AgentConfig` in `arawn-agent`
- **Benefits of Fixing**: Single source of truth, easier to document, consistent loading
- **Risk Assessment**: LOW - mostly consolidation, existing configs still work

## Acceptance Criteria

- [ ] Single `config.toml` file with all configuration sections
- [ ] `ArawnConfig` struct in `arawn-config` contains all sections
- [ ] Environment variable overrides work for all settings
- [ ] CLI flags override config file values
- [ ] Default config generated on first run
- [ ] Migration path for existing configs
- [ ] All existing tests pass

## Implementation Notes

### Target Config Schema

```toml
# ~/.arawn/config.toml

[server]
bind_address = "127.0.0.1:8080"
auth_token = "..."
cors_origins = ["http://localhost:3000"]

[agent]
max_turns = 50
memory_enabled = true
default_provider = "anthropic"

[workstreams]
base_dir = "~/.arawn/workstreams"
default_workstream = "scratch"

[memory]
database_path = "~/.arawn/memory.db"
embedding_model = "local"

[tui]
theme = "dark"
sidebar_width = 30

[llm.anthropic]
api_key = "${ANTHROPIC_API_KEY}"
model = "claude-sonnet-4-20250514"

[llm.openai]
api_key = "${OPENAI_API_KEY}"
model = "gpt-4"
```

### Config Loading Order

1. Built-in defaults
2. System config (`/etc/arawn/config.toml`)
3. User config (`~/.arawn/config.toml`)
4. Project config (`./.arawn/config.toml`)
5. Environment variables (`ARAWN_SERVER_BIND_ADDRESS`)
6. CLI flags (`--bind-address`)

### ArawnConfig Structure

```rust
// arawn-config/src/lib.rs
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ArawnConfig {
    pub server: ServerConfig,
    pub agent: AgentConfig,
    pub workstreams: WorkstreamsConfig,
    pub memory: MemoryConfig,
    pub tui: TuiConfig,
    pub llm: LlmProvidersConfig,
}

impl ArawnConfig {
    pub fn load() -> Result<Self>;
    pub fn load_with_overrides(overrides: ConfigOverrides) -> Result<Self>;
}
```

## Dependencies

- Can be done in parallel with ARAWN-T-0218 (Domain Facade)
- Not blocking other tasks

## Status Updates

*To be added during implementation*