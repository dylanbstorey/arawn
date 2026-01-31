//! Notes command - note management.

use anyhow::Result;
use clap::{Args, Subcommand};
use console::{Style, style};

use super::Context;
use crate::client::Client;

/// Arguments for the notes command.
#[derive(Args, Debug)]
pub struct NotesArgs {
    #[command(subcommand)]
    pub command: NotesCommand,
}

#[derive(Subcommand, Debug)]
pub enum NotesCommand {
    /// Add a quick note
    Add {
        /// Note content
        content: String,

        /// Tags for the note
        #[arg(short, long)]
        tags: Vec<String>,
    },

    /// List all notes
    List {
        /// Maximum notes to show
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },

    /// Search notes
    Search {
        /// Search query
        query: String,
    },

    /// Show a specific note
    Show {
        /// Note ID
        id: String,
    },

    /// Delete a note
    Delete {
        /// Note ID
        id: String,
    },
}

/// Run the notes command.
pub async fn run(args: NotesArgs, ctx: &Context) -> Result<()> {
    let client = Client::new(&ctx.server_url)?;
    let dim = Style::new().dim();

    match args.command {
        NotesCommand::Add { content, tags } => {
            if ctx.verbose && !tags.is_empty() {
                println!("{}", dim.apply_to(format!("Tags: {:?}", tags)));
            }

            match client.create_note(&content).await {
                Ok(note) => {
                    if ctx.json_output {
                        println!("{}", serde_json::to_string_pretty(&note)?);
                    } else {
                        let green = Style::new().green();
                        println!(
                            "{} Note created: {}",
                            green.apply_to("✓"),
                            dim.apply_to(&note.id)
                        );
                    }
                }
                Err(e) => {
                    let red = Style::new().red();
                    eprintln!("{} {}", red.apply_to("Error:"), e);
                }
            }
        }
        NotesCommand::List { limit } => match client.list_notes().await {
            Ok(notes) => {
                if ctx.json_output {
                    println!("{}", serde_json::to_string_pretty(&notes)?);
                } else {
                    println!("{}", style("Notes").bold());
                    println!("{}", dim.apply_to("─".repeat(50)));
                    println!();

                    if notes.is_empty() {
                        println!("{}", dim.apply_to("No notes found"));
                    } else {
                        for note in notes.iter().take(limit) {
                            println!(
                                "{} {}",
                                dim.apply_to(format!("[{}]", &note.id[..8])),
                                truncate(&note.content, 60)
                            );
                        }
                        if notes.len() > limit {
                            println!();
                            println!(
                                "{}",
                                dim.apply_to(format!("... and {} more", notes.len() - limit))
                            );
                        }
                    }
                }
            }
            Err(e) => {
                let red = Style::new().red();
                eprintln!("{} {}", red.apply_to("Error:"), e);
            }
        },
        NotesCommand::Search { query } => {
            println!("{}", style("Note Search").bold());
            println!("{}", dim.apply_to("─".repeat(50)));
            println!();
            println!(
                "{}",
                dim.apply_to(format!("Searching: \"{}\" (not yet implemented)", query))
            );
        }
        NotesCommand::Show { id } => {
            println!("{}", style("Note Details").bold());
            println!("{}", dim.apply_to("─".repeat(50)));
            println!();
            println!(
                "{}",
                dim.apply_to(format!("Note ID: {} (not yet implemented)", id))
            );
        }
        NotesCommand::Delete { id } => {
            println!(
                "{}",
                dim.apply_to(format!("Deleting note: {} (not yet implemented)", id))
            );
        }
    }

    Ok(())
}

fn truncate(s: &str, max_len: usize) -> String {
    let s = s.replace('\n', " ");
    if s.len() <= max_len {
        s
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
