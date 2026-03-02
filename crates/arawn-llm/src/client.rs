//! High-level LLM client with provider routing and fallback.
//!
//! The [`LlmClient`] provides a unified interface for working with multiple LLM
//! providers. It supports:
//!
//! - **Primary/Fallback routing**: Automatically fail over to backup providers
//! - **Provider selection**: Route requests to specific backends by name
//! - **Unified configuration**: Configure all providers from a single config
//!
//! # Example
//!
//! ```rust,ignore
//! use arawn_llm::{LlmClient, LlmClientConfig};
//!
//! let client = LlmClient::from_env().await?;
//!
//! // Use default provider
//! let response = client.complete(request).await?;
//!
//! // Use specific provider
//! let response = client.complete_with("anthropic", request).await?;
//! ```

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

use crate::CompletionResponse;
use crate::anthropic::{AnthropicBackend, AnthropicConfig};
use crate::backend::{LlmBackend, ResponseStream, SharedBackend};
use crate::error::{LlmError, Result};
use crate::openai::{OpenAiBackend, OpenAiConfig};
use crate::types::CompletionRequest;

// ─────────────────────────────────────────────────────────────────────────────
// Provider Enum
// ─────────────────────────────────────────────────────────────────────────────

/// Supported LLM providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Provider {
    /// Anthropic Claude API
    Anthropic,
    /// OpenAI API
    OpenAi,
    /// Groq cloud inference
    Groq,
    /// Local Ollama instance
    Ollama,
}

impl Provider {
    /// Get the string name for this provider.
    pub fn name(&self) -> &'static str {
        match self {
            Provider::Anthropic => "anthropic",
            Provider::OpenAi => "openai",
            Provider::Groq => "groq",
            Provider::Ollama => "ollama",
        }
    }

    /// Parse a provider from a string name.
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "anthropic" | "claude" => Some(Provider::Anthropic),
            "openai" | "gpt" => Some(Provider::OpenAi),
            "groq" => Some(Provider::Groq),
            "ollama" | "local" => Some(Provider::Ollama),
            _ => None,
        }
    }

    /// Check if this provider requires an API key.
    pub fn requires_api_key(&self) -> bool {
        match self {
            Provider::Anthropic | Provider::OpenAi | Provider::Groq => true,
            Provider::Ollama => false,
        }
    }
}

impl std::fmt::Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Client Configuration
// ─────────────────────────────────────────────────────────────────────────────

/// Configuration for the LLM client.
#[derive(Debug, Clone, Default)]
pub struct LlmClientConfig {
    /// Anthropic configuration (optional).
    pub anthropic: Option<AnthropicConfig>,

    /// OpenAI configuration (optional).
    pub openai: Option<OpenAiConfig>,

    /// Groq configuration (optional).
    pub groq: Option<OpenAiConfig>,

    /// Ollama configuration (optional).
    pub ollama: Option<OpenAiConfig>,

    /// Primary provider to use.
    pub primary: Option<Provider>,

    /// Fallback providers in order of preference.
    pub fallbacks: Vec<Provider>,

    /// Enable automatic fallback on provider errors.
    pub auto_fallback: bool,
}

impl LlmClientConfig {
    /// Create a new empty configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Configure Anthropic backend.
    pub fn with_anthropic(mut self, config: AnthropicConfig) -> Self {
        self.anthropic = Some(config);
        self
    }

    /// Configure OpenAI backend.
    pub fn with_openai(mut self, config: OpenAiConfig) -> Self {
        self.openai = Some(config);
        self
    }

    /// Configure Groq backend.
    pub fn with_groq(mut self, config: OpenAiConfig) -> Self {
        self.groq = Some(config);
        self
    }

    /// Configure Ollama backend.
    pub fn with_ollama(mut self, config: OpenAiConfig) -> Self {
        self.ollama = Some(config);
        self
    }

    /// Set the primary provider.
    pub fn with_primary(mut self, provider: Provider) -> Self {
        self.primary = Some(provider);
        self
    }

    /// Set fallback providers.
    pub fn with_fallbacks(mut self, providers: Vec<Provider>) -> Self {
        self.fallbacks = providers;
        self
    }

    /// Enable automatic fallback.
    pub fn with_auto_fallback(mut self, enabled: bool) -> Self {
        self.auto_fallback = enabled;
        self
    }

    /// Create configuration from environment variables.
    ///
    /// Looks for:
    /// - `ANTHROPIC_API_KEY` - Anthropic configuration
    /// - `OPENAI_API_KEY` - OpenAI configuration
    /// - `GROQ_API_KEY` - Groq configuration
    /// - `OLLAMA_HOST` - Ollama configuration (defaults to localhost)
    /// - `LLM_PRIMARY` - Primary provider name
    /// - `LLM_FALLBACK` - Comma-separated fallback providers
    pub fn from_env() -> Self {
        let mut config = Self::new();

        // Configure available providers
        if let Ok(config_result) = AnthropicConfig::from_env() {
            config.anthropic = Some(config_result);
        }

        if let Ok(config_result) = OpenAiConfig::openai_from_env() {
            config.openai = Some(config_result);
        }

        if let Ok(config_result) = OpenAiConfig::groq_from_env() {
            config.groq = Some(config_result);
        }

        // Ollama is always available (no key required)
        let ollama_config = if let Ok(host) = std::env::var("OLLAMA_HOST") {
            OpenAiConfig::ollama().with_base_url(format!("{}/v1", host))
        } else {
            OpenAiConfig::ollama()
        };
        config.ollama = Some(ollama_config);

        // Set primary provider
        if let Ok(primary) = std::env::var("LLM_PRIMARY")
            && let Some(provider) = Provider::from_name(&primary)
        {
            config.primary = Some(provider);
        }

        // Set fallbacks
        if let Ok(fallbacks) = std::env::var("LLM_FALLBACK") {
            config.fallbacks = fallbacks
                .split(',')
                .filter_map(|s| Provider::from_name(s.trim()))
                .collect();
        }

        config
    }

    /// Determine the primary provider based on what's configured.
    fn determine_primary(&self) -> Option<Provider> {
        if let Some(p) = self.primary {
            // Check if configured primary is actually available
            if self.is_provider_configured(p) {
                return Some(p);
            }
        }

        // Fall back to first configured provider (preference order)
        let preference = [
            Provider::Anthropic,
            Provider::OpenAi,
            Provider::Groq,
            Provider::Ollama,
        ];

        preference
            .into_iter()
            .find(|&provider| self.is_provider_configured(provider))
    }

    /// Check if a provider is configured.
    fn is_provider_configured(&self, provider: Provider) -> bool {
        match provider {
            Provider::Anthropic => self.anthropic.is_some(),
            Provider::OpenAi => self.openai.is_some(),
            Provider::Groq => self.groq.is_some(),
            Provider::Ollama => self.ollama.is_some(),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// LLM Client
// ─────────────────────────────────────────────────────────────────────────────

/// High-level LLM client with multi-provider support.
///
/// The client wraps multiple backends and provides:
/// - Unified interface for all providers
/// - Automatic fallback on failures
/// - Provider selection by name
pub struct LlmClient {
    backends: HashMap<Provider, SharedBackend>,
    primary: Provider,
    fallbacks: Vec<Provider>,
    auto_fallback: bool,
}

impl LlmClient {
    /// Create a new client from configuration.
    pub fn new(config: LlmClientConfig) -> Result<Self> {
        let mut backends: HashMap<Provider, SharedBackend> = HashMap::new();

        // Initialize configured backends
        if let Some(ref anthropic_config) = config.anthropic {
            let backend = AnthropicBackend::new(anthropic_config.clone())?;
            backends.insert(Provider::Anthropic, Arc::new(backend));
        }

        if let Some(ref openai_config) = config.openai {
            let backend = OpenAiBackend::new(openai_config.clone())?;
            backends.insert(Provider::OpenAi, Arc::new(backend));
        }

        if let Some(ref groq_config) = config.groq {
            let backend = OpenAiBackend::new(groq_config.clone())?;
            backends.insert(Provider::Groq, Arc::new(backend));
        }

        if let Some(ref ollama_config) = config.ollama {
            let backend = OpenAiBackend::new(ollama_config.clone())?;
            backends.insert(Provider::Ollama, Arc::new(backend));
        }

        if backends.is_empty() {
            return Err(LlmError::Config(
                "No LLM providers configured. Set ANTHROPIC_API_KEY, OPENAI_API_KEY, GROQ_API_KEY, or ensure Ollama is running.".to_string()
            ));
        }

        let primary = config.determine_primary().ok_or_else(|| {
            LlmError::Config("No primary provider could be determined".to_string())
        })?;

        // Filter fallbacks to only include configured providers
        let fallbacks: Vec<Provider> = config
            .fallbacks
            .into_iter()
            .filter(|p| backends.contains_key(p) && *p != primary)
            .collect();

        Ok(Self {
            backends,
            primary,
            fallbacks,
            auto_fallback: config.auto_fallback,
        })
    }

    /// Create a client from environment variables.
    pub fn from_env() -> Result<Self> {
        Self::new(LlmClientConfig::from_env())
    }

    /// Create a client with just an Anthropic backend.
    pub fn anthropic(config: AnthropicConfig) -> Result<Self> {
        Self::new(
            LlmClientConfig::new()
                .with_anthropic(config)
                .with_primary(Provider::Anthropic),
        )
    }

    /// Create a client with just an OpenAI backend.
    pub fn openai(config: OpenAiConfig) -> Result<Self> {
        Self::new(
            LlmClientConfig::new()
                .with_openai(config)
                .with_primary(Provider::OpenAi),
        )
    }

    /// Create a client from environment with Anthropic as primary.
    pub fn anthropic_from_env() -> Result<Self> {
        Self::anthropic(AnthropicConfig::from_env()?)
    }

    /// Create a client from environment with OpenAI as primary.
    pub fn openai_from_env() -> Result<Self> {
        Self::openai(OpenAiConfig::openai_from_env()?)
    }

    /// Get the primary provider.
    pub fn primary(&self) -> Provider {
        self.primary
    }

    /// Get all available providers.
    pub fn available_providers(&self) -> Vec<Provider> {
        self.backends.keys().copied().collect()
    }

    /// Check if a provider is available.
    pub fn has_provider(&self, provider: Provider) -> bool {
        self.backends.contains_key(&provider)
    }

    /// Get a backend by provider.
    pub fn get_backend(&self, provider: Provider) -> Option<&SharedBackend> {
        self.backends.get(&provider)
    }

    /// Execute a completion using the primary provider.
    pub async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        self.complete_with_fallback(self.primary, request).await
    }

    /// Execute a completion using a specific provider.
    pub async fn complete_with(
        &self,
        provider: Provider,
        request: CompletionRequest,
    ) -> Result<CompletionResponse> {
        let backend = self.backends.get(&provider).ok_or_else(|| {
            LlmError::Config(format!("Provider '{}' is not configured", provider))
        })?;

        backend.complete(request).await
    }

    /// Execute a completion with automatic fallback.
    async fn complete_with_fallback(
        &self,
        provider: Provider,
        request: CompletionRequest,
    ) -> Result<CompletionResponse> {
        let backend = self.backends.get(&provider).ok_or_else(|| {
            LlmError::Config(format!("Provider '{}' is not configured", provider))
        })?;

        match backend.complete(request.clone()).await {
            Ok(response) => Ok(response),
            Err(e) if self.auto_fallback && self.should_fallback(&e) => {
                tracing::warn!(
                    provider = %provider,
                    error = %e,
                    "Primary provider failed, trying fallbacks"
                );

                for fallback in &self.fallbacks {
                    if let Some(fallback_backend) = self.backends.get(fallback) {
                        tracing::info!(provider = %fallback, "Trying fallback provider");

                        match fallback_backend.complete(request.clone()).await {
                            Ok(response) => {
                                tracing::info!(provider = %fallback, "Fallback succeeded");
                                return Ok(response);
                            }
                            Err(fallback_error) => {
                                tracing::warn!(
                                    provider = %fallback,
                                    error = %fallback_error,
                                    "Fallback provider also failed"
                                );
                            }
                        }
                    }
                }

                // All fallbacks failed, return original error
                Err(e)
            }
            Err(e) => Err(e),
        }
    }

    /// Execute a streaming completion using the primary provider.
    pub async fn complete_stream(&self, request: CompletionRequest) -> Result<ResponseStream> {
        self.complete_stream_with(self.primary, request).await
    }

    /// Execute a streaming completion using a specific provider.
    pub async fn complete_stream_with(
        &self,
        provider: Provider,
        request: CompletionRequest,
    ) -> Result<ResponseStream> {
        let backend = self.backends.get(&provider).ok_or_else(|| {
            LlmError::Config(format!("Provider '{}' is not configured", provider))
        })?;

        backend.complete_stream(request).await
    }

    /// Determine if we should attempt fallback for this error.
    fn should_fallback(&self, error: &LlmError) -> bool {
        // Network errors include timeouts (via reqwest::Error conversion)
        matches!(
            error,
            LlmError::Network(_) | LlmError::Backend(_) | LlmError::RateLimit(_)
        )
    }
}

// Implement LlmBackend for LlmClient so it can be used interchangeably
#[async_trait]
impl LlmBackend for LlmClient {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        LlmClient::complete(self, request).await
    }

    async fn complete_stream(&self, request: CompletionRequest) -> Result<ResponseStream> {
        LlmClient::complete_stream(self, request).await
    }

    fn name(&self) -> &str {
        "llm-client"
    }

    fn supports_native_tools(&self) -> bool {
        // Delegate to primary backend
        self.backends
            .get(&self.primary)
            .map(|b| b.supports_native_tools())
            .unwrap_or(false)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Message;

    #[test]
    fn test_provider_name() {
        assert_eq!(Provider::Anthropic.name(), "anthropic");
        assert_eq!(Provider::OpenAi.name(), "openai");
        assert_eq!(Provider::Groq.name(), "groq");
        assert_eq!(Provider::Ollama.name(), "ollama");
    }

    #[test]
    fn test_provider_from_name() {
        assert_eq!(Provider::from_name("anthropic"), Some(Provider::Anthropic));
        assert_eq!(Provider::from_name("claude"), Some(Provider::Anthropic));
        assert_eq!(Provider::from_name("openai"), Some(Provider::OpenAi));
        assert_eq!(Provider::from_name("gpt"), Some(Provider::OpenAi));
        assert_eq!(Provider::from_name("groq"), Some(Provider::Groq));
        assert_eq!(Provider::from_name("ollama"), Some(Provider::Ollama));
        assert_eq!(Provider::from_name("local"), Some(Provider::Ollama));
        assert_eq!(Provider::from_name("unknown"), None);
    }

    #[test]
    fn test_provider_requires_api_key() {
        assert!(Provider::Anthropic.requires_api_key());
        assert!(Provider::OpenAi.requires_api_key());
        assert!(Provider::Groq.requires_api_key());
        assert!(!Provider::Ollama.requires_api_key());
    }

    #[test]
    fn test_client_config_builder() {
        let config = LlmClientConfig::new()
            .with_primary(Provider::Anthropic)
            .with_fallbacks(vec![Provider::OpenAi, Provider::Ollama])
            .with_auto_fallback(true);

        assert_eq!(config.primary, Some(Provider::Anthropic));
        assert_eq!(config.fallbacks.len(), 2);
        assert!(config.auto_fallback);
    }

    #[test]
    fn test_config_is_provider_configured() {
        let config = LlmClientConfig::new().with_ollama(OpenAiConfig::ollama());

        assert!(config.is_provider_configured(Provider::Ollama));
        assert!(!config.is_provider_configured(Provider::Anthropic));
        assert!(!config.is_provider_configured(Provider::OpenAi));
    }

    #[test]
    fn test_config_determine_primary() {
        // Explicit primary that's configured
        let config = LlmClientConfig::new()
            .with_ollama(OpenAiConfig::ollama())
            .with_primary(Provider::Ollama);
        assert_eq!(config.determine_primary(), Some(Provider::Ollama));

        // Explicit primary that's NOT configured - should fall back
        let config = LlmClientConfig::new()
            .with_ollama(OpenAiConfig::ollama())
            .with_primary(Provider::Anthropic);
        assert_eq!(config.determine_primary(), Some(Provider::Ollama));

        // No explicit primary - uses preference order
        let config = LlmClientConfig::new().with_ollama(OpenAiConfig::ollama());
        assert_eq!(config.determine_primary(), Some(Provider::Ollama));
    }

    #[tokio::test]
    async fn test_client_with_ollama() {
        // This test uses Ollama config but won't make real requests
        let config = LlmClientConfig::new()
            .with_ollama(OpenAiConfig::ollama())
            .with_primary(Provider::Ollama);

        let client = LlmClient::new(config).unwrap();
        assert_eq!(client.primary(), Provider::Ollama);
        assert!(client.has_provider(Provider::Ollama));
        assert!(!client.has_provider(Provider::Anthropic));
    }

    #[test]
    fn test_client_no_providers_error() {
        let config = LlmClientConfig::new();
        let result = LlmClient::new(config);
        assert!(result.is_err());
    }

    #[test]
    fn test_client_available_providers() {
        let config = LlmClientConfig::new()
            .with_ollama(OpenAiConfig::ollama())
            .with_groq(OpenAiConfig::groq("test-key"));

        let client = LlmClient::new(config).unwrap();
        let providers = client.available_providers();

        assert!(providers.contains(&Provider::Ollama));
        assert!(providers.contains(&Provider::Groq));
        assert!(!providers.contains(&Provider::Anthropic));
    }

    #[tokio::test]
    async fn test_complete_with_unavailable_provider() {
        let config = LlmClientConfig::new().with_ollama(OpenAiConfig::ollama());

        let client = LlmClient::new(config).unwrap();
        let request = CompletionRequest::new("test", vec![Message::user("hi")], 100);

        let result = client.complete_with(Provider::Anthropic, request).await;
        assert!(result.is_err());
    }
}
