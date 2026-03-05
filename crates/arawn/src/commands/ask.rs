//! Ask command - one-shot question to the agent.

use anyhow::Result;
use clap::Args;
use console::Style;
use std::io::Write;

use super::Context;
use crate::client::{ChatEvent, Client};

/// Arguments for the ask command.
#[derive(Args, Debug)]
#[command(after_help = "\x1b[1mExamples:\x1b[0m
  arawn ask \"Explain ownership in Rust\"
  arawn ask -s abc123 \"Follow up on that\"
  arawn ask --no-memory \"What is 2+2?\"")]
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
    let mut stream = match client
        .chat_stream(&args.prompt, args.session.as_deref())
        .await
    {
        Ok(s) => s,
        Err(e) => {
            super::print_cli_error(&e, &ctx.server_url, ctx.verbose);
            return Err(e);
        }
    };

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
                let err = anyhow::anyhow!(e);
                super::print_cli_error(&err, &ctx.server_url, ctx.verbose);
                return Err(err);
            }
        }
    }

    Ok(())
}
