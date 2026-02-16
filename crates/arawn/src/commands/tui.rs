//! TUI command handler.

use anyhow::Result;
use clap::Args;

use super::Context;

/// TUI command arguments.
#[derive(Args, Debug)]
pub struct TuiArgs {
    /// Workstream to open (default: from context or "default")
    #[arg(short, long)]
    pub workstream: Option<String>,
}

/// Run the TUI.
pub async fn run(args: TuiArgs, ctx: &Context) -> Result<()> {
    // Build TUI config - server URL is already resolved from context/flag
    let mut config = arawn_tui::TuiConfig::new(&ctx.server_url);

    // Try to get context name for display
    if let Ok(client_config) = arawn_config::load_client_config() {
        // Find which context matches the server URL
        if let Some(ctx) = client_config
            .contexts
            .iter()
            .find(|c| c.server == ctx.server_url)
        {
            config.context_name = Some(ctx.name.clone());

            // Use context workstream if no explicit override
            if args.workstream.is_none() {
                config.workstream = ctx
                    .workstream
                    .clone()
                    .or_else(|| Some(client_config.defaults.workstream.clone()));
            }
        }
    }

    // CLI arg overrides config
    if let Some(ws) = args.workstream {
        config.workstream = Some(ws);
    }

    arawn_tui::run_with_config(config).await
}
