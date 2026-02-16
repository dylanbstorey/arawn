//! Configuration error types.

/// Result type alias for config operations.
pub type Result<T> = std::result::Result<T, ConfigError>;

/// Errors that can occur during configuration loading and resolution.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// Failed to read a config file.
    #[error("failed to read config file '{path}': {source}")]
    ReadFile {
        path: String,
        source: std::io::Error,
    },

    /// Failed to write a config file.
    #[error("failed to write config file '{path}': {source}")]
    WriteFile {
        path: String,
        source: std::io::Error,
    },

    /// Failed to parse TOML.
    #[error("failed to parse config: {0}")]
    Parse(#[from] toml::de::Error),

    /// Failed to serialize config.
    #[error("failed to serialize config: {0}")]
    Serialize(#[from] toml::ser::Error),

    /// Referenced LLM config not found.
    #[error("LLM config '{name}' not found (referenced by {context})")]
    LlmNotFound { name: String, context: String },

    /// No default LLM configured.
    #[error("no default LLM configured — add an [llm] section to your config")]
    NoDefaultLlm,

    /// Missing required field.
    #[error("missing required field '{field}' in {context}")]
    MissingField { field: String, context: String },

    /// API key not found through any resolution method.
    #[error(
        "API key not found for backend '{backend}'. Set via keyring (arawn config set-secret), env var ({env_var}), or config file"
    )]
    ApiKeyNotFound { backend: String, env_var: String },

    /// Context limit not configured for model.
    #[error(
        "max_context_tokens not configured for model '{model}'. Add max_context_tokens to your LLM config"
    )]
    MissingContextLimit { model: String },

    /// Failed to parse YAML.
    #[error("failed to parse YAML config: {0}")]
    ParseYaml(String),

    /// Context not found.
    #[error("context '{0}' not found")]
    ContextNotFound(String),

    /// Other error.
    #[error("{0}")]
    Other(String),
}
