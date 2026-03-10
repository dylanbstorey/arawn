//! Hot-reload file watcher for plugins.
//!
//! Watches plugin directories for changes and triggers per-plugin reloads.
//! Uses debouncing to coalesce rapid file edits (e.g., editor save patterns).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use notify_debouncer_mini::{DebouncedEventKind, new_debouncer};
use tokio::sync::{RwLock, mpsc};

use crate::manager::{LoadedPlugin, MANIFEST_PATH, PluginManager};

/// Event emitted when a plugin is reloaded, added, or removed.
#[derive(Debug, Clone)]
pub enum PluginEvent {
    /// A plugin was loaded or reloaded.
    Reloaded { name: String, plugin_dir: PathBuf },
    /// A plugin was removed (directory deleted).
    Removed { name: String, plugin_dir: PathBuf },
    /// A plugin failed to reload.
    Error { plugin_dir: PathBuf, error: String },
}

/// Shared plugin state that can be read concurrently and swapped on reload.
#[derive(Debug, Default)]
pub struct PluginState {
    /// Plugins keyed by directory path (canonical).
    plugins: HashMap<PathBuf, LoadedPlugin>,
}

impl PluginState {
    /// Get all loaded plugins.
    pub fn plugins(&self) -> Vec<&LoadedPlugin> {
        self.plugins.values().collect()
    }

    /// Get a plugin by its name.
    pub fn get_by_name(&self, name: &str) -> Option<&LoadedPlugin> {
        self.plugins.values().find(|p| p.manifest.name == name)
    }

    /// Get the number of loaded plugins.
    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }
}

/// File watcher that monitors plugin directories and triggers reloads.
pub struct PluginWatcher {
    /// The plugin manager used for loading.
    manager: PluginManager,
    /// Shared plugin state.
    state: Arc<RwLock<PluginState>>,
    /// Debounce duration.
    debounce: Duration,
}

impl PluginWatcher {
    /// Create a new plugin watcher.
    pub fn new(manager: PluginManager) -> Self {
        Self {
            manager,
            state: Arc::new(RwLock::new(PluginState::default())),
            debounce: Duration::from_millis(500),
        }
    }

    /// Set the debounce duration.
    pub fn with_debounce(mut self, duration: Duration) -> Self {
        self.debounce = duration;
        self
    }

    /// Get a reference to the shared plugin state.
    pub fn state(&self) -> Arc<RwLock<PluginState>> {
        self.state.clone()
    }

    /// Perform initial load of all plugins.
    pub async fn load_initial(&self) -> Vec<PluginEvent> {
        let loaded = self.manager.load_all();
        let mut events = Vec::new();
        let mut state = self.state.write().await;

        for plugin in loaded {
            let name = plugin.manifest.name.clone();
            let dir = plugin.plugin_dir.clone();
            state.plugins.insert(dir.clone(), plugin);
            events.push(PluginEvent::Reloaded {
                name,
                plugin_dir: dir,
            });
        }

        events
    }

    /// Reload a single plugin by its directory path.
    pub async fn reload_plugin(&self, plugin_dir: &Path) -> PluginEvent {
        match self.manager.load_single(plugin_dir) {
            Ok(plugin) => {
                let name = plugin.manifest.name.clone();
                let dir = plugin.plugin_dir.clone();
                let mut state = self.state.write().await;
                state.plugins.insert(dir.clone(), plugin);
                tracing::info!(name = %name, dir = %dir.display(), "plugin reloaded");
                PluginEvent::Reloaded {
                    name,
                    plugin_dir: dir,
                }
            }
            Err(e) => {
                tracing::warn!(dir = %plugin_dir.display(), error = %e, "failed to reload plugin");
                PluginEvent::Error {
                    plugin_dir: plugin_dir.to_path_buf(),
                    error: e.to_string(),
                }
            }
        }
    }

    /// Remove a plugin by its directory path.
    pub async fn remove_plugin(&self, plugin_dir: &Path) -> Option<PluginEvent> {
        let mut state = self.state.write().await;
        if let Some(plugin) = state.plugins.remove(plugin_dir) {
            let name = plugin.manifest.name.clone();
            tracing::info!(name = %name, dir = %plugin_dir.display(), "plugin removed");
            Some(PluginEvent::Removed {
                name,
                plugin_dir: plugin_dir.to_path_buf(),
            })
        } else {
            None
        }
    }

    /// Start watching all plugin directories for changes.
    ///
    /// Returns a channel that emits `PluginEvent`s and a handle that keeps
    /// the watcher alive. Drop the handle to stop watching.
    pub fn watch(
        &self,
    ) -> Result<(mpsc::Receiver<PluginEvent>, WatcherHandle), crate::PluginError> {
        let (event_tx, event_rx) = mpsc::channel(64);
        let (notify_tx, notify_rx) = std::sync::mpsc::channel();

        let mut debouncer = new_debouncer(self.debounce, notify_tx).map_err(|e| {
            crate::PluginError::Io(std::io::Error::other(format!("watcher init: {}", e)))
        })?;

        // Watch each plugin directory recursively
        for dir in self.manager.plugin_dirs() {
            if dir.exists()
                && let Err(e) = debouncer
                    .watcher()
                    .watch(dir, notify::RecursiveMode::Recursive)
            {
                tracing::warn!(dir = %dir.display(), error = %e, "failed to watch directory");
            }
        }

        let state = self.state.clone();
        let plugin_dirs: Vec<PathBuf> = self.manager.plugin_dirs().to_vec();

        let handle = std::thread::spawn(move || {
            let _debouncer = debouncer;

            while let Ok(Ok(events)) = notify_rx.recv() {
                // Collect unique plugin directories affected
                let mut affected: HashMap<PathBuf, bool> = HashMap::new();

                for event in &events {
                    if event.kind != DebouncedEventKind::Any {
                        continue;
                    }

                    // Find which plugin directory this file belongs to
                    if let Some(plugin_dir) = find_plugin_dir(&event.path, &plugin_dirs) {
                        let exists = plugin_dir.join(MANIFEST_PATH).exists();
                        affected.insert(plugin_dir, exists);
                    }
                }

                // Process each affected plugin
                for (plugin_dir, manifest_exists) in affected {
                    let state = state.clone();
                    let event_tx = event_tx.clone();

                    if manifest_exists {
                        // Reload
                        let evt = tokio::runtime::Handle::current()
                            .block_on(async { reload_from_dir(&state, &plugin_dir).await });
                        if let Err(e) = event_tx.blocking_send(evt) {
                            tracing::warn!("failed to send plugin event: {e}");
                        }
                    } else {
                        // Plugin directory or manifest removed
                        let evt = tokio::runtime::Handle::current().block_on(async {
                            let mut st = state.write().await;
                            st.plugins.remove(&plugin_dir).map(|p| {
                                let name = p.manifest.name.clone();
                                tracing::info!(name = %name, "plugin removed (directory/manifest gone)");
                                PluginEvent::Removed {
                                    name,
                                    plugin_dir: plugin_dir.clone(),
                                }
                            })
                        });
                        if let Some(evt) = evt
                            && let Err(e) = event_tx.blocking_send(evt)
                        {
                            tracing::warn!("failed to send plugin removal event: {e}");
                        }
                    }
                }
            }
        });

        Ok((event_rx, WatcherHandle { _thread: handle }))
    }
}

/// Find the plugin directory containing a given path.
///
/// A plugin directory is a direct subdirectory of a plugin search directory
/// that contains `.claude-plugin/plugin.json`.
fn find_plugin_dir(path: &Path, plugin_dirs: &[PathBuf]) -> Option<PathBuf> {
    for search_dir in plugin_dirs {
        // Walk up from the changed path to find the plugin root
        let mut candidate = path;
        while let Some(parent) = candidate.parent() {
            if parent == search_dir.as_path() {
                // `candidate` is a direct child of a search directory
                return Some(candidate.to_path_buf());
            }
            candidate = parent;
        }
    }
    None
}

/// Reload a plugin from its directory into the shared state.
async fn reload_from_dir(state: &Arc<RwLock<PluginState>>, plugin_dir: &Path) -> PluginEvent {
    // Use a temporary manager to load the single plugin
    let manager = PluginManager::new(vec![]);
    match manager.load_single(plugin_dir) {
        Ok(plugin) => {
            let name = plugin.manifest.name.clone();
            let dir = plugin.plugin_dir.clone();
            let mut st = state.write().await;
            st.plugins.insert(dir.clone(), plugin);
            tracing::info!(name = %name, "plugin hot-reloaded");
            PluginEvent::Reloaded {
                name,
                plugin_dir: dir,
            }
        }
        Err(e) => {
            tracing::warn!(dir = %plugin_dir.display(), error = %e, "hot-reload failed");
            PluginEvent::Error {
                plugin_dir: plugin_dir.to_path_buf(),
                error: e.to_string(),
            }
        }
    }
}

/// Handle that keeps the file watcher alive. Drop to stop watching.
pub struct WatcherHandle {
    _thread: std::thread::JoinHandle<()>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_plugin(base_dir: &Path, name: &str) -> PathBuf {
        let plugin_dir = base_dir.join(name);
        fs::create_dir_all(plugin_dir.join(".claude-plugin")).unwrap();

        fs::write(
            plugin_dir.join(MANIFEST_PATH),
            format!(
                r#"{{
  "name": "{name}",
  "version": "0.1.0",
  "description": "Test plugin"
}}"#
            ),
        )
        .unwrap();

        plugin_dir
    }

    #[tokio::test]
    async fn test_load_initial() {
        let tmp = TempDir::new().unwrap();
        create_test_plugin(tmp.path(), "alpha");
        create_test_plugin(tmp.path(), "beta");

        let manager = PluginManager::new(vec![tmp.path().to_path_buf()]);
        let watcher = PluginWatcher::new(manager);

        let events = watcher.load_initial().await;
        assert_eq!(events.len(), 2);

        let state = watcher.state();
        let st = state.read().await;
        assert_eq!(st.len(), 2);
        assert!(st.get_by_name("alpha").is_some());
        assert!(st.get_by_name("beta").is_some());
    }

    #[tokio::test]
    async fn test_reload_plugin() {
        let tmp = TempDir::new().unwrap();
        let plugin_dir = create_test_plugin(tmp.path(), "reloadme");

        let manager = PluginManager::new(vec![tmp.path().to_path_buf()]);
        let watcher = PluginWatcher::new(manager);
        watcher.load_initial().await;

        // Modify and reload
        fs::write(
            plugin_dir.join(MANIFEST_PATH),
            r#"{
  "name": "reloadme",
  "version": "0.2.0",
  "description": "Updated"
}"#,
        )
        .unwrap();

        let event = watcher.reload_plugin(&plugin_dir).await;
        assert!(matches!(event, PluginEvent::Reloaded { ref name, .. } if name == "reloadme"));

        let state = watcher.state();
        let st = state.read().await;
        let p = st.get_by_name("reloadme").unwrap();
        assert_eq!(p.manifest.version.as_deref(), Some("0.2.0"));
    }

    #[tokio::test]
    async fn test_remove_plugin() {
        let tmp = TempDir::new().unwrap();
        create_test_plugin(tmp.path(), "removeme");

        let manager = PluginManager::new(vec![tmp.path().to_path_buf()]);
        let watcher = PluginWatcher::new(manager);
        watcher.load_initial().await;

        let dir = tmp.path().join("removeme");
        let event = watcher.remove_plugin(&dir).await;
        assert!(matches!(event, Some(PluginEvent::Removed { ref name, .. }) if name == "removeme"));

        let state = watcher.state();
        let st = state.read().await;
        assert!(st.is_empty());
    }

    #[tokio::test]
    async fn test_remove_nonexistent() {
        let tmp = TempDir::new().unwrap();
        let manager = PluginManager::new(vec![tmp.path().to_path_buf()]);
        let watcher = PluginWatcher::new(manager);

        let event = watcher.remove_plugin(Path::new("/nonexistent")).await;
        assert!(event.is_none());
    }

    #[tokio::test]
    async fn test_reload_invalid_plugin() {
        let tmp = TempDir::new().unwrap();
        let plugin_dir = tmp.path().join("bad");
        fs::create_dir_all(plugin_dir.join(".claude-plugin")).unwrap();
        fs::write(plugin_dir.join(MANIFEST_PATH), "not valid {{{").unwrap();

        let manager = PluginManager::new(vec![tmp.path().to_path_buf()]);
        let watcher = PluginWatcher::new(manager);

        let event = watcher.reload_plugin(&plugin_dir).await;
        assert!(matches!(event, PluginEvent::Error { .. }));
    }

    #[tokio::test]
    async fn test_state_get_by_name() {
        let tmp = TempDir::new().unwrap();
        create_test_plugin(tmp.path(), "findme");

        let manager = PluginManager::new(vec![tmp.path().to_path_buf()]);
        let watcher = PluginWatcher::new(manager);
        watcher.load_initial().await;

        let state = watcher.state();
        let st = state.read().await;
        assert!(st.get_by_name("findme").is_some());
        assert!(st.get_by_name("nothere").is_none());
    }

    #[test]
    fn test_find_plugin_dir() {
        let search = PathBuf::from("/plugins");
        let dirs = vec![search];

        assert_eq!(
            find_plugin_dir(Path::new("/plugins/my-plugin/skills/test.md"), &dirs),
            Some(PathBuf::from("/plugins/my-plugin"))
        );

        assert_eq!(
            find_plugin_dir(Path::new("/plugins/my-plugin/plugin.toml"), &dirs),
            Some(PathBuf::from("/plugins/my-plugin"))
        );

        assert_eq!(
            find_plugin_dir(Path::new("/other/path/file.txt"), &dirs),
            None
        );
    }

    #[test]
    fn test_debounce_config() {
        let manager = PluginManager::new(vec![]);
        let watcher = PluginWatcher::new(manager).with_debounce(Duration::from_millis(100));
        assert_eq!(watcher.debounce, Duration::from_millis(100));
    }

    // ─────────────────────────────────────────────────────────────────────
    // Hot-reload state preservation tests
    // ─────────────────────────────────────────────────────────────────────

    /// Create a plugin with a skill file for reload testing.
    fn create_plugin_with_skill(base_dir: &Path, name: &str, skill_body: &str) -> PathBuf {
        let plugin_dir = base_dir.join(name);
        fs::create_dir_all(plugin_dir.join(".claude-plugin")).unwrap();
        fs::create_dir_all(plugin_dir.join("skills").join("my-skill")).unwrap();

        fs::write(
            plugin_dir.join(MANIFEST_PATH),
            format!(
                r#"{{
  "name": "{name}",
  "version": "1.0.0",
  "description": "Test plugin",
  "skills": "./skills/"
}}"#
            ),
        )
        .unwrap();

        fs::write(
            plugin_dir.join("skills/my-skill/SKILL.md"),
            format!("---\nname: my-skill\ndescription: A skill\n---\n\n{skill_body}"),
        )
        .unwrap();

        plugin_dir
    }

    #[tokio::test]
    async fn test_reload_updates_skill_content() {
        let tmp = TempDir::new().unwrap();
        let plugin_dir = create_plugin_with_skill(tmp.path(), "myplug", "Version 1 content");

        let manager = PluginManager::new(vec![tmp.path().to_path_buf()]);
        let watcher = PluginWatcher::new(manager);
        watcher.load_initial().await;

        // Verify initial content
        {
            let state = watcher.state();
            let st = state.read().await;
            let p = st.get_by_name("myplug").unwrap();
            assert_eq!(p.skill_contents.len(), 1);
            assert!(p.skill_contents[0].content.contains("Version 1 content"));
        }

        // Modify skill content on disk
        fs::write(
            plugin_dir.join("skills/my-skill/SKILL.md"),
            "---\nname: my-skill\ndescription: Updated\n---\n\nVersion 2 content",
        )
        .unwrap();

        // Reload
        let event = watcher.reload_plugin(&plugin_dir).await;
        assert!(matches!(event, PluginEvent::Reloaded { .. }));

        // Verify updated content
        let state = watcher.state();
        let st = state.read().await;
        let p = st.get_by_name("myplug").unwrap();
        assert!(p.skill_contents[0].content.contains("Version 2 content"));
        assert!(!p.skill_contents[0].content.contains("Version 1 content"));
    }

    #[tokio::test]
    async fn test_reload_updates_version() {
        let tmp = TempDir::new().unwrap();
        let plugin_dir = create_test_plugin(tmp.path(), "vtest");

        let manager = PluginManager::new(vec![tmp.path().to_path_buf()]);
        let watcher = PluginWatcher::new(manager);
        watcher.load_initial().await;

        {
            let state = watcher.state();
            let st = state.read().await;
            assert_eq!(
                st.get_by_name("vtest").unwrap().manifest.version.as_deref(),
                Some("0.1.0")
            );
        }

        // Update version on disk
        fs::write(
            plugin_dir.join(MANIFEST_PATH),
            r#"{ "name": "vtest", "version": "2.0.0", "description": "Updated" }"#,
        )
        .unwrap();

        watcher.reload_plugin(&plugin_dir).await;

        let state = watcher.state();
        let st = state.read().await;
        assert_eq!(
            st.get_by_name("vtest").unwrap().manifest.version.as_deref(),
            Some("2.0.0")
        );
    }

    #[tokio::test]
    async fn test_reload_added_skill_appears() {
        let tmp = TempDir::new().unwrap();
        let plugin_dir = create_test_plugin(tmp.path(), "addskill");

        let manager = PluginManager::new(vec![tmp.path().to_path_buf()]);
        let watcher = PluginWatcher::new(manager);
        watcher.load_initial().await;

        // Initially no skills (create_test_plugin doesn't add skills dir)
        {
            let state = watcher.state();
            let st = state.read().await;
            let p = st.get_by_name("addskill").unwrap();
            assert_eq!(p.skill_contents.len(), 0);
        }

        // Add a skill on disk and update manifest
        fs::write(
            plugin_dir.join(MANIFEST_PATH),
            r#"{ "name": "addskill", "version": "0.1.0", "skills": "./skills/" }"#,
        )
        .unwrap();
        fs::create_dir_all(plugin_dir.join("skills/new-skill")).unwrap();
        fs::write(
            plugin_dir.join("skills/new-skill/SKILL.md"),
            "---\nname: new-skill\ndescription: Brand new\n---\n\nNew skill body.",
        )
        .unwrap();

        watcher.reload_plugin(&plugin_dir).await;

        let state = watcher.state();
        let st = state.read().await;
        let p = st.get_by_name("addskill").unwrap();
        assert_eq!(p.skill_contents.len(), 1);
        assert_eq!(p.skill_contents[0].def.name, "new-skill");
    }

    #[tokio::test]
    async fn test_remove_and_readd_plugin() {
        let tmp = TempDir::new().unwrap();
        let plugin_dir = create_test_plugin(tmp.path(), "bounce");

        let manager = PluginManager::new(vec![tmp.path().to_path_buf()]);
        let watcher = PluginWatcher::new(manager);
        watcher.load_initial().await;

        assert_eq!(watcher.state().read().await.len(), 1);

        // Remove
        watcher.remove_plugin(&plugin_dir).await;
        assert_eq!(watcher.state().read().await.len(), 0);

        // Re-add via reload
        let event = watcher.reload_plugin(&plugin_dir).await;
        assert!(matches!(event, PluginEvent::Reloaded { .. }));
        assert_eq!(watcher.state().read().await.len(), 1);
    }

    #[tokio::test]
    async fn test_reload_other_plugins_unaffected() {
        let tmp = TempDir::new().unwrap();
        let _dir_a = create_plugin_with_skill(tmp.path(), "alpha", "Alpha content");
        let dir_b = create_plugin_with_skill(tmp.path(), "beta", "Beta content");

        let manager = PluginManager::new(vec![tmp.path().to_path_buf()]);
        let watcher = PluginWatcher::new(manager);
        watcher.load_initial().await;

        // Reload only beta
        fs::write(
            dir_b.join("skills/my-skill/SKILL.md"),
            "---\nname: my-skill\ndescription: Updated\n---\n\nBeta v2 content",
        )
        .unwrap();
        watcher.reload_plugin(&dir_b).await;

        let state = watcher.state();
        let st = state.read().await;

        // Alpha should be unchanged
        let alpha = st.get_by_name("alpha").unwrap();
        assert!(alpha.skill_contents[0].content.contains("Alpha content"));

        // Beta should be updated
        let beta = st.get_by_name("beta").unwrap();
        assert!(beta.skill_contents[0].content.contains("Beta v2 content"));
    }

    #[tokio::test]
    async fn test_concurrent_read_during_reload() {
        let tmp = TempDir::new().unwrap();
        let plugin_dir = create_plugin_with_skill(tmp.path(), "concurrent", "Initial");

        let manager = PluginManager::new(vec![tmp.path().to_path_buf()]);
        let watcher = PluginWatcher::new(manager);
        watcher.load_initial().await;

        let state = watcher.state();

        // Simulate an "active session" reading state
        let state_clone = state.clone();
        let reader = tokio::spawn(async move {
            // Read state multiple times — should never panic
            for _ in 0..10 {
                {
                    let st = state_clone.read().await;
                    let _ = st.plugins();
                    let _ = st.get_by_name("concurrent");
                }
                tokio::task::yield_now().await;
            }
        });

        // Simultaneously reload the plugin
        fs::write(
            plugin_dir.join("skills/my-skill/SKILL.md"),
            "---\nname: my-skill\ndescription: Updated\n---\n\nReloaded content",
        )
        .unwrap();
        watcher.reload_plugin(&plugin_dir).await;

        // Reader should complete without panicking
        reader.await.unwrap();

        // State should be consistent after concurrent operations
        let state_ref = state.clone();
        let st = state_ref.read().await;
        let p = st.get_by_name("concurrent").unwrap();
        assert!(p.skill_contents[0].content.contains("Reloaded content"));
    }

    // ── PluginState Tests ──────────────────────────────────────────────

    #[test]
    fn test_plugin_state_default() {
        let state = PluginState::default();
        assert!(state.is_empty());
        assert_eq!(state.len(), 0);
        assert!(state.plugins().is_empty());
        assert!(state.get_by_name("anything").is_none());
    }

    #[tokio::test]
    async fn test_plugin_state_plugins_returns_all() {
        let tmp = TempDir::new().unwrap();
        create_test_plugin(tmp.path(), "p1");
        create_test_plugin(tmp.path(), "p2");
        create_test_plugin(tmp.path(), "p3");

        let manager = PluginManager::new(vec![tmp.path().to_path_buf()]);
        let watcher = PluginWatcher::new(manager);
        watcher.load_initial().await;

        let state = watcher.state();
        let st = state.read().await;
        let plugins = st.plugins();
        assert_eq!(plugins.len(), 3);

        let names: Vec<&str> = plugins.iter().map(|p| p.manifest.name.as_str()).collect();
        assert!(names.contains(&"p1"));
        assert!(names.contains(&"p2"));
        assert!(names.contains(&"p3"));
    }

    // ── find_plugin_dir Tests ──────────────────────────────────────────

    #[test]
    fn test_find_plugin_dir_multiple_search_dirs() {
        let dirs = vec![PathBuf::from("/plugins"), PathBuf::from("/extra-plugins")];

        // File in first search dir
        assert_eq!(
            find_plugin_dir(Path::new("/plugins/my-plugin/file.txt"), &dirs),
            Some(PathBuf::from("/plugins/my-plugin"))
        );

        // File in second search dir
        assert_eq!(
            find_plugin_dir(Path::new("/extra-plugins/other/deep/file.txt"), &dirs),
            Some(PathBuf::from("/extra-plugins/other"))
        );

        // File not under any search dir
        assert_eq!(
            find_plugin_dir(Path::new("/somewhere/else/file.txt"), &dirs),
            None
        );
    }

    #[test]
    fn test_find_plugin_dir_deeply_nested_file() {
        let dirs = vec![PathBuf::from("/plugins")];

        // Deeply nested file should still resolve to direct child of search dir
        assert_eq!(
            find_plugin_dir(
                Path::new("/plugins/my-plugin/src/deep/nested/module.rs"),
                &dirs
            ),
            Some(PathBuf::from("/plugins/my-plugin"))
        );
    }

    #[test]
    fn test_find_plugin_dir_empty_search_dirs() {
        let dirs: Vec<PathBuf> = vec![];
        assert_eq!(
            find_plugin_dir(Path::new("/anything/file.txt"), &dirs),
            None
        );
    }

    #[test]
    fn test_find_plugin_dir_file_at_search_root() {
        let dirs = vec![PathBuf::from("/plugins")];
        // File directly under the search dir (not inside a plugin subdir)
        // The parent of "/plugins/file.txt" is "/plugins" which matches search_dir,
        // so it returns "/plugins/file.txt" as the "plugin dir"
        assert_eq!(
            find_plugin_dir(Path::new("/plugins/file.txt"), &dirs),
            Some(PathBuf::from("/plugins/file.txt"))
        );
    }

    // ── reload_from_dir Tests ──────────────────────────────────────────

    #[tokio::test]
    async fn test_reload_from_dir_success() {
        let tmp = TempDir::new().unwrap();
        let plugin_dir = create_test_plugin(tmp.path(), "direct-reload");

        let state = Arc::new(RwLock::new(PluginState::default()));
        let event = reload_from_dir(&state, &plugin_dir).await;

        match event {
            PluginEvent::Reloaded {
                name,
                plugin_dir: dir,
            } => {
                assert_eq!(name, "direct-reload");
                assert_eq!(dir, plugin_dir);
            }
            other => panic!("Expected Reloaded, got {:?}", other),
        }

        let st = state.read().await;
        assert_eq!(st.len(), 1);
        assert!(st.get_by_name("direct-reload").is_some());
    }

    #[tokio::test]
    async fn test_reload_from_dir_failure() {
        let tmp = TempDir::new().unwrap();
        let bad_dir = tmp.path().join("bad-plugin");
        fs::create_dir_all(bad_dir.join(".claude-plugin")).unwrap();
        fs::write(bad_dir.join(MANIFEST_PATH), "invalid json{{{").unwrap();

        let state = Arc::new(RwLock::new(PluginState::default()));
        let event = reload_from_dir(&state, &bad_dir).await;

        match event {
            PluginEvent::Error { plugin_dir, error } => {
                assert_eq!(plugin_dir, bad_dir);
                assert!(!error.is_empty());
            }
            other => panic!("Expected Error, got {:?}", other),
        }

        // State should remain empty
        assert!(state.read().await.is_empty());
    }

    #[tokio::test]
    async fn test_reload_from_dir_replaces_existing() {
        let tmp = TempDir::new().unwrap();
        let plugin_dir = create_test_plugin(tmp.path(), "replace-me");

        let state = Arc::new(RwLock::new(PluginState::default()));

        // Load initially
        reload_from_dir(&state, &plugin_dir).await;
        assert_eq!(
            state
                .read()
                .await
                .get_by_name("replace-me")
                .unwrap()
                .manifest
                .version
                .as_deref(),
            Some("0.1.0")
        );

        // Update manifest
        fs::write(
            plugin_dir.join(MANIFEST_PATH),
            r#"{ "name": "replace-me", "version": "3.0.0", "description": "Upgraded" }"#,
        )
        .unwrap();

        // Reload should replace
        reload_from_dir(&state, &plugin_dir).await;
        assert_eq!(
            state
                .read()
                .await
                .get_by_name("replace-me")
                .unwrap()
                .manifest
                .version
                .as_deref(),
            Some("3.0.0")
        );

        // Should still be just one plugin
        assert_eq!(state.read().await.len(), 1);
    }

    // ── PluginEvent Variant Tests ──────────────────────────────────────

    #[test]
    fn test_plugin_event_debug() {
        let event = PluginEvent::Reloaded {
            name: "test".to_string(),
            plugin_dir: PathBuf::from("/plugins/test"),
        };
        let debug = format!("{:?}", event);
        assert!(debug.contains("Reloaded"));
        assert!(debug.contains("test"));

        let event = PluginEvent::Removed {
            name: "gone".to_string(),
            plugin_dir: PathBuf::from("/plugins/gone"),
        };
        let debug = format!("{:?}", event);
        assert!(debug.contains("Removed"));

        let event = PluginEvent::Error {
            plugin_dir: PathBuf::from("/plugins/bad"),
            error: "parse error".to_string(),
        };
        let debug = format!("{:?}", event);
        assert!(debug.contains("Error"));
        assert!(debug.contains("parse error"));
    }

    // ── Watch Setup Test ───────────────────────────────────────────────

    #[tokio::test]
    async fn test_watch_creates_receiver_and_handle() {
        let tmp = TempDir::new().unwrap();
        create_test_plugin(tmp.path(), "watchable");

        let manager = PluginManager::new(vec![tmp.path().to_path_buf()]);
        let watcher = PluginWatcher::new(manager).with_debounce(Duration::from_millis(100));
        watcher.load_initial().await;

        let result = watcher.watch();
        assert!(result.is_ok());

        let (_rx, _handle) = result.unwrap();
        // Handle keeps watcher alive; dropping it stops watching
    }

    #[tokio::test]
    async fn test_watch_with_nonexistent_dir() {
        let manager = PluginManager::new(vec![PathBuf::from("/nonexistent/plugins")]);
        let watcher = PluginWatcher::new(manager);

        // watch() should succeed even if dirs don't exist (warns but continues)
        let result = watcher.watch();
        assert!(result.is_ok());
    }

    // ── Load Initial No Plugins ────────────────────────────────────────

    #[tokio::test]
    async fn test_load_initial_empty_dir() {
        let tmp = TempDir::new().unwrap();
        let manager = PluginManager::new(vec![tmp.path().to_path_buf()]);
        let watcher = PluginWatcher::new(manager);

        let events = watcher.load_initial().await;
        assert!(events.is_empty());

        let state = watcher.state();
        let st = state.read().await;
        assert!(st.is_empty());
    }

    #[tokio::test]
    async fn test_load_initial_returns_events_for_each_plugin() {
        let tmp = TempDir::new().unwrap();
        create_test_plugin(tmp.path(), "a");
        create_test_plugin(tmp.path(), "b");

        let manager = PluginManager::new(vec![tmp.path().to_path_buf()]);
        let watcher = PluginWatcher::new(manager);

        let events = watcher.load_initial().await;
        assert_eq!(events.len(), 2);

        let names: Vec<&str> = events
            .iter()
            .filter_map(|e| match e {
                PluginEvent::Reloaded { name, .. } => Some(name.as_str()),
                _ => None,
            })
            .collect();
        assert!(names.contains(&"a"));
        assert!(names.contains(&"b"));
    }
}
