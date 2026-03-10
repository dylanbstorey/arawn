//! Passthrough client for forwarding requests to upstream Anthropic API.
//!
//! Handles authentication (OAuth Bearer or API key), system prompt injection,
//! field stripping, and anthropic-beta header injection for MAX plan.

use std::collections::HashMap;

use crate::error::{OAuthError, Result};
use crate::token_manager::SharedTokenManager;
use reqwest::{Client, header};

/// Anthropic API base URL.
pub const ANTHROPIC_API_URL: &str = "https://api.anthropic.com";

/// Anthropic API version header.
pub const ANTHROPIC_VERSION: &str = "2023-06-01";

/// Required anthropic-beta header for OAuth/MAX plan.
pub const ANTHROPIC_BETA: &str = "oauth-2025-04-20,claude-code-20250219,interleaved-thinking-2025-05-14,fine-grained-tool-streaming-2025-05-14";

/// Required system prompt for Claude Code with MAX plan.
pub const CLAUDE_CODE_SYSTEM_PROMPT: &str =
    "You are Claude Code, Anthropic's official CLI for Claude.";

/// Authentication mode for passthrough requests.
#[derive(Debug, Clone, PartialEq)]
pub enum AuthMode {
    /// Use API key from request headers.
    ApiKey,
    /// Use OAuth Bearer token from token manager.
    OAuth,
    /// Try OAuth first, fall back to API key.
    OAuthWithFallback,
}

/// Configuration for the passthrough client.
#[derive(Debug, Clone)]
pub struct PassthroughConfig {
    pub base_url: String,
    pub messages_path: String,
    pub auth_header: String,
    pub extra_headers: HashMap<String, String>,
    pub auth_mode: AuthMode,
    pub inject_system_prompt: bool,
}

impl PassthroughConfig {
    /// Create config for Anthropic API with OAuth (MAX plan).
    pub fn anthropic_oauth() -> Self {
        let mut extra_headers = HashMap::new();
        extra_headers.insert(
            "anthropic-version".to_string(),
            ANTHROPIC_VERSION.to_string(),
        );
        extra_headers.insert("anthropic-beta".to_string(), ANTHROPIC_BETA.to_string());

        Self {
            base_url: ANTHROPIC_API_URL.to_string(),
            messages_path: "/v1/messages".to_string(),
            auth_header: "Authorization".to_string(),
            extra_headers,
            auth_mode: AuthMode::OAuthWithFallback,
            inject_system_prompt: true,
        }
    }

    /// Create config for Anthropic API with API key auth.
    pub fn anthropic_api_key() -> Self {
        let mut extra_headers = HashMap::new();
        extra_headers.insert(
            "anthropic-version".to_string(),
            ANTHROPIC_VERSION.to_string(),
        );

        Self {
            base_url: ANTHROPIC_API_URL.to_string(),
            messages_path: "/v1/messages".to_string(),
            auth_header: "x-api-key".to_string(),
            extra_headers,
            auth_mode: AuthMode::ApiKey,
            inject_system_prompt: false,
        }
    }
}

impl Default for PassthroughConfig {
    fn default() -> Self {
        Self::anthropic_oauth()
    }
}

/// Passthrough client for forwarding requests to upstream APIs.
#[derive(Debug, Clone)]
pub struct Passthrough {
    client: Client,
    config: PassthroughConfig,
    token_manager: Option<SharedTokenManager>,
}

impl Passthrough {
    /// Create a new passthrough client with default config (OAuth mode).
    pub fn new() -> Self {
        Self::with_config(PassthroughConfig::default())
    }

    /// Create with custom config.
    pub fn with_config(config: PassthroughConfig) -> Self {
        Self {
            client: Client::new(),
            config,
            token_manager: None,
        }
    }

    /// Set the token manager for OAuth authentication.
    pub fn with_token_manager(mut self, manager: SharedTokenManager) -> Self {
        self.token_manager = Some(manager);
        self
    }

    /// Get the config.
    pub fn config(&self) -> &PassthroughConfig {
        &self.config
    }

    /// Forward a raw JSON request to the upstream API (non-streaming).
    pub async fn forward_raw(
        &self,
        request: serde_json::Value,
        api_key: Option<&str>,
    ) -> Result<serde_json::Value> {
        let url = format!("{}{}", self.config.base_url, self.config.messages_path);

        let forward_request = self.prepare_raw_request(request);

        let mut req = self
            .client
            .post(&url)
            .header(header::CONTENT_TYPE, "application/json");

        let auth_value = self.get_auth_value(api_key).await?;
        req = req.header(&self.config.auth_header, &auth_value);

        for (key, value) in &self.config.extra_headers {
            req = req.header(key, value);
        }

        let response = req
            .json(&forward_request)
            .send()
            .await
            .map_err(|e| OAuthError::Backend(format!("Failed to forward request: {}", e)))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| OAuthError::Backend(format!("Failed to read response: {}", e)))?;

        if !status.is_success() {
            return Err(OAuthError::Backend(format!(
                "Upstream API error ({}): {}",
                status, body
            )));
        }

        let response_json: serde_json::Value = serde_json::from_str(&body)
            .map_err(|e| OAuthError::Backend(format!("Failed to parse response: {}", e)))?;

        Ok(response_json)
    }

    /// Forward a raw JSON streaming request, returning the raw response.
    pub async fn forward_raw_stream(
        &self,
        request: serde_json::Value,
        api_key: Option<&str>,
    ) -> Result<reqwest::Response> {
        let url = format!("{}{}", self.config.base_url, self.config.messages_path);

        let forward_request = self.prepare_raw_request(request);

        let mut req = self
            .client
            .post(&url)
            .header(header::CONTENT_TYPE, "application/json");

        let auth_value = self.get_auth_value(api_key).await?;
        req = req.header(&self.config.auth_header, &auth_value);

        for (key, value) in &self.config.extra_headers {
            req = req.header(key, value);
        }

        let response = req
            .json(&forward_request)
            .send()
            .await
            .map_err(|e| OAuthError::Backend(format!("Failed to forward request: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read error body".to_string());
            return Err(OAuthError::Backend(format!(
                "Upstream API error ({}): {}",
                status, body
            )));
        }

        Ok(response)
    }

    /// Get the authentication value based on the configured mode.
    async fn get_auth_value(&self, api_key: Option<&str>) -> Result<String> {
        match self.config.auth_mode {
            AuthMode::ApiKey => {
                let key = api_key.ok_or_else(|| {
                    OAuthError::InvalidRequest("API key required but not provided".to_string())
                })?;
                Ok(key.to_string())
            }
            AuthMode::OAuth => {
                let manager = self.token_manager.as_ref().ok_or_else(|| {
                    OAuthError::Config("OAuth mode requires token manager".to_string())
                })?;
                let token = manager.get_valid_access_token().await?;
                Ok(format!("Bearer {}", token))
            }
            AuthMode::OAuthWithFallback => {
                if let Some(manager) = &self.token_manager
                    && manager.has_tokens()
                {
                    match manager.get_valid_access_token().await {
                        Ok(token) => return Ok(format!("Bearer {}", token)),
                        Err(e) => {
                            tracing::warn!(
                                "OAuth token refresh failed, trying API key fallback: {}",
                                e
                            );
                        }
                    }
                }
                if let Some(key) = api_key {
                    Ok(key.to_string())
                } else {
                    Err(OAuthError::InvalidRequest(
                        "No OAuth tokens available and no API key provided. Run 'arawn oauth' to authenticate.".to_string(),
                    ))
                }
            }
        }
    }

    /// Prepare a raw JSON request: strip unknown fields, inject system prompt.
    fn prepare_raw_request(&self, request: serde_json::Value) -> serde_json::Value {
        let mut sanitized = strip_unknown_fields(&request);

        if self.config.inject_system_prompt {
            inject_system_prompt(&mut sanitized);
        }

        sanitized
    }
}

impl Default for Passthrough {
    fn default() -> Self {
        Self::new()
    }
}

/// Valid top-level fields for Anthropic API requests.
const VALID_REQUEST_FIELDS: &[&str] = &[
    "model",
    "max_tokens",
    "system",
    "messages",
    "tools",
    "tool_choice",
    "stream",
    "temperature",
    "top_p",
    "top_k",
    "stop_sequences",
    "metadata",
    "thinking",
];

/// Strip unknown fields from a raw JSON request.
fn strip_unknown_fields(request: &serde_json::Value) -> serde_json::Value {
    match request {
        serde_json::Value::Object(map) => {
            let mut sanitized = serde_json::Map::new();
            for (key, value) in map {
                if VALID_REQUEST_FIELDS.contains(&key.as_str()) {
                    sanitized.insert(key.clone(), value.clone());
                }
            }
            serde_json::Value::Object(sanitized)
        }
        _ => request.clone(),
    }
}

/// Inject the required system prompt into a raw JSON request.
fn inject_system_prompt(request: &mut serde_json::Value) {
    let required_prompt = serde_json::json!({
        "type": "text",
        "text": CLAUDE_CODE_SYSTEM_PROMPT
    });

    if let serde_json::Value::Object(map) = request {
        let system = map
            .entry("system")
            .or_insert(serde_json::Value::Array(vec![]));

        let system_array = match system {
            serde_json::Value::String(s) => {
                vec![serde_json::json!({"type": "text", "text": s.clone()})]
            }
            serde_json::Value::Array(arr) => arr.clone(),
            _ => vec![],
        };

        let has_required = system_array.first().is_some_and(|first| {
            first.get("type").and_then(|t| t.as_str()) == Some("text")
                && first.get("text").and_then(|t| t.as_str()) == Some(CLAUDE_CODE_SYSTEM_PROMPT)
        });

        if !has_required {
            let mut new_system = vec![required_prompt];
            new_system.extend(system_array);
            *system = serde_json::Value::Array(new_system);
        } else {
            *system = serde_json::Value::Array(system_array);
        }
    }
}

/// Extract API key from request headers.
pub fn extract_api_key(
    headers: &axum::http::HeaderMap,
    config: &PassthroughConfig,
) -> Option<String> {
    if let Some(value) = headers.get(&config.auth_header)
        && let Ok(s) = value.to_str()
    {
        let key = s.strip_prefix("Bearer ").unwrap_or(s);
        return Some(key.to_string());
    }

    if config.auth_header != "x-api-key"
        && let Some(value) = headers.get("x-api-key")
        && let Ok(s) = value.to_str()
    {
        return Some(s.to_string());
    }

    if config.auth_header != "Authorization"
        && let Some(value) = headers.get("Authorization")
        && let Ok(s) = value.to_str()
    {
        let key = s.strip_prefix("Bearer ").unwrap_or(s);
        return Some(key.to_string());
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default_is_oauth() {
        let config = PassthroughConfig::default();
        assert_eq!(config.auth_mode, AuthMode::OAuthWithFallback);
        assert!(config.inject_system_prompt);
        assert!(config.extra_headers.contains_key("anthropic-beta"));
    }

    #[test]
    fn test_strip_unknown_fields() {
        let request = serde_json::json!({
            "model": "claude-3",
            "max_tokens": 100,
            "messages": [],
            "unknown_field": "should be stripped",
            "context_management": {"should": "strip"}
        });

        let stripped = strip_unknown_fields(&request);
        assert!(stripped.get("model").is_some());
        assert!(stripped.get("max_tokens").is_some());
        assert!(stripped.get("messages").is_some());
        assert!(stripped.get("unknown_field").is_none());
        assert!(stripped.get("context_management").is_none());
    }

    #[test]
    fn test_inject_system_prompt_empty() {
        let mut request = serde_json::json!({"model": "claude-3", "messages": []});
        inject_system_prompt(&mut request);

        let system = request.get("system").unwrap().as_array().unwrap();
        assert_eq!(system.len(), 1);
        assert_eq!(
            system[0].get("text").unwrap().as_str().unwrap(),
            CLAUDE_CODE_SYSTEM_PROMPT
        );
    }

    #[test]
    fn test_inject_system_prompt_prepend() {
        let mut request = serde_json::json!({
            "model": "claude-3",
            "messages": [],
            "system": [{"type": "text", "text": "Custom prompt"}]
        });
        inject_system_prompt(&mut request);

        let system = request.get("system").unwrap().as_array().unwrap();
        assert_eq!(system.len(), 2);
        assert_eq!(
            system[0].get("text").unwrap().as_str().unwrap(),
            CLAUDE_CODE_SYSTEM_PROMPT
        );
        assert_eq!(
            system[1].get("text").unwrap().as_str().unwrap(),
            "Custom prompt"
        );
    }

    #[test]
    fn test_inject_system_prompt_already_present() {
        let mut request = serde_json::json!({
            "model": "claude-3",
            "messages": [],
            "system": [
                {"type": "text", "text": CLAUDE_CODE_SYSTEM_PROMPT},
                {"type": "text", "text": "Custom"}
            ]
        });
        inject_system_prompt(&mut request);

        let system = request.get("system").unwrap().as_array().unwrap();
        assert_eq!(system.len(), 2); // Not duplicated
    }

    #[test]
    fn test_inject_system_prompt_string_format() {
        let mut request = serde_json::json!({
            "model": "claude-3",
            "messages": [],
            "system": "Original string prompt"
        });
        inject_system_prompt(&mut request);

        let system = request.get("system").unwrap().as_array().unwrap();
        assert_eq!(system.len(), 2);
        assert_eq!(
            system[0].get("text").unwrap().as_str().unwrap(),
            CLAUDE_CODE_SYSTEM_PROMPT
        );
    }

    // ── extract_api_key tests ─────────────────────────────────────────────

    #[test]
    fn test_extract_api_key_from_auth_header_bearer() {
        let config = PassthroughConfig::anthropic_oauth();
        let mut headers = axum::http::HeaderMap::new();
        headers.insert("authorization", "Bearer sk-ant-123".parse().unwrap());

        let key = extract_api_key(&headers, &config);
        assert_eq!(key, Some("sk-ant-123".to_string()));
    }

    #[test]
    fn test_extract_api_key_from_auth_header_no_bearer() {
        let config = PassthroughConfig::anthropic_oauth();
        let mut headers = axum::http::HeaderMap::new();
        headers.insert("authorization", "sk-ant-123".parse().unwrap());

        let key = extract_api_key(&headers, &config);
        assert_eq!(key, Some("sk-ant-123".to_string()));
    }

    #[test]
    fn test_extract_api_key_from_x_api_key_header() {
        let config = PassthroughConfig::anthropic_oauth();
        let mut headers = axum::http::HeaderMap::new();
        headers.insert("x-api-key", "sk-ant-456".parse().unwrap());

        let key = extract_api_key(&headers, &config);
        assert_eq!(key, Some("sk-ant-456".to_string()));
    }

    #[test]
    fn test_extract_api_key_prefers_config_auth_header() {
        // When config auth_header is "Authorization", it should be checked first
        let config = PassthroughConfig::anthropic_oauth();
        let mut headers = axum::http::HeaderMap::new();
        headers.insert("authorization", "Bearer primary-key".parse().unwrap());
        headers.insert("x-api-key", "secondary-key".parse().unwrap());

        let key = extract_api_key(&headers, &config);
        assert_eq!(key, Some("primary-key".to_string()));
    }

    #[test]
    fn test_extract_api_key_api_key_mode_uses_x_api_key() {
        let config = PassthroughConfig::anthropic_api_key();
        let mut headers = axum::http::HeaderMap::new();
        headers.insert("x-api-key", "sk-ant-789".parse().unwrap());

        let key = extract_api_key(&headers, &config);
        assert_eq!(key, Some("sk-ant-789".to_string()));
    }

    #[test]
    fn test_extract_api_key_api_key_mode_fallback_to_authorization() {
        let config = PassthroughConfig::anthropic_api_key();
        let mut headers = axum::http::HeaderMap::new();
        headers.insert("authorization", "Bearer sk-fallback".parse().unwrap());

        let key = extract_api_key(&headers, &config);
        assert_eq!(key, Some("sk-fallback".to_string()));
    }

    #[test]
    fn test_extract_api_key_no_headers() {
        let config = PassthroughConfig::anthropic_oauth();
        let headers = axum::http::HeaderMap::new();
        let key = extract_api_key(&headers, &config);
        assert_eq!(key, None);
    }

    // ── PassthroughConfig tests ───────────────────────────────────────────

    #[test]
    fn test_config_anthropic_api_key() {
        let config = PassthroughConfig::anthropic_api_key();
        assert_eq!(config.auth_mode, AuthMode::ApiKey);
        assert!(!config.inject_system_prompt);
        assert_eq!(config.auth_header, "x-api-key");
        assert!(config.extra_headers.contains_key("anthropic-version"));
        assert!(!config.extra_headers.contains_key("anthropic-beta"));
    }

    #[test]
    fn test_config_anthropic_oauth() {
        let config = PassthroughConfig::anthropic_oauth();
        assert_eq!(config.auth_mode, AuthMode::OAuthWithFallback);
        assert!(config.inject_system_prompt);
        assert_eq!(config.auth_header, "Authorization");
        assert!(config.extra_headers.contains_key("anthropic-beta"));
    }

    // ── Passthrough construction ──────────────────────────────────────────

    #[test]
    fn test_passthrough_new() {
        let pt = Passthrough::new();
        assert_eq!(pt.config().auth_mode, AuthMode::OAuthWithFallback);
        assert!(pt.token_manager.is_none());
    }

    #[test]
    fn test_passthrough_default() {
        let pt = Passthrough::default();
        assert_eq!(pt.config().auth_mode, AuthMode::OAuthWithFallback);
    }

    #[test]
    fn test_passthrough_with_config() {
        let config = PassthroughConfig::anthropic_api_key();
        let pt = Passthrough::with_config(config);
        assert_eq!(pt.config().auth_mode, AuthMode::ApiKey);
    }

    // ── get_auth_value tests ──────────────────────────────────────────────

    #[tokio::test]
    async fn test_get_auth_value_api_key_mode() {
        let config = PassthroughConfig::anthropic_api_key();
        let pt = Passthrough::with_config(config);
        let value = pt.get_auth_value(Some("sk-ant-test")).await.unwrap();
        assert_eq!(value, "sk-ant-test");
    }

    #[tokio::test]
    async fn test_get_auth_value_api_key_mode_missing() {
        let config = PassthroughConfig::anthropic_api_key();
        let pt = Passthrough::with_config(config);
        let result = pt.get_auth_value(None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_auth_value_oauth_no_manager() {
        let mut config = PassthroughConfig::anthropic_oauth();
        config.auth_mode = AuthMode::OAuth;
        let pt = Passthrough::with_config(config);
        let result = pt.get_auth_value(None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_auth_value_oauth_fallback_no_tokens_uses_api_key() {
        let config = PassthroughConfig::anthropic_oauth();
        let pt = Passthrough::with_config(config);
        // No token manager → falls through to API key
        let value = pt.get_auth_value(Some("sk-fallback")).await.unwrap();
        assert_eq!(value, "sk-fallback");
    }

    #[tokio::test]
    async fn test_get_auth_value_oauth_fallback_no_tokens_no_key() {
        let config = PassthroughConfig::anthropic_oauth();
        let pt = Passthrough::with_config(config);
        let result = pt.get_auth_value(None).await;
        assert!(result.is_err());
    }

    // ── prepare_raw_request tests ─────────────────────────────────────────

    #[test]
    fn test_prepare_raw_request_strips_and_injects() {
        let config = PassthroughConfig::anthropic_oauth();
        let pt = Passthrough::with_config(config);
        let request = serde_json::json!({
            "model": "claude-3",
            "max_tokens": 100,
            "messages": [],
            "extra_field": "stripped"
        });
        let prepared = pt.prepare_raw_request(request);
        assert!(prepared.get("model").is_some());
        assert!(prepared.get("extra_field").is_none());
        // System prompt injected
        assert!(prepared.get("system").is_some());
    }

    #[test]
    fn test_prepare_raw_request_no_inject_when_disabled() {
        let config = PassthroughConfig::anthropic_api_key();
        let pt = Passthrough::with_config(config);
        let request = serde_json::json!({
            "model": "claude-3",
            "messages": []
        });
        let prepared = pt.prepare_raw_request(request);
        // No system prompt injected in api_key mode
        assert!(prepared.get("system").is_none());
    }

    // ── strip_unknown_fields edge cases ───────────────────────────────────

    #[test]
    fn test_strip_unknown_fields_non_object() {
        let input = serde_json::json!("just a string");
        let result = strip_unknown_fields(&input);
        assert_eq!(result, input);
    }

    #[test]
    fn test_strip_unknown_fields_preserves_all_valid() {
        let request = serde_json::json!({
            "model": "claude-3",
            "max_tokens": 100,
            "system": [],
            "messages": [],
            "tools": [],
            "tool_choice": "auto",
            "stream": true,
            "temperature": 0.7,
            "top_p": 0.9,
            "top_k": 40,
            "stop_sequences": ["END"],
            "metadata": {},
            "thinking": {}
        });
        let stripped = strip_unknown_fields(&request);
        assert_eq!(stripped.as_object().unwrap().len(), 13); // All 13 valid fields
    }

    // ── Constants tests ───────────────────────────────────────────────────

    #[test]
    fn test_constants() {
        assert!(ANTHROPIC_API_URL.starts_with("https://"));
        assert!(!ANTHROPIC_VERSION.is_empty());
        assert!(ANTHROPIC_BETA.contains("oauth"));
        assert!(!CLAUDE_CODE_SYSTEM_PROMPT.is_empty());
    }
}
