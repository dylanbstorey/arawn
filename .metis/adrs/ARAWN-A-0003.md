---
id: 001-age-encrypted-secrets-over-system
level: adr
title: "Age-Encrypted Secrets Over System Keyring"
number: 1
short_code: "ARAWN-A-0003"
created_at: 2026-03-05T05:15:58.790329+00:00
updated_at: 2026-03-05T05:18:59.334381+00:00
decision_date: 
decision_maker: 
parent: 
archived: false

tags:
  - "#adr"
  - "#phase/decided"


exit_criteria_met: false
initiative_id: NULL
---

# ADR-3: Age-Encrypted Secrets Over System Keyring

## Context

ADR-1 originally specified system keyring (macOS Keychain, Linux secret-service, Windows Credential Manager) as the primary secret storage mechanism. In practice, this created several problems:

- **UX friction on macOS**: Every keyring access triggers a "allow access?" security dialog, disrupting CLI workflows.
- **Platform inconsistency**: Each OS has a different keyring implementation with different behaviors, failure modes, and capability limits.
- **No agent-level isolation**: The keyring stores secrets as plaintext values. An agent with tool access could be prompt-injected to read and exfiltrate credentials (the "molt bot" pattern — instruct the agent to read a secret and include it in a web request).
- **Not portable**: Secrets are tied to the OS account and cannot be transferred between machines or backed up alongside project config.
- **OAuth tokens exposed**: OAuth refresh tokens were stored as plaintext JSON in the keyring, readable by any process with keyring access.

The core requirement is that **agents must never see raw secret values** at any point in the conversation loop, while users must be able to store and manage secrets with zero friction.

## Decision

Replace the system keyring with **age-encrypted file-based secret storage** and a **handle-based resolution system** that prevents secrets from entering the LLM context.

### Storage

Secrets are stored in `~/.config/arawn/secrets.age`, encrypted with an x25519 keypair generated on first use (`~/.config/arawn/identity.age`). The encrypted file contains a JSON map of name-value pairs.

### Resolution Flow

1. User stores a secret: `arawn secrets set github_token`
2. Agent references it via handle syntax: `${{secrets.github_token}}`
3. During tool execution, `ToolRegistry::execute_with_config()` detects handles in tool parameters
4. The `SecretResolver` decrypts and substitutes real values **transiently** — only the tool sees the real value
5. Session history and logs record only the handle string, never the real value

### Resolution Order

API keys follow a 4-tier resolution cascade:

1. Age-encrypted store (`secrets.age`)
2. System keyring (legacy, feature-gated behind `keyring` feature, disabled by default)
3. Environment variable (`ANTHROPIC_API_KEY`, etc.)
4. Config file (plaintext TOML, with a warning)

## Alternatives Analysis

| Option | Pros | Cons | Risk Level |
|--------|------|------|------------|
| **Age encryption (chosen)** | Zero UX friction, portable, pure Rust, agent-isolated | Identity file is plaintext on disk (owner-only perms) | Low |
| **System keyring (original)** | Native OS security, automatic unlock | macOS dialogs, platform-specific, no agent isolation | Medium |
| **SOPS** | In-place encryption, git-diff friendly | Complex setup, external tool dependency, overkill for single-user | Medium |
| **Sealed secrets / Vault** | Enterprise-grade | Requires running infrastructure, not laptop-friendly | High |
| **Env vars only** | Simple, CI-compatible | No persistence, repetitive, visible in process listing | Low |

## Rationale

- **Agent isolation is the primary driver.** The handle-based system (`${{secrets.name}}`) means the LLM never sees real values. Even if the agent is prompt-injected, it can only reference handles — the resolution happens outside the conversation context, in the tool execution layer.
- **Age is pure Rust** (`age` crate) with no platform-specific dependencies. It works identically on macOS, Linux, and Windows.
- **Zero friction** — no OS dialogs, no unlock prompts. Secrets are available instantly on CLI startup.
- **Portability** — copy `identity.age` and `secrets.age` to a new machine and secrets follow.
- **Keyring retained as legacy fallback** for users who already stored secrets there, but disabled by default.

## Consequences

### Positive
- Agents cannot exfiltrate secrets even under prompt injection — handles are opaque references
- No OS-level permission dialogs interrupting workflows
- Platform-independent — same behavior on macOS, Linux, Windows
- Secrets are portable between machines
- Session history is safe to inspect — only handles appear in logs

### Negative
- Identity file (`identity.age`) is stored with owner-only permissions but not further protected — someone with filesystem access to the user's home directory can decrypt secrets
- Users must copy `identity.age` when migrating between machines (vs keyring auto-sync on some platforms)
- Adds `age` crate as a dependency

### Neutral
- Environment variables still work as a fallback, maintaining CI/container compatibility
- Keyring support remains available as a feature-gated option for users who prefer it