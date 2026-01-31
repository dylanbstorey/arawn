//! Chat command - interactive REPL mode.

use anyhow::Result;
use clap::Args;

use super::Context;
use super::repl::Repl;
use crate::client::Client;

/// Arguments for the chat command.
#[derive(Args, Debug)]
pub struct ChatArgs {
    /// Resume an existing session
    #[arg(short, long)]
    pub session: Option<String>,

    /// Force start a new session
    #[arg(short, long)]
    pub new: bool,
}

/// Run the chat command (REPL).
pub async fn run(args: ChatArgs, ctx: &Context) -> Result<()> {
    let client = Client::new(&ctx.server_url)?;

    // Determine session ID
    let session_id = if args.new { None } else { args.session };

    // Create and run the REPL
    let mut repl = Repl::new(client, session_id, ctx.verbose)?;
    repl.run().await
}
