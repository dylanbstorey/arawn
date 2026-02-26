//! CLI integration tests for the Arawn command-line interface.
//!
//! These tests verify:
//! - Help text is displayed correctly
//! - Argument parsing works as expected
//! - Invalid inputs are rejected with appropriate messages
//!
//! Note: These tests do not require a running server - they test
//! CLI parsing and help output only.

use assert_cmd::Command;
use predicates::prelude::*;

/// Get a command for the arawn binary.
fn arawn() -> Command {
    Command::cargo_bin("arawn").unwrap()
}

// ─────────────────────────────────────────────────────────────────────────────
// Help and Version Tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_help_displays() {
    arawn()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Arawn"))
        .stdout(predicate::str::contains("Personal Research Agent"));
}

#[test]
fn test_version_displays() {
    arawn()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("arawn"));
}

#[test]
fn test_help_lists_subcommands() {
    arawn()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("start"))
        .stdout(predicate::str::contains("status"))
        .stdout(predicate::str::contains("ask"))
        .stdout(predicate::str::contains("chat"))
        .stdout(predicate::str::contains("memory"))
        .stdout(predicate::str::contains("notes"))
        .stdout(predicate::str::contains("config"))
        .stdout(predicate::str::contains("auth"))
        .stdout(predicate::str::contains("plugin"))
        .stdout(predicate::str::contains("agent"))
        .stdout(predicate::str::contains("mcp"))
        .stdout(predicate::str::contains("tui"));
}

// ─────────────────────────────────────────────────────────────────────────────
// Global Flag Tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_verbose_flag_accepted() {
    // --verbose is global and should be parsed without error
    // (Will fail with connection error, but that's expected - we're testing parsing)
    arawn().args(["--verbose", "--help"]).assert().success();
}

#[test]
fn test_json_flag_accepted() {
    arawn().args(["--json", "--help"]).assert().success();
}

#[test]
fn test_server_flag_accepted() {
    arawn()
        .args(["--server", "http://localhost:9999", "--help"])
        .assert()
        .success();
}

#[test]
fn test_context_flag_accepted() {
    arawn()
        .args(["--context", "mycontext", "--help"])
        .assert()
        .success();
}

// ─────────────────────────────────────────────────────────────────────────────
// Subcommand Help Tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_start_help() {
    arawn()
        .args(["start", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Start the Arawn server"));
}

#[test]
fn test_status_help() {
    arawn()
        .args(["status", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("status"));
}

#[test]
fn test_ask_help() {
    arawn()
        .args(["ask", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("question"));
}

#[test]
fn test_chat_help() {
    arawn()
        .args(["chat", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("chat"));
}

#[test]
fn test_memory_help() {
    arawn()
        .args(["memory", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("memory").or(predicate::str::contains("Memory")));
}

#[test]
fn test_notes_help() {
    arawn()
        .args(["notes", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("note").or(predicate::str::contains("Note")));
}

#[test]
fn test_config_help() {
    arawn()
        .args(["config", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("config").or(predicate::str::contains("Config")));
}

#[test]
fn test_auth_help() {
    arawn()
        .args(["auth", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("auth").or(predicate::str::contains("Auth")));
}

#[test]
fn test_plugin_help() {
    arawn()
        .args(["plugin", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("plugin").or(predicate::str::contains("Plugin")));
}

#[test]
fn test_agent_help() {
    arawn()
        .args(["agent", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("agent").or(predicate::str::contains("Agent")));
}

#[test]
fn test_mcp_help() {
    arawn()
        .args(["mcp", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("mcp").or(predicate::str::contains("MCP")));
}

#[test]
fn test_tui_help() {
    arawn().args(["tui", "--help"]).assert().success().stdout(
        predicate::str::contains("tui")
            .or(predicate::str::contains("TUI"))
            .or(predicate::str::contains("Terminal")),
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Invalid Input Tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_unknown_subcommand_fails() {
    arawn()
        .arg("unknown-command")
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}

#[test]
fn test_invalid_flag_fails() {
    arawn()
        .arg("--invalid-flag")
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}

// ─────────────────────────────────────────────────────────────────────────────
// Config Subcommand Tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_config_subcommands_listed() {
    arawn().args(["config", "--help"]).assert().success();
}

// ─────────────────────────────────────────────────────────────────────────────
// Auth Subcommand Tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_auth_subcommands_listed() {
    arawn().args(["auth", "--help"]).assert().success();
}

// ─────────────────────────────────────────────────────────────────────────────
// Plugin Subcommand Tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_plugin_subcommands_listed() {
    arawn().args(["plugin", "--help"]).assert().success();
}

// ─────────────────────────────────────────────────────────────────────────────
// MCP Subcommand Tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_mcp_subcommands_listed() {
    arawn().args(["mcp", "--help"]).assert().success();
}
