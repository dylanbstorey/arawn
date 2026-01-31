//! Research command - start long-running research tasks.

use anyhow::Result;
use clap::{Args, ValueEnum};

use super::Context;

/// Research depth level.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ResearchDepth {
    /// Quick surface-level research
    Shallow,
    /// Moderate depth research
    Medium,
    /// Thorough deep research
    Deep,
}

/// Arguments for the research command.
#[derive(Args, Debug)]
pub struct ResearchArgs {
    /// The research topic or question
    #[arg(required = true)]
    pub topic: String,

    /// Research depth level
    #[arg(short, long, value_enum, default_value = "medium")]
    pub depth: ResearchDepth,

    /// Send notification when complete
    #[arg(short, long)]
    pub notify: bool,
}

/// Run the research command.
pub async fn run(args: ResearchArgs, ctx: &Context) -> Result<()> {
    println!("Starting research task...");
    println!("Topic: {}", args.topic);
    println!("Depth: {:?}", args.depth);

    if args.notify {
        println!("Will notify when complete");
    }

    if ctx.verbose {
        println!("Server: {}", ctx.server_url);
    }

    // TODO: Submit research task to server
    println!("\nResearch tasks not yet implemented");

    Ok(())
}
