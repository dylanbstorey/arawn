//! Status command - shows server status and resource usage.

use anyhow::Result;
use clap::Args;
use console::Style;
use serde::Serialize;

use super::Context;
use super::output;
use crate::client::Client;

/// Arguments for the status command.
#[derive(Args, Debug)]
#[command(after_help = "\x1b[1mExamples:\x1b[0m
  arawn status                      Check default server
  arawn status --json               Machine-readable output
  arawn status --server http://remote:8080")]
pub struct StatusArgs {}

/// Status response for JSON output.
#[derive(Debug, Serialize)]
struct StatusOutput {
    running: bool,
    version: Option<String>,
    server_url: String,
}

/// Run the status command.
pub async fn run(_args: StatusArgs, ctx: &Context) -> Result<()> {
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
                println!();
                output::header("Arawn Server Status");
                output::kv("Status", Style::new().green().apply_to("● running"));
                output::kv("Version", &health.version);
                output::kv("Server", &ctx.server_url);
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
                println!();
                output::header("Arawn Server Status");
                output::kv("Status", Style::new().red().apply_to("● not running"));
                output::kv("Server", &ctx.server_url);

                if ctx.verbose {
                    println!();
                    output::kv("Error", &e);
                }

                println!();
                output::hint("  Start the server with: arawn start");
                println!();
            }
        }
    }

    Ok(())
}
