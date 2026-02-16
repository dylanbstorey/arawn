//! LLM config resolution — turns named config references into concrete backend configs.
//!
//! Takes an [`ArawnConfig`] and resolves a fully-qualified [`ResolvedLlm`] for
//! a given agent, handling cascading defaults and API key lookup.

use crate::secrets::{self, SecretSource};
use crate::{ArawnConfig, Backend, ConfigError, LlmConfig, Result};

/// A fully resolved LLM configuration ready to construct a backend.
#[derive(Debug, Clone)]
pub struct ResolvedLlm {
    /// The backend provider.
    pub backend: Backend,
    /// Model identifier.
    pub model: String,
    /// API base URL (if custom).
    pub base_url: Option<String>,
    /// Resolved API key.
    pub api_key: Option<String>,
    /// Where the API key was resolved from.
    pub api_key_source: Option<SecretSource>,
    /// How the config was resolved.
    pub resolved_from: ResolvedFrom,
    /// Maximum retry attempts for failed requests.
    pub retry_max: Option<u32>,
    /// Backoff delay between retries in milliseconds.
    pub retry_backoff_ms: Option<u64>,
}

/// Tracks how the LLM config was resolved for diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedFrom {
    /// Resolved from agent-specific config (`agent.<name>.llm`).
    AgentSpecific { agent: String, profile: String },
    /// Resolved from agent default (`agent.default.llm`).
    AgentDefault { profile: String },
    /// Resolved from global default (`[llm]`).
    GlobalDefault,
}

impl std::fmt::Display for ResolvedFrom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolvedFrom::AgentSpecific { agent, profile } => {
                write!(f, "agent.{}.llm -> llm.{}", agent, profile)
            }
            ResolvedFrom::AgentDefault { profile } => {
                write!(f, "agent.default.llm -> llm.{}", profile)
            }
            ResolvedFrom::GlobalDefault => write!(f, "[llm] (global default)"),
        }
    }
}

/// How an API key was resolved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiKeySource {
    /// From OS keyring.
    Keyring,
    /// From environment variable.
    EnvVar(String),
    /// From config file (not recommended).
    ConfigFile,
    /// No API key found (may be fine for local backends like Ollama).
    NotFound,
}

/// Resolve the LLM config for a given agent name.
///
/// This performs the full cascading resolution and API key lookup:
/// 1. Resolve which `LlmConfig` applies (agent-specific → agent.default → global)
/// 2. Validate required fields (backend, model)
/// 3. Resolve API key (keyring → env var → config file)
pub fn resolve_for_agent(config: &ArawnConfig, agent_name: &str) -> Result<ResolvedLlm> {
    let (llm_config, resolved_from) = resolve_llm_config(config, agent_name)?;

    let backend = llm_config
        .backend
        .ok_or_else(|| ConfigError::MissingField {
            field: "backend".to_string(),
            context: format!("LLM config (resolved via {})", resolved_from),
        })?;

    let model = llm_config
        .model
        .clone()
        .ok_or_else(|| ConfigError::MissingField {
            field: "model".to_string(),
            context: format!("LLM config (resolved via {})", resolved_from),
        })?;

    // Resolve API key via secrets module (keyring → env var → config file)
    let resolved_secret = secrets::resolve_api_key(&backend, llm_config.api_key.as_deref());

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
        resolved_from,
        retry_max: llm_config.retry_max,
        retry_backoff_ms: llm_config.retry_backoff_ms,
    })
}

/// Resolve all named LLM configs into a summary for diagnostics.
pub fn resolve_all_profiles(config: &ArawnConfig) -> Vec<(String, Backend, String)> {
    let mut profiles = Vec::new();

    if let Some(ref llm) = config.llm {
        if let (Some(backend), Some(model)) = (llm.backend, &llm.model) {
            profiles.push(("default".to_string(), backend, model.clone()));
        }
    }

    for (name, llm) in &config.llm_profiles {
        if let (Some(backend), Some(model)) = (llm.backend, &llm.model) {
            profiles.push((name.clone(), backend, model.clone()));
        }
    }

    profiles.sort_by(|a, b| a.0.cmp(&b.0));
    profiles
}

/// Inner resolution that returns both the config ref and how it was resolved.
fn resolve_llm_config<'a>(
    config: &'a ArawnConfig,
    agent_name: &str,
) -> Result<(&'a LlmConfig, ResolvedFrom)> {
    // 1. Agent-specific
    if let Some(agent_cfg) = config.agent.get(agent_name) {
        if let Some(ref llm_name) = agent_cfg.llm {
            let llm = config.llm_profiles.get(llm_name.as_str()).ok_or_else(|| {
                ConfigError::LlmNotFound {
                    name: llm_name.clone(),
                    context: format!("agent.{}", agent_name),
                }
            })?;
            return Ok((
                llm,
                ResolvedFrom::AgentSpecific {
                    agent: agent_name.to_string(),
                    profile: llm_name.clone(),
                },
            ));
        }
    }

    // 2. Agent default
    if let Some(default_cfg) = config.agent.get("default") {
        if let Some(ref llm_name) = default_cfg.llm {
            let llm = config.llm_profiles.get(llm_name.as_str()).ok_or_else(|| {
                ConfigError::LlmNotFound {
                    name: llm_name.clone(),
                    context: "agent.default".to_string(),
                }
            })?;
            return Ok((
                llm,
                ResolvedFrom::AgentDefault {
                    profile: llm_name.clone(),
                },
            ));
        }
    }

    // 3. Global default
    let llm = config.llm.as_ref().ok_or(ConfigError::NoDefaultLlm)?;
    Ok((llm, ResolvedFrom::GlobalDefault))
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> ArawnConfig {
        ArawnConfig::from_toml(
            r#"
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
"#,
        )
        .unwrap()
    }

    #[test]
    fn test_resolve_agent_specific() {
        let config = test_config();
        let resolved = resolve_for_agent(&config, "summarizer").unwrap();

        assert_eq!(resolved.backend, Backend::Groq);
        assert_eq!(resolved.model, "llama-3.1-8b-instant");
        assert!(matches!(
            resolved.resolved_from,
            ResolvedFrom::AgentSpecific { ref agent, ref profile }
            if agent == "summarizer" && profile == "fast"
        ));
    }

    #[test]
    fn test_resolve_agent_default_fallback() {
        let config = test_config();
        let resolved = resolve_for_agent(&config, "researcher").unwrap();

        assert_eq!(resolved.backend, Backend::Anthropic);
        assert_eq!(resolved.model, "claude-sonnet-4-20250514");
        assert!(matches!(
            resolved.resolved_from,
            ResolvedFrom::AgentDefault { ref profile } if profile == "claude"
        ));
    }

    #[test]
    fn test_resolve_global_default() {
        let toml = r#"
[llm]
backend = "groq"
model = "global-model"
"#;
        let config = ArawnConfig::from_toml(toml).unwrap();
        let resolved = resolve_for_agent(&config, "anything").unwrap();

        assert_eq!(resolved.backend, Backend::Groq);
        assert_eq!(resolved.model, "global-model");
        assert_eq!(resolved.resolved_from, ResolvedFrom::GlobalDefault);
    }

    #[test]
    fn test_resolve_no_config() {
        let config = ArawnConfig::new();
        let err = resolve_for_agent(&config, "agent").unwrap_err();
        assert!(matches!(err, ConfigError::NoDefaultLlm));
    }

    #[test]
    fn test_resolve_missing_backend() {
        let toml = r#"
[llm]
model = "some-model"
"#;
        let config = ArawnConfig::from_toml(toml).unwrap();
        let err = resolve_for_agent(&config, "agent").unwrap_err();
        assert!(matches!(err, ConfigError::MissingField { ref field, .. } if field == "backend"));
    }

    #[test]
    fn test_resolve_missing_model() {
        let toml = r#"
[llm]
backend = "groq"
"#;
        let config = ArawnConfig::from_toml(toml).unwrap();
        let err = resolve_for_agent(&config, "agent").unwrap_err();
        assert!(matches!(err, ConfigError::MissingField { ref field, .. } if field == "model"));
    }

    #[test]
    fn test_resolve_missing_profile_reference() {
        let toml = r#"
[llm]
backend = "groq"
model = "default"

[agent.default]
llm = "nonexistent"
"#;
        let config = ArawnConfig::from_toml(toml).unwrap();
        let err = resolve_for_agent(&config, "agent").unwrap_err();
        assert!(matches!(err, ConfigError::LlmNotFound { ref name, .. } if name == "nonexistent"));
    }

    #[test]
    fn test_resolve_with_base_url() {
        let config = test_config();
        let resolved = resolve_for_agent(&config, "local_agent").unwrap();

        // Falls back to agent.default → claude (no base_url)
        assert!(resolved.base_url.is_none());
    }

    #[test]
    fn test_resolve_ollama_no_api_key_needed() {
        let toml = r#"
[llm]
backend = "ollama"
model = "llama3.2"
base_url = "http://localhost:11434/v1"
"#;
        let config = ArawnConfig::from_toml(toml).unwrap();
        let resolved = resolve_for_agent(&config, "agent").unwrap();

        assert_eq!(resolved.backend, Backend::Ollama);
        // No API key is fine for Ollama
        assert!(resolved.api_key.is_none());
    }

    #[test]
    fn test_resolve_api_key_from_config() {
        let toml = r#"
[llm]
backend = "groq"
model = "model"
api_key = "config-key"
"#;
        let config = ArawnConfig::from_toml(toml).unwrap();
        let resolved = resolve_for_agent(&config, "agent").unwrap();
        assert_eq!(resolved.api_key.as_deref(), Some("config-key"));
    }

    #[test]
    fn test_resolve_all_profiles() {
        let config = test_config();
        let profiles = resolve_all_profiles(&config);

        assert!(profiles.iter().any(|(n, _, _)| n == "default"));
        assert!(profiles.iter().any(|(n, _, _)| n == "claude"));
        assert!(profiles.iter().any(|(n, _, _)| n == "fast"));
        assert!(profiles.iter().any(|(n, _, _)| n == "local"));
    }

    #[test]
    fn test_resolved_from_display() {
        let from = ResolvedFrom::AgentSpecific {
            agent: "summarizer".to_string(),
            profile: "fast".to_string(),
        };
        assert_eq!(from.to_string(), "agent.summarizer.llm -> llm.fast");

        let from = ResolvedFrom::AgentDefault {
            profile: "claude".to_string(),
        };
        assert_eq!(from.to_string(), "agent.default.llm -> llm.claude");

        let from = ResolvedFrom::GlobalDefault;
        assert_eq!(from.to_string(), "[llm] (global default)");
    }
}
