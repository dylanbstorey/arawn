//! Arawn - Personal Research Agent for Edge Computing
//!
//! Main entry point for the Arawn CLI.

use anyhow::Result;
use clap::{Parser, Subcommand};

mod client;
mod commands;

use commands::{
    agent, ask, auth, chat, config, mcp, memory, notes, plugin, start, status, tui,
};

// ─────────────────────────────────────────────────────────────────────────────
// CLI Structure
// ─────────────────────────────────────────────────────────────────────────────

/// Arawn - Personal Research Agent for Edge Computing
#[derive(Parser)]
#[command(name = "arawn")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Output as JSON (for scripting)
    #[arg(long, global = true)]
    pub json: bool,

    /// Server URL (overrides current context)
    #[arg(long, global = true, env = "ARAWN_SERVER_URL")]
    pub server: Option<String>,

    /// Use a specific context instead of the current one
    #[arg(long, global = true)]
    pub context: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the Arawn server
    Start(start::StartArgs),

    /// Show server status and resource usage
    Status(status::StatusArgs),

    /// Ask a one-shot question
    Ask(ask::AskArgs),

    /// Enter interactive chat mode (REPL)
    Chat(chat::ChatArgs),

    /// Memory operations
    Memory(memory::MemoryArgs),

    /// Note management
    Notes(notes::NotesArgs),

    /// Configuration management
    Config(config::ConfigArgs),

    /// Authentication management
    Auth(auth::AuthArgs),

    /// Plugin management
    Plugin(plugin::PluginArgs),

    /// Subagent management
    Agent(agent::AgentArgs),

    /// MCP server management
    Mcp(mcp::McpArgs),

    /// Launch Terminal UI
    Tui(tui::TuiArgs),
}

// ─────────────────────────────────────────────────────────────────────────────
// Server URL Resolution
// ─────────────────────────────────────────────────────────────────────────────

/// Resolve the server URL from various sources.
///
/// Priority order:
/// 1. CLI `--server` flag (already checked by clap via `cli.server`)
/// 2. `--context` flag → lookup in client config
/// 3. Current context from client config
/// 4. ARAWN_SERVER_URL environment variable (already checked by clap)
/// 5. Default: http://localhost:8080
fn resolve_server_url(server_flag: Option<&str>, context_flag: Option<&str>) -> String {
    // 1. Explicit --server flag takes priority
    if let Some(url) = server_flag {
        return url.to_string();
    }

    // Try to load client config
    let config = arawn_config::load_client_config().ok();

    // 2. Explicit --context flag
    if let Some(ctx_name) = context_flag {
        if let Some(config) = &config {
            if let Some(ctx) = config.get_context(ctx_name) {
                return ctx.server.clone();
            }
            // Context not found — fall through to default
            tracing::warn!("Context '{}' not found, using default", ctx_name);
        }
    }

    // 3. Current context from config
    if let Some(config) = &config {
        if let Some(url) = config.current_server_url() {
            return url;
        }
    }

    // 4. Default
    "http://localhost:8080".to_string()
}

// ─────────────────────────────────────────────────────────────────────────────
// Main
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Check if running TUI - need to skip console logging to avoid corrupting display
    let is_tui = matches!(cli.command, Commands::Tui(_));

    // Initialize tracing — console (human-readable) + rotating JSON file
    let filter = if cli.verbose {
        "arawn=debug,arawn_agent=debug,arawn_llm=debug,arawn_server=debug,arawn_oauth=debug,arawn_config=debug,info"
    } else {
        "arawn=info,arawn_agent=info,arawn_llm=info,arawn_server=info,arawn_oauth=info,warn"
    };

    let log_dir = arawn_config::xdg_config_dir()
        .map(|d| d.join("logs"))
        .unwrap_or_else(|| std::path::PathBuf::from("logs"));
    let file_appender = tracing_appender::rolling::daily(&log_dir, "arawn.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    use tracing_subscriber::prelude::*;

    if is_tui {
        // TUI mode: tracing is set up by the TUI itself with a log buffer
        // Don't initialize here - the TUI will handle it
    } else {
        // Normal mode: console + file logging
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .with_target(true)
                    .with_filter(tracing_subscriber::EnvFilter::new(filter)),
            )
            .with(
                tracing_subscriber::fmt::layer()
                    .json()
                    .with_writer(non_blocking)
                    .with_filter(tracing_subscriber::EnvFilter::new(
                        "arawn=trace,arawn_agent=trace,arawn_llm=trace,arawn_server=trace,arawn_oauth=trace,arawn_config=trace,info"
                    )),
            )
            .init();
    }

    // Get server URL: CLI flag > context > env var > default
    let server_url = resolve_server_url(cli.server.as_deref(), cli.context.as_deref());

    // Create context for commands
    let ctx = commands::Context {
        server_url,
        json_output: cli.json,
        verbose: cli.verbose,
    };

    // Dispatch to command handlers
    match cli.command {
        Commands::Start(args) => start::run(args, &ctx).await,
        Commands::Status(args) => status::run(args, &ctx).await,
        Commands::Ask(args) => ask::run(args, &ctx).await,
        Commands::Chat(args) => chat::run(args, &ctx).await,
        Commands::Memory(args) => memory::run(args, &ctx).await,
        Commands::Notes(args) => notes::run(args, &ctx).await,
        Commands::Config(args) => config::run(args, &ctx).await,
        Commands::Auth(args) => auth::run(args, &ctx).await,
        Commands::Plugin(args) => plugin::run(args, &ctx).await,
        Commands::Agent(args) => agent::run(args, &ctx).await,
        Commands::Mcp(args) => mcp::run(args, &ctx).await,
        Commands::Tui(args) => tui::run(args, &ctx).await,
    }
}
