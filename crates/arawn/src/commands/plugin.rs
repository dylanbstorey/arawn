//! Plugin management commands.
//!
//! Provides CLI subcommands for managing plugin subscriptions:
//! - `arawn plugin add` - Subscribe to a plugin
//! - `arawn plugin update` - Update subscribed plugins
//! - `arawn plugin remove` - Unsubscribe from a plugin
//! - `arawn plugin list` - List all plugins

use std::path::PathBuf;

use anyhow::Result;
use clap::{Args, Subcommand};

use arawn_config::PluginSubscription;
use arawn_plugin::{PluginManager, SubscriptionManager, SyncAction};

use super::Context;

/// Plugin management commands.
#[derive(Args, Debug)]
pub struct PluginArgs {
    #[command(subcommand)]
    pub command: PluginCommand,
}

#[derive(Subcommand, Debug)]
pub enum PluginCommand {
    /// Subscribe to a plugin from a git URL or GitHub shorthand
    Add(AddArgs),

    /// Update subscribed plugins (all if no name given)
    Update(UpdateArgs),

    /// Unsubscribe and remove a plugin
    Remove(RemoveArgs),

    /// List all installed plugins
    List(ListArgs),
}

/// Arguments for `arawn plugin add`.
#[derive(Args, Debug)]
pub struct AddArgs {
    /// Plugin source: GitHub shorthand (owner/repo) or full git URL
    pub source: String,

    /// Git ref (branch, tag, or commit) to checkout
    #[arg(long, short)]
    pub r#ref: Option<String>,

    /// Add to project-local config instead of global
    #[arg(long)]
    pub project: bool,
}

/// Arguments for `arawn plugin update`.
#[derive(Args, Debug)]
pub struct UpdateArgs {
    /// Plugin name to update (updates all if not specified)
    pub name: Option<String>,
}

/// Arguments for `arawn plugin remove`.
#[derive(Args, Debug)]
pub struct RemoveArgs {
    /// Plugin name or subscription ID to remove
    pub name: String,

    /// Remove from project-local config instead of global
    #[arg(long)]
    pub project: bool,

    /// Also delete cached plugin files
    #[arg(long)]
    pub delete_cache: bool,
}

/// Arguments for `arawn plugin list`.
#[derive(Args, Debug)]
pub struct ListArgs {
    /// Show only subscribed plugins (not local)
    #[arg(long)]
    pub subscribed: bool,

    /// Show only local plugins
    #[arg(long)]
    pub local: bool,
}

/// Run the plugin command.
pub async fn run(args: PluginArgs, ctx: &Context) -> Result<()> {
    match args.command {
        PluginCommand::Add(add_args) => run_add(add_args, ctx).await,
        PluginCommand::Update(update_args) => run_update(update_args, ctx).await,
        PluginCommand::Remove(remove_args) => run_remove(remove_args, ctx).await,
        PluginCommand::List(list_args) => run_list(list_args, ctx).await,
    }
}

/// Parse a source string into a PluginSubscription.
fn parse_source(source: &str, git_ref: Option<String>) -> PluginSubscription {
    // Check if it's a GitHub shorthand (owner/repo)
    if !source.contains("://") && !source.starts_with("git@") {
        // Treat as GitHub shorthand
        let mut sub = PluginSubscription::github(source);
        if let Some(r) = git_ref {
            sub = sub.with_ref(r);
        }
        sub
    } else {
        // Full URL
        let mut sub = PluginSubscription::url(source);
        if let Some(r) = git_ref {
            sub = sub.with_ref(r);
        }
        sub
    }
}

/// Run `arawn plugin add`.
async fn run_add(args: AddArgs, ctx: &Context) -> Result<()> {
    let subscription = parse_source(&args.source, args.r#ref);
    let sub_id = subscription.id();

    if ctx.verbose {
        println!("Adding plugin subscription: {}", sub_id);
        if let Some(url) = subscription.clone_url() {
            println!("  Clone URL: {}", url);
        }
        println!("  Ref: {}", subscription.effective_ref());
    }

    // Load subscription manager
    let project_dir = if args.project {
        Some(std::env::current_dir()?)
    } else {
        None
    };

    let mut manager = SubscriptionManager::new(Vec::new(), project_dir.as_deref())?;

    // Add subscription to appropriate config
    if args.project {
        manager.add_project_subscription(subscription.clone());
        manager.save_project_config()?;
        println!("Added to project config: {}", sub_id);
    } else {
        manager.add_global_subscription(subscription.clone());
        manager.save_global_config()?;
        println!("Added to global config: {}", sub_id);
    }

    // Clone the plugin
    println!("Cloning plugin...");
    let result = manager.sync_subscription(&subscription);

    match result.action {
        SyncAction::Cloned => {
            println!("Successfully cloned: {}", sub_id);
            if let Some(path) = result.path {
                println!("  Location: {}", path.display());
            }
        }
        SyncAction::Updated => {
            println!("Plugin already exists, updated: {}", sub_id);
        }
        SyncAction::Skipped => {
            println!("Skipped (local path): {}", sub_id);
        }
        SyncAction::CloneFailed | SyncAction::UpdateFailed => {
            let err = result.error.unwrap_or_else(|| "unknown error".to_string());
            eprintln!("Failed to clone plugin: {}", err);
            return Err(anyhow::anyhow!("Clone failed: {}", err));
        }
    }

    Ok(())
}

/// Run `arawn plugin update`.
async fn run_update(args: UpdateArgs, ctx: &Context) -> Result<()> {
    // Load config to get subscriptions
    let loaded = arawn_config::load_config(None)?;
    let plugins_cfg = loaded.config.plugins.clone().unwrap_or_default();

    let project_dir = std::env::current_dir().ok();
    let manager =
        SubscriptionManager::new(plugins_cfg.subscriptions.clone(), project_dir.as_deref())?;

    let subscriptions = manager.all_subscriptions();

    if subscriptions.is_empty() {
        println!("No subscribed plugins found.");
        return Ok(());
    }

    // Filter by name if specified
    let to_update: Vec<_> = if let Some(ref name) = args.name {
        subscriptions
            .into_iter()
            .filter(|s| s.id().contains(name) || s.repo.as_deref() == Some(name))
            .collect()
    } else {
        subscriptions
    };

    if to_update.is_empty() {
        if let Some(name) = args.name {
            println!("No plugin found matching: {}", name);
        }
        return Ok(());
    }

    println!("Updating {} plugin(s)...", to_update.len());

    let results = manager.sync_all_async().await;

    // Filter results to only show the ones we're updating
    let update_ids: std::collections::HashSet<_> = to_update.iter().map(|s| s.id()).collect();
    let filtered_results: Vec<_> = results
        .into_iter()
        .filter(|r| update_ids.contains(&r.subscription_id))
        .collect();

    // Print results
    let mut cloned = 0;
    let mut updated = 0;
    let mut failed = 0;
    let mut skipped = 0;

    for result in &filtered_results {
        match result.action {
            SyncAction::Cloned => {
                cloned += 1;
                if ctx.verbose {
                    println!("  Cloned: {}", result.subscription_id);
                }
            }
            SyncAction::Updated => {
                updated += 1;
                if ctx.verbose {
                    println!("  Updated: {}", result.subscription_id);
                }
            }
            SyncAction::Skipped => {
                skipped += 1;
                if ctx.verbose {
                    println!("  Skipped: {}", result.subscription_id);
                }
            }
            SyncAction::CloneFailed | SyncAction::UpdateFailed => {
                failed += 1;
                let err = result.error.as_deref().unwrap_or("unknown error");
                eprintln!("  Failed: {} - {}", result.subscription_id, err);
            }
        }
    }

    println!(
        "Update complete: {} cloned, {} updated, {} skipped, {} failed",
        cloned, updated, skipped, failed
    );

    if failed > 0 {
        return Err(anyhow::anyhow!("{} plugin(s) failed to update", failed));
    }

    Ok(())
}

/// Run `arawn plugin remove`.
async fn run_remove(args: RemoveArgs, ctx: &Context) -> Result<()> {
    let project_dir = if args.project {
        Some(std::env::current_dir()?)
    } else {
        None
    };

    let mut manager = SubscriptionManager::new(Vec::new(), project_dir.as_deref())?;

    // Find the subscription by name or ID
    let all_subs = manager.all_subscriptions();
    let matching: Vec<_> = all_subs
        .iter()
        .filter(|s| {
            s.id().contains(&args.name)
                || s.repo.as_deref() == Some(&args.name)
                || s.id() == args.name
        })
        .collect();

    if matching.is_empty() {
        println!("No plugin found matching: {}", args.name);
        return Ok(());
    }

    if matching.len() > 1 {
        println!(
            "Multiple plugins match '{}'. Please be more specific:",
            args.name
        );
        for sub in matching {
            println!("  - {}", sub.id());
        }
        return Ok(());
    }

    let sub = matching[0];
    let sub_id = sub.id();

    if ctx.verbose {
        println!("Removing plugin: {}", sub_id);
    }

    // Remove from config
    if args.project {
        manager.project_config_mut().remove_subscription(&sub_id);
        manager.save_project_config()?;
        println!("Removed from project config: {}", sub_id);
    } else {
        manager.global_config_mut().remove_subscription(&sub_id);
        manager.save_global_config()?;
        println!("Removed from global config: {}", sub_id);
    }

    // Delete cached files if requested
    if args.delete_cache {
        let cache_dir = manager.cache_dir_for(sub);
        if cache_dir.exists() {
            if ctx.verbose {
                println!("Deleting cache: {}", cache_dir.display());
            }
            std::fs::remove_dir_all(&cache_dir)?;
            println!("Deleted cached files: {}", cache_dir.display());
        }
    }

    Ok(())
}

/// Run `arawn plugin list`.
async fn run_list(args: ListArgs, ctx: &Context) -> Result<()> {
    // Load config
    let loaded = arawn_config::load_config(None)?;
    let plugins_cfg = loaded.config.plugins.clone().unwrap_or_default();

    // Get subscribed plugins
    let project_dir = std::env::current_dir().ok();
    let manager =
        SubscriptionManager::new(plugins_cfg.subscriptions.clone(), project_dir.as_deref())?;

    let subscriptions = manager.all_subscriptions();

    // Get local plugins
    let mut plugin_dirs: Vec<PathBuf> = Vec::new();
    if let Some(config_dir) = dirs::config_dir() {
        plugin_dirs.push(config_dir.join("arawn").join("plugins"));
    }
    plugin_dirs.push(PathBuf::from("./plugins"));
    plugin_dirs.extend(plugins_cfg.dirs.clone());

    let local_manager = PluginManager::new(plugin_dirs);
    let local_plugins = local_manager.load_all();

    // Print header
    if ctx.json_output {
        print_list_json(&subscriptions, &local_plugins, &manager)?;
    } else {
        print_list_table(&subscriptions, &local_plugins, &manager, &args, ctx.verbose)?;
    }

    Ok(())
}

/// Print plugin list as JSON.
fn print_list_json(
    subscriptions: &[PluginSubscription],
    local_plugins: &[arawn_plugin::LoadedPlugin],
    manager: &SubscriptionManager,
) -> Result<()> {
    use serde_json::json;

    let mut plugins = Vec::new();

    // Add subscribed plugins
    for sub in subscriptions {
        let path = manager.plugin_dir_for(sub);
        let status = if path.is_some() {
            "synced"
        } else {
            "not synced"
        };

        plugins.push(json!({
            "name": sub.id(),
            "source": match sub.source {
                arawn_config::PluginSource::GitHub => format!("github.com/{}", sub.repo.as_deref().unwrap_or("unknown")),
                arawn_config::PluginSource::Url => sub.url.as_deref().unwrap_or("unknown").to_string(),
                arawn_config::PluginSource::Local => "local".to_string(),
            },
            "ref": sub.effective_ref(),
            "status": status,
            "type": "subscribed",
            "path": path.map(|p| p.display().to_string()),
        }));
    }

    // Add local plugins
    for plugin in local_plugins {
        plugins.push(json!({
            "name": plugin.manifest.name,
            "source": "local",
            "version": plugin.manifest.version.as_deref().unwrap_or("unknown"),
            "status": "enabled",
            "type": "local",
            "path": plugin.plugin_dir.display().to_string(),
        }));
    }

    println!("{}", serde_json::to_string_pretty(&plugins)?);
    Ok(())
}

/// Print plugin list as a table.
fn print_list_table(
    subscriptions: &[PluginSubscription],
    local_plugins: &[arawn_plugin::LoadedPlugin],
    manager: &SubscriptionManager,
    args: &ListArgs,
    verbose: bool,
) -> Result<()> {
    let show_subscribed = !args.local;
    let show_local = !args.subscribed;

    let mut has_output = false;

    // Print subscribed plugins
    if show_subscribed && !subscriptions.is_empty() {
        println!("Subscribed plugins:");
        println!("{:<30} {:<15} {:<40} STATUS", "ID", "REF", "SOURCE");
        println!("{}", "-".repeat(95));

        for sub in subscriptions {
            let path = manager.plugin_dir_for(sub);
            let status = if path.is_some() {
                "synced"
            } else {
                "not synced"
            };

            let source = match sub.source {
                arawn_config::PluginSource::GitHub => {
                    format!("github.com/{}", sub.repo.as_deref().unwrap_or("unknown"))
                }
                arawn_config::PluginSource::Url => {
                    sub.url.as_deref().unwrap_or("unknown").to_string()
                }
                arawn_config::PluginSource::Local => "local".to_string(),
            };

            println!(
                "{:<30} {:<15} {:<40} {}",
                truncate(&sub.id(), 30),
                truncate(sub.effective_ref(), 15),
                truncate(&source, 40),
                status
            );

            if verbose {
                if let Some(p) = path {
                    println!("  Path: {}", p.display());
                }
            }
        }

        has_output = true;
    }

    // Print local plugins
    if show_local && !local_plugins.is_empty() {
        if has_output {
            println!();
        }

        println!("Local plugins:");
        println!("{:<30} {:<15} {:<40}", "NAME", "VERSION", "PATH");
        println!("{}", "-".repeat(85));

        for plugin in local_plugins {
            let version = plugin.manifest.version.as_deref().unwrap_or("unknown");
            println!(
                "{:<30} {:<15} {:<40}",
                truncate(&plugin.manifest.name, 30),
                truncate(version, 15),
                truncate(&plugin.plugin_dir.display().to_string(), 40)
            );
        }

        has_output = true;
    }

    if !has_output {
        println!("No plugins found.");
    }

    Ok(())
}

/// Truncate a string to a maximum length.
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
