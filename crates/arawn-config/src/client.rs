//! Client configuration for connecting to Arawn servers.
//!
//! Implements a kubeconfig-style configuration with named contexts:
//!
//! ```yaml
//! apiVersion: v1
//! kind: ClientConfig
//!
//! current-context: local
//!
//! contexts:
//!   - name: local
//!     server: http://localhost:8080
//!   - name: home
//!     server: https://arawn.home.lan:8443
//!     auth:
//!       type: api-key
//!       key-file: ~/.config/arawn/keys/home.key
//! ```

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{ConfigError, Result};

// ─────────────────────────────────────────────────────────────────────────────
// Client Config
// ─────────────────────────────────────────────────────────────────────────────

/// API version for the client config file format.
pub const API_VERSION: &str = "v1";

/// Kind identifier for client config files.
pub const KIND: &str = "ClientConfig";

/// Default config filename.
const CLIENT_CONFIG_FILE: &str = "client.yaml";

/// Root client configuration structure.
///
/// This is the main config file for TUI and CLI client connections.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ClientConfig {
    /// API version (always "v1" currently).
    #[serde(default = "default_api_version")]
    pub api_version: String,

    /// Config kind (always "ClientConfig").
    #[serde(default = "default_kind")]
    pub kind: String,

    /// Name of the current/default context.
    #[serde(default)]
    pub current_context: Option<String>,

    /// Named connection contexts.
    #[serde(default)]
    pub contexts: Vec<Context>,

    /// Default settings applied to all contexts.
    #[serde(default)]
    pub defaults: ClientDefaults,
}

fn default_api_version() -> String {
    API_VERSION.to_string()
}

fn default_kind() -> String {
    KIND.to_string()
}

impl ClientConfig {
    /// Create an empty client config.
    pub fn new() -> Self {
        Self {
            api_version: API_VERSION.to_string(),
            kind: KIND.to_string(),
            ..Default::default()
        }
    }

    /// Parse from a YAML string.
    pub fn from_yaml(yaml_str: &str) -> Result<Self> {
        serde_yaml::from_str(yaml_str).map_err(|e| ConfigError::ParseYaml(e.to_string()))
    }

    /// Serialize to a YAML string.
    pub fn to_yaml(&self) -> Result<String> {
        serde_yaml::to_string(self).map_err(|e| ConfigError::ParseYaml(e.to_string()))
    }

    /// Get the current context, if set and valid.
    pub fn current(&self) -> Option<&Context> {
        self.current_context
            .as_ref()
            .and_then(|name| self.get_context(name))
    }

    /// Get a context by name.
    pub fn get_context(&self, name: &str) -> Option<&Context> {
        self.contexts.iter().find(|c| c.name == name)
    }

    /// Get a mutable context by name.
    pub fn get_context_mut(&mut self, name: &str) -> Option<&mut Context> {
        self.contexts.iter_mut().find(|c| c.name == name)
    }

    /// Add or update a context.
    pub fn set_context(&mut self, context: Context) {
        if let Some(existing) = self.get_context_mut(&context.name) {
            *existing = context;
        } else {
            self.contexts.push(context);
        }
    }

    /// Remove a context by name.
    pub fn remove_context(&mut self, name: &str) -> Option<Context> {
        if let Some(pos) = self.contexts.iter().position(|c| c.name == name) {
            // If removing current context, clear it
            if self.current_context.as_deref() == Some(name) {
                self.current_context = None;
            }
            Some(self.contexts.remove(pos))
        } else {
            None
        }
    }

    /// Set the current context by name.
    ///
    /// Returns an error if the context doesn't exist.
    pub fn use_context(&mut self, name: &str) -> Result<()> {
        if self.get_context(name).is_some() {
            self.current_context = Some(name.to_string());
            Ok(())
        } else {
            Err(ConfigError::ContextNotFound(name.to_string()))
        }
    }

    /// List all context names.
    pub fn context_names(&self) -> Vec<&str> {
        self.contexts.iter().map(|c| c.name.as_str()).collect()
    }

    /// Get the effective server URL for a context, applying defaults.
    pub fn server_url(&self, context_name: &str) -> Option<String> {
        self.get_context(context_name).map(|c| c.server.clone())
    }

    /// Get the effective server URL for the current context.
    pub fn current_server_url(&self) -> Option<String> {
        self.current().map(|c| c.server.clone())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Context
// ─────────────────────────────────────────────────────────────────────────────

/// A named connection context (server + auth bundle).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Context {
    /// Unique name for this context.
    pub name: String,

    /// Server URL (e.g., "http://localhost:8080" or "https://arawn.example.com").
    pub server: String,

    /// Authentication configuration.
    #[serde(default)]
    pub auth: Option<AuthConfig>,

    /// Default workstream for this context.
    #[serde(default)]
    pub workstream: Option<String>,

    /// Connection timeout override (seconds).
    #[serde(default)]
    pub timeout: Option<u64>,
}

impl Context {
    /// Create a new context with just a name and server URL.
    pub fn new(name: impl Into<String>, server: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            server: server.into(),
            auth: None,
            workstream: None,
            timeout: None,
        }
    }

    /// Set the auth configuration.
    pub fn with_auth(mut self, auth: AuthConfig) -> Self {
        self.auth = Some(auth);
        self
    }

    /// Set the default workstream.
    pub fn with_workstream(mut self, workstream: impl Into<String>) -> Self {
        self.workstream = Some(workstream.into());
        self
    }

    /// Set the connection timeout.
    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.timeout = Some(timeout);
        self
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Authentication
// ─────────────────────────────────────────────────────────────────────────────

/// Authentication configuration for a context.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum AuthConfig {
    /// No authentication.
    None,

    /// API key authentication.
    #[serde(rename_all = "kebab-case")]
    ApiKey {
        /// Path to file containing the API key.
        key_file: Option<PathBuf>,
        /// Environment variable containing the API key.
        key_env: Option<String>,
    },

    /// OAuth 2.0 authentication.
    #[serde(rename_all = "kebab-case")]
    Oauth {
        /// OAuth client ID.
        client_id: String,
        /// Path to cached tokens file.
        token_file: Option<PathBuf>,
    },

    /// Bearer token authentication.
    #[serde(rename_all = "kebab-case")]
    Bearer {
        /// Path to file containing the bearer token.
        token_file: Option<PathBuf>,
        /// Environment variable containing the token.
        token_env: Option<String>,
    },
}

impl AuthConfig {
    /// Create API key auth referencing a file.
    pub fn api_key_file(path: impl Into<PathBuf>) -> Self {
        Self::ApiKey {
            key_file: Some(path.into()),
            key_env: None,
        }
    }

    /// Create API key auth referencing an environment variable.
    pub fn api_key_env(var: impl Into<String>) -> Self {
        Self::ApiKey {
            key_file: None,
            key_env: Some(var.into()),
        }
    }

    /// Create OAuth auth.
    pub fn oauth(client_id: impl Into<String>) -> Self {
        Self::Oauth {
            client_id: client_id.into(),
            token_file: None,
        }
    }

    /// Resolve the actual credential value.
    ///
    /// Reads from file or environment variable as configured.
    pub fn resolve(&self) -> Result<Option<String>> {
        match self {
            AuthConfig::None => Ok(None),

            AuthConfig::ApiKey { key_file, key_env } => {
                // Try file first, then env var
                if let Some(path) = key_file {
                    let expanded = expand_path(path);
                    if expanded.exists() {
                        let key = std::fs::read_to_string(&expanded)
                            .map_err(|e| ConfigError::ReadFile {
                                path: expanded.display().to_string(),
                                source: e,
                            })?
                            .trim()
                            .to_string();
                        return Ok(Some(key));
                    }
                }
                if let Some(var) = key_env {
                    if let Ok(key) = std::env::var(var) {
                        return Ok(Some(key));
                    }
                }
                Ok(None)
            }

            AuthConfig::Bearer {
                token_file,
                token_env,
            } => {
                if let Some(path) = token_file {
                    let expanded = expand_path(path);
                    if expanded.exists() {
                        let token = std::fs::read_to_string(&expanded)
                            .map_err(|e| ConfigError::ReadFile {
                                path: expanded.display().to_string(),
                                source: e,
                            })?
                            .trim()
                            .to_string();
                        return Ok(Some(token));
                    }
                }
                if let Some(var) = token_env {
                    if let Ok(token) = std::env::var(var) {
                        return Ok(Some(token));
                    }
                }
                Ok(None)
            }

            AuthConfig::Oauth { .. } => {
                // OAuth tokens are handled separately by the OAuth flow
                Ok(None)
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Defaults
// ─────────────────────────────────────────────────────────────────────────────

/// Default settings applied to all contexts.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct ClientDefaults {
    /// Default connection timeout in seconds.
    pub timeout: u64,

    /// Default workstream name.
    pub workstream: String,
}

impl Default for ClientDefaults {
    fn default() -> Self {
        Self {
            timeout: 30,
            workstream: "default".to_string(),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Loading / Saving
// ─────────────────────────────────────────────────────────────────────────────

/// Get the path to the client config file.
pub fn client_config_path() -> Option<PathBuf> {
    crate::xdg_config_dir().map(|d| d.join(CLIENT_CONFIG_FILE))
}

/// Load the client configuration.
///
/// Returns a default config if the file doesn't exist.
pub fn load_client_config() -> Result<ClientConfig> {
    load_client_config_from(client_config_path().as_deref())
}

/// Load client config from a specific path.
pub fn load_client_config_from(path: Option<&Path>) -> Result<ClientConfig> {
    let Some(path) = path else {
        return Ok(ClientConfig::new());
    };

    if !path.exists() {
        return Ok(ClientConfig::new());
    }

    let contents = std::fs::read_to_string(path).map_err(|e| ConfigError::ReadFile {
        path: path.display().to_string(),
        source: e,
    })?;

    ClientConfig::from_yaml(&contents)
}

/// Save the client configuration.
pub fn save_client_config(config: &ClientConfig) -> Result<()> {
    let path = client_config_path()
        .ok_or_else(|| ConfigError::Other("Could not determine config directory".to_string()))?;
    save_client_config_to(config, &path)
}

/// Save client config to a specific path.
pub fn save_client_config_to(config: &ClientConfig, path: &Path) -> Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| ConfigError::WriteFile {
            path: parent.display().to_string(),
            source: e,
        })?;
    }

    let contents = config.to_yaml()?;
    std::fs::write(path, contents).map_err(|e| ConfigError::WriteFile {
        path: path.display().to_string(),
        source: e,
    })?;

    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Expand ~ to home directory in paths.
fn expand_path(path: &Path) -> PathBuf {
    if let Some(s) = path.to_str() {
        if let Some(rest) = s.strip_prefix("~/") {
            if let Some(home) = dirs::home_dir() {
                return home.join(rest);
            }
        }
    }
    path.to_path_buf()
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_config() {
        let config = ClientConfig::new();
        assert_eq!(config.api_version, API_VERSION);
        assert_eq!(config.kind, KIND);
        assert!(config.current_context.is_none());
        assert!(config.contexts.is_empty());
    }

    #[test]
    fn test_parse_minimal_yaml() {
        let yaml = r#"
api-version: v1
kind: ClientConfig
current-context: local
contexts:
  - name: local
    server: http://localhost:8080
"#;
        let config = ClientConfig::from_yaml(yaml).unwrap();
        assert_eq!(config.current_context.as_deref(), Some("local"));
        assert_eq!(config.contexts.len(), 1);
        assert_eq!(config.contexts[0].name, "local");
        assert_eq!(config.contexts[0].server, "http://localhost:8080");
    }

    #[test]
    fn test_parse_full_yaml() {
        let yaml = r#"
api-version: v1
kind: ClientConfig
current-context: home

contexts:
  - name: local
    server: http://localhost:8080

  - name: home
    server: https://arawn.home.lan:8443
    auth:
      type: api-key
      key-file: ~/.config/arawn/keys/home.key
    workstream: personal
    timeout: 60

  - name: work
    server: https://arawn.company.com
    auth:
      type: oauth
      client-id: arawn-tui

defaults:
  timeout: 30
  workstream: default
"#;
        let config = ClientConfig::from_yaml(yaml).unwrap();

        assert_eq!(config.current_context.as_deref(), Some("home"));
        assert_eq!(config.contexts.len(), 3);

        // Check local context
        let local = config.get_context("local").unwrap();
        assert_eq!(local.server, "http://localhost:8080");
        assert!(local.auth.is_none());

        // Check home context
        let home = config.get_context("home").unwrap();
        assert_eq!(home.server, "https://arawn.home.lan:8443");
        assert_eq!(home.workstream.as_deref(), Some("personal"));
        assert_eq!(home.timeout, Some(60));
        match &home.auth {
            Some(AuthConfig::ApiKey { key_file, .. }) => {
                assert_eq!(
                    key_file.as_ref().unwrap().to_str().unwrap(),
                    "~/.config/arawn/keys/home.key"
                );
            }
            _ => panic!("Expected ApiKey auth"),
        }

        // Check work context
        let work = config.get_context("work").unwrap();
        match &work.auth {
            Some(AuthConfig::Oauth { client_id, .. }) => {
                assert_eq!(client_id, "arawn-tui");
            }
            _ => panic!("Expected OAuth auth"),
        }

        // Check defaults
        assert_eq!(config.defaults.timeout, 30);
        assert_eq!(config.defaults.workstream, "default");
    }

    #[test]
    fn test_current_context() {
        let yaml = r#"
current-context: local
contexts:
  - name: local
    server: http://localhost:8080
"#;
        let config = ClientConfig::from_yaml(yaml).unwrap();
        let current = config.current().unwrap();
        assert_eq!(current.name, "local");
        assert_eq!(config.current_server_url(), Some("http://localhost:8080".to_string()));
    }

    #[test]
    fn test_set_context() {
        let mut config = ClientConfig::new();

        // Add new context
        config.set_context(Context::new("local", "http://localhost:8080"));
        assert_eq!(config.contexts.len(), 1);

        // Update existing context
        config.set_context(Context::new("local", "http://localhost:9090"));
        assert_eq!(config.contexts.len(), 1);
        assert_eq!(config.contexts[0].server, "http://localhost:9090");
    }

    #[test]
    fn test_remove_context() {
        let mut config = ClientConfig::new();
        config.set_context(Context::new("local", "http://localhost:8080"));
        config.set_context(Context::new("remote", "https://remote.example.com"));
        config.current_context = Some("local".to_string());

        // Remove non-current context
        let removed = config.remove_context("remote").unwrap();
        assert_eq!(removed.name, "remote");
        assert_eq!(config.contexts.len(), 1);
        assert_eq!(config.current_context.as_deref(), Some("local"));

        // Remove current context - should clear current
        config.remove_context("local");
        assert!(config.current_context.is_none());
    }

    #[test]
    fn test_use_context() {
        let mut config = ClientConfig::new();
        config.set_context(Context::new("local", "http://localhost:8080"));

        // Valid context
        config.use_context("local").unwrap();
        assert_eq!(config.current_context.as_deref(), Some("local"));

        // Invalid context
        let err = config.use_context("nonexistent").unwrap_err();
        assert!(matches!(err, ConfigError::ContextNotFound(_)));
    }

    #[test]
    fn test_context_names() {
        let mut config = ClientConfig::new();
        config.set_context(Context::new("local", "http://localhost:8080"));
        config.set_context(Context::new("remote", "https://remote.example.com"));

        let names = config.context_names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"local"));
        assert!(names.contains(&"remote"));
    }

    #[test]
    fn test_roundtrip_yaml() {
        let mut config = ClientConfig::new();
        config.set_context(
            Context::new("local", "http://localhost:8080")
                .with_workstream("default")
                .with_timeout(30),
        );
        config.set_context(
            Context::new("remote", "https://remote.example.com")
                .with_auth(AuthConfig::api_key_file("~/.config/arawn/keys/remote.key")),
        );
        config.current_context = Some("local".to_string());

        let yaml = config.to_yaml().unwrap();
        let reparsed = ClientConfig::from_yaml(&yaml).unwrap();

        assert_eq!(reparsed.current_context, config.current_context);
        assert_eq!(reparsed.contexts.len(), config.contexts.len());
    }

    #[test]
    fn test_context_builder() {
        let context = Context::new("test", "http://test.local")
            .with_auth(AuthConfig::api_key_env("TEST_API_KEY"))
            .with_workstream("testing")
            .with_timeout(60);

        assert_eq!(context.name, "test");
        assert_eq!(context.server, "http://test.local");
        assert_eq!(context.workstream.as_deref(), Some("testing"));
        assert_eq!(context.timeout, Some(60));
        assert!(context.auth.is_some());
    }

    #[test]
    fn test_auth_api_key_env_resolve() {
        // SAFETY: Test is single-threaded, env var is test-specific
        unsafe {
            std::env::set_var("TEST_ARAWN_KEY", "secret123");
        }
        let auth = AuthConfig::api_key_env("TEST_ARAWN_KEY");
        let resolved = auth.resolve().unwrap();
        assert_eq!(resolved, Some("secret123".to_string()));
        // SAFETY: Cleanup test env var
        unsafe {
            std::env::remove_var("TEST_ARAWN_KEY");
        }
    }

    #[test]
    fn test_auth_none_resolve() {
        let auth = AuthConfig::None;
        let resolved = auth.resolve().unwrap();
        assert!(resolved.is_none());
    }

    #[test]
    fn test_expand_path() {
        let path = PathBuf::from("~/test/file.key");
        let expanded = expand_path(&path);
        // Should expand on systems with a home dir
        if dirs::home_dir().is_some() {
            assert!(!expanded.to_str().unwrap().starts_with("~/"));
        }

        // Non-tilde paths unchanged
        let path2 = PathBuf::from("/absolute/path");
        assert_eq!(expand_path(&path2), path2);
    }
}
