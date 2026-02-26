//! OAuth 2.0 PKCE flow for Anthropic MAX plan authentication.

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::{OAuthError, Result};

/// OAuth configuration for Anthropic MAX plan.
#[derive(Debug, Clone)]
pub struct OAuthConfig {
    pub client_id: String,
    pub authorize_url: String,
    pub token_url: String,
    pub redirect_uri: String,
    pub scope: String,
}

impl Default for OAuthConfig {
    fn default() -> Self {
        Self::anthropic_max()
    }
}

impl OAuthConfig {
    /// Create OAuth config for Anthropic MAX plan.
    ///
    /// TODO: Make these configurable via `arawn.toml` if we need to support
    /// mock OAuth servers for testing or if Anthropic changes endpoints.
    /// See ARAWN-T-0224 (P2 tech debt).
    pub fn anthropic_max() -> Self {
        Self {
            client_id: "9d1c250a-e61b-44d9-88ed-5944d1962f5e".to_string(),
            authorize_url: "https://claude.ai/oauth/authorize".to_string(),
            token_url: "https://console.anthropic.com/v1/oauth/token".to_string(),
            redirect_uri: "https://console.anthropic.com/oauth/code/callback".to_string(),
            scope: "org:create_api_key user:profile user:inference".to_string(),
        }
    }
}

/// PKCE code verifier and challenge pair.
#[derive(Debug, Clone)]
pub struct PkceChallenge {
    pub verifier: String,
    pub challenge: String,
}

impl PkceChallenge {
    /// Generate a new PKCE challenge pair.
    pub fn generate() -> Self {
        let mut verifier_bytes = [0u8; 32];
        rand::rng().fill_bytes(&mut verifier_bytes);
        let verifier = URL_SAFE_NO_PAD.encode(verifier_bytes);

        let mut hasher = Sha256::new();
        hasher.update(verifier.as_bytes());
        let challenge_bytes = hasher.finalize();
        let challenge = URL_SAFE_NO_PAD.encode(challenge_bytes);

        Self {
            verifier,
            challenge,
        }
    }
}

/// Generate a random state string for CSRF protection.
pub fn generate_state() -> String {
    let mut state_bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut state_bytes);
    URL_SAFE_NO_PAD.encode(state_bytes)
}

/// Build the authorization URL for the OAuth flow.
pub fn build_authorization_url(config: &OAuthConfig, challenge: &str, state: &str) -> String {
    let params = [
        ("code", "true"),
        ("client_id", &config.client_id),
        ("redirect_uri", &config.redirect_uri),
        ("response_type", "code"),
        ("scope", &config.scope),
        ("code_challenge", challenge),
        ("code_challenge_method", "S256"),
        ("state", state),
    ];

    let query = params
        .iter()
        .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
        .collect::<Vec<_>>()
        .join("&");

    format!("{}?{}", config.authorize_url, query)
}

/// OAuth tokens returned from token exchange.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
    pub token_type: String,
    pub scope: String,
    #[serde(default)]
    pub expires_at: u64,
    #[serde(default)]
    pub created_at: String,
}

#[derive(Debug, Serialize)]
struct TokenExchangeRequest {
    code: String,
    state: String,
    grant_type: String,
    client_id: String,
    redirect_uri: String,
    code_verifier: String,
}

#[derive(Debug, Serialize)]
struct TokenRefreshRequest {
    grant_type: String,
    client_id: String,
    refresh_token: String,
}

/// Exchange an authorization code for OAuth tokens.
pub async fn exchange_code_for_tokens(
    config: &OAuthConfig,
    code: &str,
    verifier: &str,
    state: &str,
) -> Result<OAuthTokens> {
    let request_body = TokenExchangeRequest {
        code: code.to_string(),
        state: state.to_string(),
        grant_type: "authorization_code".to_string(),
        client_id: config.client_id.clone(),
        redirect_uri: config.redirect_uri.clone(),
        code_verifier: verifier.to_string(),
    };

    let client = reqwest::Client::new();
    let response = client
        .post(&config.token_url)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| OAuthError::Network(format!("Token exchange request failed: {}", e)))?;

    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(OAuthError::Backend(format!(
            "Token exchange failed: {}",
            error_text
        )));
    }

    let mut tokens: OAuthTokens = response
        .json()
        .await
        .map_err(|e| OAuthError::Backend(format!("Failed to parse token response: {}", e)))?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    tokens.expires_at = now + (tokens.expires_in * 1000);
    tokens.created_at = chrono::Utc::now().to_rfc3339();

    Ok(tokens)
}

/// Refresh an access token using a refresh token.
pub async fn refresh_access_token(
    config: &OAuthConfig,
    refresh_token: &str,
) -> Result<OAuthTokens> {
    let request_body = TokenRefreshRequest {
        grant_type: "refresh_token".to_string(),
        client_id: config.client_id.clone(),
        refresh_token: refresh_token.to_string(),
    };

    let client = reqwest::Client::new();
    let response = client
        .post(&config.token_url)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| OAuthError::Network(format!("Token refresh request failed: {}", e)))?;

    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(OAuthError::Backend(format!(
            "Token refresh failed: {}",
            error_text
        )));
    }

    let mut tokens: OAuthTokens = response
        .json()
        .await
        .map_err(|e| OAuthError::Backend(format!("Failed to parse refresh response: {}", e)))?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    tokens.expires_at = now + (tokens.expires_in * 1000);
    tokens.created_at = chrono::Utc::now().to_rfc3339();

    Ok(tokens)
}

/// Parse the code#state response from the OAuth callback.
pub fn parse_code_state(input: &str) -> Result<(String, String)> {
    let trimmed = input.trim();
    if !trimmed.contains('#') {
        return Err(OAuthError::InvalidRequest(
            "Invalid format. Expected: code#state".to_string(),
        ));
    }

    let parts: Vec<&str> = trimmed.splitn(2, '#').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        return Err(OAuthError::InvalidRequest(
            "Missing code or state".to_string(),
        ));
    }

    Ok((parts[0].to_string(), parts[1].to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pkce_generation() {
        let pkce = PkceChallenge::generate();
        assert!(!pkce.verifier.is_empty());
        assert!(!pkce.challenge.is_empty());
        assert_ne!(pkce.verifier, pkce.challenge);
    }

    #[test]
    fn test_state_generation() {
        let state1 = generate_state();
        let state2 = generate_state();
        assert!(!state1.is_empty());
        assert_ne!(state1, state2);
    }

    #[test]
    fn test_authorization_url() {
        let config = OAuthConfig::anthropic_max();
        let url = build_authorization_url(&config, "test_challenge", "test_state");

        assert!(url.starts_with("https://claude.ai/oauth/authorize?"));
        assert!(url.contains("client_id="));
        assert!(url.contains("code_challenge=test_challenge"));
        assert!(url.contains("state=test_state"));
        assert!(url.contains("code_challenge_method=S256"));
    }

    #[test]
    fn test_parse_code_state_valid() {
        let (code, state) = parse_code_state("abc123#xyz789").unwrap();
        assert_eq!(code, "abc123");
        assert_eq!(state, "xyz789");
    }

    #[test]
    fn test_parse_code_state_with_whitespace() {
        let (code, state) = parse_code_state("  abc123#xyz789  ").unwrap();
        assert_eq!(code, "abc123");
        assert_eq!(state, "xyz789");
    }

    #[test]
    fn test_parse_code_state_invalid() {
        assert!(parse_code_state("no_separator").is_err());
        assert!(parse_code_state("#only_state").is_err());
        assert!(parse_code_state("only_code#").is_err());
    }

    #[test]
    fn test_oauth_config_default() {
        let config = OAuthConfig::default();
        assert_eq!(config.client_id, "9d1c250a-e61b-44d9-88ed-5944d1962f5e");
        assert!(config.authorize_url.contains("claude.ai"));
    }
}
