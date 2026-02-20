---
id: tui-client-configuration
level: task
title: "TUI: Client configuration (kubeconfig-style)"
short_code: "ARAWN-T-0169"
created_at: 2026-02-11T18:26:43.450400+00:00
updated_at: 2026-02-19T18:10:44.045822+00:00
parent: ARAWN-I-0025
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0025
---

# TUI: Client configuration (kubeconfig-style)

## Objective

Implement a kubeconfig-style client configuration system that allows:
- Multiple server contexts (local, remote, production, etc.)
- Easy switching between contexts
- Pre-configured distributions (just update DNS/endpoint before shipping)
- Secure credential storage per context

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Config file at `~/.config/arawn/client.yaml` (or XDG equivalent)
- [x] Support multiple named contexts with server URL + credentials
- [x] `current-context` field to set default
- [x] `arawn config use-context <name>` to switch
- [x] `arawn config get-contexts` to list available
- [x] `arawn config set-context <name> --server=<url>` to create/update
- [x] TUI reads config and connects to current context
- [x] `--context` flag overrides current-context for single command

## Implementation Notes

### Config File Format
```yaml
apiVersion: v1
kind: Config

current-context: local

contexts:
  - name: local
    server: http://localhost:8080
    # No auth for local
    
  - name: home-server
    server: https://arawn.home.lan:8443
    auth:
      type: api-key
      key-file: ~/.config/arawn/keys/home-server.key
      
  - name: work
    server: https://arawn.company.com
    auth:
      type: oauth
      client-id: arawn-tui
      # Token cached after first auth flow

defaults:
  timeout: 30s
  workstream: default
```

### Key Design Points
- **Contexts** = server + auth bundle (like kubeconfig)
- **Auth types**: none, api-key, oauth (extensible)
- **Secrets**: Reference files or use system keyring, never inline
- **Distributable**: Ship config with just `server` filled in, user adds auth

### CLI Commands
```bash
# List contexts
arawn config get-contexts

# Switch context
arawn config use-context home-server

# Create/update context
arawn config set-context staging --server=https://staging.example.com

# View current
arawn config current-context

# Use specific context for one command
arawn --context=work tui
```

### Integration Points
- `arawn-config` crate needs to load/parse this
- TUI uses it to determine server URL (replaces `--server` flag as primary)
- OAuth flow stores tokens per-context
- WebSocket client reads auth from context

### Dependencies
- ARAWN-T-0161 (app shell) - needs to read config
- ARAWN-T-0162 (WebSocket client) - needs auth from context

## Status Updates

### 2026-02-11: Implementation Complete

**Files Created:**
- `crates/arawn-config/src/client.rs` - Client configuration with contexts, auth types, loading/saving

**Files Modified:**
- `crates/arawn-config/src/lib.rs` - Export client module and types
- `crates/arawn-config/src/error.rs` - Added ParseYaml, ContextNotFound, Other error variants
- `crates/arawn-config/Cargo.toml` - Added serde_yaml dependency
- `crates/arawn/src/main.rs` - Added --context flag, resolve_server_url function
- `crates/arawn/src/commands/config.rs` - Added context management subcommands
- `crates/arawn/src/commands/tui.rs` - Updated to use client config
- `crates/arawn-tui/src/lib.rs` - Added TuiConfig, from_client_config loading
- `crates/arawn-tui/src/app.rs` - Added context_name field
- `crates/arawn-tui/src/ui/layout.rs` - Display context name in header
- `crates/arawn-tui/Cargo.toml` - Added arawn-config dependency

**Features Implemented:**
- YAML config file at `~/.config/arawn/client.yaml`
- Multiple named contexts with server URL + optional auth
- Auth types: none, api-key (file or env), oauth, bearer
- `current-context` field to set default
- CLI commands:
  - `arawn config current-context` - Show current context
  - `arawn config get-contexts` - List all contexts
  - `arawn config use-context <name>` - Switch context
  - `arawn config set-context <name> --server=<url>` - Create/update
  - `arawn config delete-context <name>` - Remove context
- `--context` global flag to override for single command
- Server URL resolution order: --server > --context > current-context > env > default
- TUI reads config, displays context name in header

**Config File Format:**
```yaml
api-version: v1
kind: ClientConfig
current-context: local

contexts:
  - name: local
    server: http://localhost:8080
  - name: home
    server: https://arawn.home.lan:8443
    auth:
      type: api-key
      key-file: ~/.config/arawn/keys/home.key
    workstream: personal

defaults:
  timeout: 30
  workstream: default
```

**Tests:**
- 104 total tests in arawn-config (67 existing + 37 new client tests)
- All tests pass
- Clippy clean on arawn-config, arawn-tui