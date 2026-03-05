//! OAuth 2.0 PKCE flow for Anthropic MAX plan authentication.

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::{OAuthError, Result};

/// OAuth configuration for Anthropic MAX plan.
///
/// # Examples
///
/// ```rust,ignore
/// use arawn_oauth::OAuthConfig;
///
/// let config = OAuthConfig::anthropic_max();
/// let custom = config.with_overrides(
///     Some("custom-client-id"), None, None, None, None,
/// );
/// ```
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
    /// Default client ID for Anthropic MAX plan OAuth.
    const DEFAULT_CLIENT_ID: &str = "9d1c250a-e61b-44d9-88ed-5944d1962f5e";
    const DEFAULT_AUTHORIZE_URL: &str = "https://claude.ai/oauth/authorize";
    const DEFAULT_TOKEN_URL: &str = "https://console.anthropic.com/v1/oauth/token";
    const DEFAULT_REDIRECT_URI: &str = "https://console.anthropic.com/oauth/code/callback";
    const DEFAULT_SCOPE: &str = "org:create_api_key user:profile user:inference";

    /// Create OAuth config for Anthropic MAX plan.
    ///
    /// Environment variable overrides (checked in order):
    /// - `ARAWN_OAUTH_CLIENT_ID`
    /// - `ARAWN_OAUTH_AUTHORIZE_URL`
    /// - `ARAWN_OAUTH_TOKEN_URL`
    /// - `ARAWN_OAUTH_REDIRECT_URI`
    /// - `ARAWN_OAUTH_SCOPE`
    pub fn anthropic_max() -> Self {
        Self {
            client_id: std::env::var("ARAWN_OAUTH_CLIENT_ID")
                .unwrap_or_else(|_| Self::DEFAULT_CLIENT_ID.to_string()),
            authorize_url: std::env::var("ARAWN_OAUTH_AUTHORIZE_URL")
                .unwrap_or_else(|_| Self::DEFAULT_AUTHORIZE_URL.to_string()),
            token_url: std::env::var("ARAWN_OAUTH_TOKEN_URL")
                .unwrap_or_else(|_| Self::DEFAULT_TOKEN_URL.to_string()),
            redirect_uri: std::env::var("ARAWN_OAUTH_REDIRECT_URI")
                .unwrap_or_else(|_| Self::DEFAULT_REDIRECT_URI.to_string()),
            scope: std::env::var("ARAWN_OAUTH_SCOPE")
                .unwrap_or_else(|_| Self::DEFAULT_SCOPE.to_string()),
        }
    }

    /// Apply config overrides. Any `Some` value replaces the current field.
    pub fn with_overrides(
        mut self,
        client_id: Option<&str>,
        authorize_url: Option<&str>,
        token_url: Option<&str>,
        redirect_uri: Option<&str>,
        scope: Option<&str>,
    ) -> Self {
        if let Some(v) = client_id {
            self.client_id = v.to_string();
        }
        if let Some(v) = authorize_url {
            self.authorize_url = v.to_string();
        }
        if let Some(v) = token_url {
            self.token_url = v.to_string();
        }
        if let Some(v) = redirect_uri {
            self.redirect_uri = v.to_string();
        }
        if let Some(v) = scope {
            self.scope = v.to_string();
        }
        self
    }
}

/// PKCE code verifier and challenge pair.
///
/// # Examples
///
/// ```rust,ignore
/// use arawn_oauth::PkceChallenge;
///
/// let pkce = PkceChallenge::generate();
/// println!("verifier: {}", pkce.verifier);
/// println!("challenge: {}", pkce.challenge);
/// ```
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
///
/// # Examples
///
/// ```rust,ignore
/// use arawn_oauth::{OAuthConfig, PkceChallenge, build_authorization_url, generate_state};
///
/// let config = OAuthConfig::anthropic_max();
/// let pkce = PkceChallenge::generate();
/// let state = generate_state();
/// let url = build_authorization_url(&config, &pkce.challenge, &state);
/// // Open `url` in the user's browser
/// ```
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
///
/// # Examples
///
/// ```rust,ignore
/// use arawn_oauth::parse_code_state;
///
/// let (code, state) = parse_code_state("auth_code_123#state_xyz").unwrap();
/// assert_eq!(code, "auth_code_123");
/// assert_eq!(state, "state_xyz");
/// ```
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
