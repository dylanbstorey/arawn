//! Ask command - one-shot question to the agent.

use anyhow::Result;
use clap::Args;
use console::Style;
use std::io::Write;

use super::Context;
use crate::client::{ChatEvent, Client};

/// Arguments for the ask command.
#[derive(Args, Debug)]
pub struct AskArgs {
    /// The question or prompt to send
    #[arg(required = true)]
    pub prompt: String,

    /// Continue an existing session
    #[arg(short, long)]
    pub session: Option<String>,

    /// Skip memory context
    #[arg(long)]
    pub no_memory: bool,
}

/// Run the ask command.
pub async fn run(args: AskArgs, ctx: &Context) -> Result<()> {
    let client = Client::new(&ctx.server_url)?;
    let dim = Style::new().dim();

    if ctx.verbose {
        println!(
            "{}",
            dim.apply_to(format!("Sending to: {}", ctx.server_url))
        );
        if let Some(ref session) = args.session {
            println!("{}", dim.apply_to(format!("Session: {}", session)));
        }
        println!();
    }

    // Send the chat request and stream the response
    let mut stream = client
        .chat_stream(&args.prompt, args.session.as_deref())
        .await?;

    // Track if we've printed anything (for final newline)
    let mut has_output = false;

    // Print streaming response
    while let Some(event) = stream.next_event().await {
        match event? {
            ChatEvent::Text(text) => {
                print!("{}", text);
                std::io::stdout().flush()?;
                has_output = true;
            }
            ChatEvent::ToolStart { name, .. } => {
                if has_output {
                    println!();
                }
                println!("{}", dim.apply_to(format!("[Running: {}]", name)));
                has_output = false;
            }
            ChatEvent::ToolEnd { success, .. } => {
                let status = if success { "done" } else { "failed" };
                println!("{}", dim.apply_to(format!("[{}]", status)));
            }
            ChatEvent::Done => {
                if has_output {
                    println!();
                }
            }
            ChatEvent::Error(e) => {
                let red = Style::new().red();
                eprintln!();
                eprintln!("{} {}", red.apply_to("Error:"), e);
                return Err(anyhow::anyhow!(e));
            }
        }
    }

    Ok(())
}
