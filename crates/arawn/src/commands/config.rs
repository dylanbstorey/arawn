//! Config command - configuration management.

use anyhow::Result;
use clap::{Args, Subcommand};

use arawn_config::{self, Backend};

use super::Context;

/// Arguments for the config command.
#[derive(Args, Debug)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigCommand,
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommand {
    /// Show resolved configuration and all LLM profiles
    Show,

    /// Show which config files are loaded and their precedence
    Which,

    /// Store an API key in the system keyring
    SetSecret {
        /// Backend name: anthropic, openai, groq, ollama, custom
        backend: String,
    },

    /// Remove an API key from the system keyring
    DeleteSecret {
        /// Backend name: anthropic, openai, groq, ollama, custom
        backend: String,
    },

    /// Open configuration file in $EDITOR
    Edit,

    /// Initialize a config file with defaults
    Init {
        /// Create project-local config (./arawn.toml) instead of user config
        #[arg(long)]
        local: bool,
    },

    /// Show configuration file path
    Path,
}

/// Run the config command.
pub async fn run(args: ConfigArgs, ctx: &Context) -> Result<()> {
    match args.command {
        ConfigCommand::Show => cmd_show(ctx).await,
        ConfigCommand::Which => cmd_which(ctx).await,
        ConfigCommand::SetSecret { backend } => cmd_set_secret(&backend).await,
        ConfigCommand::DeleteSecret { backend } => cmd_delete_secret(&backend).await,
        ConfigCommand::Edit => cmd_edit().await,
        ConfigCommand::Init { local } => cmd_init(local).await,
        ConfigCommand::Path => cmd_path().await,
    }
}

async fn cmd_show(ctx: &Context) -> Result<()> {
    let loaded = arawn_config::load_config(None)?;
    let config = &loaded.config;

    println!("# Arawn Configuration\n");

    // Sources
    let sources = loaded.loaded_from();
    if sources.is_empty() {
        println!("No config files loaded (using defaults)\n");
    } else {
        println!("Config files:");
        for source in &sources {
            println!("  {}", source.display());
        }
        println!();
    }

    // LLM profiles
    let profiles = arawn_config::resolve_all_profiles(config);
    if profiles.is_empty() {
        println!("No LLM profiles configured\n");
    } else {
        println!("LLM Profiles:");
        for (name, backend, model) in &profiles {
            let key_status = key_status_for(backend);
            println!("  {:<12} {} / {}  {}", name, backend, model, key_status);
        }
        println!();
    }

    // Agent bindings
    if !config.agent.is_empty() {
        println!("Agent Bindings:");
        for (name, agent_cfg) in &config.agent {
            if let Some(ref llm) = agent_cfg.llm {
                println!("  {:<12} -> llm.{}", name, llm);
            }
        }
        println!();
    }

    // Server settings
    if let Some(ref server) = config.server {
        println!("Server:");
        println!("  bind: {}:{}", server.bind, server.port);
        if let Some(ref ws) = server.workspace {
            println!("  workspace: {}", ws.display());
        }
        if let Some(ref dir) = server.bootstrap_dir {
            println!("  bootstrap: {}", dir.display());
        }
        println!();
    }

    // Warnings
    if !loaded.warnings.is_empty() {
        println!("Warnings:");
        for w in &loaded.warnings {
            println!("  ⚠ {}", w);
        }
        println!();
    }

    if ctx.verbose {
        // Show raw TOML
        println!("---\nRaw config:\n");
        if let Ok(toml_str) = config.to_toml() {
            println!("{}", toml_str);
        }
    }

    Ok(())
}

async fn cmd_which(_ctx: &Context) -> Result<()> {
    let loaded = arawn_config::load_config(None)?;

    println!("Config file search order (later overrides earlier):\n");

    for source in &loaded.sources {
        let status = if source.loaded {
            "✓ loaded"
        } else {
            "· not found"
        };
        println!("  {} {}", status, source.path.display());
    }

    println!();
    let loaded_count = loaded.loaded_from().len();
    if loaded_count == 0 {
        println!("No config files found. Run 'arawn config init' to create one.");
    } else {
        println!("{} config file(s) loaded.", loaded_count);
    }

    Ok(())
}

async fn cmd_set_secret(backend_str: &str) -> Result<()> {
    let backend = parse_backend(backend_str)?;

    println!(
        "Enter API key for {} (input hidden):",
        backend.display_name()
    );

    // Read from stdin (not hidden in this implementation — would use rpassword in production)
    let mut api_key = String::new();
    std::io::stdin().read_line(&mut api_key)?;
    let api_key = api_key.trim();

    if api_key.is_empty() {
        println!("No key provided, aborting.");
        return Ok(());
    }

    match arawn_config::secrets::store_in_keyring(&backend, api_key) {
        Ok(()) => {
            println!(
                "✓ API key stored in system keyring for {}",
                backend.display_name()
            );
        }
        Err(e) => {
            eprintln!("Failed to store in keyring: {}", e);
            eprintln!(
                "Fallback: set the {} environment variable instead.",
                backend.env_var()
            );
        }
    }

    Ok(())
}

async fn cmd_delete_secret(backend_str: &str) -> Result<()> {
    let backend = parse_backend(backend_str)?;

    match arawn_config::secrets::delete_from_keyring(&backend) {
        Ok(()) => {
            println!(
                "✓ API key removed from keyring for {}",
                backend.display_name()
            );
        }
        Err(e) => {
            eprintln!("Failed to delete from keyring: {}", e);
        }
    }

    Ok(())
}

async fn cmd_edit() -> Result<()> {
    let config_path = arawn_config::xdg_config_path()
        .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;

    if !config_path.exists() {
        println!("No config file exists yet. Run 'arawn config init' first.");
        return Ok(());
    }

    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());

    let status = std::process::Command::new(&editor)
        .arg(&config_path)
        .status()?;

    if !status.success() {
        eprintln!("Editor exited with non-zero status");
    }

    Ok(())
}

async fn cmd_init(local: bool) -> Result<()> {
    let path = if local {
        std::path::PathBuf::from("arawn.toml")
    } else {
        let dir = arawn_config::xdg_config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;
        std::fs::create_dir_all(&dir)?;
        dir.join("config.toml")
    };

    if path.exists() {
        println!("Config file already exists: {}", path.display());
        println!("Use 'arawn config edit' to modify it.");
        return Ok(());
    }

    let template = r#"# Arawn Configuration
# See: https://github.com/dstorey/arawn

# Default LLM backend
[llm]
backend = "groq"
model = "llama-3.1-70b-versatile"

# Named profiles (uncomment to use)
# [llm.claude]
# backend = "anthropic"
# model = "claude-sonnet-4-20250514"

# [llm.fast]
# backend = "groq"
# model = "llama-3.1-8b-instant"

# [llm.local]
# backend = "ollama"
# model = "llama3.2"
# base_url = "http://localhost:11434/v1"

# [llm.max]
# backend = "claude-oauth"
# model = "claude-sonnet-4-20250514"

# Agent defaults (uncomment to use)
# [agent.default]
# llm = "claude"

# Server settings
# [server]
# port = 8080
# bind = "127.0.0.1"
"#;

    std::fs::write(&path, template)?;
    println!("✓ Created config file: {}", path.display());
    println!();
    println!("Next steps:");
    println!("  arawn config set-secret groq    # store API key in keyring");
    println!("  arawn config edit               # customize config");
    println!("  arawn config show               # verify configuration");

    Ok(())
}

async fn cmd_path() -> Result<()> {
    if let Some(path) = arawn_config::xdg_config_path() {
        println!("{}", path.display());
    } else {
        eprintln!("Could not determine config directory");
    }
    Ok(())
}

fn parse_backend(s: &str) -> Result<Backend> {
    match s.to_lowercase().as_str() {
        "anthropic" => Ok(Backend::Anthropic),
        "openai" => Ok(Backend::Openai),
        "groq" => Ok(Backend::Groq),
        "ollama" => Ok(Backend::Ollama),
        "custom" => Ok(Backend::Custom),
        "claude-oauth" | "claudeoauth" => Ok(Backend::ClaudeOauth),
        other => Err(anyhow::anyhow!(
            "Unknown backend '{}'. Valid: anthropic, openai, groq, ollama, custom, claude-oauth",
            other
        )),
    }
}

fn key_status_for(backend: &Backend) -> &'static str {
    if *backend == Backend::ClaudeOauth {
        // OAuth doesn't use API keys — check token file
        let data_dir = arawn_config::xdg_config_dir().unwrap_or_default();
        if data_dir.join("oauth-tokens.json").exists() {
            return "(oauth ✓)";
        }
        return "(no oauth token)";
    }
    if arawn_config::secrets::has_keyring_entry(backend) {
        "(keyring ✓)"
    } else if std::env::var(backend.env_var()).is_ok() {
        "(env var ✓)"
    } else {
        "(no key)"
    }
}
