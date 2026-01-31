//! Agent management commands.
//!
//! Provides CLI subcommands for listing and inspecting available subagents:
//! - `arawn agent list` - List all available subagents
//! - `arawn agent info <name>` - Show details for a specific agent

use std::path::PathBuf;

use anyhow::Result;
use clap::{Args, Subcommand};
use serde_json::json;

use arawn_plugin::{PluginManager, SubscriptionManager};

use super::Context;

/// Agent management commands.
#[derive(Args, Debug)]
pub struct AgentArgs {
    #[command(subcommand)]
    pub command: AgentCommand,
}

#[derive(Subcommand, Debug)]
pub enum AgentCommand {
    /// List all available subagents
    List(ListArgs),

    /// Show detailed information about a specific agent
    Info(InfoArgs),
}

/// Arguments for `arawn agent list`.
#[derive(Args, Debug)]
pub struct ListArgs {
    /// Filter agents by source plugin name
    #[arg(long)]
    pub plugin: Option<String>,
}

/// Arguments for `arawn agent info`.
#[derive(Args, Debug)]
pub struct InfoArgs {
    /// Name of the agent to show details for
    pub name: String,
}

/// Run the agent command.
pub async fn run(args: AgentArgs, ctx: &Context) -> Result<()> {
    match args.command {
        AgentCommand::List(list_args) => run_list(list_args, ctx).await,
        AgentCommand::Info(info_args) => run_info(info_args, ctx).await,
    }
}

/// Information about an agent for display.
#[derive(Debug)]
struct AgentInfo {
    name: String,
    description: String,
    tools: Vec<String>,
    source_plugin: String,
    system_prompt: Option<String>,
    max_iterations: Option<usize>,
    model: Option<String>,
}

/// Load all plugins and extract agent information.
fn load_agents() -> Result<Vec<AgentInfo>> {
    // Load config to get plugin settings
    let loaded = arawn_config::load_config(None)?;
    let plugins_cfg = loaded.config.plugins.clone().unwrap_or_default();

    // Get subscribed plugins via SubscriptionManager
    let project_dir = std::env::current_dir().ok();
    let manager =
        SubscriptionManager::new(plugins_cfg.subscriptions.clone(), project_dir.as_deref())?;

    // Build plugin directories list
    let mut plugin_dirs: Vec<PathBuf> = Vec::new();

    // Add subscribed plugin cache directories
    for sub in manager.all_subscriptions() {
        if let Some(path) = manager.plugin_dir_for(&sub) {
            plugin_dirs.push(path);
        }
    }

    // Add default plugin directories
    if let Some(config_dir) = dirs::config_dir() {
        plugin_dirs.push(config_dir.join("arawn").join("plugins"));
    }
    plugin_dirs.push(PathBuf::from("./plugins"));
    plugin_dirs.extend(plugins_cfg.dirs.clone());

    // Load all plugins
    let plugin_manager = PluginManager::new(plugin_dirs);
    let plugins = plugin_manager.load_all();

    // Extract agent information
    let mut agents = Vec::new();
    for plugin in plugins {
        let plugin_name = plugin.manifest.name.clone();

        for loaded_agent in &plugin.agent_configs {
            let config = &loaded_agent.config;

            let tools = config
                .agent
                .constraints
                .as_ref()
                .map(|c| c.tools.clone())
                .unwrap_or_default();

            let max_iterations = config
                .agent
                .constraints
                .as_ref()
                .and_then(|c| c.max_iterations);

            let system_prompt = config.agent.system_prompt.as_ref().map(|p| p.text.clone());

            agents.push(AgentInfo {
                name: config.agent.name.clone(),
                description: config.agent.description.clone(),
                tools,
                source_plugin: plugin_name.clone(),
                system_prompt,
                max_iterations,
                model: config.agent.model.clone(),
            });
        }
    }

    Ok(agents)
}

/// Run `arawn agent list`.
async fn run_list(args: ListArgs, ctx: &Context) -> Result<()> {
    let agents = load_agents()?;

    // Filter by plugin if specified
    let agents: Vec<_> = if let Some(ref plugin_filter) = args.plugin {
        agents
            .into_iter()
            .filter(|a| a.source_plugin.contains(plugin_filter))
            .collect()
    } else {
        agents
    };

    if agents.is_empty() {
        if ctx.json_output {
            println!("[]");
        } else {
            println!("No subagents available.");
            println!();
            println!("Subagents are defined in plugins. To add plugins:");
            println!("  arawn plugin add <owner/repo>");
            println!();
            println!("Or create agents in ./plugins/<plugin>/agents/<name>.md");
        }
        return Ok(());
    }

    if ctx.json_output {
        print_list_json(&agents)?;
    } else {
        print_list_table(&agents, ctx.verbose)?;
    }

    Ok(())
}

/// Print agent list as JSON.
fn print_list_json(agents: &[AgentInfo]) -> Result<()> {
    let output: Vec<_> = agents
        .iter()
        .map(|a| {
            json!({
                "name": a.name,
                "description": a.description,
                "tools": a.tools,
                "source": a.source_plugin,
                "model": a.model,
                "max_iterations": a.max_iterations,
            })
        })
        .collect();

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

/// Print agent list as a table.
fn print_list_table(agents: &[AgentInfo], verbose: bool) -> Result<()> {
    println!("Available Subagents:");
    println!();
    println!(
        "  {:<20} {:<35} {:<30} PLUGIN",
        "NAME", "DESCRIPTION", "TOOLS"
    );
    println!("  {}", "-".repeat(95));

    for agent in agents {
        let tools_str = if agent.tools.is_empty() {
            "(none)".to_string()
        } else if agent.tools.len() <= 3 {
            format!("[{}]", agent.tools.join(", "))
        } else {
            format!(
                "[{}, +{}]",
                agent.tools[..2].join(", "),
                agent.tools.len() - 2
            )
        };

        println!(
            "  {:<20} {:<35} {:<30} ({})",
            truncate(&agent.name, 20),
            truncate(&agent.description, 35),
            truncate(&tools_str, 30),
            truncate(&agent.source_plugin, 15)
        );

        if verbose {
            if let Some(model) = &agent.model {
                println!("    Model: {}", model);
            }
            if let Some(max_iter) = agent.max_iterations {
                println!("    Max iterations: {}", max_iter);
            }
        }
    }

    println!();
    println!("Use 'arawn agent info <name>' for detailed information.");

    Ok(())
}

/// Run `arawn agent info`.
async fn run_info(args: InfoArgs, ctx: &Context) -> Result<()> {
    let agents = load_agents()?;

    // Find the agent by name (case-insensitive partial match)
    let matches: Vec<_> = agents
        .iter()
        .filter(|a| {
            a.name.to_lowercase().contains(&args.name.to_lowercase())
                || a.name.eq_ignore_ascii_case(&args.name)
        })
        .collect();

    if matches.is_empty() {
        if ctx.json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "error": "Agent not found",
                    "name": args.name,
                }))?
            );
        } else {
            println!("Agent '{}' not found.", args.name);
            println!();
            println!("Use 'arawn agent list' to see available agents.");
        }
        return Ok(());
    }

    if matches.len() > 1 {
        if ctx.json_output {
            let names: Vec<_> = matches.iter().map(|a| &a.name).collect();
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "error": "Multiple agents match",
                    "matches": names,
                }))?
            );
        } else {
            println!(
                "Multiple agents match '{}'. Please be more specific:",
                args.name
            );
            for agent in matches {
                println!("  - {} ({})", agent.name, agent.source_plugin);
            }
        }
        return Ok(());
    }

    let agent = matches[0];

    if ctx.json_output {
        print_info_json(agent)?;
    } else {
        print_info_detail(agent)?;
    }

    Ok(())
}

/// Print agent info as JSON.
fn print_info_json(agent: &AgentInfo) -> Result<()> {
    let output = json!({
        "name": agent.name,
        "description": agent.description,
        "plugin": agent.source_plugin,
        "tools": agent.tools,
        "model": agent.model,
        "max_iterations": agent.max_iterations,
        "system_prompt": agent.system_prompt,
    });

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

/// Print detailed agent info.
fn print_info_detail(agent: &AgentInfo) -> Result<()> {
    println!("Agent: {}", agent.name);
    println!();
    println!("Description: {}", agent.description);
    println!("Plugin: {}", agent.source_plugin);

    if let Some(model) = &agent.model {
        println!("Model: {}", model);
    }

    if let Some(max_iter) = agent.max_iterations {
        println!("Max Iterations: {}", max_iter);
    }

    println!();
    println!("Allowed Tools:");
    if agent.tools.is_empty() {
        println!("  (none)");
    } else {
        for tool in &agent.tools {
            println!("  - {}", tool);
        }
    }

    if let Some(prompt) = &agent.system_prompt {
        println!();
        println!("System Prompt:");
        println!("  {}", truncate_multiline(prompt, 500));
    }

    Ok(())
}

/// Truncate a string to a maximum length.
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

/// Truncate a multiline string, showing first N characters.
fn truncate_multiline(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.replace('\n', "\n  ")
    } else {
        format!(
            "{}... (truncated, {} chars total)",
            s[..max_len].replace('\n', "\n  "),
            s.len()
        )
    }
}
