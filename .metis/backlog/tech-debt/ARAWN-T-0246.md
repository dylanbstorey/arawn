---
id: gpg-encrypted-secret-store-with
level: task
title: "GPG-encrypted secret store with opaque agent handles"
short_code: "ARAWN-T-0246"
created_at: 2026-03-02T15:25:56.783430+00:00
updated_at: 2026-03-02T17:21:07.428289+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#tech-debt"
  - "#phase/active"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# GPG-encrypted secret store with opaque agent handles

## Objective

Replace the OS keychain (`keyring` crate) secret storage with an `age`-encrypted local file, and introduce opaque secret handles (`${{secrets.*}}`) so the agent LLM never sees raw secret values. Secrets are resolved at the `ToolRegistry::execute_with_config()` chokepoint — the same centralized enforcement point used by FsGate (ARAWN-T-0245).

Uses the `age` crate (pure Rust, modern encryption) instead of GPG — simpler API, no keypair confusion, designed for exactly this use case.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P1 - High (important for user experience)

### Technical Debt Impact
- **Current Problems**: OS keychain triggers macOS "allow access?" prompts on every use — bad UX. OAuth tokens stored as plaintext JSON. Env vars visible in process lists. No mechanism for tool-level auth that hides secrets from the agent. Exfiltration risk: a prompt-injected agent could be directed to read and leak credentials (molt bot pattern).
- **Benefits of Fixing**: Zero-friction secret storage (no OS prompts). Agent never sees raw secret values at any point. Secrets encrypted at rest with `age` identity. Tool-level auth (e.g. GitHub API) works without exposing tokens to LLM context. Exfiltration-resistant: no tool exists that returns secret values — only the ToolRegistry can resolve handles.
- **Risk Assessment**: Without this, the keychain UX friction pushes users toward plaintext config files or env vars, and any tool-level auth exposes secrets directly in the agent's conversation context.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Arawn auto-generates an `age` identity on first startup, stored in `~/.config/arawn/identity.age`
- [ ] `arawn secrets set <name>` prompts for value, encrypts with `age` recipient, writes to `~/.config/arawn/secrets.age`
- [ ] `arawn secrets list` shows stored secret names (never values)
- [ ] `arawn secrets delete <name>` removes a secret
- [ ] Existing `resolve_api_key()` updated to check age store (replaces keyring tier)
- [ ] Secret handle syntax `${{secrets.<name>}}` recognized in tool params
- [ ] `ToolRegistry::execute_with_config()` scans params for handles, resolves from age store, passes resolved clone to tool
- [ ] Original params (with handles, not values) are what get logged/recorded in session history
- [ ] No tool or agent API exists that returns raw secret values
- [ ] `keyring` crate dependency removed (or feature-gated off by default)
- [ ] LLM backend API keys (GROQ, ANTHROPIC, etc.) work through the new GPG store
- [ ] Existing OAuth flow unaffected (separate concern)

## Implementation Notes

### Architecture

```
User: arawn secrets set github_token
  → prompts for value
  → encrypts with arawn's age recipient (public key)
  → writes to ~/.config/arawn/secrets.age

Agent tool call:
  { "headers": { "Authorization": "Bearer ${{secrets.github_token}}" } }

ToolRegistry::execute_with_config():
  1. Scan params for ${{secrets.*}} handles
  2. Decrypt matching values from secrets.age (identity loaded in memory)
  3. Replace handles in a CLONED params object
  4. Pass resolved params to tool.execute()
  5. Tool makes HTTP call with real token
  6. Original params (with handles) are what get logged

Agent receives tool result — never saw the raw value
```

### Resolution Order (replaces current 3-tier)
1. **Age secret store** (`~/.config/arawn/secrets.age`) — primary
2. **Environment variables** — fallback for CI/container environments
3. **Config file** (plaintext TOML) — last resort, with warning

### Key Components

| Component | Location | Action |
|-----------|----------|--------|
| Age identity management | `crates/arawn-config/src/age_crypto.rs` | **Create** — generate identity, encrypt/decrypt |
| Secret store | `crates/arawn-config/src/secrets.rs` | **Rewrite** — replace keyring with age-encrypted file |
| CLI commands | `crates/arawn/src/commands/secrets.rs` | **Create** — `set`, `list`, `delete` subcommands |
| Handle resolution | `crates/arawn-agent/src/tool.rs` | **Edit** — scan/resolve `${{secrets.*}}` in `execute_with_config()` |
| API key resolution | `crates/arawn-config/src/secrets.rs` | **Edit** — update `resolve_api_key()` to use age store |
| Cargo.toml | workspace root | **Edit** — remove/feature-gate `keyring` dependency |

### Security Properties
- **At rest**: Secrets encrypted with `age`, file is inert without the identity
- **In LLM context**: Agent only ever sees `${{secrets.github_token}}`, never the value
- **In logs/history**: Only handle strings recorded (params logged pre-resolution)
- **At execution**: Tool gets real value transiently, never flows back to agent
- **Exfiltration defense**: No tool returns secret values — store is decrypt-only at ToolRegistry level

### Dependencies
- Pairs with ARAWN-T-0245 (FsGate) — both enforce at `execute_with_config()` chokepoint
- `age` crate — pure Rust, modern file encryption, minimal API

## Status Updates

### Implementation Complete

**Files created:**
- `crates/arawn-types/src/secret_resolver.rs` — `SecretResolver` trait, handle syntax constants, `resolve_handles_in_json()` recursive resolver, comprehensive tests
- `crates/arawn-config/src/age_crypto.rs` — Age identity management (generate, load, encrypt, decrypt), `AgeError` type, tests
- `crates/arawn-config/src/secret_store.rs` — `AgeSecretStore` (JSON map encrypted with age identity), implements `SecretResolver`, RwLock cache, persistence, tests
- `crates/arawn/src/commands/secrets.rs` — CLI commands: `arawn secrets set/list/delete`

**Files modified:**
- `crates/arawn-types/src/lib.rs` — Re-exports for `SecretResolver`, `SharedSecretResolver`, handle utilities
- `crates/arawn-config/src/lib.rs` — Added `age_crypto`, `secret_store` modules, re-exports
- `crates/arawn-config/src/secrets.rs` — Rewritten: 4-tier resolution (age→keyring→env→config), new `store_secret/delete_secret/list_secrets` functions
- `crates/arawn-config/Cargo.toml` — Added `age` dependency, keyring feature disabled by default
- `crates/arawn-agent/src/tool.rs` — Added `secret_resolver` to `ToolContext`, `resolve_secret_handles()` method on `ToolRegistry`, 5 tests
- `crates/arawn-agent/src/agent.rs` — Added `secret_resolver` to `Agent` and `AgentBuilder`, wired into `execute_tools()` and `turn_stream()`
- `crates/arawn-agent/src/stream.rs` — Threaded `secret_resolver` through `StreamState` and `create_turn_stream()`
- `crates/arawn/src/commands/start.rs` — Opens `AgeSecretStore::open_default()` at startup, passes to agent builder
- `crates/arawn/src/commands/config.rs` — Updated `set-secret`/`delete-secret` to use age store instead of keyring
- `crates/arawn/src/commands/mod.rs` — Added `secrets` module
- `crates/arawn/src/main.rs` — Registered `Secrets` command variant

**Acceptance criteria status:**
- [x] Auto-generates age identity on first startup (`load_or_generate_identity`)
- [x] `arawn secrets set <name>` — stores in age-encrypted file
- [x] `arawn secrets list` — shows names, never values
- [x] `arawn secrets delete <name>` — removes secret
- [x] `resolve_api_key()` checks age store first (new tier 1)
- [x] `${{secrets.<name>}}` handle syntax recognized in tool params
- [x] `ToolRegistry::execute_with_config()` resolves handles before tool execution
- [x] Original params (with handles) logged — resolved copy passed to tool
- [x] No tool or API returns raw secret values
- [x] Keyring feature-gated off by default
- [x] Backend API keys work through age store (via updated `resolve_api_key`)
- [x] OAuth flow unaffected (separate concern)

**Verification:** `angreal check all` clean, `angreal test unit` all pass (1800+ tests, 0 failures).