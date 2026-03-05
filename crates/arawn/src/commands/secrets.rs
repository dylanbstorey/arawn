//! Secrets command - manage age-encrypted secret store.

use anyhow::Result;
use clap::{Args, Subcommand};

use super::output;

/// Arguments for the secrets command.
#[derive(Args, Debug)]
#[command(after_help = "\x1b[1mExamples:\x1b[0m
  arawn secrets set github_token    Store a secret (prompts for value)
  arawn secrets list                List stored secret names
  arawn secrets delete github_token")]
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
    let value = rpassword::prompt_password(format!("Enter value for '{}' (input hidden): ", name))?;
    let value = value.trim();

    if value.is_empty() {
        output::hint("No value provided, aborting.");
        return Ok(());
    }

    match arawn_config::secrets::store_named_secret(name, value) {
        Ok(()) => {
            output::success(format!("Secret '{}' stored in encrypted store.", name));
            output::hint(format!(
                "Use ${{{{secrets.{}}}}} in tool parameters to reference it.",
                name
            ));
        }
        Err(e) => {
            output::error(format!("Failed to store secret: {}", e));
        }
    }

    Ok(())
}

async fn cmd_list() -> Result<()> {
    match arawn_config::secrets::list_secrets() {
        Ok(names) => {
            if names.is_empty() {
                output::hint("No secrets stored.");
            } else {
                println!("Stored secrets:");
                for name in &names {
                    println!("  {}", name);
                }
                println!();
                output::hint(format!("{} secret(s) total.", names.len()));
            }
        }
        Err(e) => {
            output::error(format!("Failed to list secrets: {}", e));
        }
    }

    Ok(())
}

async fn cmd_delete(name: &str) -> Result<()> {
    match arawn_config::secrets::delete_named_secret(name) {
        Ok(()) => {
            output::success(format!("Secret '{}' deleted.", name));
        }
        Err(e) => {
            output::error(format!("Failed to delete secret: {}", e));
        }
    }

    Ok(())
}
