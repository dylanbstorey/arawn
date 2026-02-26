//! CLI command handlers.

pub mod agent;
pub mod ask;
pub mod auth;
pub mod chat;
pub mod config;
pub mod mcp;
pub mod memory;
pub mod notes;
pub mod plugin;
pub mod repl;
pub mod start;
pub mod status;
pub mod tui;

/// Shared context for all commands.
#[derive(Debug, Clone)]
pub struct Context {
    /// Server URL to connect to.
    pub server_url: String,
    /// Output as JSON for scripting.
    pub json_output: bool,
    /// Verbose output enabled.
    pub verbose: bool,
}
