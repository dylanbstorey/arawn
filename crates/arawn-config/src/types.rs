//! Configuration types mapping to the TOML schema.
//!
//! Top-level config:
//! ```toml
//! [llm]                    # default LLM
//! [llm.claude]             # named LLM configs
//! [agent.default]          # default agent settings
//! [agent.summarizer]       # per-agent overrides
//! [server]                 # server settings
//! ```

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────────────────────────────────────
// Top-level Config
// ─────────────────────────────────────────────────────────────────────────────

/// Root configuration structure.
///
/// Maps to the full TOML config file. All sections are optional so that
/// partial configs (e.g., project-local overrides) can be loaded and merged.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ArawnConfig {
    /// Default LLM configuration (the bare `[llm]` section).
    pub llm: Option<LlmConfig>,

    /// Named LLM configurations (`[llm.claude]`, `[llm.fast]`, etc.).
    /// Stored under a wrapper to avoid conflict with the default `llm` key.
    #[serde(default, rename = "llm_profiles")]
    pub llm_profiles: HashMap<String, LlmConfig>,

    /// Agent configurations.
    #[serde(default)]
    pub agent: HashMap<String, AgentConfig>,

    /// Server configuration.
    pub server: Option<ServerConfig>,

    /// Interaction logging configuration.
    pub logging: Option<LoggingConfig>,

    /// Embedding provider configuration.
    pub embedding: Option<EmbeddingConfig>,

    /// Pipeline / workflow engine configuration.
    pub pipeline: Option<PipelineSection>,

    /// Memory subsystem configuration.
    pub memory: Option<MemoryConfig>,

    /// Plugin system configuration.
    pub plugins: Option<PluginsConfig>,

    /// Subagent delegation configuration.
    pub delegation: Option<DelegationConfig>,

    /// MCP (Model Context Protocol) server configuration.
    pub mcp: Option<McpConfig>,

    /// Workstream configuration.
    pub workstream: Option<WorkstreamConfig>,

    /// Session cache configuration.
    pub session: Option<SessionConfig>,

    /// Tool execution configuration.
    pub tools: Option<ToolsConfig>,

    /// Path management configuration.
    pub paths: Option<crate::paths::PathConfig>,

    /// RLM (Recursive Language Model) exploration agent configuration.
    pub rlm: Option<RlmTomlConfig>,
}

impl ArawnConfig {
    /// Create an empty config.
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse from a TOML string.
    pub fn from_toml(toml_str: &str) -> crate::Result<Self> {
        // Parse into raw TOML value first to handle the llm table split
        let raw: RawConfig = toml::from_str(toml_str)?;
        Ok(raw.into())
    }

    /// Serialize to a TOML string.
    pub fn to_toml(&self) -> crate::Result<String> {
        // Convert back to raw format for serialization
        let raw: RawConfig = self.clone().into();
        Ok(toml::to_string_pretty(&raw)?)
    }

    /// Merge another config on top of this one (other takes priority).
    pub fn merge(&mut self, other: ArawnConfig) {
        if other.llm.is_some() {
            self.llm = other.llm;
        }

        for (name, config) in other.llm_profiles {
            self.llm_profiles.insert(name, config);
        }

        for (name, config) in other.agent {
            self.agent.insert(name, config);
        }

        if other.server.is_some() {
            self.server = other.server;
        }

        if other.logging.is_some() {
            self.logging = other.logging;
        }

        if other.embedding.is_some() {
            self.embedding = other.embedding;
        }

        if other.pipeline.is_some() {
            self.pipeline = other.pipeline;
        }

        if other.memory.is_some() {
            self.memory = other.memory;
        }

        if other.plugins.is_some() {
            self.plugins = other.plugins;
        }

        if other.delegation.is_some() {
            self.delegation = other.delegation;
        }

        if other.mcp.is_some() {
            self.mcp = other.mcp;
        }

        if other.workstream.is_some() {
            self.workstream = other.workstream;
        }

        if other.session.is_some() {
            self.session = other.session;
        }

        if other.tools.is_some() {
            self.tools = other.tools;
        }

        if other.paths.is_some() {
            self.paths = other.paths;
        }

        if other.rlm.is_some() {
            self.rlm = other.rlm;
        }
    }

    /// Resolve the LLM config for a given agent name.
    ///
    /// Resolution order:
    /// 1. `agent.<name>.llm` → lookup in `llm_profiles`
    /// 2. `agent.default.llm` → lookup in `llm_profiles`
    /// 3. `[llm]` (global default)
    pub fn resolve_llm(&self, agent_name: &str) -> crate::Result<&LlmConfig> {
        // 1. Agent-specific
        if let Some(agent_cfg) = self.agent.get(agent_name) {
            if let Some(ref llm_name) = agent_cfg.llm {
                return self.lookup_llm(llm_name, &format!("agent.{}", agent_name));
            }
        }

        // 2. Agent default
        if let Some(default_cfg) = self.agent.get("default") {
            if let Some(ref llm_name) = default_cfg.llm {
                return self.lookup_llm(llm_name, "agent.default");
            }
        }

        // 3. Global default
        self.llm.as_ref().ok_or(crate::ConfigError::NoDefaultLlm)
    }

    /// Look up a named LLM config.
    fn lookup_llm<'a>(&'a self, name: &str, context: &str) -> crate::Result<&'a LlmConfig> {
        self.llm_profiles
            .get(name)
            .ok_or_else(|| crate::ConfigError::LlmNotFound {
                name: name.to_string(),
                context: context.to_string(),
            })
    }

    /// Get all defined LLM config names (including "default" for the bare [llm]).
    pub fn llm_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        if self.llm.is_some() {
            names.push("default".to_string());
        }
        names.extend(self.llm_profiles.keys().cloned());
        names.sort();
        names
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Raw TOML structure (for serde)
// ─────────────────────────────────────────────────────────────────────────────

/// Internal raw config matching the actual TOML layout.
///
/// In TOML, `[llm]` and `[llm.claude]` coexist as a table with both
/// direct keys and sub-tables. This struct handles that mapping.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
struct RawConfig {
    llm: Option<RawLlmSection>,
    #[serde(default)]
    agent: HashMap<String, AgentConfig>,
    server: Option<ServerConfig>,
    logging: Option<LoggingConfig>,
    embedding: Option<EmbeddingConfig>,
    pipeline: Option<PipelineSection>,
    memory: Option<MemoryConfig>,
    plugins: Option<PluginsConfig>,
    delegation: Option<DelegationConfig>,
    mcp: Option<McpConfig>,
    workstream: Option<WorkstreamConfig>,
    session: Option<SessionConfig>,
    tools: Option<ToolsConfig>,
    paths: Option<crate::paths::PathConfig>,
    rlm: Option<RlmTomlConfig>,
}

/// The `[llm]` section which can contain both direct fields and named sub-tables.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
struct RawLlmSection {
    /// Default backend.
    backend: Option<Backend>,
    /// Default model.
    model: Option<String>,
    /// Default base URL.
    base_url: Option<String>,
    /// Default API key (will warn if present).
    api_key: Option<String>,
    /// Maximum retry attempts for failed requests.
    retry_max: Option<u32>,
    /// Backoff delay between retries in milliseconds.
    retry_backoff_ms: Option<u64>,
    /// Maximum context window size in tokens.
    max_context_tokens: Option<usize>,

    /// Named profiles are captured via flatten.
    #[serde(flatten)]
    profiles: HashMap<String, LlmConfig>,
}

impl From<RawConfig> for ArawnConfig {
    fn from(raw: RawConfig) -> Self {
        let (llm, llm_profiles) = match raw.llm {
            Some(section) => {
                let default = if section.backend.is_some() || section.model.is_some() {
                    Some(LlmConfig {
                        backend: section.backend,
                        model: section.model,
                        base_url: section.base_url,
                        api_key: section.api_key,
                        retry_max: section.retry_max,
                        retry_backoff_ms: section.retry_backoff_ms,
                        max_context_tokens: section.max_context_tokens,
                    })
                } else {
                    None
                };
                (default, section.profiles)
            }
            None => (None, HashMap::new()),
        };

        ArawnConfig {
            llm,
            llm_profiles,
            agent: raw.agent,
            server: raw.server,
            logging: raw.logging,
            embedding: raw.embedding,
            pipeline: raw.pipeline,
            memory: raw.memory,
            plugins: raw.plugins,
            delegation: raw.delegation,
            mcp: raw.mcp,
            workstream: raw.workstream,
            session: raw.session,
            tools: raw.tools,
            paths: raw.paths,
            rlm: raw.rlm,
        }
    }
}

impl From<ArawnConfig> for RawConfig {
    fn from(config: ArawnConfig) -> Self {
        let llm = if config.llm.is_some() || !config.llm_profiles.is_empty() {
            let default = config.llm.unwrap_or_default();
            Some(RawLlmSection {
                backend: default.backend,
                model: default.model,
                base_url: default.base_url,
                api_key: default.api_key,
                retry_max: default.retry_max,
                retry_backoff_ms: default.retry_backoff_ms,
                max_context_tokens: default.max_context_tokens,
                profiles: config.llm_profiles,
            })
        } else {
            None
        };

        RawConfig {
            llm,
            agent: config.agent,
            server: config.server,
            logging: config.logging,
            embedding: config.embedding,
            pipeline: config.pipeline,
            memory: config.memory,
            plugins: config.plugins,
            delegation: config.delegation,
            mcp: config.mcp,
            workstream: config.workstream,
            session: config.session,
            tools: config.tools,
            paths: config.paths,
            rlm: config.rlm,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// LLM Configuration
// ─────────────────────────────────────────────────────────────────────────────

/// Configuration for an LLM backend.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct LlmConfig {
    /// Backend provider.
    pub backend: Option<Backend>,
    /// Model identifier.
    pub model: Option<String>,
    /// Custom API base URL (for proxies, custom endpoints).
    pub base_url: Option<String>,
    /// API key (prefer keyring or env var; warns if set here).
    pub api_key: Option<String>,
    /// Maximum retry attempts for failed requests.
    pub retry_max: Option<u32>,
    /// Backoff delay between retries in milliseconds.
    pub retry_backoff_ms: Option<u64>,
    /// Maximum context window size in tokens.
    /// If not specified, uses default for the model (see `effective_max_context_tokens`).
    pub max_context_tokens: Option<usize>,
}

impl LlmConfig {
    /// Returns true if an API key is stored directly in the config file.
    pub fn has_plaintext_api_key(&self) -> bool {
        self.api_key.is_some()
    }

    /// Get the environment variable name for this backend's API key.
    pub fn api_key_env_var(&self) -> Option<&'static str> {
        self.backend.as_ref().map(|b| b.env_var())
    }

    /// Get the maximum context tokens, returning an error if not configured.
    pub fn require_max_context_tokens(&self) -> crate::Result<usize> {
        self.max_context_tokens.ok_or_else(|| {
            let model = self.model.as_deref().unwrap_or("unknown");
            crate::ConfigError::MissingContextLimit {
                model: model.to_string(),
            }
        })
    }
}

/// Supported LLM backend providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Backend {
    Anthropic,
    Openai,
    Groq,
    Ollama,
    Custom,
    #[serde(rename = "claude-oauth")]
    ClaudeOauth,
}

impl Backend {
    /// Environment variable name for this backend's API key.
    pub fn env_var(&self) -> &'static str {
        match self {
            Backend::Anthropic => "ANTHROPIC_API_KEY",
            Backend::Openai => "OPENAI_API_KEY",
            Backend::Groq => "GROQ_API_KEY",
            Backend::Ollama => "OLLAMA_API_KEY",
            Backend::Custom => "LLM_API_KEY",
            Backend::ClaudeOauth => "ANTHROPIC_API_KEY",
        }
    }

    /// Human-readable name.
    pub fn display_name(&self) -> &'static str {
        match self {
            Backend::Anthropic => "Anthropic",
            Backend::Openai => "OpenAI",
            Backend::Groq => "Groq",
            Backend::Ollama => "Ollama",
            Backend::Custom => "Custom",
            Backend::ClaudeOauth => "Claude OAuth",
        }
    }
}

impl std::fmt::Display for Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.display_name())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Agent Configuration
// ─────────────────────────────────────────────────────────────────────────────

/// Per-agent configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AgentConfig {
    /// Name of the LLM config to use (references a key in `llm_profiles`).
    pub llm: Option<String>,
    /// System prompt override.
    pub system_prompt: Option<String>,
    /// Max tool iterations.
    pub max_iterations: Option<u32>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Server Configuration
// ─────────────────────────────────────────────────────────────────────────────

/// Server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    /// Port to listen on.
    pub port: u16,
    /// Address to bind to.
    pub bind: String,
    /// Enable rate limiting.
    pub rate_limiting: bool,
    /// API rate limit: requests per minute per IP.
    pub api_rpm: u32,
    /// Enable request logging.
    pub request_logging: bool,
    /// Path to bootstrap files directory.
    pub bootstrap_dir: Option<PathBuf>,
    /// Working directory.
    pub workspace: Option<PathBuf>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: arawn_types::config::defaults::DEFAULT_PORT,
            bind: arawn_types::config::defaults::DEFAULT_BIND.to_string(),
            rate_limiting: true,
            api_rpm: arawn_types::config::defaults::REQUESTS_PER_MINUTE,
            request_logging: true,
            bootstrap_dir: None,
            workspace: None,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Logging Configuration
// ─────────────────────────────────────────────────────────────────────────────

/// Logging configuration section.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct LoggingConfig {
    /// Interaction log settings.
    pub interactions: InteractionLogConfig,
}

/// Settings for structured interaction logging (JSONL).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct InteractionLogConfig {
    /// Whether interaction logging is enabled.
    pub enabled: bool,
    /// Directory for JSONL log files.
    pub path: Option<PathBuf>,
    /// Days to retain log files before cleanup.
    pub retention_days: u32,
}

impl Default for InteractionLogConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            path: None,
            retention_days: 90,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Embedding Configuration
// ─────────────────────────────────────────────────────────────────────────────

/// Embedding provider configuration.
///
/// Controls which embedding backend is used for semantic search and memory
/// indexing. Default is local ONNX inference (offline-first, zero-config).
///
/// ```toml
/// [embedding]
/// provider = "local"        # "local", "openai", or "mock"
/// dimensions = 384
///
/// [embedding.openai]
/// model = "text-embedding-3-small"
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EmbeddingConfig {
    /// Provider: "local" (ONNX), "openai", or "mock".
    pub provider: EmbeddingProvider,
    /// Output embedding dimensions. Default depends on provider.
    pub dimensions: Option<usize>,
    /// OpenAI-specific embedding settings.
    pub openai: Option<EmbeddingOpenAiConfig>,
    /// Local ONNX-specific settings.
    pub local: Option<EmbeddingLocalConfig>,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            provider: EmbeddingProvider::Local,
            dimensions: None,
            openai: None,
            local: None,
        }
    }
}

impl EmbeddingConfig {
    /// Effective dimensions for the configured provider.
    pub fn effective_dimensions(&self) -> usize {
        if let Some(d) = self.dimensions {
            return d;
        }
        match self.provider {
            EmbeddingProvider::Local => 384,
            EmbeddingProvider::OpenAi => {
                // Check provider-specific config
                self.openai
                    .as_ref()
                    .and_then(|c| c.dimensions)
                    .unwrap_or(1536)
            }
            EmbeddingProvider::Mock => 384,
        }
    }
}

/// Supported embedding providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EmbeddingProvider {
    /// Local ONNX Runtime inference (default, offline-first).
    Local,
    /// OpenAI embeddings API.
    OpenAi,
    /// Mock embedder for testing.
    Mock,
}

/// OpenAI embedding provider settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EmbeddingOpenAiConfig {
    /// Model name. Default: "text-embedding-3-small".
    pub model: String,
    /// Override dimensions (OpenAI supports reduced output).
    pub dimensions: Option<usize>,
    /// Custom base URL (for proxies).
    pub base_url: Option<String>,
    /// API key (prefer keyring or env var).
    pub api_key: Option<String>,
}

impl Default for EmbeddingOpenAiConfig {
    fn default() -> Self {
        Self {
            model: "text-embedding-3-small".to_string(),
            dimensions: None,
            base_url: None,
            api_key: None,
        }
    }
}

/// Local ONNX embedding settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
#[derive(Default)]
pub struct EmbeddingLocalConfig {
    /// Path to ONNX model file.
    pub model_path: Option<PathBuf>,
    /// Path to tokenizer.json file.
    pub tokenizer_path: Option<PathBuf>,
    /// Download URL for the ONNX model file.
    /// Defaults to HuggingFace all-MiniLM-L6-v2.
    pub model_url: Option<String>,
    /// Download URL for the tokenizer JSON file.
    /// Defaults to HuggingFace all-MiniLM-L6-v2.
    pub tokenizer_url: Option<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Memory Configuration
// ─────────────────────────────────────────────────────────────────────────────

/// Memory subsystem configuration.
///
/// ```toml
/// [memory]
/// [memory.recall]
/// enabled = true
/// threshold = 0.6
/// limit = 5
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct MemoryConfig {
    /// Path to the SQLite database for memory storage.
    /// Relative paths are resolved from the data directory.
    pub database: Option<PathBuf>,
    /// Active recall configuration.
    pub recall: RecallConfig,
    /// Session indexing pipeline configuration.
    pub indexing: IndexingConfig,
    /// Confidence scoring parameters.
    pub confidence: ConfidenceConfig,
}

/// Configuration for active recall behavior.
///
/// Controls whether and how the agent automatically recalls relevant
/// memories at the start of each turn.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RecallConfig {
    /// Whether active recall is enabled.
    pub enabled: bool,
    /// Minimum similarity score threshold (0.0–1.0).
    pub threshold: f32,
    /// Maximum number of memories to recall per turn.
    pub limit: usize,
}

impl Default for RecallConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            threshold: 0.6,
            limit: 5,
        }
    }
}

/// Configuration for session indexing pipeline.
///
/// Controls the LLM backend used for fact extraction and session summarization.
///
/// ```toml
/// [memory.indexing]
/// enabled = true
/// backend = "openai"
/// model = "gpt-4o-mini"
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct IndexingConfig {
    /// Whether session indexing is enabled.
    pub enabled: bool,
    /// LLM backend for extraction/summarization (e.g., "openai", "groq", "anthropic").
    pub backend: String,
    /// Model to use for extraction/summarization.
    pub model: String,
    /// Path to GLiNER ONNX model file for local NER extraction.
    /// When set, enables hybrid extraction (NER for entities, LLM for facts only).
    /// If unset and the `gliner` feature is enabled, the default model is
    /// auto-downloaded from HuggingFace on first use.
    pub ner_model_path: Option<String>,
    /// Path to GLiNER tokenizer JSON file.
    pub ner_tokenizer_path: Option<String>,
    /// Minimum confidence threshold for NER spans (0.0 to 1.0).
    pub ner_threshold: f32,
    /// Download URL for the GLiNER ONNX model file.
    /// Defaults to HuggingFace onnx-community/gliner_small-v2.1.
    pub ner_model_url: Option<String>,
    /// Download URL for the GLiNER tokenizer JSON file.
    /// Defaults to HuggingFace onnx-community/gliner_small-v2.1.
    pub ner_tokenizer_url: Option<String>,
}

impl Default for IndexingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            backend: "openai".to_string(),
            model: "gpt-4o-mini".to_string(),
            ner_model_path: None,
            ner_tokenizer_path: None,
            ner_threshold: 0.5,
            ner_model_url: None,
            ner_tokenizer_url: None,
        }
    }
}

/// Configuration for confidence scoring parameters.
///
/// ```toml
/// [memory.confidence]
/// staleness_days = 365
/// staleness_floor = 0.3
/// reinforcement_cap = 1.5
/// fresh_days = 30
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ConfidenceConfig {
    /// Number of days before staleness decay begins.
    pub fresh_days: f32,
    /// Number of days at which staleness reaches the floor.
    pub staleness_days: f32,
    /// Minimum staleness multiplier.
    pub staleness_floor: f32,
    /// Maximum reinforcement multiplier.
    pub reinforcement_cap: f32,
}

impl Default for ConfidenceConfig {
    fn default() -> Self {
        Self {
            fresh_days: 30.0,
            staleness_days: 365.0,
            staleness_floor: 0.3,
            reinforcement_cap: 1.5,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Delegation Configuration
// ─────────────────────────────────────────────────────────────────────────────

/// Subagent delegation configuration.
///
/// Controls behavior for delegating tasks to subagents, including result
/// compaction (LLM-based summarization of long responses).
///
/// ```toml
/// [delegation]
/// max_result_len = 8000
///
/// [delegation.compaction]
/// enabled = true
/// threshold = 8000
/// backend = "default"
/// model = "gpt-4o-mini"
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DelegationConfig {
    /// Maximum length for subagent results before truncation/compaction.
    pub max_result_len: usize,
    /// Context compaction configuration.
    pub compaction: CompactionConfig,
}

impl Default for DelegationConfig {
    fn default() -> Self {
        Self {
            max_result_len: 8000,
            compaction: CompactionConfig::default(),
        }
    }
}

/// Configuration for LLM-based result compaction.
///
/// When enabled, long subagent responses are summarized using an LLM
/// instead of being truncated. This preserves more useful information
/// at the cost of an additional LLM call.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CompactionConfig {
    /// Whether compaction is enabled.
    pub enabled: bool,
    /// Minimum length to trigger compaction (characters).
    pub threshold: usize,
    /// LLM backend name for compaction (references `llm_profiles`).
    /// Use "default" to use the global default LLM.
    pub backend: String,
    /// Model to use for compaction.
    pub model: String,
    /// Target length for compacted output (characters).
    pub target_len: usize,
}

impl Default for CompactionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            threshold: 8000,
            backend: "default".to_string(),
            model: "gpt-4o-mini".to_string(),
            target_len: 4000,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Plugin Configuration
// ─────────────────────────────────────────────────────────────────────────────

/// Plugin system configuration.
///
/// ```toml
/// [plugins]
/// enabled = true
/// dirs = ["~/.config/arawn/plugins", "./plugins"]
/// auto_update = true
///
/// [[plugins.subscriptions]]
/// source = "github"
/// repo = "author/plugin-name"
/// ref = "main"
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PluginsConfig {
    /// Whether the plugin system is enabled.
    pub enabled: bool,
    /// Additional directories to scan for plugins.
    /// Default dirs (`~/.config/arawn/plugins/` and `./plugins/`) are always included.
    pub dirs: Vec<PathBuf>,
    /// Whether hot-reload file watching is enabled.
    pub hot_reload: bool,
    /// Whether to automatically update subscribed plugins on startup.
    pub auto_update: bool,
    /// Plugin subscriptions (sources to fetch plugins from).
    #[serde(default)]
    pub subscriptions: Vec<PluginSubscription>,
}

impl Default for PluginsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            dirs: Vec::new(),
            hot_reload: true,
            auto_update: true,
            subscriptions: Vec::new(),
        }
    }
}

/// A plugin subscription defining where to fetch a plugin from.
///
/// Supports multiple source types:
/// - GitHub repositories
/// - Git URLs (any git remote)
/// - Local filesystem paths
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginSubscription {
    /// Source type for this subscription.
    pub source: PluginSource,
    /// GitHub repository in "owner/repo" format (for source = "github").
    #[serde(default)]
    pub repo: Option<String>,
    /// Git URL for cloning (for source = "url").
    #[serde(default)]
    pub url: Option<String>,
    /// Local filesystem path (for source = "local").
    #[serde(default)]
    pub path: Option<PathBuf>,
    /// Git ref (branch, tag, or commit) to checkout. Defaults to "main".
    #[serde(default, rename = "ref")]
    pub git_ref: Option<String>,
    /// Whether this subscription is enabled. Defaults to true.
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}

impl PluginSubscription {
    /// Create a GitHub subscription.
    pub fn github(repo: impl Into<String>) -> Self {
        Self {
            source: PluginSource::GitHub,
            repo: Some(repo.into()),
            url: None,
            path: None,
            git_ref: None,
            enabled: true,
        }
    }

    /// Create a URL subscription.
    pub fn url(url: impl Into<String>) -> Self {
        Self {
            source: PluginSource::Url,
            repo: None,
            url: Some(url.into()),
            path: None,
            git_ref: None,
            enabled: true,
        }
    }

    /// Create a local path subscription.
    pub fn local(path: impl Into<PathBuf>) -> Self {
        Self {
            source: PluginSource::Local,
            repo: None,
            url: None,
            path: Some(path.into()),
            git_ref: None,
            enabled: true,
        }
    }

    /// Set the git ref (branch, tag, or commit).
    pub fn with_ref(mut self, git_ref: impl Into<String>) -> Self {
        self.git_ref = Some(git_ref.into());
        self
    }

    /// Get the effective git ref, defaulting to "main".
    pub fn effective_ref(&self) -> &str {
        self.git_ref.as_deref().unwrap_or("main")
    }

    /// Generate a unique identifier for this subscription.
    ///
    /// Used for cache directory naming and deduplication.
    pub fn id(&self) -> String {
        match self.source {
            PluginSource::GitHub => {
                let repo = self.repo.as_deref().unwrap_or("unknown");
                format!("github/{}", repo.replace('/', "-"))
            }
            PluginSource::Url => {
                let url = self.url.as_deref().unwrap_or("unknown");
                // Hash the URL to create a stable identifier
                let hash = simple_hash(url);
                format!("url/{:016x}", hash)
            }
            PluginSource::Local => {
                let path = self
                    .path
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_default();
                let hash = simple_hash(&path);
                format!("local/{:016x}", hash)
            }
        }
    }

    /// Get the clone URL for this subscription.
    pub fn clone_url(&self) -> Option<String> {
        match self.source {
            PluginSource::GitHub => self
                .repo
                .as_ref()
                .map(|r| format!("https://github.com/{}.git", r)),
            PluginSource::Url => self.url.clone(),
            PluginSource::Local => None,
        }
    }
}

/// Simple hash function for generating stable identifiers.
fn simple_hash(s: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

/// Source type for plugin subscriptions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PluginSource {
    /// GitHub repository (uses HTTPS clone URL).
    GitHub,
    /// Arbitrary git URL.
    Url,
    /// Local filesystem path (not cloned, just referenced).
    Local,
}

// ─────────────────────────────────────────────────────────────────────────────
// Pipeline Configuration
// ─────────────────────────────────────────────────────────────────────────────

/// Pipeline / workflow engine configuration.
///
/// ```toml
/// [pipeline]
/// enabled = true
/// database = "pipeline.db"
/// workflow_dir = "workflows"
/// max_concurrent_tasks = 4
/// task_timeout_secs = 300
/// pipeline_timeout_secs = 600
/// cron_enabled = true
/// triggers_enabled = true
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PipelineSection {
    /// Whether the pipeline engine is enabled.
    pub enabled: bool,
    /// Path to the SQLite database for pipeline state.
    /// Relative paths are resolved from the data directory.
    pub database: Option<PathBuf>,
    /// Directory containing workflow TOML definitions.
    /// Relative paths are resolved from the data directory.
    pub workflow_dir: Option<PathBuf>,
    /// Maximum concurrent task executions.
    pub max_concurrent_tasks: usize,
    /// Per-task timeout in seconds.
    pub task_timeout_secs: u64,
    /// Per-pipeline (workflow) timeout in seconds.
    pub pipeline_timeout_secs: u64,
    /// Enable cron-based scheduling.
    pub cron_enabled: bool,
    /// Enable event-based triggers.
    pub triggers_enabled: bool,
}

impl Default for PipelineSection {
    fn default() -> Self {
        Self {
            enabled: true,
            database: None,
            workflow_dir: None,
            max_concurrent_tasks: 4,
            task_timeout_secs: 300,
            pipeline_timeout_secs: 600,
            cron_enabled: true,
            triggers_enabled: true,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// MCP Configuration
// ─────────────────────────────────────────────────────────────────────────────

/// MCP (Model Context Protocol) configuration.
///
/// Configures MCP servers that provide external tools to the agent.
/// Servers can be configured globally or per-plugin.
///
/// ```toml
/// [mcp]
/// enabled = true
///
/// [[mcp.servers]]
/// name = "sqlite"
/// command = "mcp-server-sqlite"
/// args = ["--db", "/path/to/db.sqlite"]
///
/// [[mcp.servers]]
/// name = "filesystem"
/// command = "mcp-server-filesystem"
/// args = ["--allowed-dirs", "/home/user/projects"]
/// env = [["DEBUG", "1"]]
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct McpConfig {
    /// Whether MCP is enabled globally.
    pub enabled: bool,
    /// Configured MCP servers.
    #[serde(default)]
    pub servers: Vec<McpServerEntry>,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            servers: Vec::new(),
        }
    }
}

/// Transport type for MCP server connections.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum McpTransportType {
    /// Stdio transport - spawns a child process.
    #[default]
    Stdio,
    /// HTTP transport - connects to a remote server via HTTP POST.
    Http,
}

/// Configuration for a single MCP server.
///
/// Defines how to spawn and connect to an MCP server.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct McpServerEntry {
    /// Unique name for this server (used in tool namespacing).
    pub name: String,
    /// Transport type (stdio or http). Defaults to stdio.
    #[serde(default)]
    pub transport: McpTransportType,
    /// Command to execute to start the server (for stdio transport).
    #[serde(default)]
    pub command: String,
    /// URL for the server (for HTTP transport).
    pub url: Option<String>,
    /// Arguments to pass to the command (for stdio transport).
    #[serde(default)]
    pub args: Vec<String>,
    /// Environment variables to set (as [key, value] pairs, for stdio transport).
    #[serde(default)]
    pub env: Vec<[String; 2]>,
    /// HTTP headers to set (as [key, value] pairs, for HTTP transport).
    #[serde(default)]
    pub headers: Vec<[String; 2]>,
    /// Request timeout in seconds (for HTTP transport). Defaults to 30.
    pub timeout_secs: Option<u64>,
    /// Number of retries (for HTTP transport). Defaults to 3.
    pub retries: Option<u32>,
    /// Whether this server is enabled. Defaults to true.
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

impl McpServerEntry {
    /// Create a new MCP server entry for stdio transport.
    pub fn new(name: impl Into<String>, command: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            transport: McpTransportType::Stdio,
            command: command.into(),
            url: None,
            args: Vec::new(),
            env: Vec::new(),
            headers: Vec::new(),
            timeout_secs: None,
            retries: None,
            enabled: true,
        }
    }

    /// Create a new MCP server entry for HTTP transport.
    pub fn http(name: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            transport: McpTransportType::Http,
            command: String::new(),
            url: Some(url.into()),
            args: Vec::new(),
            env: Vec::new(),
            headers: Vec::new(),
            timeout_secs: None,
            retries: None,
            enabled: true,
        }
    }

    /// Add an argument (for stdio transport).
    pub fn with_arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Add arguments (for stdio transport).
    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    /// Add an environment variable (for stdio transport).
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.push([key.into(), value.into()]);
        self
    }

    /// Add an HTTP header (for HTTP transport).
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push([key.into(), value.into()]);
        self
    }

    /// Set request timeout in seconds (for HTTP transport).
    pub fn with_timeout_secs(mut self, timeout: u64) -> Self {
        self.timeout_secs = Some(timeout);
        self
    }

    /// Set number of retries (for HTTP transport).
    pub fn with_retries(mut self, retries: u32) -> Self {
        self.retries = Some(retries);
        self
    }

    /// Set enabled state.
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Check if this is an HTTP transport.
    pub fn is_http(&self) -> bool {
        matches!(self.transport, McpTransportType::Http)
    }

    /// Check if this is a stdio transport.
    pub fn is_stdio(&self) -> bool {
        matches!(self.transport, McpTransportType::Stdio)
    }

    /// Convert environment variables to the tuple format expected by McpServerConfig.
    pub fn env_tuples(&self) -> Vec<(String, String)> {
        self.env
            .iter()
            .map(|[k, v]| (k.clone(), v.clone()))
            .collect()
    }

    /// Convert HTTP headers to the tuple format.
    pub fn header_tuples(&self) -> Vec<(String, String)> {
        self.headers
            .iter()
            .map(|[k, v]| (k.clone(), v.clone()))
            .collect()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Workstream Configuration
// ─────────────────────────────────────────────────────────────────────────────

/// Configuration for workstreams (persistent conversation contexts).
///
/// Workstreams are always enabled - this config only customizes paths.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WorkstreamConfig {
    /// Path to the SQLite database for workstream storage.
    /// Relative paths are resolved from the data directory.
    pub database: Option<PathBuf>,
    /// Root directory for JSONL message files.
    /// Relative paths are resolved from the data directory.
    pub data_dir: Option<PathBuf>,
    /// Session timeout in minutes (default: 60).
    pub session_timeout_minutes: i64,
    /// Session/workstream compression configuration.
    pub compression: Option<CompressionConfig>,
}

impl Default for WorkstreamConfig {
    fn default() -> Self {
        Self {
            database: None,
            data_dir: None,
            session_timeout_minutes: 60,
            compression: None,
        }
    }
}

/// Configuration for automatic session/workstream compression.
///
/// When enabled, sessions are automatically compressed via LLM summarization
/// when they end, and workstream summaries are updated from session summaries.
///
/// ```toml
/// [workstream.compression]
/// enabled = true
/// backend = "default"
/// model = "claude-sonnet-4-20250514"
/// max_summary_tokens = 1024
/// token_threshold_chars = 32000
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CompressionConfig {
    /// Whether automatic compression is enabled (default: false).
    pub enabled: bool,
    /// LLM backend name for compression (references `llm_profiles`).
    /// Use "default" to use the global default LLM.
    pub backend: String,
    /// Model to use for summarization calls.
    pub model: String,
    /// Max tokens for summary generation (default: 1024).
    pub max_summary_tokens: u32,
    /// Character threshold that triggers compression (default: 32000, ~8k tokens).
    pub token_threshold_chars: usize,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            backend: "default".to_string(),
            model: "claude-sonnet".to_string(),
            max_summary_tokens: 1024,
            token_threshold_chars: 32_000,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Session Configuration
// ─────────────────────────────────────────────────────────────────────────────

/// Session cache configuration.
///
/// Controls behavior of the in-memory session cache including eviction
/// and cleanup settings.
///
/// ```toml
/// [session]
/// max_sessions = 10000
/// cleanup_interval_secs = 60
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SessionConfig {
    /// Maximum number of sessions to keep in cache before eviction.
    pub max_sessions: usize,
    /// Interval in seconds between cache cleanup runs.
    pub cleanup_interval_secs: u64,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            max_sessions: 10000,
            cleanup_interval_secs: 60,
        }
    }
}

impl arawn_types::ConfigProvider for SessionConfig {}

impl arawn_types::HasSessionConfig for SessionConfig {
    fn max_sessions(&self) -> usize {
        self.max_sessions
    }

    fn cleanup_interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.cleanup_interval_secs)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tools Configuration
// ─────────────────────────────────────────────────────────────────────────────

/// Tool execution configuration.
///
/// Controls timeouts and output limits for various tool types.
///
/// ```toml
/// [tools]
/// [tools.output]
/// max_size_bytes = 102400
///
/// [tools.shell]
/// timeout_secs = 30
///
/// [tools.web]
/// timeout_secs = 30
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ToolsConfig {
    /// Tool output configuration.
    pub output: ToolOutputConfig,
    /// Shell tool configuration.
    pub shell: ShellToolConfig,
    /// Web tool configuration.
    pub web: WebToolConfig,
}

/// Tool output configuration.
///
/// Per-tool limits override the global `max_size_bytes` default.
/// When a per-tool value is `None`, the hardcoded default for that
/// tool type is used (shell=100KB, file_read=500KB, web_fetch=200KB,
/// search=50KB).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ToolOutputConfig {
    /// Default maximum size of tool output in bytes before truncation.
    /// Used for tools without a specific override.
    pub max_size_bytes: usize,
    /// Max output size for shell/bash tool (default: 100KB).
    pub shell: Option<usize>,
    /// Max output size for file_read tool (default: 500KB).
    pub file_read: Option<usize>,
    /// Max output size for web_fetch tool (default: 200KB).
    pub web_fetch: Option<usize>,
    /// Max output size for search/grep/glob tools (default: 50KB).
    pub search: Option<usize>,
}

impl Default for ToolOutputConfig {
    fn default() -> Self {
        Self {
            max_size_bytes: 102400, // 100KB
            shell: None,
            file_read: None,
            web_fetch: None,
            search: None,
        }
    }
}

/// Shell tool configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ShellToolConfig {
    /// Timeout for shell command execution in seconds.
    pub timeout_secs: u64,
}

impl Default for ShellToolConfig {
    fn default() -> Self {
        Self { timeout_secs: 30 }
    }
}

/// Web tool configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WebToolConfig {
    /// Timeout for web requests in seconds.
    pub timeout_secs: u64,
}

impl Default for WebToolConfig {
    fn default() -> Self {
        Self { timeout_secs: 30 }
    }
}

impl arawn_types::ConfigProvider for ToolsConfig {}

impl arawn_types::HasToolConfig for ToolsConfig {
    fn shell_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.shell.timeout_secs)
    }

    fn web_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.web.timeout_secs)
    }

    fn max_output_bytes(&self) -> usize {
        self.output.max_size_bytes
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// RLM Configuration
// ─────────────────────────────────────────────────────────────────────────────

/// Configuration for the RLM (Recursive Language Model) exploration agent.
///
/// Maps to the `[rlm]` section in `arawn.toml`. All fields are optional;
/// when absent, the agent-side `RlmConfig` defaults apply.
///
/// ```toml
/// [rlm]
/// model = "claude-haiku-4-5-20251001"
/// max_turns = 25
/// max_context_tokens = 50000
/// compaction_threshold = 0.7
/// max_compactions = 10
/// compaction_model = "claude-haiku-4-5-20251001"
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
#[derive(Default)]
pub struct RlmTomlConfig {
    /// Model to use for exploration. Empty string or absent = inherit from backend.
    pub model: Option<String>,
    /// Maximum agent turns before stopping (safety valve).
    pub max_turns: Option<u32>,
    /// Maximum estimated tokens before triggering compaction.
    pub max_context_tokens: Option<usize>,
    /// Fraction of `max_context_tokens` that triggers compaction (0.0–1.0).
    pub compaction_threshold: Option<f32>,
    /// Maximum compaction cycles before stopping.
    pub max_compactions: Option<u32>,
    /// Cumulative token budget for the entire exploration.
    pub max_total_tokens: Option<usize>,
    /// Separate model for compaction (cheaper/faster).
    pub compaction_model: Option<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_config() {
        let config = ArawnConfig::new();
        assert!(config.llm.is_none());
        assert!(config.llm_profiles.is_empty());
        assert!(config.agent.is_empty());
        assert!(config.server.is_none());
    }

    #[test]
    fn test_parse_minimal() {
        let toml = r#"
[llm]
backend = "groq"
model = "llama-3.1-70b-versatile"
"#;
        let config = ArawnConfig::from_toml(toml).unwrap();
        let llm = config.llm.as_ref().unwrap();
        assert_eq!(llm.backend, Some(Backend::Groq));
        assert_eq!(llm.model.as_deref(), Some("llama-3.1-70b-versatile"));
    }

    #[test]
    fn test_parse_named_profiles() {
        let toml = r#"
[llm]
backend = "groq"
model = "llama-3.1-70b-versatile"

[llm.claude]
backend = "anthropic"
model = "claude-sonnet-4-20250514"

[llm.fast]
backend = "groq"
model = "llama-3.1-8b-instant"
"#;
        let config = ArawnConfig::from_toml(toml).unwrap();
        assert!(config.llm.is_some());
        assert_eq!(config.llm_profiles.len(), 2);

        let claude = &config.llm_profiles["claude"];
        assert_eq!(claude.backend, Some(Backend::Anthropic));
        assert_eq!(claude.model.as_deref(), Some("claude-sonnet-4-20250514"));

        let fast = &config.llm_profiles["fast"];
        assert_eq!(fast.backend, Some(Backend::Groq));
    }

    #[test]
    fn test_parse_agents() {
        let toml = r#"
[llm]
backend = "groq"
model = "default-model"

[llm.claude]
backend = "anthropic"
model = "claude-sonnet-4-20250514"

[llm.fast]
backend = "groq"
model = "llama-3.1-8b-instant"

[agent.default]
llm = "claude"

[agent.summarizer]
llm = "fast"
"#;
        let config = ArawnConfig::from_toml(toml).unwrap();
        assert_eq!(config.agent["default"].llm.as_deref(), Some("claude"));
        assert_eq!(config.agent["summarizer"].llm.as_deref(), Some("fast"));
    }

    #[test]
    fn test_resolve_llm_agent_specific() {
        let toml = r#"
[llm]
backend = "groq"
model = "default-model"

[llm.fast]
backend = "groq"
model = "fast-model"

[agent.summarizer]
llm = "fast"
"#;
        let config = ArawnConfig::from_toml(toml).unwrap();
        let resolved = config.resolve_llm("summarizer").unwrap();
        assert_eq!(resolved.model.as_deref(), Some("fast-model"));
    }

    #[test]
    fn test_resolve_llm_agent_default() {
        let toml = r#"
[llm]
backend = "groq"
model = "default-model"

[llm.claude]
backend = "anthropic"
model = "claude-model"

[agent.default]
llm = "claude"
"#;
        let config = ArawnConfig::from_toml(toml).unwrap();
        // "researcher" has no specific config, falls back to agent.default
        let resolved = config.resolve_llm("researcher").unwrap();
        assert_eq!(resolved.model.as_deref(), Some("claude-model"));
    }

    #[test]
    fn test_resolve_llm_global_default() {
        let toml = r#"
[llm]
backend = "groq"
model = "global-default"
"#;
        let config = ArawnConfig::from_toml(toml).unwrap();
        let resolved = config.resolve_llm("anything").unwrap();
        assert_eq!(resolved.model.as_deref(), Some("global-default"));
    }

    #[test]
    fn test_resolve_llm_no_default() {
        let config = ArawnConfig::new();
        let err = config.resolve_llm("agent").unwrap_err();
        assert!(matches!(err, crate::ConfigError::NoDefaultLlm));
    }

    #[test]
    fn test_resolve_llm_missing_reference() {
        let toml = r#"
[llm]
backend = "groq"
model = "default"

[agent.default]
llm = "nonexistent"
"#;
        let config = ArawnConfig::from_toml(toml).unwrap();
        let err = config.resolve_llm("agent").unwrap_err();
        assert!(matches!(err, crate::ConfigError::LlmNotFound { .. }));
    }

    #[test]
    fn test_merge_override() {
        let base_toml = r#"
[llm]
backend = "groq"
model = "base-model"

[server]
port = 8080
"#;
        let override_toml = r#"
[llm]
backend = "anthropic"
model = "override-model"

[server]
port = 9090
"#;
        let mut base = ArawnConfig::from_toml(base_toml).unwrap();
        let over = ArawnConfig::from_toml(override_toml).unwrap();
        base.merge(over);

        let llm = base.llm.as_ref().unwrap();
        assert_eq!(llm.backend, Some(Backend::Anthropic));
        assert_eq!(llm.model.as_deref(), Some("override-model"));
        assert_eq!(base.server.as_ref().unwrap().port, 9090);
    }

    #[test]
    fn test_merge_adds_profiles() {
        let base_toml = r#"
[llm]
backend = "groq"
model = "default"

[llm.claude]
backend = "anthropic"
model = "claude-model"
"#;
        let override_toml = r#"
[llm.fast]
backend = "groq"
model = "fast-model"
"#;
        let mut base = ArawnConfig::from_toml(base_toml).unwrap();
        let over = ArawnConfig::from_toml(override_toml).unwrap();
        base.merge(over);

        // Base default preserved
        assert!(base.llm.is_some());
        // Both profiles present
        assert!(base.llm_profiles.contains_key("claude"));
        assert!(base.llm_profiles.contains_key("fast"));
    }

    #[test]
    fn test_server_defaults() {
        let toml = r#"
[server]
"#;
        let config = ArawnConfig::from_toml(toml).unwrap();
        let server = config.server.as_ref().unwrap();
        assert_eq!(server.port, 8080);
        assert_eq!(server.bind, "127.0.0.1");
        assert!(server.rate_limiting);
    }

    #[test]
    fn test_backend_env_var() {
        assert_eq!(Backend::Anthropic.env_var(), "ANTHROPIC_API_KEY");
        assert_eq!(Backend::Groq.env_var(), "GROQ_API_KEY");
        assert_eq!(Backend::Openai.env_var(), "OPENAI_API_KEY");
    }

    #[test]
    fn test_plaintext_api_key_warning() {
        let toml = r#"
[llm]
backend = "groq"
model = "model"
api_key = "gsk_secret123"
"#;
        let config = ArawnConfig::from_toml(toml).unwrap();
        assert!(config.llm.as_ref().unwrap().has_plaintext_api_key());
    }

    #[test]
    fn test_llm_names() {
        let toml = r#"
[llm]
backend = "groq"
model = "default"

[llm.claude]
backend = "anthropic"
model = "claude"

[llm.fast]
backend = "groq"
model = "fast"
"#;
        let config = ArawnConfig::from_toml(toml).unwrap();
        let names = config.llm_names();
        assert_eq!(names, vec!["claude", "default", "fast"]);
    }

    #[test]
    fn test_parse_full_example() {
        let toml = r#"
[llm]
backend = "groq"
model = "llama-3.1-70b-versatile"

[llm.claude]
backend = "anthropic"
model = "claude-sonnet-4-20250514"

[llm.fast]
backend = "groq"
model = "llama-3.1-8b-instant"

[llm.local]
backend = "ollama"
model = "llama3.2"
base_url = "http://localhost:11434/v1"

[agent.default]
llm = "claude"

[agent.summarizer]
llm = "fast"
max_iterations = 3

[server]
port = 8080
bind = "127.0.0.1"
bootstrap_dir = "/home/user/.config/arawn/prompts"
"#;
        let config = ArawnConfig::from_toml(toml).unwrap();

        // Default LLM
        let llm = config.llm.as_ref().unwrap();
        assert_eq!(llm.backend, Some(Backend::Groq));

        // Named profiles
        assert_eq!(config.llm_profiles.len(), 3);
        assert_eq!(
            config.llm_profiles["local"].base_url.as_deref(),
            Some("http://localhost:11434/v1")
        );

        // Agent resolution
        let summarizer_llm = config.resolve_llm("summarizer").unwrap();
        assert_eq!(
            summarizer_llm.model.as_deref(),
            Some("llama-3.1-8b-instant")
        );

        let researcher_llm = config.resolve_llm("researcher").unwrap();
        assert_eq!(
            researcher_llm.model.as_deref(),
            Some("claude-sonnet-4-20250514")
        );

        // Server
        let server = config.server.as_ref().unwrap();
        assert_eq!(server.port, 8080);
    }

    #[test]
    fn test_roundtrip_toml() {
        let toml = r#"
[llm]
backend = "groq"
model = "default-model"

[llm.claude]
backend = "anthropic"
model = "claude-model"
"#;
        let config = ArawnConfig::from_toml(toml).unwrap();
        let serialized = config.to_toml().unwrap();
        let reparsed = ArawnConfig::from_toml(&serialized).unwrap();

        assert_eq!(reparsed.llm.as_ref().unwrap().backend, Some(Backend::Groq));
        assert!(reparsed.llm_profiles.contains_key("claude"));
    }

    // ── Embedding Config Tests ──────────────────────────────────────────

    #[test]
    fn test_embedding_defaults() {
        let config = EmbeddingConfig::default();
        assert_eq!(config.provider, EmbeddingProvider::Local);
        assert_eq!(config.effective_dimensions(), 384);
    }

    #[test]
    fn test_embedding_explicit_dimensions() {
        let config = EmbeddingConfig {
            provider: EmbeddingProvider::OpenAi,
            dimensions: Some(512),
            ..Default::default()
        };
        assert_eq!(config.effective_dimensions(), 512);
    }

    #[test]
    fn test_embedding_openai_default_dimensions() {
        let config = EmbeddingConfig {
            provider: EmbeddingProvider::OpenAi,
            dimensions: None,
            openai: None,
            ..Default::default()
        };
        assert_eq!(config.effective_dimensions(), 1536);
    }

    #[test]
    fn test_embedding_openai_provider_dimensions() {
        let config = EmbeddingConfig {
            provider: EmbeddingProvider::OpenAi,
            dimensions: None,
            openai: Some(EmbeddingOpenAiConfig {
                dimensions: Some(768),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert_eq!(config.effective_dimensions(), 768);
    }

    #[test]
    fn test_parse_embedding_config() {
        let toml = r#"
[llm]
backend = "groq"
model = "default"

[embedding]
provider = "openai"
dimensions = 512

[embedding.openai]
model = "text-embedding-3-large"
"#;
        let config = ArawnConfig::from_toml(toml).unwrap();
        let emb = config.embedding.as_ref().unwrap();
        assert_eq!(emb.provider, EmbeddingProvider::OpenAi);
        assert_eq!(emb.effective_dimensions(), 512);
        assert_eq!(emb.openai.as_ref().unwrap().model, "text-embedding-3-large");
    }

    #[test]
    fn test_parse_embedding_local_default() {
        let toml = r#"
[embedding]
provider = "local"
"#;
        let config = ArawnConfig::from_toml(toml).unwrap();
        let emb = config.embedding.as_ref().unwrap();
        assert_eq!(emb.provider, EmbeddingProvider::Local);
        assert_eq!(emb.effective_dimensions(), 384);
    }

    #[test]
    fn test_no_embedding_section_uses_default() {
        let toml = r#"
[llm]
backend = "groq"
model = "default"
"#;
        let config = ArawnConfig::from_toml(toml).unwrap();
        assert!(config.embedding.is_none());
        // Callers use unwrap_or_default()
        let emb = config.embedding.unwrap_or_default();
        assert_eq!(emb.provider, EmbeddingProvider::Local);
    }

    #[test]
    fn test_merge_embedding_override() {
        let base_toml = r#"
[embedding]
provider = "local"
"#;
        let override_toml = r#"
[embedding]
provider = "openai"
dimensions = 1536
"#;
        let mut base = ArawnConfig::from_toml(base_toml).unwrap();
        let over = ArawnConfig::from_toml(override_toml).unwrap();
        base.merge(over);

        let emb = base.embedding.as_ref().unwrap();
        assert_eq!(emb.provider, EmbeddingProvider::OpenAi);
        assert_eq!(emb.effective_dimensions(), 1536);
    }

    // ── Pipeline Config Tests ──────────────────────────────────────────

    #[test]
    fn test_pipeline_defaults() {
        let cfg = PipelineSection::default();
        assert!(cfg.enabled);
        assert!(cfg.database.is_none());
        assert!(cfg.workflow_dir.is_none());
        assert_eq!(cfg.max_concurrent_tasks, 4);
        assert!(cfg.cron_enabled);
        assert!(cfg.triggers_enabled);
    }

    #[test]
    fn test_parse_pipeline_config() {
        let toml = r#"
[pipeline]
enabled = true
database = "/data/pipeline.db"
workflow_dir = "/data/workflows"
max_concurrent_tasks = 8
task_timeout_secs = 600
cron_enabled = false
"#;
        let config = ArawnConfig::from_toml(toml).unwrap();
        let p = config.pipeline.as_ref().unwrap();
        assert!(p.enabled);
        assert_eq!(
            p.database.as_ref().unwrap().to_str().unwrap(),
            "/data/pipeline.db"
        );
        assert_eq!(
            p.workflow_dir.as_ref().unwrap().to_str().unwrap(),
            "/data/workflows"
        );
        assert_eq!(p.max_concurrent_tasks, 8);
        assert_eq!(p.task_timeout_secs, 600);
        assert!(!p.cron_enabled);
        assert!(p.triggers_enabled); // default
    }

    #[test]
    fn test_parse_pipeline_disabled() {
        let toml = r#"
[pipeline]
enabled = false
"#;
        let config = ArawnConfig::from_toml(toml).unwrap();
        let p = config.pipeline.as_ref().unwrap();
        assert!(!p.enabled);
    }

    #[test]
    fn test_no_pipeline_section_uses_default() {
        let toml = r#"
[llm]
backend = "groq"
model = "default"
"#;
        let config = ArawnConfig::from_toml(toml).unwrap();
        assert!(config.pipeline.is_none());
        let p = config.pipeline.unwrap_or_default();
        assert!(p.enabled);
    }

    // ── Memory / Recall Config Tests ────────────────────────────────────

    #[test]
    fn test_recall_defaults() {
        let cfg = RecallConfig::default();
        assert!(cfg.enabled);
        assert!((cfg.threshold - 0.6).abs() < f32::EPSILON);
        assert_eq!(cfg.limit, 5);
    }

    #[test]
    fn test_parse_recall_config() {
        let toml = r#"
[memory.recall]
enabled = false
threshold = 0.8
limit = 10
"#;
        let config = ArawnConfig::from_toml(toml).unwrap();
        let recall = &config.memory.as_ref().unwrap().recall;
        assert!(!recall.enabled);
        assert!((recall.threshold - 0.8).abs() < f32::EPSILON);
        assert_eq!(recall.limit, 10);
    }

    #[test]
    fn test_no_memory_section_uses_default() {
        let toml = r#"
[llm]
backend = "groq"
model = "default"
"#;
        let config = ArawnConfig::from_toml(toml).unwrap();
        assert!(config.memory.is_none());
        let recall = &config.memory.unwrap_or_default().recall;
        assert!(recall.enabled);
        assert!((recall.threshold - 0.6).abs() < f32::EPSILON);
        assert_eq!(recall.limit, 5);
    }

    #[test]
    fn test_merge_memory_override() {
        let base_toml = r#"
[memory.recall]
enabled = true
threshold = 0.5
limit = 3
"#;
        let override_toml = r#"
[memory.recall]
enabled = false
threshold = 0.9
limit = 20
"#;
        let mut base = ArawnConfig::from_toml(base_toml).unwrap();
        let over = ArawnConfig::from_toml(override_toml).unwrap();
        base.merge(over);

        let recall = &base.memory.as_ref().unwrap().recall;
        assert!(!recall.enabled);
        assert!((recall.threshold - 0.9).abs() < f32::EPSILON);
        assert_eq!(recall.limit, 20);
    }

    #[test]
    fn test_memory_indexing_defaults() {
        let toml = r#"
[memory]
"#;
        let config = ArawnConfig::from_toml(toml).unwrap();
        let memory = config.memory.unwrap();
        assert!(memory.indexing.enabled);
        assert_eq!(memory.indexing.backend, "openai");
        assert_eq!(memory.indexing.model, "gpt-4o-mini");
    }

    #[test]
    fn test_memory_confidence_defaults() {
        let toml = r#"
[memory]
"#;
        let config = ArawnConfig::from_toml(toml).unwrap();
        let memory = config.memory.unwrap();
        assert!((memory.confidence.fresh_days - 30.0).abs() < f32::EPSILON);
        assert!((memory.confidence.staleness_days - 365.0).abs() < f32::EPSILON);
        assert!((memory.confidence.staleness_floor - 0.3).abs() < f32::EPSILON);
        assert!((memory.confidence.reinforcement_cap - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_memory_indexing_override() {
        let toml = r#"
[memory.indexing]
enabled = false
backend = "groq"
model = "llama-3.3-70b-versatile"
"#;
        let config = ArawnConfig::from_toml(toml).unwrap();
        let indexing = &config.memory.unwrap().indexing;
        assert!(!indexing.enabled);
        assert_eq!(indexing.backend, "groq");
        assert_eq!(indexing.model, "llama-3.3-70b-versatile");
    }

    #[test]
    fn test_memory_confidence_override() {
        let toml = r#"
[memory.confidence]
fresh_days = 7.0
staleness_days = 90.0
staleness_floor = 0.1
reinforcement_cap = 2.0
"#;
        let config = ArawnConfig::from_toml(toml).unwrap();
        let conf = &config.memory.unwrap().confidence;
        assert!((conf.fresh_days - 7.0).abs() < f32::EPSILON);
        assert!((conf.staleness_days - 90.0).abs() < f32::EPSILON);
        assert!((conf.staleness_floor - 0.1).abs() < f32::EPSILON);
        assert!((conf.reinforcement_cap - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_memory_partial_sections() {
        // Only indexing specified, confidence should default
        let toml = r#"
[memory.indexing]
enabled = false
"#;
        let config = ArawnConfig::from_toml(toml).unwrap();
        let memory = config.memory.unwrap();
        assert!(!memory.indexing.enabled);
        // Confidence defaults
        assert!((memory.confidence.staleness_days - 365.0).abs() < f32::EPSILON);
        // Recall defaults
        assert!(memory.recall.enabled);
    }

    #[test]
    fn test_merge_memory_with_indexing() {
        let base_toml = r#"
[memory.indexing]
enabled = true
backend = "openai"
model = "gpt-4o-mini"
"#;
        let override_toml = r#"
[memory.indexing]
backend = "groq"
model = "llama-3.3-70b-versatile"
"#;
        let mut base = ArawnConfig::from_toml(base_toml).unwrap();
        let over = ArawnConfig::from_toml(override_toml).unwrap();
        base.merge(over);

        let indexing = &base.memory.as_ref().unwrap().indexing;
        assert_eq!(indexing.backend, "groq");
        assert_eq!(indexing.model, "llama-3.3-70b-versatile");
    }

    #[test]
    fn test_merge_pipeline_override() {
        let base_toml = r#"
[pipeline]
enabled = true
max_concurrent_tasks = 2
"#;
        let override_toml = r#"
[pipeline]
max_concurrent_tasks = 8
cron_enabled = false
"#;
        let mut base = ArawnConfig::from_toml(base_toml).unwrap();
        let over = ArawnConfig::from_toml(override_toml).unwrap();
        base.merge(over);

        let p = base.pipeline.as_ref().unwrap();
        assert!(p.enabled); // default from override
        assert_eq!(p.max_concurrent_tasks, 8);
        assert!(!p.cron_enabled);
    }

    // ── Plugin Subscription Tests ────────────────────────────────────────

    #[test]
    fn test_plugins_defaults() {
        let cfg = PluginsConfig::default();
        assert!(cfg.enabled);
        assert!(cfg.hot_reload);
        assert!(cfg.auto_update);
        assert!(cfg.subscriptions.is_empty());
        assert!(cfg.dirs.is_empty());
    }

    #[test]
    fn test_plugin_subscription_github() {
        let sub = PluginSubscription::github("owner/repo");
        assert_eq!(sub.source, PluginSource::GitHub);
        assert_eq!(sub.repo.as_deref(), Some("owner/repo"));
        assert!(sub.enabled);
        assert_eq!(sub.effective_ref(), "main");
    }

    #[test]
    fn test_plugin_subscription_url() {
        let sub = PluginSubscription::url("https://gitlab.com/team/plugin.git");
        assert_eq!(sub.source, PluginSource::Url);
        assert_eq!(
            sub.url.as_deref(),
            Some("https://gitlab.com/team/plugin.git")
        );
    }

    #[test]
    fn test_plugin_subscription_local() {
        let sub = PluginSubscription::local("/path/to/plugin");
        assert_eq!(sub.source, PluginSource::Local);
        assert_eq!(
            sub.path.as_ref().unwrap().to_str().unwrap(),
            "/path/to/plugin"
        );
    }

    #[test]
    fn test_plugin_subscription_with_ref() {
        let sub = PluginSubscription::github("owner/repo").with_ref("v1.0.0");
        assert_eq!(sub.git_ref.as_deref(), Some("v1.0.0"));
        assert_eq!(sub.effective_ref(), "v1.0.0");
    }

    #[test]
    fn test_plugin_subscription_id() {
        let github = PluginSubscription::github("owner/repo");
        assert_eq!(github.id(), "github/owner-repo");

        let url = PluginSubscription::url("https://example.com/plugin.git");
        assert!(url.id().starts_with("url/"));

        let local = PluginSubscription::local("/path/to/plugin");
        assert!(local.id().starts_with("local/"));
    }

    #[test]
    fn test_plugin_subscription_clone_url() {
        let github = PluginSubscription::github("owner/repo");
        assert_eq!(
            github.clone_url(),
            Some("https://github.com/owner/repo.git".to_string())
        );

        let url = PluginSubscription::url("https://gitlab.com/team/plugin.git");
        assert_eq!(
            url.clone_url(),
            Some("https://gitlab.com/team/plugin.git".to_string())
        );

        let local = PluginSubscription::local("/path/to/plugin");
        assert_eq!(local.clone_url(), None);
    }

    #[test]
    fn test_parse_plugin_subscriptions() {
        let toml = r#"
[plugins]
enabled = true
hot_reload = true
auto_update = false

[[plugins.subscriptions]]
source = "github"
repo = "owner/repo"
ref = "main"

[[plugins.subscriptions]]
source = "url"
url = "https://gitlab.com/team/plugin.git"
ref = "v1.0.0"

[[plugins.subscriptions]]
source = "local"
path = "/path/to/plugin"
enabled = false
"#;
        let config = ArawnConfig::from_toml(toml).unwrap();
        let plugins = config.plugins.as_ref().unwrap();
        assert!(plugins.enabled);
        assert!(plugins.hot_reload);
        assert!(!plugins.auto_update);
        assert_eq!(plugins.subscriptions.len(), 3);

        let sub0 = &plugins.subscriptions[0];
        assert_eq!(sub0.source, PluginSource::GitHub);
        assert_eq!(sub0.repo.as_deref(), Some("owner/repo"));
        assert_eq!(sub0.git_ref.as_deref(), Some("main"));
        assert!(sub0.enabled);

        let sub1 = &plugins.subscriptions[1];
        assert_eq!(sub1.source, PluginSource::Url);
        assert_eq!(
            sub1.url.as_deref(),
            Some("https://gitlab.com/team/plugin.git")
        );
        assert_eq!(sub1.git_ref.as_deref(), Some("v1.0.0"));

        let sub2 = &plugins.subscriptions[2];
        assert_eq!(sub2.source, PluginSource::Local);
        assert_eq!(
            sub2.path.as_ref().unwrap().to_str().unwrap(),
            "/path/to/plugin"
        );
        assert!(!sub2.enabled);
    }

    #[test]
    fn test_parse_plugins_no_subscriptions() {
        let toml = r#"
[plugins]
enabled = true
"#;
        let config = ArawnConfig::from_toml(toml).unwrap();
        let plugins = config.plugins.as_ref().unwrap();
        assert!(plugins.enabled);
        assert!(plugins.subscriptions.is_empty());
    }

    // ── Delegation Config Tests ────────────────────────────────────────────

    #[test]
    fn test_delegation_defaults() {
        let cfg = DelegationConfig::default();
        assert_eq!(cfg.max_result_len, 8000);
        assert!(!cfg.compaction.enabled);
        assert_eq!(cfg.compaction.threshold, 8000);
        assert_eq!(cfg.compaction.backend, "default");
        assert_eq!(cfg.compaction.model, "gpt-4o-mini");
        assert_eq!(cfg.compaction.target_len, 4000);
    }

    #[test]
    fn test_compaction_defaults() {
        let cfg = CompactionConfig::default();
        assert!(!cfg.enabled);
        assert_eq!(cfg.threshold, 8000);
        assert_eq!(cfg.backend, "default");
        assert_eq!(cfg.model, "gpt-4o-mini");
        assert_eq!(cfg.target_len, 4000);
    }

    #[test]
    fn test_parse_delegation_config() {
        let toml = r#"
[delegation]
max_result_len = 10000

[delegation.compaction]
enabled = true
threshold = 5000
backend = "fast"
model = "gpt-4o-mini"
target_len = 2000
"#;
        let config = ArawnConfig::from_toml(toml).unwrap();
        let delegation = config.delegation.as_ref().unwrap();
        assert_eq!(delegation.max_result_len, 10000);
        assert!(delegation.compaction.enabled);
        assert_eq!(delegation.compaction.threshold, 5000);
        assert_eq!(delegation.compaction.backend, "fast");
        assert_eq!(delegation.compaction.model, "gpt-4o-mini");
        assert_eq!(delegation.compaction.target_len, 2000);
    }

    #[test]
    fn test_parse_delegation_compaction_disabled() {
        let toml = r#"
[delegation]
max_result_len = 8000

[delegation.compaction]
enabled = false
"#;
        let config = ArawnConfig::from_toml(toml).unwrap();
        let delegation = config.delegation.as_ref().unwrap();
        assert_eq!(delegation.max_result_len, 8000);
        assert!(!delegation.compaction.enabled);
        // Other fields get defaults
        assert_eq!(delegation.compaction.threshold, 8000);
        assert_eq!(delegation.compaction.target_len, 4000);
    }

    #[test]
    fn test_no_delegation_section_uses_default() {
        let toml = r#"
[llm]
backend = "groq"
model = "default"
"#;
        let config = ArawnConfig::from_toml(toml).unwrap();
        assert!(config.delegation.is_none());
        let delegation = config.delegation.unwrap_or_default();
        assert_eq!(delegation.max_result_len, 8000);
        assert!(!delegation.compaction.enabled);
    }

    #[test]
    fn test_merge_delegation_override() {
        let base_toml = r#"
[delegation]
max_result_len = 8000

[delegation.compaction]
enabled = false
threshold = 8000
"#;
        let override_toml = r#"
[delegation]
max_result_len = 12000

[delegation.compaction]
enabled = true
threshold = 6000
model = "claude-haiku"
"#;
        let mut base = ArawnConfig::from_toml(base_toml).unwrap();
        let over = ArawnConfig::from_toml(override_toml).unwrap();
        base.merge(over);

        let delegation = base.delegation.as_ref().unwrap();
        assert_eq!(delegation.max_result_len, 12000);
        assert!(delegation.compaction.enabled);
        assert_eq!(delegation.compaction.threshold, 6000);
        assert_eq!(delegation.compaction.model, "claude-haiku");
    }

    // ── MCP Config Tests ────────────────────────────────────────────

    #[test]
    fn test_mcp_defaults() {
        let cfg = McpConfig::default();
        assert!(cfg.enabled);
        assert!(cfg.servers.is_empty());
    }

    #[test]
    fn test_mcp_server_entry_new() {
        let entry = McpServerEntry::new("sqlite", "mcp-server-sqlite");
        assert_eq!(entry.name, "sqlite");
        assert_eq!(entry.command, "mcp-server-sqlite");
        assert!(entry.args.is_empty());
        assert!(entry.env.is_empty());
        assert!(entry.enabled);
    }

    #[test]
    fn test_mcp_server_entry_builder() {
        let entry = McpServerEntry::new("sqlite", "mcp-server-sqlite")
            .with_arg("--db")
            .with_arg("/path/to/db.sqlite")
            .with_env("DEBUG", "1")
            .with_enabled(true);
        assert_eq!(entry.args, vec!["--db", "/path/to/db.sqlite"]);
        assert_eq!(entry.env, vec![["DEBUG".to_string(), "1".to_string()]]);
        assert!(entry.enabled);
    }

    #[test]
    fn test_mcp_server_entry_env_tuples() {
        let entry = McpServerEntry::new("test", "cmd")
            .with_env("KEY1", "value1")
            .with_env("KEY2", "value2");
        let tuples = entry.env_tuples();
        assert_eq!(
            tuples,
            vec![
                ("KEY1".to_string(), "value1".to_string()),
                ("KEY2".to_string(), "value2".to_string())
            ]
        );
    }

    #[test]
    fn test_parse_mcp_config() {
        let toml = r#"
[mcp]
enabled = true

[[mcp.servers]]
name = "sqlite"
command = "mcp-server-sqlite"
args = ["--db", "/path/to/db.sqlite"]

[[mcp.servers]]
name = "filesystem"
command = "mcp-server-filesystem"
args = ["--allowed-dirs", "/home/user"]
env = [["DEBUG", "1"]]
enabled = false
"#;
        let config = ArawnConfig::from_toml(toml).unwrap();
        let mcp = config.mcp.as_ref().unwrap();
        assert!(mcp.enabled);
        assert_eq!(mcp.servers.len(), 2);

        let s0 = &mcp.servers[0];
        assert_eq!(s0.name, "sqlite");
        assert_eq!(s0.command, "mcp-server-sqlite");
        assert_eq!(s0.args, vec!["--db", "/path/to/db.sqlite"]);
        assert!(s0.enabled);

        let s1 = &mcp.servers[1];
        assert_eq!(s1.name, "filesystem");
        assert_eq!(s1.command, "mcp-server-filesystem");
        assert_eq!(s1.args, vec!["--allowed-dirs", "/home/user"]);
        assert_eq!(s1.env, vec![["DEBUG".to_string(), "1".to_string()]]);
        assert!(!s1.enabled);
    }

    #[test]
    fn test_parse_mcp_disabled() {
        let toml = r#"
[mcp]
enabled = false
"#;
        let config = ArawnConfig::from_toml(toml).unwrap();
        let mcp = config.mcp.as_ref().unwrap();
        assert!(!mcp.enabled);
        assert!(mcp.servers.is_empty());
    }

    #[test]
    fn test_no_mcp_section_uses_default() {
        let toml = r#"
[llm]
backend = "groq"
model = "default"
"#;
        let config = ArawnConfig::from_toml(toml).unwrap();
        assert!(config.mcp.is_none());
        let mcp = config.mcp.unwrap_or_default();
        assert!(mcp.enabled);
        assert!(mcp.servers.is_empty());
    }

    #[test]
    fn test_merge_mcp_override() {
        let base_toml = r#"
[mcp]
enabled = true

[[mcp.servers]]
name = "sqlite"
command = "mcp-server-sqlite"
"#;
        let override_toml = r#"
[mcp]
enabled = false

[[mcp.servers]]
name = "other"
command = "other-server"
"#;
        let mut base = ArawnConfig::from_toml(base_toml).unwrap();
        let over = ArawnConfig::from_toml(override_toml).unwrap();
        base.merge(over);

        let mcp = base.mcp.as_ref().unwrap();
        assert!(!mcp.enabled);
        // Override replaces entire section
        assert_eq!(mcp.servers.len(), 1);
        assert_eq!(mcp.servers[0].name, "other");
    }

    // ── Context Limit Tests ────────────────────────────────────────────

    #[test]
    fn test_model_config_parses_max_context_tokens() {
        let toml = r#"
[llm]
backend = "anthropic"
model = "claude-sonnet-4-20250514"
max_context_tokens = 200000
"#;
        let config = ArawnConfig::from_toml(toml).unwrap();
        let llm = config.llm.as_ref().unwrap();
        assert_eq!(llm.max_context_tokens, Some(200_000));
    }

    #[test]
    fn test_model_config_context_tokens_in_profile() {
        let toml = r#"
[llm.claude]
backend = "anthropic"
model = "claude-sonnet-4-20250514"
max_context_tokens = 200000

[llm.fast]
backend = "groq"
model = "llama-3.1-70b-versatile"
max_context_tokens = 32000
"#;
        let config = ArawnConfig::from_toml(toml).unwrap();

        let claude = &config.llm_profiles["claude"];
        assert_eq!(claude.max_context_tokens, Some(200_000));

        let fast = &config.llm_profiles["fast"];
        assert_eq!(fast.max_context_tokens, Some(32_000));
    }

    #[test]
    fn test_require_max_context_tokens_success() {
        let llm = LlmConfig {
            model: Some("test-model".to_string()),
            max_context_tokens: Some(100_000),
            ..Default::default()
        };
        assert_eq!(llm.require_max_context_tokens().unwrap(), 100_000);
    }

    #[test]
    fn test_require_max_context_tokens_error() {
        let llm = LlmConfig {
            model: Some("unknown-model".to_string()),
            max_context_tokens: None,
            ..Default::default()
        };
        let err = llm.require_max_context_tokens().unwrap_err();
        assert!(matches!(
            err,
            crate::ConfigError::MissingContextLimit { .. }
        ));
    }

    #[test]
    fn test_model_context_roundtrip() {
        let toml = r#"
[llm]
backend = "anthropic"
model = "claude-sonnet-4-20250514"
max_context_tokens = 200000
"#;
        let config = ArawnConfig::from_toml(toml).unwrap();
        let serialized = config.to_toml().unwrap();
        let reparsed = ArawnConfig::from_toml(&serialized).unwrap();
        assert_eq!(
            reparsed.llm.as_ref().unwrap().max_context_tokens,
            Some(200_000)
        );
    }

    // ── Path Config Tests ────────────────────────────────────────────

    #[test]
    fn test_parse_paths_config() {
        let toml = r#"
[paths]
base_path = "/custom/arawn"

[paths.usage]
total_warning_gb = 20
workstream_warning_gb = 2
session_warning_mb = 500

[paths.cleanup]
scratch_cleanup_days = 14
dry_run = true

[paths.monitoring]
enabled = false
debounce_ms = 1000
polling_interval_secs = 60
"#;
        let config = ArawnConfig::from_toml(toml).unwrap();
        let paths = config.paths.as_ref().unwrap();
        assert_eq!(
            paths.base_path.as_ref().unwrap().to_str().unwrap(),
            "/custom/arawn"
        );
        assert_eq!(paths.usage.total_warning_gb, 20);
        assert_eq!(paths.usage.workstream_warning_gb, 2);
        assert_eq!(paths.usage.session_warning_mb, 500);
        assert_eq!(paths.cleanup.scratch_cleanup_days, 14);
        assert!(paths.cleanup.dry_run);
        assert!(!paths.monitoring.enabled);
        assert_eq!(paths.monitoring.debounce_ms, 1000);
        assert_eq!(paths.monitoring.polling_interval_secs, 60);
    }

    #[test]
    fn test_no_paths_section_uses_default() {
        let toml = r#"
[llm]
backend = "groq"
model = "default"
"#;
        let config = ArawnConfig::from_toml(toml).unwrap();
        assert!(config.paths.is_none());
        let paths = config.paths.unwrap_or_default();
        assert!(paths.base_path.is_none());
        assert_eq!(paths.usage.total_warning_gb, 10);
        assert!(paths.monitoring.enabled);
    }

    #[test]
    fn test_merge_paths_override() {
        let base_toml = r#"
[paths]
base_path = "/base/path"

[paths.usage]
total_warning_gb = 10
"#;
        let override_toml = r#"
[paths]
base_path = "/override/path"

[paths.usage]
total_warning_gb = 50
"#;
        let mut base = ArawnConfig::from_toml(base_toml).unwrap();
        let over = ArawnConfig::from_toml(override_toml).unwrap();
        base.merge(over);

        let paths = base.paths.as_ref().unwrap();
        assert_eq!(
            paths.base_path.as_ref().unwrap().to_str().unwrap(),
            "/override/path"
        );
        assert_eq!(paths.usage.total_warning_gb, 50);
    }

    #[test]
    fn test_paths_roundtrip() {
        let toml = r#"
[paths]
base_path = "/my/arawn"

[paths.usage]
total_warning_gb = 15
workstream_warning_gb = 3
session_warning_mb = 300

[paths.cleanup]
scratch_cleanup_days = 30
dry_run = false

[paths.monitoring]
enabled = true
debounce_ms = 250
polling_interval_secs = 15
"#;
        let config = ArawnConfig::from_toml(toml).unwrap();
        let serialized = config.to_toml().unwrap();
        let reparsed = ArawnConfig::from_toml(&serialized).unwrap();

        let paths = reparsed.paths.as_ref().unwrap();
        assert_eq!(
            paths.base_path.as_ref().unwrap().to_str().unwrap(),
            "/my/arawn"
        );
        assert_eq!(paths.usage.total_warning_gb, 15);
        assert_eq!(paths.cleanup.scratch_cleanup_days, 30);
        assert!(paths.monitoring.enabled);
    }

    #[test]
    fn test_tool_output_config_per_tool_fields() {
        let toml = r#"
[tools.output]
max_size_bytes = 102400
shell = 204800
file_read = 1048576
web_fetch = 512000
search = 25600
"#;
        let config = ArawnConfig::from_toml(toml).unwrap();
        let tools = config.tools.unwrap();
        assert_eq!(tools.output.max_size_bytes, 102400);
        assert_eq!(tools.output.shell, Some(204800));
        assert_eq!(tools.output.file_read, Some(1048576));
        assert_eq!(tools.output.web_fetch, Some(512000));
        assert_eq!(tools.output.search, Some(25600));
    }

    #[test]
    fn test_tool_output_config_defaults_none() {
        let toml = r#"
[tools.output]
max_size_bytes = 102400
"#;
        let config = ArawnConfig::from_toml(toml).unwrap();
        let tools = config.tools.unwrap();
        assert_eq!(tools.output.max_size_bytes, 102400);
        assert!(tools.output.shell.is_none());
        assert!(tools.output.file_read.is_none());
        assert!(tools.output.web_fetch.is_none());
        assert!(tools.output.search.is_none());
    }

    #[test]
    fn test_rlm_config_deserialization() {
        let toml = r#"
[rlm]
model = "claude-haiku-4-5-20251001"
max_turns = 30
max_context_tokens = 80000
compaction_threshold = 0.6
max_compactions = 5
max_total_tokens = 200000
compaction_model = "claude-haiku-4-5-20251001"
"#;
        let config = ArawnConfig::from_toml(toml).unwrap();
        let rlm = config.rlm.as_ref().unwrap();
        assert_eq!(rlm.model.as_deref(), Some("claude-haiku-4-5-20251001"));
        assert_eq!(rlm.max_turns, Some(30));
        assert_eq!(rlm.max_context_tokens, Some(80000));
        assert_eq!(rlm.compaction_threshold, Some(0.6));
        assert_eq!(rlm.max_compactions, Some(5));
        assert_eq!(rlm.max_total_tokens, Some(200000));
        assert_eq!(
            rlm.compaction_model.as_deref(),
            Some("claude-haiku-4-5-20251001")
        );
    }

    #[test]
    fn test_rlm_config_defaults() {
        let toml = r#"
[rlm]
"#;
        let config = ArawnConfig::from_toml(toml).unwrap();
        let rlm = config.rlm.as_ref().unwrap();
        assert!(rlm.model.is_none());
        assert!(rlm.max_turns.is_none());
        assert!(rlm.max_context_tokens.is_none());
        assert!(rlm.compaction_threshold.is_none());
        assert!(rlm.max_compactions.is_none());
        assert!(rlm.max_total_tokens.is_none());
        assert!(rlm.compaction_model.is_none());
    }

    #[test]
    fn test_rlm_config_partial() {
        let toml = r#"
[rlm]
max_turns = 10
compaction_threshold = 0.5
"#;
        let config = ArawnConfig::from_toml(toml).unwrap();
        let rlm = config.rlm.as_ref().unwrap();
        assert_eq!(rlm.max_turns, Some(10));
        assert_eq!(rlm.compaction_threshold, Some(0.5));
        assert!(rlm.model.is_none());
        assert!(rlm.max_context_tokens.is_none());
    }

    #[test]
    fn test_rlm_config_absent() {
        let config = ArawnConfig::new();
        assert!(config.rlm.is_none());
    }

    #[test]
    fn test_rlm_config_merge() {
        let base_toml = r#"
[rlm]
max_turns = 20
model = "base-model"
"#;
        let override_toml = r#"
[rlm]
max_turns = 50
compaction_threshold = 0.8
"#;
        let mut base = ArawnConfig::from_toml(base_toml).unwrap();
        let over = ArawnConfig::from_toml(override_toml).unwrap();
        base.merge(over);

        let rlm = base.rlm.as_ref().unwrap();
        // Override replaces entire section
        assert_eq!(rlm.max_turns, Some(50));
        assert_eq!(rlm.compaction_threshold, Some(0.8));
        // Note: merge replaces the whole Option, so base-only fields are lost
        // This is consistent with how other config sections merge
    }
}
