//! Arawn - Personal Research Agent for Edge Computing
//!
//! Main entry point for the Arawn CLI.

use anyhow::Result;
use clap::{Parser, Subcommand};

mod client;
mod commands;

use commands::{
    agent, ask, auth, chat, config, mcp, memory, notes, plugin, research, start, status, stop,
    tasks,
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

    /// Server URL (default: http://localhost:8080)
    #[arg(long, global = true, env = "ARAWN_SERVER_URL")]
    pub server: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the Arawn server
    Start(start::StartArgs),

    /// Stop the running server
    Stop(stop::StopArgs),

    /// Show server status and resource usage
    Status(status::StatusArgs),

    /// Ask a one-shot question
    Ask(ask::AskArgs),

    /// Enter interactive chat mode (REPL)
    Chat(chat::ChatArgs),

    /// Start a long-running research task
    Research(research::ResearchArgs),

    /// Manage running and recent tasks
    Tasks(tasks::TasksArgs),

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
}

// ─────────────────────────────────────────────────────────────────────────────
// Main
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

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

    // Get server URL
    let server_url = cli
        .server
        .unwrap_or_else(|| "http://localhost:8080".to_string());

    // Create context for commands
    let ctx = commands::Context {
        server_url,
        json_output: cli.json,
        verbose: cli.verbose,
    };

    // Dispatch to command handlers
    match cli.command {
        Commands::Start(args) => start::run(args, &ctx).await,
        Commands::Stop(args) => stop::run(args, &ctx).await,
        Commands::Status(args) => status::run(args, &ctx).await,
        Commands::Ask(args) => ask::run(args, &ctx).await,
        Commands::Chat(args) => chat::run(args, &ctx).await,
        Commands::Research(args) => research::run(args, &ctx).await,
        Commands::Tasks(args) => tasks::run(args, &ctx).await,
        Commands::Memory(args) => memory::run(args, &ctx).await,
        Commands::Notes(args) => notes::run(args, &ctx).await,
        Commands::Config(args) => config::run(args, &ctx).await,
        Commands::Auth(args) => auth::run(args, &ctx).await,
        Commands::Plugin(args) => plugin::run(args, &ctx).await,
        Commands::Agent(args) => agent::run(args, &ctx).await,
        Commands::Mcp(args) => mcp::run(args, &ctx).await,
    }
}
