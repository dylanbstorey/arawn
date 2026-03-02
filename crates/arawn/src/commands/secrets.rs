//! Secrets command - manage age-encrypted secret store.

use anyhow::Result;
use clap::{Args, Subcommand};

/// Arguments for the secrets command.
#[derive(Args, Debug)]
pub struct SecretsArgs {
    #[command(subcommand)]
    pub command: SecretsCommand,
}

#[derive(Subcommand, Debug)]
pub enum SecretsCommand {
    /// Store a secret in the encrypted store
    Set {
        /// Secret name (e.g., github_token, anthropic_api_key)
        name: String,
    },

    /// List all stored secret names (never values)
    List,

    /// Delete a secret from the encrypted store
    Delete {
        /// Secret name to delete
        name: String,
    },
}

/// Run the secrets command.
pub async fn run(args: SecretsArgs) -> Result<()> {
    match args.command {
        SecretsCommand::Set { name } => cmd_set(&name).await,
        SecretsCommand::List => cmd_list().await,
        SecretsCommand::Delete { name } => cmd_delete(&name).await,
    }
}

async fn cmd_set(name: &str) -> Result<()> {
    println!("Enter value for '{}':", name);

    let mut value = String::new();
    std::io::stdin().read_line(&mut value)?;
    let value = value.trim();

    if value.is_empty() {
        println!("No value provided, aborting.");
        return Ok(());
    }

    match arawn_config::secrets::store_named_secret(name, value) {
        Ok(()) => {
            println!("Secret '{}' stored in encrypted store.", name);
            println!(
                "Use ${{{{secrets.{}}}}} in tool parameters to reference it.",
                name
            );
        }
        Err(e) => {
            eprintln!("Failed to store secret: {}", e);
        }
    }

    Ok(())
}

async fn cmd_list() -> Result<()> {
    match arawn_config::secrets::list_secrets() {
        Ok(names) => {
            if names.is_empty() {
                println!("No secrets stored.");
            } else {
                println!("Stored secrets:");
                for name in &names {
                    println!("  {}", name);
                }
                println!("\n{} secret(s) total.", names.len());
            }
        }
        Err(e) => {
            eprintln!("Failed to list secrets: {}", e);
        }
    }

    Ok(())
}

async fn cmd_delete(name: &str) -> Result<()> {
    match arawn_config::secrets::delete_named_secret(name) {
        Ok(()) => {
            println!("Secret '{}' deleted.", name);
        }
        Err(e) => {
            eprintln!("Failed to delete secret: {}", e);
        }
    }

    Ok(())
}
