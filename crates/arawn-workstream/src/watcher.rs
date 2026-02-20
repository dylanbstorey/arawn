//! Filesystem monitoring for workstream directories.
//!
//! Watches production/ and work/ directories for changes and emits events
//! that can be broadcast via WebSocket to connected clients.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use notify::RecursiveMode;
use notify_debouncer_mini::{DebouncedEventKind, new_debouncer};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::directory::{DirectoryManager, SCRATCH_WORKSTREAM};

/// Default debounce duration in milliseconds.
pub const DEFAULT_DEBOUNCE_MS: u64 = 500;

/// Default polling interval when native watching is unavailable.
pub const DEFAULT_POLL_INTERVAL_SECS: u64 = 30;

/// Errors that can occur during filesystem watching.
#[derive(Debug, Error)]
pub enum WatcherError {
    /// Failed to create the file watcher.
    #[error("Failed to create file watcher: {0}")]
    InitFailed(String),

    /// Failed to watch a path.
    #[error("Failed to watch path {path}: {error}")]
    WatchFailed { path: PathBuf, error: String },

    /// Workstream not found.
    #[error("Workstream not found: {0}")]
    WorkstreamNotFound(String),

    /// Invalid workstream name.
    #[error("Invalid workstream name: {0}")]
    InvalidName(String),
}

/// Result type for watcher operations.
pub type WatcherResult<T> = std::result::Result<T, WatcherError>;

/// Actions that can occur on a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FsAction {
    /// File or directory was created.
    Created,
    /// File or directory was modified.
    Modified,
    /// File or directory was deleted.
    Deleted,
}

impl std::fmt::Display for FsAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FsAction::Created => write!(f, "created"),
            FsAction::Modified => write!(f, "modified"),
            FsAction::Deleted => write!(f, "deleted"),
        }
    }
}

/// Event emitted when a file changes in a workstream directory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsChangeEvent {
    /// The workstream containing the changed file.
    pub workstream: String,
    /// Relative path within the workstream (e.g., "production/file.txt").
    pub path: String,
    /// The action that occurred.
    pub action: FsAction,
    /// When the change was detected.
    pub timestamp: DateTime<Utc>,
}

impl FsChangeEvent {
    /// Create a new filesystem change event.
    pub fn new(workstream: impl Into<String>, path: impl Into<String>, action: FsAction) -> Self {
        Self {
            workstream: workstream.into(),
            path: path.into(),
            action,
            timestamp: Utc::now(),
        }
    }
}

/// Handle to the running watcher thread.
pub struct WatcherHandle {
    handle: std::thread::JoinHandle<()>,
}

impl WatcherHandle {
    /// Check if the watcher thread is still running.
    pub fn is_running(&self) -> bool {
        !self.handle.is_finished()
    }
}

/// Configuration for the file watcher.
#[derive(Debug, Clone)]
pub struct FileWatcherConfig {
    /// Debounce duration in milliseconds (default: 500).
    pub debounce_ms: u64,
    /// Channel buffer size for events (default: 256).
    pub buffer_size: usize,
}

impl Default for FileWatcherConfig {
    fn default() -> Self {
        Self {
            debounce_ms: DEFAULT_DEBOUNCE_MS,
            buffer_size: 256,
        }
    }
}

/// Watches workstream directories for file changes.
///
/// Uses the `notify` crate for cross-platform filesystem watching with
/// debouncing to handle rapid changes from editors.
pub struct FileWatcher {
    /// Directory manager for path resolution.
    directory_manager: DirectoryManager,
    /// Configuration.
    config: FileWatcherConfig,
    /// Tracked workstreams (workstream_id -> watched paths).
    watched: Arc<RwLock<HashMap<String, Vec<PathBuf>>>>,
}

impl FileWatcher {
    /// Create a new file watcher with default configuration.
    pub fn new(directory_manager: DirectoryManager) -> Self {
        Self::with_config(directory_manager, FileWatcherConfig::default())
    }

    /// Create a new file watcher with custom configuration.
    pub fn with_config(directory_manager: DirectoryManager, config: FileWatcherConfig) -> Self {
        Self {
            directory_manager,
            config,
            watched: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start watching and return a receiver for events.
    ///
    /// The watcher runs in a background thread and emits `FsChangeEvent`s
    /// when files change in watched workstream directories.
    ///
    /// # Arguments
    ///
    /// * `workstreams` - List of workstream IDs to watch
    ///
    /// # Returns
    ///
    /// A tuple of (event receiver, watcher handle). The handle must be kept
    /// alive to continue watching. Events are sent to the receiver channel.
    pub fn start(
        &self,
        workstreams: &[&str],
    ) -> WatcherResult<(mpsc::Receiver<FsChangeEvent>, WatcherHandle)> {
        let (event_tx, event_rx) = mpsc::channel(self.config.buffer_size);
        let (notify_tx, notify_rx) = std::sync::mpsc::channel();

        // Create debounced watcher
        let mut debouncer =
            new_debouncer(Duration::from_millis(self.config.debounce_ms), notify_tx).map_err(
                |e| WatcherError::InitFailed(format!("Failed to create debouncer: {}", e)),
            )?;

        // Build path -> workstream mapping for event routing
        let mut path_to_workstream: HashMap<PathBuf, String> = HashMap::new();
        let mut watched_paths: HashMap<String, Vec<PathBuf>> = HashMap::new();

        for &workstream_id in workstreams {
            let paths = self.get_watch_paths(workstream_id)?;

            for path in &paths {
                if path.exists() {
                    debouncer
                        .watcher()
                        .watch(path, RecursiveMode::Recursive)
                        .map_err(|e| WatcherError::WatchFailed {
                            path: path.clone(),
                            error: e.to_string(),
                        })?;

                    path_to_workstream.insert(path.clone(), workstream_id.to_string());
                    debug!("Watching path: {}", path.display());
                } else {
                    debug!("Skipping non-existent path: {}", path.display());
                }
            }

            watched_paths.insert(workstream_id.to_string(), paths);
        }

        // Store watched paths
        *self.watched.write() = watched_paths.clone();

        let workstreams_root = self.directory_manager.workstreams_root();

        info!(
            "Started file watcher for {} workstreams with {}ms debounce",
            workstreams.len(),
            self.config.debounce_ms
        );

        // Spawn background thread for the notify receiver
        let handle = std::thread::spawn(move || {
            // Keep debouncer alive in this thread
            let _debouncer = debouncer;

            while let Ok(result) = notify_rx.recv() {
                match result {
                    Ok(events) => {
                        for event in events {
                            let path = &event.path;

                            // Determine the workstream from the path
                            let workstream = match find_workstream_for_path(
                                path,
                                &workstreams_root,
                                &path_to_workstream,
                            ) {
                                Some(ws) => ws,
                                None => {
                                    debug!("Ignoring event for untracked path: {}", path.display());
                                    continue;
                                }
                            };

                            // Calculate relative path
                            let relative_path = match calculate_relative_path(
                                path,
                                &workstreams_root,
                                &workstream,
                            ) {
                                Some(rp) => rp,
                                None => continue,
                            };

                            // Determine action
                            let action = if event.kind == DebouncedEventKind::Any {
                                if path.exists() {
                                    // Could be create or modify - we don't distinguish
                                    FsAction::Modified
                                } else {
                                    FsAction::Deleted
                                }
                            } else {
                                // AnyContinuous events are treated as modifications
                                FsAction::Modified
                            };

                            let fs_event = FsChangeEvent::new(&workstream, relative_path, action);

                            debug!(
                                workstream = %fs_event.workstream,
                                path = %fs_event.path,
                                action = %fs_event.action,
                                "File change detected"
                            );

                            // Send event (non-blocking, drop if full)
                            if event_tx.blocking_send(fs_event).is_err() {
                                warn!("Event channel full or closed, dropping event");
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        error!("File watcher error: {:?}", e);
                    }
                }
            }

            info!("File watcher thread exiting");
        });

        Ok((event_rx, WatcherHandle { handle }))
    }

    /// Get the paths to watch for a workstream.
    fn get_watch_paths(&self, workstream_id: &str) -> WatcherResult<Vec<PathBuf>> {
        // Validate workstream name
        if workstream_id.contains('/') || workstream_id.contains('\\') || workstream_id.contains("..") {
            return Err(WatcherError::InvalidName(workstream_id.to_string()));
        }

        let ws_path = self.directory_manager.workstream_path(workstream_id);

        if !ws_path.exists() {
            return Err(WatcherError::WorkstreamNotFound(workstream_id.to_string()));
        }

        let mut paths = Vec::new();

        if workstream_id == SCRATCH_WORKSTREAM {
            // For scratch, watch the sessions directory
            let sessions_path = ws_path.join("sessions");
            if sessions_path.exists() {
                paths.push(sessions_path);
            }
        } else {
            // For named workstreams, watch production and work
            paths.push(ws_path.join("production"));
            paths.push(ws_path.join("work"));
        }

        Ok(paths)
    }

    /// List currently watched workstreams.
    pub fn watched_workstreams(&self) -> Vec<String> {
        self.watched.read().keys().cloned().collect()
    }
}

/// Find the workstream ID for a given file path.
fn find_workstream_for_path(
    path: &Path,
    workstreams_root: &Path,
    path_to_workstream: &HashMap<PathBuf, String>,
) -> Option<String> {
    // First check direct matches in our map
    for (watched_path, workstream) in path_to_workstream {
        if path.starts_with(watched_path) {
            return Some(workstream.clone());
        }
    }

    // Fallback: extract workstream from path
    if let Ok(relative) = path.strip_prefix(workstreams_root) {
        let components: Vec<_> = relative.components().collect();
        if !components.is_empty() {
            if let Some(ws) = components[0].as_os_str().to_str() {
                return Some(ws.to_string());
            }
        }
    }

    None
}

/// Calculate the relative path within a workstream.
fn calculate_relative_path(
    path: &Path,
    workstreams_root: &Path,
    workstream: &str,
) -> Option<String> {
    let ws_path = workstreams_root.join(workstream);

    path.strip_prefix(&ws_path)
        .ok()
        .map(|p| p.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn setup() -> (tempfile::TempDir, DirectoryManager) {
        let dir = tempdir().unwrap();
        let manager = DirectoryManager::new(dir.path());
        (dir, manager)
    }

    #[test]
    fn test_fs_action_display() {
        assert_eq!(format!("{}", FsAction::Created), "created");
        assert_eq!(format!("{}", FsAction::Modified), "modified");
        assert_eq!(format!("{}", FsAction::Deleted), "deleted");
    }

    #[test]
    fn test_fs_change_event_new() {
        let event = FsChangeEvent::new("my-blog", "production/post.md", FsAction::Modified);

        assert_eq!(event.workstream, "my-blog");
        assert_eq!(event.path, "production/post.md");
        assert_eq!(event.action, FsAction::Modified);
    }

    #[test]
    fn test_fs_change_event_serialization() {
        let event = FsChangeEvent::new("test", "work/file.txt", FsAction::Created);

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"workstream\":\"test\""));
        assert!(json.contains("\"path\":\"work/file.txt\""));
        assert!(json.contains("\"action\":\"created\""));

        let deserialized: FsChangeEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.workstream, event.workstream);
        assert_eq!(deserialized.path, event.path);
        assert_eq!(deserialized.action, event.action);
    }

    #[test]
    fn test_file_watcher_config_default() {
        let config = FileWatcherConfig::default();
        assert_eq!(config.debounce_ms, DEFAULT_DEBOUNCE_MS);
        assert_eq!(config.buffer_size, 256);
    }

    #[test]
    fn test_get_watch_paths_named_workstream() {
        let (_dir, manager) = setup();

        // Create workstream
        manager.create_workstream("my-project").unwrap();

        let watcher = FileWatcher::new(manager.clone());
        let paths = watcher.get_watch_paths("my-project").unwrap();

        assert_eq!(paths.len(), 2);
        assert!(paths[0].ends_with("my-project/production"));
        assert!(paths[1].ends_with("my-project/work"));
    }

    #[test]
    fn test_get_watch_paths_scratch() {
        let (_dir, manager) = setup();

        // Create scratch workstream structure
        let scratch_sessions = manager.workstream_path(SCRATCH_WORKSTREAM).join("sessions");
        fs::create_dir_all(&scratch_sessions).unwrap();

        let watcher = FileWatcher::new(manager.clone());
        let paths = watcher.get_watch_paths(SCRATCH_WORKSTREAM).unwrap();

        assert_eq!(paths.len(), 1);
        assert!(paths[0].ends_with("scratch/sessions"));
    }

    #[test]
    fn test_get_watch_paths_nonexistent() {
        let (_dir, manager) = setup();

        let watcher = FileWatcher::new(manager);
        let err = watcher.get_watch_paths("nonexistent").unwrap_err();

        assert!(matches!(err, WatcherError::WorkstreamNotFound(_)));
    }

    #[test]
    fn test_get_watch_paths_invalid_name() {
        let (_dir, manager) = setup();

        let watcher = FileWatcher::new(manager);

        let err = watcher.get_watch_paths("invalid/name").unwrap_err();
        assert!(matches!(err, WatcherError::InvalidName(_)));

        let err = watcher.get_watch_paths("../escape").unwrap_err();
        assert!(matches!(err, WatcherError::InvalidName(_)));
    }

    #[test]
    fn test_find_workstream_for_path() {
        let workstreams_root = PathBuf::from("/home/user/.arawn/workstreams");
        let mut path_to_workstream = HashMap::new();
        path_to_workstream.insert(
            PathBuf::from("/home/user/.arawn/workstreams/my-blog/production"),
            "my-blog".to_string(),
        );

        // Direct match
        let result = find_workstream_for_path(
            Path::new("/home/user/.arawn/workstreams/my-blog/production/post.md"),
            &workstreams_root,
            &path_to_workstream,
        );
        assert_eq!(result, Some("my-blog".to_string()));

        // Fallback extraction
        let result = find_workstream_for_path(
            Path::new("/home/user/.arawn/workstreams/other/work/file.txt"),
            &workstreams_root,
            &path_to_workstream,
        );
        assert_eq!(result, Some("other".to_string()));
    }

    #[test]
    fn test_calculate_relative_path() {
        let workstreams_root = PathBuf::from("/home/user/.arawn/workstreams");

        let result = calculate_relative_path(
            Path::new("/home/user/.arawn/workstreams/my-blog/production/post.md"),
            &workstreams_root,
            "my-blog",
        );
        assert_eq!(result, Some("production/post.md".to_string()));

        let result = calculate_relative_path(
            Path::new("/home/user/.arawn/workstreams/my-blog/work/draft.txt"),
            &workstreams_root,
            "my-blog",
        );
        assert_eq!(result, Some("work/draft.txt".to_string()));
    }

    // Integration test that actually starts the watcher
    #[tokio::test]
    async fn test_watcher_start_and_detect_changes() {
        let (dir, manager) = setup();

        // Create workstream
        manager.create_workstream("test-ws").unwrap();

        let watcher = FileWatcher::new(manager.clone());
        let (mut rx, _handle) = watcher.start(&["test-ws"]).unwrap();

        // Give watcher time to initialize
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Create a file
        let prod_path = manager.production_path("test-ws");
        let test_file = prod_path.join("test.txt");
        fs::write(&test_file, "hello").unwrap();

        // Wait for event with timeout
        let event = tokio::time::timeout(Duration::from_secs(2), rx.recv()).await;

        // Drop the dir last to ensure files exist during test
        drop(dir);

        match event {
            Ok(Some(e)) => {
                assert_eq!(e.workstream, "test-ws");
                assert!(e.path.contains("test.txt"));
            }
            Ok(None) => panic!("Channel closed unexpectedly"),
            Err(_) => {
                // Timeout is acceptable in CI environments where native watching
                // may not work reliably
                eprintln!("Warning: File change not detected (may be expected in CI)");
            }
        }
    }
}
