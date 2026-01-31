//! Plugin subscription management.
//!
//! Handles loading, merging, and managing plugin subscriptions from multiple
//! sources:
//!
//! - `arawn.toml` `[plugins.subscriptions]` for initial/configured subscriptions
//! - `~/.config/arawn/plugins.json` for runtime-added global subscriptions
//! - `.arawn/plugins.json` for project-specific subscriptions
//!
//! ## Runtime plugins.json Format
//!
//! ```json
//! {
//!   "enabledPlugins": {
//!     "journal@local": true,
//!     "github-tools@github.com/author/repo": false
//!   },
//!   "subscriptions": [
//!     {
//!       "source": "github",
//!       "repo": "author/repo",
//!       "ref": "v1.0.0"
//!     }
//!   ]
//! }
//! ```

use arawn_config::{PluginSource, PluginSubscription};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Runtime plugins configuration file format.
///
/// This is the JSON format used for `~/.config/arawn/plugins.json` and
/// `.arawn/plugins.json` files.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct RuntimePluginsConfig {
    /// Map of plugin identifiers to enabled state.
    ///
    /// Keys are in format "name@source" where source can be:
    /// - `local` for local plugins
    /// - `github.com/owner/repo` for GitHub plugins
    /// - A URL hash for URL-based plugins
    #[serde(rename = "enabledPlugins")]
    pub enabled_plugins: HashMap<String, bool>,

    /// Plugin subscriptions added at runtime.
    #[serde(default)]
    pub subscriptions: Vec<PluginSubscription>,
}

impl RuntimePluginsConfig {
    /// Load from a JSON file, returning default if file doesn't exist.
    pub fn load(path: &Path) -> crate::Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(path)?;
        Self::from_json(&content)
    }

    /// Parse from a JSON string.
    pub fn from_json(json_str: &str) -> crate::Result<Self> {
        serde_json::from_str(json_str).map_err(|e| crate::PluginError::ManifestParse {
            reason: format!("plugins.json: {}", e),
        })
    }

    /// Serialize to a JSON string (pretty printed).
    pub fn to_json(&self) -> crate::Result<String> {
        serde_json::to_string_pretty(self).map_err(|e| crate::PluginError::ManifestParse {
            reason: format!("failed to serialize plugins.json: {}", e),
        })
    }

    /// Save to a JSON file.
    pub fn save(&self, path: &Path) -> crate::Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = self.to_json()?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Check if a plugin is enabled.
    ///
    /// Returns `None` if the plugin is not in the enabled map (use default behavior).
    pub fn is_enabled(&self, plugin_id: &str) -> Option<bool> {
        self.enabled_plugins.get(plugin_id).copied()
    }

    /// Set a plugin's enabled state.
    pub fn set_enabled(&mut self, plugin_id: impl Into<String>, enabled: bool) {
        self.enabled_plugins.insert(plugin_id.into(), enabled);
    }

    /// Add a subscription.
    pub fn add_subscription(&mut self, subscription: PluginSubscription) {
        // Check if we already have this subscription
        let id = subscription.id();
        if !self.subscriptions.iter().any(|s| s.id() == id) {
            self.subscriptions.push(subscription);
        }
    }

    /// Remove a subscription by its ID.
    pub fn remove_subscription(&mut self, subscription_id: &str) {
        self.subscriptions.retain(|s| s.id() != subscription_id);
    }

    /// Merge another config into this one.
    ///
    /// Subscriptions are deduplicated by ID. Enabled states are overwritten.
    pub fn merge(&mut self, other: RuntimePluginsConfig) {
        // Merge enabled states (other takes precedence)
        for (id, enabled) in other.enabled_plugins {
            self.enabled_plugins.insert(id, enabled);
        }

        // Merge subscriptions (deduplicate by ID)
        for sub in other.subscriptions {
            let id = sub.id();
            if !self.subscriptions.iter().any(|s| s.id() == id) {
                self.subscriptions.push(sub);
            }
        }
    }
}

/// Manager for plugin subscriptions across all sources.
#[derive(Debug, Clone)]
pub struct SubscriptionManager {
    /// Subscriptions from arawn.toml.
    config_subscriptions: Vec<PluginSubscription>,
    /// Global runtime config (~/.config/arawn/plugins.json).
    global_config: RuntimePluginsConfig,
    /// Project-local runtime config (.arawn/plugins.json).
    project_config: RuntimePluginsConfig,
    /// Path to global config file.
    global_config_path: PathBuf,
    /// Path to project config file (if any).
    project_config_path: Option<PathBuf>,
    /// Path to plugin cache directory.
    cache_dir: PathBuf,
}

impl SubscriptionManager {
    /// Create a new subscription manager.
    ///
    /// # Arguments
    ///
    /// * `config_subscriptions` - Subscriptions from arawn.toml
    /// * `project_dir` - Optional project directory for project-local plugins
    pub fn new(
        config_subscriptions: Vec<PluginSubscription>,
        project_dir: Option<&Path>,
    ) -> crate::Result<Self> {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("arawn");
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from(".cache"))
            .join("arawn")
            .join("plugins");

        let global_config_path = config_dir.join("plugins.json");
        let global_config = RuntimePluginsConfig::load(&global_config_path)?;

        let (project_config_path, project_config) = if let Some(dir) = project_dir {
            let path = dir.join(".arawn").join("plugins.json");
            let config = RuntimePluginsConfig::load(&path)?;
            (Some(path), config)
        } else {
            (None, RuntimePluginsConfig::default())
        };

        Ok(Self {
            config_subscriptions,
            global_config,
            project_config,
            global_config_path,
            project_config_path,
            cache_dir,
        })
    }

    /// Get all active subscriptions, merged from all sources.
    ///
    /// Subscriptions are deduplicated by ID, with later sources taking
    /// precedence (project > global > config).
    pub fn all_subscriptions(&self) -> Vec<PluginSubscription> {
        let mut seen = std::collections::HashSet::new();
        let mut result = Vec::new();

        // Add in reverse priority order (config first, then global, then project)
        // so that later duplicates replace earlier ones
        let sources = [
            &self.config_subscriptions,
            &self.global_config.subscriptions,
            &self.project_config.subscriptions,
        ];

        for source in sources {
            for sub in source {
                let id = sub.id();
                if !seen.contains(&id) {
                    seen.insert(id);
                    result.push(sub.clone());
                }
            }
        }

        // Filter out disabled subscriptions
        result.retain(|sub| {
            let id = sub.id();
            // Check project config first, then global
            if let Some(enabled) = self.project_config.is_enabled(&id) {
                return enabled;
            }
            if let Some(enabled) = self.global_config.is_enabled(&id) {
                return enabled;
            }
            // Default to the subscription's own enabled flag
            sub.enabled
        });

        result
    }

    /// Get the cache directory for a subscription.
    pub fn cache_dir_for(&self, subscription: &PluginSubscription) -> PathBuf {
        self.cache_dir.join(subscription.id())
    }

    /// Get the global runtime config.
    pub fn global_config(&self) -> &RuntimePluginsConfig {
        &self.global_config
    }

    /// Get the project runtime config.
    pub fn project_config(&self) -> &RuntimePluginsConfig {
        &self.project_config
    }

    /// Get a mutable reference to the global runtime config.
    pub fn global_config_mut(&mut self) -> &mut RuntimePluginsConfig {
        &mut self.global_config
    }

    /// Get a mutable reference to the project runtime config.
    pub fn project_config_mut(&mut self) -> &mut RuntimePluginsConfig {
        &mut self.project_config
    }

    /// Save the global runtime config.
    pub fn save_global_config(&self) -> crate::Result<()> {
        self.global_config.save(&self.global_config_path)
    }

    /// Save the project runtime config.
    pub fn save_project_config(&self) -> crate::Result<()> {
        if let Some(ref path) = self.project_config_path {
            self.project_config.save(path)
        } else {
            Ok(())
        }
    }

    /// Add a subscription to the global config.
    pub fn add_global_subscription(&mut self, subscription: PluginSubscription) {
        self.global_config.add_subscription(subscription);
    }

    /// Add a subscription to the project config.
    pub fn add_project_subscription(&mut self, subscription: PluginSubscription) {
        self.project_config.add_subscription(subscription);
    }

    /// Enable or disable a plugin globally.
    pub fn set_global_enabled(&mut self, plugin_id: impl Into<String>, enabled: bool) {
        self.global_config.set_enabled(plugin_id, enabled);
    }

    /// Enable or disable a plugin for the current project.
    pub fn set_project_enabled(&mut self, plugin_id: impl Into<String>, enabled: bool) {
        self.project_config.set_enabled(plugin_id, enabled);
    }

    /// Get the cache directory.
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Check if auto-update is disabled via environment variable.
    ///
    /// Returns true if `ARAWN_DISABLE_PLUGIN_UPDATES=1` is set.
    pub fn is_auto_update_disabled() -> bool {
        std::env::var("ARAWN_DISABLE_PLUGIN_UPDATES")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false)
    }

    /// Get the update timeout from environment variable.
    ///
    /// Returns the value of `ARAWN_PLUGIN_UPDATE_TIMEOUT` in seconds,
    /// or 30 seconds if not set.
    pub fn update_timeout_secs() -> u64 {
        std::env::var("ARAWN_PLUGIN_UPDATE_TIMEOUT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(30)
    }

    /// Sync all subscriptions in parallel (async version).
    ///
    /// Returns a list of sync results for each subscription.
    /// Uses tokio::task::spawn_blocking for git operations.
    pub async fn sync_all_async(&self) -> Vec<SyncResult> {
        use tokio::task;
        use tokio::time::{Duration, timeout};

        if Self::is_auto_update_disabled() {
            tracing::info!("Plugin updates disabled via ARAWN_DISABLE_PLUGIN_UPDATES");
            return self
                .all_subscriptions()
                .into_iter()
                .map(|sub| SyncResult {
                    subscription_id: sub.id(),
                    action: SyncAction::Skipped,
                    path: self.plugin_dir_for(&sub),
                    error: Some("Updates disabled".to_string()),
                })
                .collect();
        }

        let subscriptions = self.all_subscriptions();
        let timeout_duration = Duration::from_secs(Self::update_timeout_secs());

        // Spawn sync tasks in parallel
        let mut handles = Vec::with_capacity(subscriptions.len());
        for sub in subscriptions {
            let cache_dir = self.cache_dir.clone();
            let sub_id_for_handle = sub.id();
            let handle = task::spawn(async move {
                let sub_id = sub.id();
                let dest = cache_dir.join(&sub_id);

                // Local subscriptions don't need syncing
                if sub.source == PluginSource::Local {
                    return SyncResult {
                        subscription_id: sub_id,
                        action: SyncAction::Skipped,
                        path: sub.path.clone(),
                        error: None,
                    };
                }

                // Run git operations in blocking thread pool
                let dest_clone = dest.clone();
                let sub_id_for_panic = sub_id.clone();
                let sync_result = task::spawn_blocking(move || {
                    let sub_id = sub.id();
                    if dest_clone.join(".git").exists() {
                        // Update existing clone
                        match GitOps::pull(&dest_clone, sub.effective_ref()) {
                            Ok(()) => SyncResult {
                                subscription_id: sub_id,
                                action: SyncAction::Updated,
                                path: Some(dest_clone),
                                error: None,
                            },
                            Err(e) => SyncResult {
                                subscription_id: sub_id,
                                action: SyncAction::UpdateFailed,
                                path: Some(dest_clone),
                                error: Some(e),
                            },
                        }
                    } else {
                        // Fresh clone
                        match sub.clone_url() {
                            Some(url) => {
                                match GitOps::clone(&url, &dest_clone, sub.effective_ref()) {
                                    Ok(()) => SyncResult {
                                        subscription_id: sub_id,
                                        action: SyncAction::Cloned,
                                        path: Some(dest_clone),
                                        error: None,
                                    },
                                    Err(e) => SyncResult {
                                        subscription_id: sub_id,
                                        action: SyncAction::CloneFailed,
                                        path: None,
                                        error: Some(e),
                                    },
                                }
                            }
                            None => SyncResult {
                                subscription_id: sub_id,
                                action: SyncAction::CloneFailed,
                                path: None,
                                error: Some("No clone URL available".to_string()),
                            },
                        }
                    }
                })
                .await;

                match sync_result {
                    Ok(result) => result,
                    Err(e) => SyncResult {
                        subscription_id: sub_id_for_panic,
                        action: SyncAction::CloneFailed,
                        path: None,
                        error: Some(format!("Task panicked: {}", e)),
                    },
                }
            });

            handles.push((sub_id_for_handle, handle));
        }

        // Collect results with timeout
        let mut results = Vec::with_capacity(handles.len());
        for (sub_id, handle) in handles {
            match timeout(timeout_duration, handle).await {
                Ok(Ok(result)) => results.push(result),
                Ok(Err(e)) => {
                    results.push(SyncResult {
                        subscription_id: sub_id,
                        action: SyncAction::CloneFailed,
                        path: None,
                        error: Some(format!("Task error: {}", e)),
                    });
                }
                Err(_) => {
                    results.push(SyncResult {
                        subscription_id: sub_id,
                        action: SyncAction::UpdateFailed,
                        path: None,
                        error: Some(format!(
                            "Timeout after {} seconds",
                            timeout_duration.as_secs()
                        )),
                    });
                }
            }
        }

        results
    }

    /// Sync all subscriptions (clone or update).
    ///
    /// Returns a list of sync results for each subscription.
    pub fn sync_all(&self) -> Vec<SyncResult> {
        let subscriptions = self.all_subscriptions();
        let mut results = Vec::with_capacity(subscriptions.len());

        for sub in subscriptions {
            let result = self.sync_subscription(&sub);
            results.push(result);
        }

        results
    }

    /// Sync a single subscription (clone or update).
    pub fn sync_subscription(&self, subscription: &PluginSubscription) -> SyncResult {
        let dest = self.cache_dir_for(subscription);

        // Local subscriptions don't need syncing
        if subscription.source == PluginSource::Local {
            let path = subscription.path.as_ref().map(|p| p.to_path_buf());
            return SyncResult {
                subscription_id: subscription.id(),
                action: SyncAction::Skipped,
                path,
                error: None,
            };
        }

        // Check if we need to clone or update
        if dest.join(".git").exists() {
            // Already cloned, try to update
            match GitOps::pull(&dest, subscription.effective_ref()) {
                Ok(()) => SyncResult {
                    subscription_id: subscription.id(),
                    action: SyncAction::Updated,
                    path: Some(dest),
                    error: None,
                },
                Err(e) => SyncResult {
                    subscription_id: subscription.id(),
                    action: SyncAction::UpdateFailed,
                    path: Some(dest),
                    error: Some(e),
                },
            }
        } else {
            // Need to clone
            let clone_url = match subscription.clone_url() {
                Some(url) => url,
                None => {
                    return SyncResult {
                        subscription_id: subscription.id(),
                        action: SyncAction::CloneFailed,
                        path: None,
                        error: Some("No clone URL available".to_string()),
                    };
                }
            };

            match GitOps::clone(&clone_url, &dest, subscription.effective_ref()) {
                Ok(()) => SyncResult {
                    subscription_id: subscription.id(),
                    action: SyncAction::Cloned,
                    path: Some(dest),
                    error: None,
                },
                Err(e) => SyncResult {
                    subscription_id: subscription.id(),
                    action: SyncAction::CloneFailed,
                    path: None,
                    error: Some(e),
                },
            }
        }
    }

    /// Get the plugin directory for a subscription.
    ///
    /// For local subscriptions, returns the configured path.
    /// For remote subscriptions, returns the cache directory.
    pub fn plugin_dir_for(&self, subscription: &PluginSubscription) -> Option<PathBuf> {
        match subscription.source {
            PluginSource::Local => subscription.path.clone(),
            _ => {
                let cache_dir = self.cache_dir_for(subscription);
                if cache_dir.exists() {
                    Some(cache_dir)
                } else {
                    None
                }
            }
        }
    }

    /// Get all plugin directories (synced subscriptions + local paths).
    pub fn plugin_dirs(&self) -> Vec<PathBuf> {
        self.all_subscriptions()
            .iter()
            .filter_map(|sub| self.plugin_dir_for(sub))
            .collect()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Git Operations
// ─────────────────────────────────────────────────────────────────────────────

/// Git operations for plugin syncing.
///
/// Uses the system `git` command for better credential handling
/// (SSH keys, credential helpers, etc.).
pub struct GitOps;

impl GitOps {
    /// Clone a repository to a destination directory.
    ///
    /// Uses shallow clone (`--depth 1`) for faster downloads.
    pub fn clone(url: &str, dest: &Path, git_ref: &str) -> Result<(), String> {
        // Ensure parent directory exists
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        // Remove destination if it exists but isn't a git repo
        if dest.exists() && !dest.join(".git").exists() {
            std::fs::remove_dir_all(dest)
                .map_err(|e| format!("Failed to remove existing directory: {}", e))?;
        }

        let output = Command::new("git")
            .args([
                "clone",
                "--depth",
                "1",
                "--branch",
                git_ref,
                "--single-branch",
                url,
            ])
            .arg(dest)
            .output()
            .map_err(|e| format!("Failed to execute git clone: {}", e))?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("git clone failed: {}", stderr.trim()))
        }
    }

    /// Pull updates for an existing repository.
    ///
    /// Fetches and checks out the specified ref, then pulls if on a branch.
    pub fn pull(repo_dir: &Path, git_ref: &str) -> Result<(), String> {
        // First, fetch all refs
        let fetch_output = Command::new("git")
            .args(["fetch", "--all", "--prune"])
            .current_dir(repo_dir)
            .output()
            .map_err(|e| format!("Failed to execute git fetch: {}", e))?;

        if !fetch_output.status.success() {
            let stderr = String::from_utf8_lossy(&fetch_output.stderr);
            return Err(format!("git fetch failed: {}", stderr.trim()));
        }

        // Try to checkout the ref (could be branch, tag, or commit)
        let checkout_output = Command::new("git")
            .args(["checkout", git_ref])
            .current_dir(repo_dir)
            .output()
            .map_err(|e| format!("Failed to execute git checkout: {}", e))?;

        if !checkout_output.status.success() {
            // If checkout fails, try origin/<ref> for branches
            let origin_ref = format!("origin/{}", git_ref);
            let checkout_origin = Command::new("git")
                .args(["checkout", &origin_ref])
                .current_dir(repo_dir)
                .output()
                .map_err(|e| format!("Failed to execute git checkout: {}", e))?;

            if !checkout_origin.status.success() {
                let stderr = String::from_utf8_lossy(&checkout_output.stderr);
                return Err(format!("git checkout failed: {}", stderr.trim()));
            }
        }

        // Try to pull if we're on a branch (will fail gracefully for tags/commits)
        let pull_output = Command::new("git")
            .args(["pull", "--ff-only"])
            .current_dir(repo_dir)
            .output()
            .map_err(|e| format!("Failed to execute git pull: {}", e))?;

        // We don't fail if pull fails - could be a detached HEAD (tag/commit)
        if !pull_output.status.success() {
            let stderr = String::from_utf8_lossy(&pull_output.stderr);
            // Only warn, don't fail - detached HEAD is expected for tags
            tracing::debug!(
                repo = %repo_dir.display(),
                stderr = %stderr.trim(),
                "git pull skipped (possibly detached HEAD)"
            );
        }

        Ok(())
    }

    /// Check if git is available on the system.
    pub fn is_available() -> bool {
        Command::new("git")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Get the current commit hash of a repository.
    pub fn current_commit(repo_dir: &Path) -> Option<String> {
        Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(repo_dir)
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
    }

    /// Get the current branch name (if on a branch).
    pub fn current_branch(repo_dir: &Path) -> Option<String> {
        Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(repo_dir)
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .filter(|s| s != "HEAD") // Detached HEAD returns "HEAD"
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Sync Results
// ─────────────────────────────────────────────────────────────────────────────

/// Result of syncing a subscription.
#[derive(Debug, Clone)]
pub struct SyncResult {
    /// Subscription ID that was synced.
    pub subscription_id: String,
    /// Action taken.
    pub action: SyncAction,
    /// Path to the plugin directory (if available).
    pub path: Option<PathBuf>,
    /// Error message if sync failed.
    pub error: Option<String>,
}

impl SyncResult {
    /// Check if the sync was successful.
    pub fn is_success(&self) -> bool {
        matches!(
            self.action,
            SyncAction::Cloned | SyncAction::Updated | SyncAction::Skipped
        )
    }

    /// Check if this was a failure.
    pub fn is_failure(&self) -> bool {
        matches!(
            self.action,
            SyncAction::CloneFailed | SyncAction::UpdateFailed
        )
    }
}

/// Action taken during sync.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncAction {
    /// Repository was cloned (first time).
    Cloned,
    /// Repository was updated (pulled).
    Updated,
    /// Sync was skipped (e.g., local path).
    Skipped,
    /// Clone operation failed.
    CloneFailed,
    /// Update operation failed.
    UpdateFailed,
}

impl std::fmt::Display for SyncAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncAction::Cloned => write!(f, "cloned"),
            SyncAction::Updated => write!(f, "updated"),
            SyncAction::Skipped => write!(f, "skipped"),
            SyncAction::CloneFailed => write!(f, "clone failed"),
            SyncAction::UpdateFailed => write!(f, "update failed"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arawn_config::PluginSource;
    use tempfile::TempDir;

    #[test]
    fn test_runtime_config_parse() {
        let json = r#"{
            "enabledPlugins": {
                "journal@local": true,
                "github-tools@github.com/author/repo": false
            },
            "subscriptions": [
                {
                    "source": "github",
                    "repo": "author/repo",
                    "ref": "v1.0.0"
                }
            ]
        }"#;

        let config = RuntimePluginsConfig::from_json(json).unwrap();
        assert_eq!(config.enabled_plugins.get("journal@local"), Some(&true));
        assert_eq!(
            config
                .enabled_plugins
                .get("github-tools@github.com/author/repo"),
            Some(&false)
        );
        assert_eq!(config.subscriptions.len(), 1);
        assert_eq!(config.subscriptions[0].source, PluginSource::GitHub);
        assert_eq!(config.subscriptions[0].repo.as_deref(), Some("author/repo"));
        assert_eq!(config.subscriptions[0].git_ref.as_deref(), Some("v1.0.0"));
    }

    #[test]
    fn test_runtime_config_empty() {
        let json = "{}";
        let config = RuntimePluginsConfig::from_json(json).unwrap();
        assert!(config.enabled_plugins.is_empty());
        assert!(config.subscriptions.is_empty());
    }

    #[test]
    fn test_runtime_config_roundtrip() {
        let mut config = RuntimePluginsConfig::default();
        config.set_enabled("test@local", true);
        config.add_subscription(PluginSubscription::github("owner/repo").with_ref("v1.0.0"));

        let json = config.to_json().unwrap();
        let reparsed = RuntimePluginsConfig::from_json(&json).unwrap();

        assert_eq!(reparsed.enabled_plugins.get("test@local"), Some(&true));
        assert_eq!(reparsed.subscriptions.len(), 1);
    }

    #[test]
    fn test_runtime_config_save_load() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("plugins.json");

        let mut config = RuntimePluginsConfig::default();
        config.set_enabled("test@local", true);
        config.add_subscription(PluginSubscription::github("owner/repo"));

        config.save(&path).unwrap();
        let loaded = RuntimePluginsConfig::load(&path).unwrap();

        assert_eq!(loaded.enabled_plugins.get("test@local"), Some(&true));
        assert_eq!(loaded.subscriptions.len(), 1);
    }

    #[test]
    fn test_runtime_config_load_missing_file() {
        let config = RuntimePluginsConfig::load(Path::new("/nonexistent/path.json")).unwrap();
        assert!(config.enabled_plugins.is_empty());
        assert!(config.subscriptions.is_empty());
    }

    #[test]
    fn test_runtime_config_merge() {
        let mut config1 = RuntimePluginsConfig::default();
        config1.set_enabled("plugin1", true);
        config1.add_subscription(PluginSubscription::github("owner/repo1"));

        let mut config2 = RuntimePluginsConfig::default();
        config2.set_enabled("plugin1", false); // Override
        config2.set_enabled("plugin2", true);
        config2.add_subscription(PluginSubscription::github("owner/repo1")); // Duplicate
        config2.add_subscription(PluginSubscription::github("owner/repo2")); // New

        config1.merge(config2);

        assert_eq!(config1.enabled_plugins.get("plugin1"), Some(&false));
        assert_eq!(config1.enabled_plugins.get("plugin2"), Some(&true));
        assert_eq!(config1.subscriptions.len(), 2); // Deduplicated
    }

    #[test]
    fn test_subscription_id_github() {
        let sub = PluginSubscription::github("owner/repo");
        assert_eq!(sub.id(), "github/owner-repo");
    }

    #[test]
    fn test_subscription_id_url() {
        let sub = PluginSubscription::url("https://gitlab.com/team/plugin.git");
        let id = sub.id();
        assert!(id.starts_with("url/"));
    }

    #[test]
    fn test_subscription_id_local() {
        let sub = PluginSubscription::local("/path/to/plugin");
        let id = sub.id();
        assert!(id.starts_with("local/"));
    }

    #[test]
    fn test_subscription_clone_url() {
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
    fn test_subscription_effective_ref() {
        let sub1 = PluginSubscription::github("owner/repo");
        assert_eq!(sub1.effective_ref(), "main");

        let sub2 = PluginSubscription::github("owner/repo").with_ref("v1.0.0");
        assert_eq!(sub2.effective_ref(), "v1.0.0");
    }

    #[test]
    fn test_subscription_manager_merge() {
        let tmp = TempDir::new().unwrap();
        let project_dir = tmp.path();

        // Create project config
        std::fs::create_dir_all(project_dir.join(".arawn")).unwrap();
        let mut project_config = RuntimePluginsConfig::default();
        project_config.add_subscription(PluginSubscription::github("project/plugin"));
        project_config
            .save(&project_dir.join(".arawn/plugins.json"))
            .unwrap();

        let config_subs = vec![PluginSubscription::github("config/plugin")];

        let manager = SubscriptionManager::new(config_subs, Some(project_dir)).unwrap();
        let all_subs = manager.all_subscriptions();

        // Should have both config and project subscriptions
        assert_eq!(all_subs.len(), 2);
    }

    #[test]
    fn test_subscription_manager_dedup() {
        let tmp = TempDir::new().unwrap();
        let project_dir = tmp.path();

        // Create project config with same subscription as config
        std::fs::create_dir_all(project_dir.join(".arawn")).unwrap();
        let mut project_config = RuntimePluginsConfig::default();
        project_config.add_subscription(PluginSubscription::github("owner/repo"));
        project_config
            .save(&project_dir.join(".arawn/plugins.json"))
            .unwrap();

        let config_subs = vec![PluginSubscription::github("owner/repo")];

        let manager = SubscriptionManager::new(config_subs, Some(project_dir)).unwrap();
        let all_subs = manager.all_subscriptions();

        // Should deduplicate to single subscription
        assert_eq!(all_subs.len(), 1);
    }

    #[test]
    fn test_subscription_manager_enabled_filter() {
        let tmp = TempDir::new().unwrap();
        let project_dir = tmp.path();

        // Create project config that disables a subscription
        std::fs::create_dir_all(project_dir.join(".arawn")).unwrap();
        let mut project_config = RuntimePluginsConfig::default();
        project_config.set_enabled("github/config-plugin", false);
        project_config
            .save(&project_dir.join(".arawn/plugins.json"))
            .unwrap();

        let config_subs = vec![
            PluginSubscription::github("config/plugin"),
            PluginSubscription::github("another/plugin"),
        ];

        let manager = SubscriptionManager::new(config_subs, Some(project_dir)).unwrap();
        let all_subs = manager.all_subscriptions();

        // config/plugin should be filtered out (disabled)
        assert_eq!(all_subs.len(), 1);
        assert_eq!(all_subs[0].repo.as_deref(), Some("another/plugin"));
    }

    // ── Git Operations Tests ─────────────────────────────────────────────

    #[test]
    fn test_git_is_available() {
        // Git should be available in most development environments
        // This test documents the expected behavior
        let available = GitOps::is_available();
        // We don't assert because CI might not have git
        println!("Git available: {}", available);
    }

    #[test]
    fn test_sync_result_is_success() {
        let success_actions = [SyncAction::Cloned, SyncAction::Updated, SyncAction::Skipped];
        for action in success_actions {
            let result = SyncResult {
                subscription_id: "test".to_string(),
                action,
                path: None,
                error: None,
            };
            assert!(result.is_success(), "Expected {:?} to be success", action);
            assert!(!result.is_failure());
        }
    }

    #[test]
    fn test_sync_result_is_failure() {
        let failure_actions = [SyncAction::CloneFailed, SyncAction::UpdateFailed];
        for action in failure_actions {
            let result = SyncResult {
                subscription_id: "test".to_string(),
                action,
                path: None,
                error: Some("test error".to_string()),
            };
            assert!(result.is_failure(), "Expected {:?} to be failure", action);
            assert!(!result.is_success());
        }
    }

    #[test]
    fn test_sync_action_display() {
        assert_eq!(SyncAction::Cloned.to_string(), "cloned");
        assert_eq!(SyncAction::Updated.to_string(), "updated");
        assert_eq!(SyncAction::Skipped.to_string(), "skipped");
        assert_eq!(SyncAction::CloneFailed.to_string(), "clone failed");
        assert_eq!(SyncAction::UpdateFailed.to_string(), "update failed");
    }

    #[test]
    fn test_sync_local_subscription_skipped() {
        let tmp = TempDir::new().unwrap();
        let local_path = tmp.path().join("local-plugin");
        std::fs::create_dir_all(&local_path).unwrap();

        let config_subs = vec![PluginSubscription::local(&local_path)];
        let manager = SubscriptionManager::new(config_subs, None).unwrap();

        let results = manager.sync_all();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].action, SyncAction::Skipped);
        assert!(results[0].is_success());
    }

    #[test]
    fn test_plugin_dir_for_local() {
        let tmp = TempDir::new().unwrap();
        let local_path = tmp.path().join("local-plugin");
        std::fs::create_dir_all(&local_path).unwrap();

        let sub = PluginSubscription::local(&local_path);
        let config_subs = vec![sub.clone()];
        let manager = SubscriptionManager::new(config_subs, None).unwrap();

        let dir = manager.plugin_dir_for(&sub);
        assert_eq!(dir, Some(local_path));
    }

    #[test]
    fn test_plugin_dir_for_remote_not_synced() {
        let sub = PluginSubscription::github("owner/repo");
        let manager = SubscriptionManager::new(vec![sub.clone()], None).unwrap();

        // Remote subscription not yet cloned
        let dir = manager.plugin_dir_for(&sub);
        assert!(dir.is_none());
    }

    #[test]
    fn test_sync_subscription_no_clone_url() {
        // Create a malformed subscription with no URL
        let sub = PluginSubscription {
            source: PluginSource::Url,
            repo: None,
            url: None, // No URL!
            path: None,
            git_ref: None,
            enabled: true,
        };

        let manager = SubscriptionManager::new(vec![sub.clone()], None).unwrap();
        let result = manager.sync_subscription(&sub);

        assert_eq!(result.action, SyncAction::CloneFailed);
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("No clone URL"));
    }

    #[test]
    fn test_auto_update_disabled_check() {
        // Without env var set, should not be disabled
        // SAFETY: Tests run single-threaded with --test-threads=1 or we accept the race
        unsafe { std::env::remove_var("ARAWN_DISABLE_PLUGIN_UPDATES") };
        assert!(!SubscriptionManager::is_auto_update_disabled());

        // Test helper to restore env
        struct EnvGuard;
        impl Drop for EnvGuard {
            fn drop(&mut self) {
                // SAFETY: Cleanup in test context
                unsafe { std::env::remove_var("ARAWN_DISABLE_PLUGIN_UPDATES") };
            }
        }
        let _guard = EnvGuard;

        // With "1" should be disabled
        // SAFETY: Test context
        unsafe { std::env::set_var("ARAWN_DISABLE_PLUGIN_UPDATES", "1") };
        assert!(SubscriptionManager::is_auto_update_disabled());

        // With "true" should be disabled
        unsafe { std::env::set_var("ARAWN_DISABLE_PLUGIN_UPDATES", "true") };
        assert!(SubscriptionManager::is_auto_update_disabled());

        // With "TRUE" should be disabled (case insensitive)
        unsafe { std::env::set_var("ARAWN_DISABLE_PLUGIN_UPDATES", "TRUE") };
        assert!(SubscriptionManager::is_auto_update_disabled());

        // With other values should not be disabled
        unsafe { std::env::set_var("ARAWN_DISABLE_PLUGIN_UPDATES", "0") };
        assert!(!SubscriptionManager::is_auto_update_disabled());

        unsafe { std::env::set_var("ARAWN_DISABLE_PLUGIN_UPDATES", "false") };
        assert!(!SubscriptionManager::is_auto_update_disabled());
    }

    #[test]
    fn test_update_timeout_secs() {
        // SAFETY: Test context
        unsafe { std::env::remove_var("ARAWN_PLUGIN_UPDATE_TIMEOUT") };

        // Default is 30 seconds
        assert_eq!(SubscriptionManager::update_timeout_secs(), 30);

        // Test helper to restore env
        struct EnvGuard;
        impl Drop for EnvGuard {
            fn drop(&mut self) {
                // SAFETY: Cleanup in test context
                unsafe { std::env::remove_var("ARAWN_PLUGIN_UPDATE_TIMEOUT") };
            }
        }
        let _guard = EnvGuard;

        // Custom timeout
        unsafe { std::env::set_var("ARAWN_PLUGIN_UPDATE_TIMEOUT", "60") };
        assert_eq!(SubscriptionManager::update_timeout_secs(), 60);

        // Invalid value falls back to default
        unsafe { std::env::set_var("ARAWN_PLUGIN_UPDATE_TIMEOUT", "invalid") };
        assert_eq!(SubscriptionManager::update_timeout_secs(), 30);
    }

    // Note: test_sync_all_async_skips_when_disabled is skipped because env var
    // tests are flaky in parallel test environments. The logic is tested via
    // test_auto_update_disabled_check which tests the detection function directly.

    #[tokio::test]
    async fn test_sync_all_async_local_skipped() {
        // SAFETY: Test context
        unsafe { std::env::remove_var("ARAWN_DISABLE_PLUGIN_UPDATES") };

        let tmp = TempDir::new().unwrap();
        let local_path = tmp.path().join("local-plugin");
        std::fs::create_dir_all(&local_path).unwrap();

        let config_subs = vec![PluginSubscription::local(&local_path)];
        let manager = SubscriptionManager::new(config_subs, None).unwrap();

        let results = manager.sync_all_async().await;

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].action, SyncAction::Skipped);
        assert!(results[0].is_success());
    }

    // Integration test that requires git - marked as ignored for CI
    #[test]
    #[ignore]
    fn test_git_clone_real_repo() {
        if !GitOps::is_available() {
            println!("Skipping: git not available");
            return;
        }

        let tmp = TempDir::new().unwrap();
        let dest = tmp.path().join("repo");

        // Clone a small public repo
        let result = GitOps::clone(
            "https://github.com/octocat/Hello-World.git",
            &dest,
            "master",
        );

        assert!(result.is_ok(), "Clone failed: {:?}", result);
        assert!(dest.join(".git").exists());
        assert!(dest.join("README").exists());

        // Check commit info
        let commit = GitOps::current_commit(&dest);
        assert!(commit.is_some());

        let branch = GitOps::current_branch(&dest);
        assert_eq!(branch, Some("master".to_string()));
    }

    #[test]
    #[ignore]
    fn test_git_pull_real_repo() {
        if !GitOps::is_available() {
            println!("Skipping: git not available");
            return;
        }

        let tmp = TempDir::new().unwrap();
        let dest = tmp.path().join("repo");

        // Clone first
        GitOps::clone(
            "https://github.com/octocat/Hello-World.git",
            &dest,
            "master",
        )
        .unwrap();

        // Then pull
        let result = GitOps::pull(&dest, "master");
        assert!(result.is_ok(), "Pull failed: {:?}", result);
    }
}
