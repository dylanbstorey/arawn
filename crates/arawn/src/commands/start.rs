//! Start command - launches the Arawn server.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use clap::Args;

use arawn_agent::{
    Agent, IndexerConfig, McpToolAdapter, SessionIndexer, Tool, ToolRegistry, tools,
};
#[cfg(feature = "gliner")]
use arawn_agent::{GlinerEngine, NerConfig};
use arawn_config::EmbeddingProvider;
use arawn_config::{self, Backend, LlmConfig, ResolvedLlm};
use arawn_llm::{
    AnthropicBackend, AnthropicConfig, EmbedderSpec, OpenAiBackend, OpenAiConfig, SharedBackend,
};
use arawn_mcp::{McpManager, McpServerConfig};
use arawn_memory::{MemoryStore, init_vector_extension};
use arawn_oauth;
use arawn_pipeline::sandbox::ScriptExecutor;
use arawn_pipeline::{
    CatalogEntry, PipelineConfig, PipelineEngine, RuntimeCatalog, RuntimeCategory,
};
use arawn_plugin::{HookDispatcher, PluginManager, PluginWatcher, SubscriptionManager, SyncAction};
use arawn_server::{AppState, Server, ServerConfig};
use arawn_workstream::{WorkstreamConfig as WsConfig, WorkstreamManager};
use tokio::sync::RwLock;

use super::Context;

/// Arguments for the start command.
///
/// CLI arguments override config file values.
#[derive(Args, Debug)]
pub struct StartArgs {
    /// Run server in background (daemon mode)
    #[arg(short, long)]
    pub daemon: bool,

    /// Port to listen on (overrides config)
    #[arg(short, long)]
    pub port: Option<u16>,

    /// Address to bind to (overrides config)
    #[arg(short, long)]
    pub bind: Option<String>,

    /// API token for authentication (or set ARAWN_API_TOKEN env var)
    #[arg(long, env = "ARAWN_API_TOKEN")]
    pub token: Option<String>,

    /// LLM backend (overrides config)
    #[arg(long)]
    pub backend: Option<String>,

    /// API key (overrides config and keyring)
    #[arg(long)]
    pub api_key: Option<String>,

    /// Custom base URL (overrides config)
    #[arg(long)]
    pub base_url: Option<String>,

    /// Model (overrides config)
    #[arg(long)]
    pub model: Option<String>,

    /// Working directory for file operations (overrides config)
    #[arg(long)]
    pub workspace: Option<PathBuf>,

    /// Path to directory containing bootstrap files (overrides config)
    #[arg(long)]
    pub bootstrap_dir: Option<PathBuf>,

    /// Additional prompt file to load (can be specified multiple times)
    #[arg(long)]
    pub prompt_file: Vec<PathBuf>,

    /// Path to config file (overrides default discovery)
    #[arg(long)]
    pub config: Option<PathBuf>,
}

/// Run the start command.
pub async fn run(args: StartArgs, ctx: &Context) -> Result<()> {
    if args.daemon {
        println!("Daemon mode not yet implemented");
        println!("Run without --daemon to start in foreground");
        return Ok(());
    }

    // ── Load configuration ──────────────────────────────────────────────

    let loaded = if let Some(ref config_path) = args.config {
        // Explicit config file
        let config = arawn_config::load_config_file(config_path)?;
        let source = arawn_config::discovery::ConfigSource {
            path: config_path.clone(),
            loaded: true,
        };
        arawn_config::LoadedConfig {
            config,
            sources: vec![source.clone()],
            source: Some(source),
            warnings: Vec::new(),
        }
    } else {
        arawn_config::load_config(args.workspace.as_deref())?
    };

    // Print warnings (plaintext keys, parse errors, etc.)
    for warning in &loaded.warnings {
        eprintln!("warning: {}", warning);
    }

    if ctx.verbose {
        let sources = loaded.loaded_from();
        if sources.is_empty() {
            println!("No config files found, using defaults + CLI args");
        } else {
            for source in sources {
                println!("Loaded config: {}", source.display());
            }
        }
    }

    let config = &loaded.config;

    // ── Resolve LLM backend ─────────────────────────────────────────────

    let resolved = resolve_with_cli_overrides(config, &args)?;

    if ctx.verbose {
        println!("Backend: {}", resolved.backend);
        println!("Model: {}", resolved.model);
        println!("Resolved via: {}", resolved.resolved_from);
        if let Some(ref source) = resolved.api_key_source {
            println!("API key from: {}", source);
        }
    }

    let backend = create_backend(&resolved).await?;

    // ── Create named backends from profiles ──────────────────────────────

    let mut backends: HashMap<String, SharedBackend> = HashMap::new();
    backends.insert("default".to_string(), backend.clone());

    for (name, llm_config) in &config.llm_profiles {
        match resolve_profile(name, llm_config) {
            Ok(profile_resolved) => match create_backend(&profile_resolved).await {
                Ok(profile_backend) => {
                    if ctx.verbose {
                        println!(
                            "Backend '{}': {} / {}",
                            name, profile_resolved.backend, profile_resolved.model
                        );
                    }
                    backends.insert(name.clone(), profile_backend);
                }
                Err(e) => {
                    eprintln!("warning: failed to create backend '{}': {}", name, e);
                }
            },
            Err(e) => {
                eprintln!("warning: failed to resolve profile '{}': {}", name, e);
            }
        }
    }

    if ctx.verbose && backends.len() > 1 {
        println!(
            "Available backends: {}",
            backends
                .keys()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    // ── Server settings ─────────────────────────────────────────────────

    let server_cfg = config.server.as_ref();
    let port = args
        .port
        .or_else(|| server_cfg.map(|s| s.port))
        .unwrap_or(8080);
    let bind = args
        .bind
        .clone()
        .or_else(|| server_cfg.map(|s| s.bind.clone()))
        .unwrap_or_else(|| "127.0.0.1".to_string());
    let addr: SocketAddr = format!("{}:{}", bind, port).parse()?;

    let workspace = args
        .workspace
        .clone()
        .or_else(|| server_cfg.and_then(|s| s.workspace.clone()));
    let bootstrap_dir = args
        .bootstrap_dir
        .clone()
        .or_else(|| server_cfg.and_then(|s| s.bootstrap_dir.clone()));

    // ── Auth token ──────────────────────────────────────────────────────

    let explicit_token = args.token.or_else(|| std::env::var("ARAWN_API_TOKEN").ok());

    let auth_token: Option<String> = if let Some(token) = explicit_token {
        // Explicit token always wins
        Some(token)
    } else if addr.ip().is_loopback() {
        // Localhost — no auth needed
        None
    } else {
        // Non-loopback — load or generate a persisted token
        let token = load_or_generate_server_token()?;
        println!("Server auth token: {}", token);
        Some(token)
    };

    if ctx.verbose {
        println!("Bind address: {}", addr);
        match &auth_token {
            Some(t) => println!("Auth token: {}...", &t[..8.min(t.len())]),
            None => println!("Auth: disabled (localhost)"),
        }
    }

    // ── Build embedder ────────────────────────────────────────────────────

    let embedding_config = config.embedding.clone().unwrap_or_default();
    let embedder_spec = build_embedder_spec(&embedding_config);

    if ctx.verbose {
        println!(
            "Embedding provider: {:?} ({}d)",
            embedding_config.provider,
            embedding_config.effective_dimensions()
        );
    }

    let embedder = arawn_llm::build_embedder(&embedder_spec)?;

    if ctx.verbose {
        println!("Embedder: {} ({}d)", embedder.name(), embedder.dimensions());
    }

    // ── Pipeline engine ────────────────────────────────────────────────

    let pipeline_cfg = config.pipeline.clone().unwrap_or_default();
    let data_dir = arawn_config::xdg_config_dir().unwrap_or_else(|| PathBuf::from("."));

    let resolve_path = |p: Option<PathBuf>, default: &str| -> PathBuf {
        let p = p.unwrap_or_else(|| PathBuf::from(default));
        if p.is_relative() { data_dir.join(p) } else { p }
    };

    let pipeline_db_path = resolve_path(pipeline_cfg.database.clone(), "pipeline.db");
    let pipeline_workflow_dir = resolve_path(pipeline_cfg.workflow_dir.clone(), "workflows");

    let pipeline_engine: Option<Arc<PipelineEngine>> = if pipeline_cfg.enabled {
        let engine_config = PipelineConfig {
            max_concurrent_tasks: pipeline_cfg.max_concurrent_tasks,
            task_timeout_secs: pipeline_cfg.task_timeout_secs,
            pipeline_timeout_secs: pipeline_cfg.pipeline_timeout_secs,
            cron_enabled: pipeline_cfg.cron_enabled,
            triggers_enabled: pipeline_cfg.triggers_enabled,
        };

        // Ensure workflow directory exists
        if let Err(e) = std::fs::create_dir_all(&pipeline_workflow_dir) {
            eprintln!("warning: failed to create workflow directory: {}", e);
        }

        match PipelineEngine::new(&pipeline_db_path, engine_config).await {
            Ok(engine) => {
                let engine = Arc::new(engine);

                if ctx.verbose {
                    println!(
                        "Pipeline engine: enabled (db: {}, workflows: {})",
                        pipeline_db_path.display(),
                        pipeline_workflow_dir.display(),
                    );
                }

                Some(engine)
            }
            Err(e) => {
                eprintln!("warning: failed to start pipeline engine: {}", e);
                None
            }
        }
    } else {
        if ctx.verbose {
            println!("Pipeline engine: disabled");
        }
        None
    };

    // ── Build agent ─────────────────────────────────────────────────────

    let mut tool_registry = ToolRegistry::new();
    tool_registry.register(tools::ShellTool::new());
    tool_registry.register(tools::FileReadTool::new());
    tool_registry.register(tools::FileWriteTool::new());
    tool_registry.register(tools::GlobTool::new());
    tool_registry.register(tools::GrepTool::new());
    tool_registry.register(tools::WebFetchTool::new());
    tool_registry.register(tools::NoteTool::new());

    // Register workflow tool if pipeline is enabled.
    // Uses a labeled block so fallback failures skip tool registration
    // without aborting the entire server startup.
    if let Some(ref engine) = pipeline_engine {
        let pipeline_tools: Option<(Arc<ScriptExecutor>, Arc<RwLock<RuntimeCatalog>>)> = 'pipeline: {
            let runtimes_dir = resolve_path(None, "runtimes");
            let catalog = match RuntimeCatalog::load(&runtimes_dir) {
                Ok(c) => {
                    if ctx.verbose {
                        println!("Runtime catalog: {}", runtimes_dir.display());
                    }
                    Arc::new(RwLock::new(c))
                }
                Err(e) => {
                    eprintln!(
                        "warning: failed to load runtime catalog at {}: {}",
                        runtimes_dir.display(),
                        e
                    );
                    let fallback = std::env::temp_dir().join("arawn-runtimes");
                    match RuntimeCatalog::load(&fallback) {
                        Ok(c) => {
                            eprintln!("warning: using fallback catalog at {}", fallback.display());
                            Arc::new(RwLock::new(c))
                        }
                        Err(e2) => {
                            eprintln!("error: failed to create fallback catalog: {}", e2);
                            eprintln!("warning: pipeline tools will not be available");
                            break 'pipeline None;
                        }
                    }
                }
            };

            let cache_dir = data_dir.join("wasm-cache");
            let executor = match ScriptExecutor::new(
                cache_dir.clone(),
                std::time::Duration::from_secs(pipeline_cfg.task_timeout_secs),
            ) {
                Ok(e) => {
                    if ctx.verbose {
                        println!("Script executor: cache at {}", cache_dir.display());
                    }
                    Arc::new(e)
                }
                Err(e) => {
                    eprintln!("warning: failed to create script executor: {}", e);
                    let fallback_cache = std::env::temp_dir().join("arawn-wasm-cache");
                    match ScriptExecutor::new(fallback_cache, std::time::Duration::from_secs(30)) {
                        Ok(e2) => {
                            eprintln!("warning: using fallback WASM cache");
                            Arc::new(e2)
                        }
                        Err(e2) => {
                            eprintln!("error: failed to create fallback executor: {}", e2);
                            eprintln!("warning: pipeline tools will not be available");
                            break 'pipeline None;
                        }
                    }
                }
            };

            Some((executor, catalog))
        };

        if let Some((executor, catalog)) = pipeline_tools {
            // Auto-compile and register built-in WASM runtimes
            let runtimes_src_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .ancestors()
                .nth(2)
                .map(|p| p.join("runtimes"));

            if let Some(ref src_dir) = runtimes_src_dir {
                if src_dir.is_dir() {
                    register_builtin_runtimes(src_dir, &executor, &catalog, ctx.verbose).await;
                }
            }

            tool_registry.register(tools::CatalogTool::new(catalog.clone(), executor.clone()));
            tool_registry.register(tools::WorkflowTool::new(
                engine.clone(),
                pipeline_workflow_dir.clone(),
                executor,
                catalog,
            ));
        }
    }

    // ── Plugin system ────────────────────────────────────────────────────

    let plugins_cfg = config.plugins.clone().unwrap_or_default();
    let plugin_prompts: Vec<(String, String)> = Vec::new();
    let mut hook_dispatcher = HookDispatcher::new();
    // Collect agent configs from plugins for delegate tool
    let mut plugin_agent_configs: HashMap<String, arawn_plugin::PluginAgentConfig> = HashMap::new();
    let mut plugin_agent_sources: HashMap<String, String> = HashMap::new();
    let _watcher_handle: Option<arawn_plugin::WatcherHandle> = if plugins_cfg.enabled {
        // Build plugin directories: defaults + any user-configured dirs
        let mut plugin_dirs: Vec<PathBuf> = Vec::new();
        if let Some(config_dir) = dirs::config_dir() {
            plugin_dirs.push(config_dir.join("arawn").join("plugins"));
        }
        plugin_dirs.push(PathBuf::from("./plugins"));
        plugin_dirs.extend(plugins_cfg.dirs.clone());

        // Sync subscribed plugins (clone/update from git)
        if !plugins_cfg.subscriptions.is_empty() {
            let workspace_dir = workspace.as_deref();
            match SubscriptionManager::new(plugins_cfg.subscriptions.clone(), workspace_dir) {
                Ok(sub_manager) => {
                    // Check if auto-update is enabled
                    let should_update =
                        plugins_cfg.auto_update && !SubscriptionManager::is_auto_update_disabled();

                    if should_update {
                        if ctx.verbose {
                            println!(
                                "Syncing {} subscribed plugin(s)...",
                                sub_manager.all_subscriptions().len()
                            );
                        }

                        let results = sub_manager.sync_all_async().await;

                        // Log results
                        for result in &results {
                            match result.action {
                                SyncAction::Cloned => {
                                    if ctx.verbose {
                                        println!("  Cloned: {}", result.subscription_id);
                                    }
                                }
                                SyncAction::Updated => {
                                    if ctx.verbose {
                                        println!("  Updated: {}", result.subscription_id);
                                    }
                                }
                                SyncAction::Skipped => {
                                    // Silent unless verbose
                                    if ctx.verbose {
                                        println!("  Skipped: {}", result.subscription_id);
                                    }
                                }
                                SyncAction::CloneFailed | SyncAction::UpdateFailed => {
                                    let err = result.error.as_deref().unwrap_or("unknown error");
                                    eprintln!(
                                        "warning: {} {}: {}",
                                        result.action, result.subscription_id, err
                                    );
                                }
                            }
                        }

                        // Summary
                        let cloned = results
                            .iter()
                            .filter(|r| r.action == SyncAction::Cloned)
                            .count();
                        let updated = results
                            .iter()
                            .filter(|r| r.action == SyncAction::Updated)
                            .count();
                        let failed = results.iter().filter(|r| r.is_failure()).count();

                        if ctx.verbose || failed > 0 {
                            println!(
                                "Plugin sync: {} cloned, {} updated, {} failed",
                                cloned, updated, failed
                            );
                        }
                    } else if ctx.verbose {
                        println!("Plugin auto-update: disabled");
                    }

                    // Add synced plugin directories
                    plugin_dirs.extend(sub_manager.plugin_dirs());
                }
                Err(e) => {
                    eprintln!("warning: failed to load plugin subscriptions: {}", e);
                }
            }
        }

        let manager = PluginManager::new(plugin_dirs);
        let watcher = PluginWatcher::new(manager);
        let _events = watcher.load_initial().await;

        // Register plugin CLI tools, hooks, and collect prompt fragments
        {
            let state = watcher.state();
            let st = state.read().await;

            for plugin in st.plugins() {
                // Register hooks from this plugin
                if let Some(ref hooks_config) = plugin.hooks_config {
                    hook_dispatcher.register_from_config(hooks_config, &plugin.plugin_dir);
                    if ctx.verbose {
                        let hook_count =
                            hooks_config.hooks.values().map(|v| v.len()).sum::<usize>();
                        if hook_count > 0 {
                            println!(
                                "  Plugin '{}': {} hook(s) registered",
                                plugin.manifest.name, hook_count
                            );
                        }
                    }
                }

                // Collect agent configs for delegate tool
                for loaded_agent in &plugin.agent_configs {
                    plugin_agent_configs.insert(
                        loaded_agent.config.agent.name.clone(),
                        loaded_agent.config.clone(),
                    );
                    plugin_agent_sources.insert(
                        loaded_agent.config.agent.name.clone(),
                        plugin.manifest.name.clone(),
                    );
                    if ctx.verbose {
                        println!(
                            "  Plugin '{}': agent '{}' registered",
                            plugin.manifest.name, loaded_agent.config.agent.name
                        );
                    }
                }

                // Note: CLI tools (commands/) and prompt fragments are not yet implemented
                // in the Claude plugin format migration. Those will be handled in future tasks.
            }

            if ctx.verbose || !st.is_empty() {
                println!(
                    "Plugins: {} loaded ({})",
                    st.len(),
                    st.plugins()
                        .iter()
                        .map(|p| p.manifest.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }

            if !hook_dispatcher.is_empty() && ctx.verbose {
                println!("Hooks: {} total registered", hook_dispatcher.len());
            }
        }

        // Start hot-reload watcher if enabled
        if plugins_cfg.hot_reload {
            match watcher.watch() {
                Ok((mut rx, handle)) => {
                    // Spawn background task to log reload events
                    tokio::spawn(async move {
                        while let Some(event) = rx.recv().await {
                            match event {
                                arawn_plugin::PluginEvent::Reloaded { name, .. } => {
                                    tracing::info!(plugin = %name, "plugin reloaded");
                                }
                                arawn_plugin::PluginEvent::Removed { name, .. } => {
                                    tracing::info!(plugin = %name, "plugin removed");
                                }
                                arawn_plugin::PluginEvent::Error { plugin_dir, error } => {
                                    tracing::warn!(
                                        dir = %plugin_dir.display(),
                                        error = %error,
                                        "plugin reload failed"
                                    );
                                }
                            }
                        }
                    });
                    Some(handle)
                }
                Err(e) => {
                    eprintln!("warning: failed to start plugin watcher: {}", e);
                    None
                }
            }
        } else {
            if ctx.verbose {
                println!("Plugin hot-reload: disabled");
            }
            None
        }
    } else {
        if ctx.verbose {
            println!("Plugin system: disabled");
        }
        None
    };

    // ── MCP (Model Context Protocol) servers ────────────────────────────────

    let mcp_cfg = config.mcp.clone().unwrap_or_default();

    // Create MCP manager if enabled (allows runtime additions via API even with no initial servers)
    let mut mcp_manager: Option<McpManager> = if mcp_cfg.enabled {
        Some(McpManager::new())
    } else {
        if ctx.verbose {
            println!("MCP: disabled");
        }
        None
    };

    // Connect pre-configured servers if any
    if let Some(ref mut manager) = mcp_manager {
        if !mcp_cfg.servers.is_empty() {
            // Convert config entries to McpServerConfig
            let enabled_servers: Vec<McpServerConfig> = mcp_cfg
                .servers
                .iter()
                .filter(|s| s.enabled)
                .filter_map(|entry| {
                    if entry.is_http() {
                        // HTTP transport
                        let url = match &entry.url {
                            Some(u) => u.clone(),
                            None => {
                                eprintln!(
                                    "warning: MCP server '{}' is HTTP but has no URL, skipping",
                                    entry.name
                                );
                                return None;
                            }
                        };
                        let mut config = McpServerConfig::http(&entry.name, &url);
                        for (k, v) in entry.header_tuples() {
                            config = config.with_header(k, v);
                        }
                        if let Some(timeout) = entry.timeout_secs {
                            config = config.with_timeout(std::time::Duration::from_secs(timeout));
                        }
                        if let Some(retries) = entry.retries {
                            config = config.with_retries(retries);
                        }
                        Some(config)
                    } else {
                        // Stdio transport
                        Some(
                            McpServerConfig::new(&entry.name, &entry.command)
                                .with_args(entry.args.clone())
                                .with_env(entry.env_tuples()),
                        )
                    }
                })
                .collect();

            if enabled_servers.is_empty() {
                if ctx.verbose {
                    println!("MCP: enabled (no servers configured)");
                }
            } else {
                // Add configs to manager
                for config in enabled_servers {
                    manager.add_server(config);
                }

                if ctx.verbose {
                    println!("MCP: connecting to {} server(s)...", manager.config_count());
                }

                match manager.connect_all() {
                    Ok(connected) => {
                        if connected > 0 {
                            // Register MCP tools in the tool registry
                            match manager.list_all_tools() {
                                Ok(all_tools) => {
                                    let mut total_tools = 0;
                                    for server_name in all_tools.keys() {
                                        if let Some(client) = manager.get_client(server_name) {
                                            match McpToolAdapter::from_client(client) {
                                                Ok(adapters) => {
                                                    for adapter in adapters {
                                                        if ctx.verbose {
                                                            println!(
                                                                "  Registered: {}",
                                                                adapter.name()
                                                            );
                                                        }
                                                        tool_registry.register(adapter);
                                                        total_tools += 1;
                                                    }
                                                }
                                                Err(e) => {
                                                    eprintln!(
                                                        "warning: failed to create adapters for {}: {}",
                                                        server_name, e
                                                    );
                                                }
                                            }
                                        }
                                    }

                                    println!(
                                        "MCP: {} server(s) connected, {} tool(s) registered",
                                        connected, total_tools
                                    );
                                }
                                Err(e) => {
                                    eprintln!("warning: failed to list MCP tools: {}", e);
                                }
                            }
                        } else {
                            eprintln!("warning: no MCP servers could be connected");
                        }
                    }
                    Err(e) => {
                        eprintln!("warning: MCP connection failed: {}", e);
                    }
                }
            }
        } else {
            println!("MCP: enabled (no servers configured)");
        }
    };

    // ── Hook dispatcher (shared between agent and subagent spawner) ─────────

    // Create the shared hook dispatcher early so it can be used by both
    // the agent and the subagent spawner for background execution events
    let shared_hook_dispatcher: Option<arawn_types::SharedHookDispatcher> =
        if !hook_dispatcher.is_empty() {
            Some(Arc::new(hook_dispatcher))
        } else {
            None
        };

    // ── Delegate tool (subagent delegation) ────────────────────────────────

    // Create delegate tool if any plugin agents are defined
    if !plugin_agent_configs.is_empty() {
        let parent_tools = Arc::new(tool_registry);
        let mut spawner = arawn_plugin::PluginSubagentSpawner::with_sources(
            parent_tools.clone(),
            backend.clone(),
            plugin_agent_configs,
            plugin_agent_sources,
        );

        // Wire hook dispatcher for background subagent events
        if let Some(ref dispatcher) = shared_hook_dispatcher {
            spawner = spawner.with_hook_dispatcher(dispatcher.clone());
        }

        // Create a new mutable registry and copy tools from the Arc'd one
        let mut new_registry = ToolRegistry::new();
        for name in parent_tools.names() {
            if let Some(tool) = parent_tools.get(name) {
                new_registry.register_arc(tool);
            }
        }
        new_registry.register(tools::DelegateTool::new(Arc::new(spawner)));

        if ctx.verbose {
            println!(
                "Delegate tool: {} subagent(s) available",
                new_registry
                    .names()
                    .iter()
                    .filter(|n| *n != &"delegate")
                    .count()
            );
        }

        tool_registry = new_registry;
    }

    let mut builder = Agent::builder()
        .with_shared_backend(backend)
        .with_tools(tool_registry)
        .with_plugin_prompts(plugin_prompts)
        .with_model(&resolved.model);

    // Wire up hook dispatcher to the agent
    if let Some(ref dispatcher) = shared_hook_dispatcher {
        builder = builder.with_hook_dispatcher(dispatcher.clone());
    }

    if let Some(ref ws) = workspace {
        if ctx.verbose {
            println!("Workspace: {}", ws.display());
        }
        builder = builder.with_workspace(ws);
    }

    if let Some(ref dir) = bootstrap_dir {
        if ctx.verbose {
            println!("Loading bootstrap files from: {}", dir.display());
        }
        builder = builder.with_bootstrap_dir(dir);
    }

    for file in &args.prompt_file {
        if ctx.verbose {
            println!("Loading prompt file: {}", file.display());
        }
        builder = builder.with_prompt_file(file);
    }

    let agent = builder.build()?;

    // ── Session indexer ──────────────────────────────────────────────────

    let memory_cfg = config.memory.clone().unwrap_or_default();
    let indexer: Option<SessionIndexer> = if memory_cfg.indexing.enabled {
        let memory_db_path = memory_cfg
            .database
            .clone()
            .map(|p| if p.is_relative() { data_dir.join(p) } else { p })
            .unwrap_or_else(|| data_dir.join("memory.db"));

        init_vector_extension();
        match MemoryStore::open(&memory_db_path) {
            Ok(mut store) => {
                let graph_db_path = memory_db_path.with_extension("graph.db");
                if let Err(e) = store.init_graph_at_path(&graph_db_path) {
                    eprintln!("warning: failed to init knowledge graph: {}", e);
                }
                if let Err(e) = store.init_vectors(embedder.dimensions(), embedder.name()) {
                    eprintln!("warning: failed to init vector store: {}", e);
                }
                let store = Arc::new(store);

                // Resolve the indexing LLM backend
                let indexing_backend_name = &memory_cfg.indexing.backend;
                let indexing_backend = backends
                    .get(indexing_backend_name)
                    .or_else(|| backends.get("default"))
                    .cloned();

                match indexing_backend {
                    Some(ib) => {
                        let indexer_config = IndexerConfig {
                            model: memory_cfg.indexing.model.clone(),
                            ..Default::default()
                        };

                        #[allow(unused_mut)]
                        let mut idx = SessionIndexer::with_backend(
                            store,
                            ib,
                            Some(embedder.clone()),
                            indexer_config,
                        );

                        // Optionally attach GLiNER NER engine
                        #[cfg(feature = "gliner")]
                        if let Some(ref model_path) = memory_cfg.indexing.ner_model_path {
                            let ner_config = NerConfig {
                                model_path: model_path.clone(),
                                tokenizer_path: memory_cfg
                                    .indexing
                                    .ner_tokenizer_path
                                    .clone()
                                    .unwrap_or_else(|| {
                                        // Default: tokenizer.json next to model
                                        std::path::Path::new(model_path)
                                            .parent()
                                            .map(|p| {
                                                p.join("tokenizer.json").to_string_lossy().into()
                                            })
                                            .unwrap_or_else(|| "tokenizer.json".to_string())
                                    }),
                                threshold: memory_cfg.indexing.ner_threshold,
                            };
                            match GlinerEngine::new(&ner_config) {
                                Ok(engine) => {
                                    idx = idx.with_ner_engine(Arc::new(engine));
                                    if ctx.verbose {
                                        println!("NER engine: GLiNER ({})", model_path);
                                    }
                                }
                                Err(e) => {
                                    eprintln!("warning: failed to load GLiNER model: {}", e);
                                }
                            }
                        }

                        if ctx.verbose {
                            println!(
                                "Session indexer: enabled (backend={}, model={}, db={})",
                                indexing_backend_name,
                                memory_cfg.indexing.model,
                                memory_db_path.display(),
                            );
                        }

                        Some(idx)
                    }
                    None => {
                        eprintln!(
                            "warning: indexing backend '{}' not found, indexer disabled",
                            indexing_backend_name
                        );
                        None
                    }
                }
            }
            Err(e) => {
                eprintln!(
                    "warning: failed to open memory store: {}, indexer disabled",
                    e
                );
                None
            }
        }
    } else {
        if ctx.verbose {
            println!("Session indexer: disabled");
        }
        None
    };

    // ── Start server ────────────────────────────────────────────────────

    let server_config = ServerConfig::new(auth_token)
        .with_bind_address(addr)
        .with_rate_limiting(true)
        .with_request_logging(true);

    let mut app_state = AppState::new(agent, server_config);
    if let Some(idx) = indexer {
        app_state = app_state.with_indexer(idx);
    }
    if let Some(dispatcher) = shared_hook_dispatcher {
        app_state = app_state.with_hook_dispatcher(dispatcher);
    }
    if let Some(manager) = mcp_manager.take() {
        app_state = app_state.with_mcp_manager(manager);
    }

    // ── Workstream manager ────────────────────────────────────────────────
    let ws_cfg = config.workstream.clone().unwrap_or_default();
    let ws_config = WsConfig {
        db_path: ws_cfg
            .database
            .map(|p| if p.is_relative() { data_dir.join(p) } else { p })
            .unwrap_or_else(|| data_dir.join("workstreams.db")),
        data_dir: ws_cfg
            .data_dir
            .map(|p| if p.is_relative() { data_dir.join(p) } else { p })
            .unwrap_or_else(|| data_dir.join("workstreams")),
        session_timeout_minutes: ws_cfg.session_timeout_minutes,
    };

    match WorkstreamManager::new(&ws_config) {
        Ok(mgr) => {
            app_state = app_state.with_workstreams(mgr);
            if ctx.verbose {
                println!(
                    "Workstreams: db={}, data={}",
                    ws_config.db_path.display(),
                    ws_config.data_dir.display()
                );
            }
        }
        Err(e) => {
            eprintln!("warning: failed to init workstreams: {}", e);
        }
    }

    let server = Server::from_state(app_state);

    println!("Arawn server starting on http://{}", addr);
    println!("Press Ctrl+C to stop");

    server.run().await?;

    // ── Graceful shutdown ──────────────────────────────────────────────

    if let Some(engine) = pipeline_engine {
        if let Ok(engine) = Arc::try_unwrap(engine) {
            if let Err(e) = engine.shutdown().await {
                eprintln!("warning: pipeline shutdown error: {}", e);
            }
        }
    }

    // Shutdown MCP servers
    if let Some(ref mut manager) = mcp_manager {
        if ctx.verbose {
            println!("Shutting down MCP servers...");
        }
        if let Err(e) = manager.shutdown_all() {
            eprintln!("warning: MCP shutdown error: {}", e);
        }
    }

    Ok(())
}

/// Resolve LLM config, applying CLI overrides on top of config file values.
fn resolve_with_cli_overrides(
    config: &arawn_config::ArawnConfig,
    args: &StartArgs,
) -> Result<ResolvedLlm> {
    // Try config-based resolution first
    let mut resolved = match arawn_config::resolve_for_agent(config, "default") {
        Ok(r) => r,
        Err(_) => {
            // No config — build from CLI args or fail
            let backend_str = args.backend.as_deref().unwrap_or("anthropic");
            let backend = parse_backend(backend_str)?;
            let model = args
                .model
                .clone()
                .unwrap_or_else(|| default_model(&backend));

            let api_key = args.api_key.clone().or_else(|| {
                let resolved = arawn_config::secrets::resolve_api_key(&backend, None);
                resolved.map(|r| r.value)
            });

            ResolvedLlm {
                backend,
                model,
                base_url: args.base_url.clone(),
                api_key,
                api_key_source: None,
                resolved_from: arawn_config::ResolvedFrom::GlobalDefault,
            }
        }
    };

    // CLI overrides
    if let Some(ref backend_str) = args.backend {
        resolved.backend = parse_backend(backend_str)?;
    }
    if let Some(ref model) = args.model {
        resolved.model = model.clone();
    }
    if let Some(ref base_url) = args.base_url {
        resolved.base_url = Some(base_url.clone());
    }
    if let Some(ref api_key) = args.api_key {
        resolved.api_key = Some(api_key.clone());
        resolved.api_key_source = None; // CLI override, no tracked source
    }

    Ok(resolved)
}

/// Create an LLM backend from a resolved config.
async fn create_backend(resolved: &ResolvedLlm) -> Result<SharedBackend> {
    match resolved.backend {
        Backend::Anthropic => {
            let api_key = resolved.api_key.as_deref().ok_or_else(|| {
                anyhow::anyhow!(
                    "Anthropic API key required. Use 'arawn config set-secret anthropic', \
                     set ANTHROPIC_API_KEY, or add api_key to config"
                )
            })?;
            let config = AnthropicConfig::new(api_key);
            Ok(Arc::new(AnthropicBackend::new(config)?))
        }
        Backend::Openai => {
            let api_key = resolved.api_key.as_deref().ok_or_else(|| {
                anyhow::anyhow!(
                    "OpenAI API key required. Use 'arawn config set-secret openai', \
                     set OPENAI_API_KEY, or add api_key to config"
                )
            })?;
            let mut config = OpenAiConfig::openai(api_key);
            if let Some(ref base_url) = resolved.base_url {
                config = config.with_base_url(base_url);
            }
            config = config.with_model(&resolved.model);
            Ok(Arc::new(OpenAiBackend::new(config)?))
        }
        Backend::Groq => {
            let api_key = resolved.api_key.as_deref().ok_or_else(|| {
                anyhow::anyhow!(
                    "Groq API key required. Use 'arawn config set-secret groq', \
                     set GROQ_API_KEY, or add api_key to config"
                )
            })?;
            let mut config = OpenAiConfig::groq(api_key);
            config = config.with_model(&resolved.model);
            Ok(Arc::new(OpenAiBackend::new(config)?))
        }
        Backend::Ollama => {
            let mut config = OpenAiConfig::ollama();
            if let Some(ref base_url) = resolved.base_url {
                config = config.with_base_url(base_url);
            }
            config = config.with_model(&resolved.model);
            Ok(Arc::new(OpenAiBackend::new(config)?))
        }
        Backend::Custom => {
            let base_url = resolved.base_url.as_deref().ok_or_else(|| {
                anyhow::anyhow!("Custom backend requires base_url in config or --base-url")
            })?;
            let mut config = OpenAiConfig::openai("")
                .with_base_url(base_url)
                .with_name("custom")
                .with_model(&resolved.model);
            if let Some(ref api_key) = resolved.api_key {
                config.api_key = Some(api_key.clone());
            } else {
                config.api_key = None;
            }
            Ok(Arc::new(OpenAiBackend::new(config)?))
        }
        Backend::ClaudeOauth => {
            // Start the OAuth proxy on a random port, then point AnthropicBackend at it
            let data_dir = arawn_config::xdg_config_dir()
                .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;

            let token_manager = arawn_oauth::token_manager::create_token_manager(&data_dir);
            if !token_manager.has_tokens() {
                return Err(anyhow::anyhow!(
                    "No OAuth tokens found. Run 'arawn auth login' first to authenticate."
                ));
            }

            let proxy_config =
                arawn_oauth::ProxyConfig::default().with_token_manager(token_manager);

            let proxy = arawn_oauth::ProxyServer::new(proxy_config);
            let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
            let proxy_addr = proxy
                .run_with_shutdown(async {
                    shutdown_rx.await.ok();
                })
                .await
                .map_err(|e| anyhow::anyhow!("Failed to start OAuth proxy: {}", e))?;

            // Leak the shutdown sender so the proxy lives for the process lifetime
            std::mem::forget(shutdown_tx);

            let proxy_url = format!("http://{}", proxy_addr);
            println!("OAuth proxy running on {}", proxy_url);

            // Point Anthropic backend at the proxy — no API key needed,
            // proxy handles auth via OAuth tokens
            let config = AnthropicConfig::new("oauth-proxy-managed").with_base_url(&proxy_url);
            Ok(Arc::new(AnthropicBackend::new(config)?))
        }
    }
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

/// Load a persisted server token, or generate and save a new one.
fn load_or_generate_server_token() -> Result<String> {
    let dir = arawn_config::xdg_config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;
    let token_path = dir.join("server-token");

    if token_path.exists() {
        let token = std::fs::read_to_string(&token_path)?.trim().to_string();
        if !token.is_empty() {
            return Ok(token);
        }
    }

    let token = uuid::Uuid::new_v4().to_string();
    std::fs::create_dir_all(&dir)?;
    std::fs::write(&token_path, &token)?;
    Ok(token)
}

/// Resolve a named LLM profile into a ResolvedLlm ready for backend creation.
fn resolve_profile(name: &str, llm_config: &LlmConfig) -> Result<ResolvedLlm> {
    let backend = llm_config
        .backend
        .ok_or_else(|| anyhow::anyhow!("Profile '{}' is missing 'backend' field", name))?;

    let model = llm_config
        .model
        .clone()
        .ok_or_else(|| anyhow::anyhow!("Profile '{}' is missing 'model' field", name))?;

    let resolved_secret =
        arawn_config::secrets::resolve_api_key(&backend, llm_config.api_key.as_deref());

    let (api_key, api_key_source) = match resolved_secret {
        Some(s) => (Some(s.value), Some(s.source)),
        None => (None, None),
    };

    Ok(ResolvedLlm {
        backend,
        model,
        base_url: llm_config.base_url.clone(),
        api_key,
        api_key_source,
        resolved_from: arawn_config::ResolvedFrom::AgentSpecific {
            agent: "profile".to_string(),
            profile: name.to_string(),
        },
    })
}

/// Build an `EmbedderSpec` from the application's `EmbeddingConfig`.
fn build_embedder_spec(config: &arawn_config::EmbeddingConfig) -> EmbedderSpec {
    let provider = match config.provider {
        EmbeddingProvider::Local => "local",
        EmbeddingProvider::OpenAi => "openai",
        EmbeddingProvider::Mock => "mock",
    };

    let (openai_api_key, openai_model, openai_base_url) = config
        .openai
        .as_ref()
        .map(|c| {
            // Try config api_key, then env var, then keyring
            let api_key = c.api_key.clone().or_else(|| {
                std::env::var("OPENAI_API_KEY").ok().or_else(|| {
                    arawn_config::secrets::resolve_api_key(&arawn_config::Backend::Openai, None)
                        .map(|r| r.value)
                })
            });
            (api_key, Some(c.model.clone()), c.base_url.clone())
        })
        .unwrap_or((None, None, None));

    let (local_model_path, local_tokenizer_path) = config
        .local
        .as_ref()
        .map(|c| (c.model_path.clone(), c.tokenizer_path.clone()))
        .unwrap_or((None, None));

    EmbedderSpec {
        provider: provider.to_string(),
        openai_api_key,
        openai_model,
        openai_base_url,
        local_model_path,
        local_tokenizer_path,
        dimensions: Some(config.effective_dimensions()),
    }
}

fn default_model(backend: &Backend) -> String {
    match backend {
        Backend::Anthropic | Backend::ClaudeOauth => "claude-sonnet-4-20250514".to_string(),
        Backend::Openai => "gpt-4o".to_string(),
        Backend::Groq => "llama-3.1-70b-versatile".to_string(),
        Backend::Ollama => "llama3.2".to_string(),
        Backend::Custom => "default".to_string(),
    }
}

/// Compile and register built-in WASM runtimes from source crate directories.
///
/// Scans `runtimes_src_dir` for subdirectories, each expected to be a Cargo crate.
/// For each, if the runtime isn't already in the catalog, compiles it to wasm32-wasip1
/// and registers the `.wasm` as a builtin entry.
async fn register_builtin_runtimes(
    runtimes_src_dir: &std::path::Path,
    executor: &Arc<ScriptExecutor>,
    catalog: &Arc<RwLock<RuntimeCatalog>>,
    verbose: bool,
) {
    let entries = match std::fs::read_dir(runtimes_src_dir) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("warning: cannot read runtimes source dir: {e}");
            return;
        }
    };

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_dir() || !path.join("Cargo.toml").exists() {
            continue;
        }

        let runtime_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };

        // Skip if already registered
        {
            let cat = catalog.read().await;
            if cat.get(&runtime_name).is_some() {
                if verbose {
                    println!("Runtime '{}' already registered, skipping", runtime_name);
                }
                continue;
            }
        }

        if verbose {
            println!("Compiling runtime '{}'...", runtime_name);
        }

        let wasm_path = match executor.compile_crate(&path).await {
            Ok(p) => p,
            Err(e) => {
                eprintln!(
                    "warning: failed to compile runtime '{}': {}",
                    runtime_name, e
                );
                continue;
            }
        };

        // Copy .wasm to catalog's builtin/ directory
        let mut cat = catalog.write().await;
        let builtin_dir = cat.root().join("builtin");
        if let Err(e) = std::fs::create_dir_all(&builtin_dir) {
            eprintln!("warning: cannot create builtin dir: {e}");
            continue;
        }

        let dest = builtin_dir.join(format!("{runtime_name}.wasm"));
        if let Err(e) = std::fs::copy(&wasm_path, &dest) {
            eprintln!("warning: failed to copy wasm for '{}': {}", runtime_name, e);
            continue;
        }

        if let Err(e) = cat.add(
            &runtime_name,
            CatalogEntry {
                description: format!("Built-in {runtime_name} runtime"),
                path: format!("builtin/{runtime_name}.wasm"),
                category: RuntimeCategory::Builtin,
            },
        ) {
            eprintln!(
                "warning: failed to register runtime '{}': {}",
                runtime_name, e
            );
            continue;
        }

        if verbose {
            println!("Registered runtime '{}'", runtime_name);
        }
    }
}
