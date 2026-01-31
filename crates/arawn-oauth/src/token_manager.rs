//! Token management for OAuth tokens.
//!
//! Handles saving, loading, and refreshing OAuth tokens for
//! Anthropic MAX plan authentication.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::error::{OAuthError, Result};
use crate::oauth::{OAuthConfig, OAuthTokens, refresh_access_token};

/// Default token file name within the arawn data directory.
pub const TOKEN_FILE: &str = "oauth-tokens.json";

/// Buffer time before expiry to trigger refresh (5 minutes in milliseconds).
const REFRESH_BUFFER_MS: u64 = 5 * 60 * 1000;

// ============================================================================
// TokenManager Trait
// ============================================================================

/// Trait for managing OAuth token lifecycle.
#[async_trait]
pub trait TokenManager: Send + Sync + std::fmt::Debug {
    /// Get a valid access token, refreshing if necessary.
    async fn get_valid_access_token(&self) -> Result<String>;

    /// Check if tokens exist.
    fn has_tokens(&self) -> bool;

    /// Save tokens to storage.
    async fn save_tokens(&self, tokens: &OAuthTokens) -> Result<()>;

    /// Load tokens from storage.
    async fn load_tokens(&self) -> Result<Option<OAuthTokens>>;

    /// Delete stored tokens.
    async fn delete_tokens(&self) -> Result<()>;

    /// Clear cached tokens.
    async fn clear_cache(&self);

    /// Get token expiry information for display.
    async fn get_token_info(&self) -> Result<Option<TokenInfo>>;
}

// ============================================================================
// FileTokenManager
// ============================================================================

/// File-based token manager for production use.
#[derive(Debug)]
pub struct FileTokenManager {
    token_path: PathBuf,
    config: OAuthConfig,
    cached_tokens: Arc<RwLock<Option<OAuthTokens>>>,
}

impl FileTokenManager {
    /// Create a new file-based token manager.
    pub fn new(data_dir: &Path) -> Self {
        Self {
            token_path: data_dir.join(TOKEN_FILE),
            config: OAuthConfig::default(),
            cached_tokens: Arc::new(RwLock::new(None)),
        }
    }

    /// Create with a custom token path.
    pub fn with_path(token_path: PathBuf) -> Self {
        Self {
            token_path,
            config: OAuthConfig::default(),
            cached_tokens: Arc::new(RwLock::new(None)),
        }
    }

    /// Get the token file path.
    pub fn token_path(&self) -> &Path {
        &self.token_path
    }

    /// Check if tokens are expired (with buffer time).
    pub fn is_token_expired(tokens: &OAuthTokens) -> bool {
        if tokens.expires_at == 0 {
            return true;
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        now >= tokens.expires_at.saturating_sub(REFRESH_BUFFER_MS)
    }
}

#[async_trait]
impl TokenManager for FileTokenManager {
    fn has_tokens(&self) -> bool {
        self.token_path.exists()
    }

    async fn save_tokens(&self, tokens: &OAuthTokens) -> Result<()> {
        if let Some(parent) = self.token_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                OAuthError::Config(format!("Failed to create token directory: {}", e))
            })?;
        }

        let json = serde_json::to_string_pretty(tokens)
            .map_err(|e| OAuthError::Serialization(format!("Failed to serialize tokens: {}", e)))?;

        std::fs::write(&self.token_path, json)
            .map_err(|e| OAuthError::Config(format!("Failed to write token file: {}", e)))?;

        let mut cache = self.cached_tokens.write().await;
        *cache = Some(tokens.clone());

        tracing::info!("Tokens saved to {}", self.token_path.display());
        Ok(())
    }

    async fn load_tokens(&self) -> Result<Option<OAuthTokens>> {
        {
            let cache = self.cached_tokens.read().await;
            if cache.is_some() {
                return Ok(cache.clone());
            }
        }

        if !self.token_path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&self.token_path)
            .map_err(|e| OAuthError::Config(format!("Failed to read token file: {}", e)))?;

        let tokens: OAuthTokens = serde_json::from_str(&content)
            .map_err(|e| OAuthError::Serialization(format!("Failed to parse token file: {}", e)))?;

        let mut cache = self.cached_tokens.write().await;
        *cache = Some(tokens.clone());

        Ok(Some(tokens))
    }

    async fn get_valid_access_token(&self) -> Result<String> {
        let tokens = self.load_tokens().await?.ok_or_else(|| {
            OAuthError::Config("No OAuth tokens found. Run 'arawn oauth' first.".to_string())
        })?;

        if Self::is_token_expired(&tokens) {
            tracing::info!("Token expired, refreshing...");
            let mut new_tokens = refresh_access_token(&self.config, &tokens.refresh_token).await?;

            if new_tokens.refresh_token.is_empty() {
                new_tokens.refresh_token = tokens.refresh_token;
            }

            self.save_tokens(&new_tokens).await?;
            tracing::info!("Token refreshed successfully");
            return Ok(new_tokens.access_token);
        }

        Ok(tokens.access_token)
    }

    async fn clear_cache(&self) {
        let mut cache = self.cached_tokens.write().await;
        *cache = None;
    }

    async fn delete_tokens(&self) -> Result<()> {
        if self.token_path.exists() {
            std::fs::remove_file(&self.token_path)
                .map_err(|e| OAuthError::Config(format!("Failed to delete token file: {}", e)))?;
        }
        self.clear_cache().await;
        Ok(())
    }

    async fn get_token_info(&self) -> Result<Option<TokenInfo>> {
        let tokens = self.load_tokens().await?;
        match tokens {
            Some(t) => {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64;

                let expires_in_secs = if t.expires_at > now {
                    (t.expires_at - now) / 1000
                } else {
                    0
                };

                let is_expired = FileTokenManager::is_token_expired(&t);
                Ok(Some(TokenInfo {
                    created_at: t.created_at,
                    expires_in_secs,
                    is_expired,
                    scope: t.scope,
                }))
            }
            None => Ok(None),
        }
    }
}

// ============================================================================
// InMemoryTokenManager (for testing)
// ============================================================================

/// In-memory token manager for testing.
#[derive(Debug)]
pub struct InMemoryTokenManager {
    tokens: RwLock<Option<OAuthTokens>>,
    refresh_count: std::sync::atomic::AtomicU32,
}

impl InMemoryTokenManager {
    pub fn new() -> Self {
        Self {
            tokens: RwLock::new(None),
            refresh_count: std::sync::atomic::AtomicU32::new(0),
        }
    }

    pub fn with_tokens(tokens: OAuthTokens) -> Self {
        Self {
            tokens: RwLock::new(Some(tokens)),
            refresh_count: std::sync::atomic::AtomicU32::new(0),
        }
    }

    pub fn refresh_count(&self) -> u32 {
        self.refresh_count.load(std::sync::atomic::Ordering::SeqCst)
    }
}

impl Default for InMemoryTokenManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TokenManager for InMemoryTokenManager {
    fn has_tokens(&self) -> bool {
        self.tokens
            .try_read()
            .map(|guard| guard.is_some())
            .unwrap_or(false)
    }

    async fn save_tokens(&self, tokens: &OAuthTokens) -> Result<()> {
        let mut cache = self.tokens.write().await;
        *cache = Some(tokens.clone());
        Ok(())
    }

    async fn load_tokens(&self) -> Result<Option<OAuthTokens>> {
        let cache = self.tokens.read().await;
        Ok(cache.clone())
    }

    async fn get_valid_access_token(&self) -> Result<String> {
        let tokens = self
            .load_tokens()
            .await?
            .ok_or_else(|| OAuthError::Config("No OAuth tokens available".to_string()))?;

        if FileTokenManager::is_token_expired(&tokens) {
            self.refresh_count
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            tracing::debug!("InMemoryTokenManager: simulated token refresh");
        }

        Ok(tokens.access_token)
    }

    async fn clear_cache(&self) {
        let mut cache = self.tokens.write().await;
        *cache = None;
    }

    async fn delete_tokens(&self) -> Result<()> {
        self.clear_cache().await;
        Ok(())
    }

    async fn get_token_info(&self) -> Result<Option<TokenInfo>> {
        let tokens = self.load_tokens().await?;
        match tokens {
            Some(t) => {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64;

                let expires_in_secs = if t.expires_at > now {
                    (t.expires_at - now) / 1000
                } else {
                    0
                };

                let is_expired = FileTokenManager::is_token_expired(&t);
                Ok(Some(TokenInfo {
                    created_at: t.created_at,
                    expires_in_secs,
                    is_expired,
                    scope: t.scope,
                }))
            }
            None => Ok(None),
        }
    }
}

// ============================================================================
// TokenInfo
// ============================================================================

/// Information about stored tokens for display.
#[derive(Debug, Clone)]
pub struct TokenInfo {
    pub created_at: String,
    pub expires_in_secs: u64,
    pub is_expired: bool,
    pub scope: String,
}

impl TokenInfo {
    pub fn expires_in_display(&self) -> String {
        if self.is_expired {
            "Expired (will refresh on next use)".to_string()
        } else {
            let hours = self.expires_in_secs / 3600;
            let minutes = (self.expires_in_secs % 3600) / 60;
            format!("{}h {}m", hours, minutes)
        }
    }
}

// ============================================================================
// Shared Token Manager
// ============================================================================

/// Shared token manager for use across async contexts.
pub type SharedTokenManager = Arc<dyn TokenManager>;

/// Create a shared file-based token manager.
pub fn create_token_manager(data_dir: &Path) -> SharedTokenManager {
    Arc::new(FileTokenManager::new(data_dir))
}

/// Create a shared in-memory token manager (for testing).
pub fn create_memory_token_manager() -> SharedTokenManager {
    Arc::new(InMemoryTokenManager::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_file_token_manager_new() {
        let temp = tempdir().unwrap();
        let manager = FileTokenManager::new(temp.path());
        assert!(!manager.has_tokens());
    }

    #[tokio::test]
    async fn test_file_save_and_load_tokens() {
        let temp = tempdir().unwrap();
        let manager = FileTokenManager::new(temp.path());

        let tokens = OAuthTokens {
            access_token: "test_access".to_string(),
            refresh_token: "test_refresh".to_string(),
            expires_in: 3600,
            token_type: "Bearer".to_string(),
            scope: "test".to_string(),
            expires_at: 9999999999999,
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };

        manager.save_tokens(&tokens).await.unwrap();
        assert!(manager.has_tokens());

        let loaded = manager.load_tokens().await.unwrap().unwrap();
        assert_eq!(loaded.access_token, "test_access");
        assert_eq!(loaded.refresh_token, "test_refresh");
    }

    #[test]
    fn test_is_token_expired() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let valid = OAuthTokens {
            access_token: "t".to_string(),
            refresh_token: "r".to_string(),
            expires_in: 3600,
            token_type: "Bearer".to_string(),
            scope: "test".to_string(),
            expires_at: now + 3600 * 1000,
            created_at: String::new(),
        };
        assert!(!FileTokenManager::is_token_expired(&valid));

        let expiring = OAuthTokens {
            expires_at: now + 2 * 60 * 1000,
            ..valid.clone()
        };
        assert!(FileTokenManager::is_token_expired(&expiring));

        let expired = OAuthTokens {
            expires_at: now - 1000,
            ..valid
        };
        assert!(FileTokenManager::is_token_expired(&expired));
    }

    #[tokio::test]
    async fn test_file_delete_tokens() {
        let temp = tempdir().unwrap();
        let manager = FileTokenManager::new(temp.path());

        let tokens = OAuthTokens {
            access_token: "t".to_string(),
            refresh_token: "r".to_string(),
            expires_in: 3600,
            token_type: "Bearer".to_string(),
            scope: "test".to_string(),
            expires_at: 9999999999999,
            created_at: String::new(),
        };

        manager.save_tokens(&tokens).await.unwrap();
        assert!(manager.has_tokens());

        manager.delete_tokens().await.unwrap();
        assert!(!manager.has_tokens());
    }

    #[tokio::test]
    async fn test_inmemory_token_manager() {
        let manager = InMemoryTokenManager::new();
        assert!(!manager.has_tokens());

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let tokens = OAuthTokens {
            access_token: "valid_token".to_string(),
            refresh_token: "refresh".to_string(),
            expires_in: 3600,
            token_type: "Bearer".to_string(),
            scope: "test".to_string(),
            expires_at: now + 3600 * 1000,
            created_at: String::new(),
        };

        manager.save_tokens(&tokens).await.unwrap();
        assert!(manager.has_tokens());

        let token = manager.get_valid_access_token().await.unwrap();
        assert_eq!(token, "valid_token");
        assert_eq!(manager.refresh_count(), 0);
    }

    #[tokio::test]
    async fn test_inmemory_no_tokens_error() {
        let manager = InMemoryTokenManager::new();
        let result = manager.get_valid_access_token().await;
        assert!(result.is_err());
    }

    #[test]
    fn test_token_info_display() {
        let expired = TokenInfo {
            created_at: String::new(),
            expires_in_secs: 0,
            is_expired: true,
            scope: "test".to_string(),
        };
        assert!(expired.expires_in_display().contains("Expired"));

        let valid = TokenInfo {
            created_at: String::new(),
            expires_in_secs: 7200,
            is_expired: false,
            scope: "test".to_string(),
        };
        assert_eq!(valid.expires_in_display(), "2h 0m");
    }
}
