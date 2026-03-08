---
id: add-cli-command-execution-tests
level: task
title: "Add CLI command execution tests for start, ask, chat, config"
short_code: "ARAWN-T-0285"
created_at: 2026-03-08T03:17:30.209368+00:00
updated_at: 2026-03-08T03:17:30.209368+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#tech-debt"


exit_criteria_met: false
initiative_id: NULL
---

# Add CLI command execution tests for start, ask, chat, config

## Objective

The `arawn` CLI binary has 18 command files totaling ~5,500 lines with **zero execution tests**. The existing `cli_integration.rs` only tests argument parsing and help text. Add tests that verify commands actually execute correctly — config display, status checking, and server startup validation.

### Priority
- [x] P2 - Medium (CLI is secondary to TUI, but `start` is the server entry point)
- **Size**: L

### Current Problems
- `start.rs` is 1,966 lines orchestrating LLM backends, plugins, memory, workstreams, agents — zero tests
- Config loading, validation, and display (`config.rs`, 534 lines) untested
- Auth setup (`auth.rs`, 258 lines) untested
- Plugin management (`plugin.rs`, 514 lines) untested
- Changes to startup code break production and are only caught by manual testing

## Acceptance Criteria

- [ ] `tests/command_integration.rs` — tests for command execution (not just parsing)
- [ ] Tests for `config show` — verify output format, missing config handling
- [ ] Tests for `config set` — verify config file modification
- [ ] Tests for `status` — verify server connectivity check (mock server)
- [ ] Tests for `start` — verify startup sequence with mock config (at minimum: config validation, port binding, graceful shutdown)
- [ ] Tests for `secrets` — verify secret store set/get/list/delete
- [ ] Tests for `plugin list` — verify plugin discovery
- [ ] Error handling: missing config file, invalid config values, port already in use
- [ ] At least 25 new test functions

## Implementation Notes

### Testability challenges

`start.rs` is the hardest to test because it does everything:
- Reads config from disk
- Initializes LLM backends
- Loads plugins
- Starts HTTP server
- Sets up memory store

**Approach**: Test subsections, not the monolith:

1. **Config commands** — easiest, use tempdir with test config files:
```rust
#[test]
fn test_config_show_displays_current_config() {
    let dir = tempdir().unwrap();
    write_test_config(&dir, "server.port = 9090\n");
    let output = Command::cargo_bin("arawn").unwrap()
        .args(["config", "show"])
        .env("ARAWN_CONFIG_DIR", dir.path())
        .output().unwrap();
    assert!(String::from_utf8_lossy(&output.stdout).contains("9090"));
}
```

2. **Start validation** — test config validation without actually starting:
```rust
#[test]
fn test_start_rejects_invalid_port() {
    let dir = tempdir().unwrap();
    write_test_config(&dir, "server.port = 99999\n");
    let output = Command::cargo_bin("arawn").unwrap()
        .args(["start"])
        .env("ARAWN_CONFIG_DIR", dir.path())
        .output().unwrap();
    assert!(!output.status.success());
}
```

3. **Secrets commands** — use isolated secret store:
```rust
#[test]
fn test_secrets_set_and_get() {
    let dir = tempdir().unwrap();
    // Set
    Command::cargo_bin("arawn").unwrap()
        .args(["secrets", "set", "test_key", "test_value"])
        .env("ARAWN_CONFIG_DIR", dir.path())
        .assert().success();
    // Get
    let output = Command::cargo_bin("arawn").unwrap()
        .args(["secrets", "get", "test_key"])
        .env("ARAWN_CONFIG_DIR", dir.path())
        .output().unwrap();
    assert!(String::from_utf8_lossy(&output.stdout).contains("test_value"));
}
```

### Commands by testability

| Command | Testability | Approach |
|---------|------------|----------|
| `config show/set` | Easy | Tempdir + env override |
| `secrets set/get/list/delete` | Easy | Isolated secret store |
| `status` | Medium | Mock server or check error output |
| `plugin list` | Medium | Tempdir with test plugin manifests |
| `start` | Hard | Config validation only, or short-lived server |
| `ask/chat` | Hard | Requires running server + mock LLM |

### Key files
- `crates/arawn/src/commands/*.rs` — All command implementations
- `crates/arawn/tests/cli_integration.rs` — Existing CLI tests (extend)
- `crates/arawn/tests/command_integration.rs` — New file

### Dependencies
- `assert_cmd` and `predicates` already in dev-dependencies

## Status Updates

*To be added during implementation*