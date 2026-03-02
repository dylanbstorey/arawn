//! Auth command - authentication management.

use anyhow::Result;
use clap::{Args, Subcommand};

use super::Context;

/// Arguments for the auth command.
#[derive(Args, Debug)]
pub struct AuthArgs {
    #[command(subcommand)]
    pub command: AuthCommand,
}

#[derive(Subcommand, Debug)]
pub enum AuthCommand {
    /// Authenticate with Claude MAX via OAuth
    Login,

    /// Show authentication status
    Status,

    /// Clear stored OAuth tokens
    Logout,

    /// Generate or show API token (for server auth)
    Token {
        /// Generate a new token
        #[arg(long)]
        generate: bool,
    },
}

/// Run the auth command.
pub async fn run(args: AuthArgs, ctx: &Context) -> Result<()> {
    match args.command {
        AuthCommand::Login => cmd_login(ctx).await,
        AuthCommand::Status => cmd_status(ctx).await,
        AuthCommand::Logout => cmd_logout().await,
        AuthCommand::Token { generate } => cmd_token(generate, ctx).await,
    }
}

async fn cmd_login(_ctx: &Context) -> Result<()> {
    let data_dir = arawn_config::xdg_config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;

    let token_manager = arawn_oauth::token_manager::create_token_manager(&data_dir);

    // Check if already authenticated
    if token_manager.has_tokens()
        && let Ok(Some(info)) = token_manager.get_token_info().await
        && !info.is_expired
    {
        println!(
            "Already authenticated (expires in {})",
            info.expires_in_display()
        );
        println!("Run 'arawn auth logout' first to re-authenticate.");
        return Ok(());
    }

    // Start PKCE OAuth flow
    let config = arawn_oauth::OAuthConfig::default();
    let pkce = arawn_oauth::PkceChallenge::generate();
    let state = arawn_oauth::oauth::generate_state();

    let auth_url = arawn_oauth::oauth::build_authorization_url(&config, &pkce.challenge, &state);

    println!("Claude MAX OAuth Authentication");
    println!("================================");
    println!();
    println!("Open this URL in your browser:");
    println!();
    println!("  {}", auth_url);
    println!();
    println!("After authenticating, you'll be redirected to a page.");
    println!("Copy the code#state value and paste it here:");
    println!();

    // Try to open the browser automatically
    if open_url(&auth_url).is_err() {
        println!("(Could not open browser automatically)");
        println!();
    }

    // Read the code#state from stdin
    print!("code#state> ");
    use std::io::Write;
    std::io::stdout().flush()?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let input = input.trim();

    if input.is_empty() {
        println!("No input provided, aborting.");
        return Ok(());
    }

    let (code, received_state) = arawn_oauth::oauth::parse_code_state(input)
        .map_err(|e| anyhow::anyhow!("Failed to parse response: {}", e))?;

    if received_state != state {
        return Err(anyhow::anyhow!(
            "State mismatch — possible CSRF attack. Expected state does not match."
        ));
    }

    println!("Exchanging code for tokens...");

    let tokens =
        arawn_oauth::oauth::exchange_code_for_tokens(&config, &code, &pkce.verifier, &state)
            .await
            .map_err(|e| anyhow::anyhow!("Token exchange failed: {}", e))?;

    token_manager
        .save_tokens(&tokens)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to save tokens: {}", e))?;

    println!();
    println!("Authentication successful!");
    println!("Token expires in: {} seconds", tokens.expires_in);
    println!("Scope: {}", tokens.scope);
    println!();
    println!("You can now use backend = \"claude-oauth\" in your config.");

    Ok(())
}

async fn cmd_status(_ctx: &Context) -> Result<()> {
    let data_dir = arawn_config::xdg_config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;

    let token_manager = arawn_oauth::token_manager::create_token_manager(&data_dir);

    println!("Authentication Status");
    println!("---------------------");

    // Check OAuth tokens
    if token_manager.has_tokens() {
        match token_manager.get_token_info().await {
            Ok(Some(info)) => {
                println!("OAuth: authenticated");
                println!("  Expires: {}", info.expires_in_display());
                println!("  Scope: {}", info.scope);
                if !info.created_at.is_empty() {
                    println!("  Created: {}", info.created_at);
                }
            }
            Ok(None) => {
                println!("OAuth: token file exists but could not be read");
            }
            Err(e) => {
                println!("OAuth: error reading tokens: {}", e);
            }
        }
    } else {
        println!("OAuth: not authenticated");
        println!("  Run 'arawn auth login' to authenticate with Claude MAX");
    }

    println!();

    // Check API token
    if let Ok(token) = std::env::var("ARAWN_API_TOKEN") {
        let masked = if token.len() > 8 {
            format!("{}...{}", &token[..4], &token[token.len() - 4..])
        } else {
            "****".to_string()
        };
        println!("Server token: {} (from environment)", masked);
    } else {
        println!("Server token: not set (ARAWN_API_TOKEN)");
    }

    Ok(())
}

async fn cmd_logout() -> Result<()> {
    let data_dir = arawn_config::xdg_config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;

    let token_manager = arawn_oauth::token_manager::create_token_manager(&data_dir);

    if token_manager.has_tokens() {
        token_manager
            .delete_tokens()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to delete tokens: {}", e))?;
        println!("OAuth tokens removed.");
    } else {
        println!("No OAuth tokens found.");
    }

    Ok(())
}

async fn cmd_token(generate: bool, _ctx: &Context) -> Result<()> {
    if generate {
        let token = uuid::Uuid::new_v4().to_string();
        println!("Generated token: {}", token);
        println!();
        println!("Set this as your server auth token:");
        println!("  export ARAWN_API_TOKEN={}", token);
    } else if let Ok(token) = std::env::var("ARAWN_API_TOKEN") {
        println!("{}", token);
    } else {
        eprintln!("No token configured");
        eprintln!("Use 'arawn auth token --generate' to create one");
    }

    Ok(())
}

/// Try to open a URL in the default browser.
fn open_url(url: &str) -> std::io::Result<()> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(url).status()?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open").arg(url).status()?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", url])
            .status()?;
    }
    Ok(())
}
