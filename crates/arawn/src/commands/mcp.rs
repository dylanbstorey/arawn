//! MCP (Model Context Protocol) server management commands.
//!
//! Provides CLI subcommands for managing MCP server connections:
//! - `arawn mcp list` - List configured MCP servers and their tools
//! - `arawn mcp add` - Add a new MCP server configuration
//! - `arawn mcp remove` - Remove an MCP server configuration
//! - `arawn mcp test` - Test connection to an MCP server

use std::time::Duration;

use anyhow::Result;
use clap::{Args, Subcommand};

use arawn_config::{load_config, save_config, McpServerEntry, McpTransportType};
use arawn_mcp::{McpClient, McpServerConfig};

use super::Context;

/// MCP server management commands.
#[derive(Args, Debug)]
pub struct McpArgs {
    #[command(subcommand)]
    pub command: McpCommand,
}

#[derive(Subcommand, Debug)]
pub enum McpCommand {
    /// List configured MCP servers and their tools
    List(ListArgs),

    /// Add a new MCP server configuration
    Add(AddArgs),

    /// Remove an MCP server configuration
    Remove(RemoveArgs),

    /// Test connection to an MCP server
    Test(TestArgs),
}

/// Arguments for `arawn mcp list`.
#[derive(Args, Debug)]
pub struct ListArgs {
    /// Show tools available from each server (requires connecting)
    #[arg(long)]
    pub tools: bool,
}

/// Arguments for `arawn mcp add`.
#[derive(Args, Debug)]
pub struct AddArgs {
    /// Unique name for this MCP server
    pub name: String,

    /// Command to spawn (for stdio transport) or URL (for http transport)
    pub target: String,

    /// Use HTTP transport instead of stdio
    #[arg(long)]
    pub http: bool,

    /// Arguments to pass to the command (stdio only)
    #[arg(last = true)]
    pub args: Vec<String>,

    /// Environment variables in KEY=VALUE format
    #[arg(long = "env", short = 'e')]
    pub env_vars: Vec<String>,

    /// HTTP header in KEY=VALUE format (http only)
    #[arg(long = "header", short = 'H')]
    pub headers: Vec<String>,

    /// Request timeout in seconds (http only)
    #[arg(long, default_value = "30")]
    pub timeout: u64,

    /// Number of retries for failed requests (http only)
    #[arg(long, default_value = "3")]
    pub retries: u32,

    /// Start server disabled (don't auto-connect on startup)
    #[arg(long)]
    pub disabled: bool,
}

/// Arguments for `arawn mcp remove`.
#[derive(Args, Debug)]
pub struct RemoveArgs {
    /// Name of the MCP server to remove
    pub name: String,
}

/// Arguments for `arawn mcp test`.
#[derive(Args, Debug)]
pub struct TestArgs {
    /// Name of the MCP server to test
    pub name: String,

    /// Show full tool schemas
    #[arg(long)]
    pub full: bool,
}

/// Run the MCP command.
pub async fn run(args: McpArgs, ctx: &Context) -> Result<()> {
    match args.command {
        McpCommand::List(list_args) => run_list(list_args, ctx).await,
        McpCommand::Add(add_args) => run_add(add_args, ctx).await,
        McpCommand::Remove(remove_args) => run_remove(remove_args, ctx).await,
        McpCommand::Test(test_args) => run_test(test_args, ctx).await,
    }
}

/// Run `arawn mcp list`.
async fn run_list(args: ListArgs, ctx: &Context) -> Result<()> {
    let loaded = load_config(None)?;
    let mcp_cfg = loaded.config.mcp.clone().unwrap_or_default();

    if mcp_cfg.servers.is_empty() {
        if ctx.json_output {
            println!("[]");
        } else {
            println!("No MCP servers configured.");
            println!();
            println!("Add a server with:");
            println!("  arawn mcp add <name> <command> [args...]");
            println!("  arawn mcp add <name> <url> --http");
        }
        return Ok(());
    }

    if ctx.json_output {
        print_list_json(&mcp_cfg.servers, args.tools)?;
    } else {
        print_list_table(&mcp_cfg.servers, args.tools, ctx.verbose)?;
    }

    Ok(())
}

/// Print server list as JSON.
fn print_list_json(servers: &[McpServerEntry], show_tools: bool) -> Result<()> {
    use serde_json::json;

    let mut output = Vec::new();

    for server in servers {
        let mut entry = json!({
            "name": server.name,
            "enabled": server.enabled,
            "transport": match server.transport {
                McpTransportType::Stdio => "stdio",
                McpTransportType::Http => "http",
            },
        });

        if server.is_http() {
            entry["url"] = json!(server.url);
        } else {
            entry["command"] = json!(server.command);
            if !server.args.is_empty() {
                entry["args"] = json!(server.args);
            }
        }

        if show_tools {
            match connect_and_list_tools(server) {
                Ok(tools) => {
                    entry["status"] = json!("connected");
                    entry["tools"] = json!(tools);
                }
                Err(e) => {
                    entry["status"] = json!("error");
                    entry["error"] = json!(e.to_string());
                }
            }
        }

        output.push(entry);
    }

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

/// Print server list as a table.
fn print_list_table(servers: &[McpServerEntry], show_tools: bool, verbose: bool) -> Result<()> {
    println!(
        "{:<20} {:<10} {:<10} {:<40}",
        "NAME", "TRANSPORT", "STATUS", "TARGET"
    );
    println!("{}", "-".repeat(80));

    for server in servers {
        let transport = match server.transport {
            McpTransportType::Stdio => "stdio",
            McpTransportType::Http => "http",
        };

        let status = if server.enabled { "enabled" } else { "disabled" };

        let target = if server.is_http() {
            server.url.clone().unwrap_or_default()
        } else {
            let mut cmd = server.command.clone();
            if !server.args.is_empty() {
                cmd.push(' ');
                cmd.push_str(&server.args.join(" "));
            }
            cmd
        };

        println!(
            "{:<20} {:<10} {:<10} {:<40}",
            truncate(&server.name, 20),
            transport,
            status,
            truncate(&target, 40)
        );

        if verbose {
            if !server.env.is_empty() {
                println!("  Environment:");
                for kv in &server.env {
                    println!("    {}={}", kv[0], kv[1]);
                }
            }
            if server.is_http() {
                if !server.headers.is_empty() {
                    println!("  Headers:");
                    for kv in &server.headers {
                        println!("    {}: {}", kv[0], kv[1]);
                    }
                }
                if let Some(timeout) = server.timeout_secs {
                    println!("  Timeout: {}s", timeout);
                }
                if let Some(retries) = server.retries {
                    println!("  Retries: {}", retries);
                }
            }
        }

        if show_tools {
            match connect_and_list_tools(server) {
                Ok(tools) => {
                    if tools.is_empty() {
                        println!("  Tools: (none)");
                    } else {
                        println!("  Tools ({}):", tools.len());
                        for tool in tools {
                            println!("    - {}", tool);
                        }
                    }
                }
                Err(e) => {
                    println!("  Error: {}", e);
                }
            }
        }
    }

    Ok(())
}

/// Connect to an MCP server and list its tools.
fn connect_and_list_tools(server: &McpServerEntry) -> Result<Vec<String>> {
    let config = server_entry_to_config(server)?;
    let mut client = McpClient::connect(config)?;

    // Initialize the connection
    let _ = client.initialize()?;

    // List tools
    let tools = client.list_tools()?;
    let tool_names: Vec<String> = tools.into_iter().map(|t| t.name).collect();

    // Shutdown gracefully
    let _ = client.shutdown();

    Ok(tool_names)
}

/// Convert a McpServerEntry to an McpServerConfig.
fn server_entry_to_config(entry: &McpServerEntry) -> Result<McpServerConfig> {
    if entry.is_http() {
        let url = entry
            .url
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("HTTP server '{}' missing URL", entry.name))?;

        let mut config = McpServerConfig::http(&entry.name, url);

        for kv in &entry.headers {
            config = config.with_header(kv[0].clone(), kv[1].clone());
        }

        if let Some(timeout) = entry.timeout_secs {
            config = config.with_timeout(Duration::from_secs(timeout));
        }

        if let Some(retries) = entry.retries {
            config = config.with_retries(retries);
        }

        Ok(config)
    } else {
        let mut config = McpServerConfig::new(&entry.name, &entry.command);
        config = config.with_args(entry.args.clone());
        config = config.with_env(entry.env_tuples());
        Ok(config)
    }
}

/// Run `arawn mcp add`.
async fn run_add(args: AddArgs, ctx: &Context) -> Result<()> {
    // Load existing config
    let loaded = load_config(None)?;
    let mut config = loaded.config.clone();
    let mut mcp_cfg = config.mcp.clone().unwrap_or_default();

    // Check if server already exists
    if mcp_cfg.servers.iter().any(|s| s.name == args.name) {
        return Err(anyhow::anyhow!(
            "MCP server '{}' already exists. Use 'arawn mcp remove {}' first.",
            args.name,
            args.name
        ));
    }

    // Parse environment variables as [key, value] pairs
    let mut env: Vec<[String; 2]> = Vec::new();
    for var in &args.env_vars {
        if let Some((key, value)) = var.split_once('=') {
            env.push([key.to_string(), value.to_string()]);
        } else {
            return Err(anyhow::anyhow!(
                "Invalid environment variable format: '{}'. Use KEY=VALUE.",
                var
            ));
        }
    }

    // Create server entry
    let entry = if args.http {
        // Parse headers as [key, value] pairs
        let mut headers: Vec<[String; 2]> = Vec::new();
        for header in &args.headers {
            if let Some((key, value)) = header.split_once('=') {
                headers.push([key.to_string(), value.to_string()]);
            } else {
                return Err(anyhow::anyhow!(
                    "Invalid header format: '{}'. Use KEY=VALUE.",
                    header
                ));
            }
        }

        McpServerEntry {
            name: args.name.clone(),
            enabled: !args.disabled,
            transport: McpTransportType::Http,
            command: String::new(),
            url: Some(args.target.clone()),
            args: Vec::new(),
            env,
            headers,
            timeout_secs: Some(args.timeout),
            retries: Some(args.retries),
        }
    } else {
        McpServerEntry {
            name: args.name.clone(),
            enabled: !args.disabled,
            transport: McpTransportType::Stdio,
            command: args.target.clone(),
            url: None,
            args: args.args.clone(),
            env,
            headers: Vec::new(),
            timeout_secs: None,
            retries: None,
        }
    };

    if ctx.verbose {
        println!("Adding MCP server: {}", args.name);
        if args.http {
            println!("  Transport: HTTP");
            println!("  URL: {}", args.target);
        } else {
            println!("  Transport: stdio");
            println!("  Command: {}", args.target);
            if !args.args.is_empty() {
                println!("  Args: {}", args.args.join(" "));
            }
        }
    }

    // Add to config
    mcp_cfg.servers.push(entry);
    config.mcp = Some(mcp_cfg);

    // Save config
    let config_path = loaded
        .source
        .map(|s| s.path)
        .unwrap_or_else(|| arawn_config::xdg_config_dir().unwrap().join("config.toml"));

    save_config(&config, &config_path)?;

    if ctx.json_output {
        use serde_json::json;
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "status": "added",
                "name": args.name,
                "config_path": config_path.display().to_string(),
            }))?
        );
    } else {
        println!("Added MCP server: {}", args.name);
        println!("Config saved to: {}", config_path.display());
        println!();
        println!("Test connection with:");
        println!("  arawn mcp test {}", args.name);
    }

    Ok(())
}

/// Run `arawn mcp remove`.
async fn run_remove(args: RemoveArgs, ctx: &Context) -> Result<()> {
    // Load existing config
    let loaded = load_config(None)?;
    let mut config = loaded.config.clone();
    let mut mcp_cfg = config.mcp.clone().unwrap_or_default();

    // Find and remove server
    let original_len = mcp_cfg.servers.len();
    mcp_cfg.servers.retain(|s| s.name != args.name);

    if mcp_cfg.servers.len() == original_len {
        return Err(anyhow::anyhow!(
            "MCP server '{}' not found. Use 'arawn mcp list' to see configured servers.",
            args.name
        ));
    }

    // Update config
    config.mcp = Some(mcp_cfg);

    // Save config
    let config_path = loaded
        .source
        .map(|s| s.path)
        .unwrap_or_else(|| arawn_config::xdg_config_dir().unwrap().join("config.toml"));

    save_config(&config, &config_path)?;

    if ctx.json_output {
        use serde_json::json;
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "status": "removed",
                "name": args.name,
                "config_path": config_path.display().to_string(),
            }))?
        );
    } else {
        println!("Removed MCP server: {}", args.name);
        println!("Config saved to: {}", config_path.display());
    }

    Ok(())
}

/// Run `arawn mcp test`.
async fn run_test(args: TestArgs, ctx: &Context) -> Result<()> {
    // Load config to find server
    let loaded = load_config(None)?;
    let mcp_cfg = loaded.config.mcp.clone().unwrap_or_default();

    let server = mcp_cfg
        .servers
        .iter()
        .find(|s| s.name == args.name)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "MCP server '{}' not found. Use 'arawn mcp list' to see configured servers.",
                args.name
            )
        })?;

    if !ctx.json_output {
        println!("Testing connection to MCP server: {}", args.name);
        if server.is_http() {
            println!("  URL: {}", server.url.as_deref().unwrap_or("(none)"));
        } else {
            println!("  Command: {}", server.command);
        }
        println!();
    }

    // Connect
    let config = server_entry_to_config(server)?;
    let mut client = match McpClient::connect(config) {
        Ok(c) => c,
        Err(e) => {
            if ctx.json_output {
                use serde_json::json;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "status": "error",
                        "phase": "connect",
                        "error": e.to_string(),
                    }))?
                );
            } else {
                println!("❌ Connection failed: {}", e);
            }
            return Err(e.into());
        }
    };

    if !ctx.json_output {
        println!("✓ Connected");
    }

    // Initialize and clone server info to avoid borrow issues
    let server_info = match client.initialize() {
        Ok(info) => (info.name.clone(), info.version.clone()),
        Err(e) => {
            if ctx.json_output {
                use serde_json::json;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "status": "error",
                        "phase": "initialize",
                        "error": e.to_string(),
                    }))?
                );
            } else {
                println!("❌ Initialization failed: {}", e);
            }
            return Err(e.into());
        }
    };

    if !ctx.json_output {
        println!("✓ Initialized: {} v{}", server_info.0, server_info.1);
    }

    // List tools
    let tools = match client.list_tools() {
        Ok(t) => t,
        Err(e) => {
            if ctx.json_output {
                use serde_json::json;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "status": "error",
                        "phase": "list_tools",
                        "error": e.to_string(),
                    }))?
                );
            } else {
                println!("❌ Failed to list tools: {}", e);
            }
            return Err(e.into());
        }
    };

    // Shutdown
    let _ = client.shutdown();

    if ctx.json_output {
        use serde_json::json;

        let tools_json: Vec<_> = tools
            .iter()
            .map(|t| {
                if args.full {
                    json!({
                        "name": t.name,
                        "description": t.description,
                        "schema": t.input_schema,
                    })
                } else {
                    json!({
                        "name": t.name,
                        "description": t.description,
                    })
                }
            })
            .collect();

        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "status": "success",
                "server": {
                    "name": server_info.0,
                    "version": server_info.1,
                },
                "tools": tools_json,
            }))?
        );
    } else {
        println!("✓ Listed {} tools", tools.len());
        println!();

        if tools.is_empty() {
            println!("No tools available.");
        } else {
            println!("Available tools:");
            for tool in &tools {
                println!("  • {}", tool.name);
                if let Some(desc) = &tool.description {
                    // Print description, wrapped to 70 chars, indented
                    let wrapped = textwrap_simple(desc, 70);
                    for line in wrapped.lines() {
                        println!("      {}", line);
                    }
                }
                if args.full {
                    println!(
                        "      Schema: {}",
                        serde_json::to_string(&tool.input_schema)?
                    );
                }
            }
        }

        println!();
        println!("✓ Connection test successful");
    }

    Ok(())
}

/// Simple text wrapping helper.
fn textwrap_simple(text: &str, max_width: usize) -> String {
    let mut result = String::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.len() + 1 + word.len() <= max_width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            result.push_str(&current_line);
            result.push('\n');
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() {
        result.push_str(&current_line);
    }

    result
}

/// Truncate a string to a maximum length.
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}
