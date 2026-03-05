//! Notes command - note management.

use anyhow::Result;
use clap::{Args, Subcommand};
use console::Style;

use super::Context;
use super::output;
use crate::client::Client;

/// Arguments for the notes command.
#[derive(Args, Debug)]
#[command(after_help = "\x1b[1mExamples:\x1b[0m
  arawn notes add \"Remember to refactor auth\" -t todo -t backend
  arawn notes list
  arawn notes search \"refactor\"
  arawn notes show <id>
  arawn notes delete <id>")]
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
                output::hint(format!("Tags: {:?}", tags));
            }

            match client.create_note(&content).await {
                Ok(note) => {
                    if ctx.json_output {
                        println!("{}", serde_json::to_string_pretty(&note)?);
                    } else {
                        output::success(format!("Note created: {}", dim.apply_to(&note.id)));
                    }
                }
                Err(e) => {
                    output::error(&e);
                }
            }
        }
        NotesCommand::List { limit } => match client.list_notes().await {
            Ok(notes) => {
                if ctx.json_output {
                    println!("{}", serde_json::to_string_pretty(&notes)?);
                } else {
                    output::header("Notes");

                    if notes.is_empty() {
                        output::hint("No notes found");
                    } else {
                        for note in notes.iter().take(limit) {
                            println!(
                                "{} {}",
                                dim.apply_to(format!("[{}]", &note.id[..8])),
                                output::truncate(&note.content, 60)
                            );
                        }
                        if notes.len() > limit {
                            println!();
                            output::hint(format!("... and {} more", notes.len() - limit));
                        }
                    }
                }
            }
            Err(e) => {
                super::print_cli_error(&e, &ctx.server_url, ctx.verbose);
            }
        },
        NotesCommand::Search { query } => match client.search_notes(&query, 20).await {
            Ok(results) => {
                if ctx.json_output {
                    println!("{}", serde_json::to_string_pretty(&results)?);
                } else {
                    output::header("Note Search");

                    if results.is_empty() {
                        output::hint(format!("No notes matching \"{}\"", query));
                    } else {
                        for result in &results {
                            println!(
                                "{} {}",
                                dim.apply_to(format!("[{}]", &result.id[..8.min(result.id.len())])),
                                output::truncate(&result.content, 60)
                            );
                        }
                        println!();
                        output::hint(format!("{} result(s)", results.len()));
                    }
                }
            }
            Err(e) => {
                super::print_cli_error(&e, &ctx.server_url, ctx.verbose);
            }
        },
        NotesCommand::Show { id } => match client.get_note(&id).await {
            Ok(note) => {
                if ctx.json_output {
                    println!("{}", serde_json::to_string_pretty(&note)?);
                } else {
                    output::header("Note Details");
                    output::kv("ID", &note.id);
                    if let Some(ref title) = note.title {
                        output::kv("Title", title);
                    }
                    if !note.tags.is_empty() {
                        output::kv("Tags", note.tags.join(", "));
                    }
                    output::kv("Created", &note.created_at);
                    output::kv("Updated", &note.updated_at);
                    println!();
                    println!("{}", note.content);
                }
            }
            Err(e) => {
                super::print_cli_error(&e, &ctx.server_url, ctx.verbose);
            }
        },
        NotesCommand::Delete { id } => match client.delete_note(&id).await {
            Ok(()) => {
                if ctx.json_output {
                    println!("{}", serde_json::json!({"deleted": id}));
                } else {
                    output::success(format!("Note deleted: {}", dim.apply_to(&id)));
                }
            }
            Err(e) => {
                super::print_cli_error(&e, &ctx.server_url, ctx.verbose);
            }
        },
    }

    Ok(())
}
