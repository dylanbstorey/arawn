//! Status command - shows server status and resource usage.

use anyhow::Result;
use clap::Args;
use console::{Style, style};
use serde::Serialize;

use super::Context;
use crate::client::Client;

/// Arguments for the status command.
#[derive(Args, Debug)]
pub struct StatusArgs {
    /// Show detailed status information
    #[arg(short, long)]
    pub detailed: bool,
}

/// Status response for JSON output.
#[derive(Debug, Serialize)]
struct StatusOutput {
    running: bool,
    version: Option<String>,
    server_url: String,
}

/// Run the status command.
pub async fn run(args: StatusArgs, ctx: &Context) -> Result<()> {
    let client = Client::new(&ctx.server_url)?;

    match client.health().await {
        Ok(health) => {
            if ctx.json_output {
                let output = StatusOutput {
                    running: true,
                    version: Some(health.version.clone()),
                    server_url: ctx.server_url.clone(),
                };
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                let green = Style::new().green();
                let dim = Style::new().dim();

                println!();
                println!("{}", style("Arawn Server Status").bold());
                println!("{}", dim.apply_to("─".repeat(40)));
                println!();
                println!(
                    "  {} {}",
                    dim.apply_to("Status:"),
                    green.apply_to("● running")
                );
                println!("  {} {}", dim.apply_to("Version:"), health.version);
                println!("  {} {}", dim.apply_to("Server:"), ctx.server_url);

                if args.detailed {
                    println!();
                    println!("{}", dim.apply_to("─".repeat(40)));
                    println!();
                    println!(
                        "  {} (detailed metrics not yet implemented)",
                        dim.apply_to("Metrics:")
                    );
                }

                println!();
            }
        }
        Err(e) => {
            if ctx.json_output {
                let output = StatusOutput {
                    running: false,
                    version: None,
                    server_url: ctx.server_url.clone(),
                };
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                let red = Style::new().red();
                let dim = Style::new().dim();

                println!();
                println!("{}", style("Arawn Server Status").bold());
                println!("{}", dim.apply_to("─".repeat(40)));
                println!();
                println!(
                    "  {} {}",
                    dim.apply_to("Status:"),
                    red.apply_to("● not running")
                );
                println!("  {} {}", dim.apply_to("Server:"), ctx.server_url);

                if ctx.verbose {
                    println!();
                    println!("  {} {}", dim.apply_to("Error:"), e);
                }

                println!();
                println!("  {}", dim.apply_to("Start the server with: arawn start"));
                println!();
            }
        }
    }

    Ok(())
}
