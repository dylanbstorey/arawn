//! Stop command - stops the running Arawn server.

use anyhow::Result;
use clap::Args;
use console::Style;

use super::Context;
use crate::client::Client;

/// Arguments for the stop command.
#[derive(Args, Debug)]
pub struct StopArgs {
    /// Force stop without graceful shutdown
    #[arg(short, long)]
    pub force: bool,
}

/// Run the stop command.
pub async fn run(args: StopArgs, ctx: &Context) -> Result<()> {
    let client = Client::new(&ctx.server_url)?;
    let dim = Style::new().dim();

    // First check if server is running
    match client.health().await {
        Ok(_) => {
            if args.force {
                println!("Force stopping Arawn server...");
            } else {
                println!("Stopping Arawn server gracefully...");
            }

            // TODO: Implement actual server shutdown endpoint
            // For now, the server needs to be stopped with Ctrl+C
            println!();
            println!(
                "{}",
                dim.apply_to("Note: Server shutdown endpoint not yet implemented.")
            );
            println!(
                "{}",
                dim.apply_to("Stop the server with Ctrl+C in the terminal where it's running.")
            );
            println!();
        }
        Err(_) => {
            let yellow = Style::new().yellow();
            println!();
            println!(
                "{}",
                yellow.apply_to("Server is not running or not reachable")
            );
            println!("  {} {}", dim.apply_to("Checked:"), ctx.server_url);
            println!();
        }
    }

    Ok(())
}
