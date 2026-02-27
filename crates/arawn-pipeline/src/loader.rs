//! Hot-reload workflow file watcher and loader.
//!
//! Manages a directory of workflow TOML files, loading them on startup and
//! watching for changes at runtime. New, modified, or deleted files are
//! picked up without restarting the server.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use notify_debouncer_mini::{DebouncedEventKind, new_debouncer};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::definition::WorkflowFile;
use crate::error::PipelineError;

/// Event emitted when workflow files change.
#[derive(Debug, Clone)]
pub enum WorkflowEvent {
    /// A workflow was loaded or updated.
    Loaded { name: String, path: PathBuf },
    /// A workflow was removed.
    Removed { name: String, path: PathBuf },
    /// A workflow file failed to parse.
    Error { path: PathBuf, error: String },
}

/// In-memory cache of loaded workflow definitions.
// TODO(ARAWN-T-0230): `path` field and `remove_file()` are scaffolding for hot-reload file watcher.
#[derive(Debug, Clone)]
struct LoadedWorkflow {
    definition: crate::definition::WorkflowDefinition,
    path: PathBuf,
}

/// Manages loading and hot-reloading of workflow TOML files from a directory.
pub struct WorkflowLoader {
    /// Directory containing workflow TOML files.
    workflow_dir: PathBuf,
    /// Loaded workflows keyed by workflow name.
    workflows: Arc<RwLock<HashMap<String, LoadedWorkflow>>>,
    /// Reverse map: file path → workflow name (for delete handling).
    path_to_name: Arc<RwLock<HashMap<PathBuf, String>>>,
}

impl WorkflowLoader {
    /// Create a new loader for the given workflow directory.
    ///
    /// The directory will be created if it doesn't exist.
    pub fn new(workflow_dir: impl Into<PathBuf>) -> Result<Self, PipelineError> {
        let workflow_dir = workflow_dir.into();

        if !workflow_dir.exists() {
            std::fs::create_dir_all(&workflow_dir).map_err(|e| {
                PipelineError::InitFailed(format!(
                    "Failed to create workflow directory {}: {}",
                    workflow_dir.display(),
                    e
                ))
            })?;
        }

        Ok(Self {
            workflow_dir,
            workflows: Arc::new(RwLock::new(HashMap::new())),
            path_to_name: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Load all TOML workflow files from the directory.
    ///
    /// Returns events for each file processed (loaded or errored).
    /// Invalid files are logged but don't prevent other files from loading.
    pub async fn load_all(&self) -> Vec<WorkflowEvent> {
        let mut events = Vec::new();

        let entries = match std::fs::read_dir(&self.workflow_dir) {
            Ok(entries) => entries,
            Err(e) => {
                error!(
                    "Failed to read workflow directory {}: {}",
                    self.workflow_dir.display(),
                    e
                );
                return events;
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if Self::is_workflow_file(&path) {
                let event = self.load_file(&path).await;
                events.push(event);
            }
        }

        let workflows = self.workflows.read().await;
        info!(
            "Loaded {} workflows from {}",
            workflows.len(),
            self.workflow_dir.display()
        );

        events
    }

    /// Load or reload a single workflow file.
    async fn load_file(&self, path: &Path) -> WorkflowEvent {
        debug!("Loading workflow file: {}", path.display());

        let wf_file = match WorkflowFile::from_file(path) {
            Ok(wf) => wf,
            Err(e) => {
                warn!("Failed to parse {}: {}", path.display(), e);
                return WorkflowEvent::Error {
                    path: path.to_path_buf(),
                    error: e.to_string(),
                };
            }
        };

        if let Err(e) = wf_file.workflow.validate() {
            warn!("Invalid workflow in {}: {}", path.display(), e);
            return WorkflowEvent::Error {
                path: path.to_path_buf(),
                error: e.to_string(),
            };
        }

        let name = wf_file.workflow.name.clone();

        // Store in cache
        let mut workflows = self.workflows.write().await;
        let mut path_to_name = self.path_to_name.write().await;

        workflows.insert(
            name.clone(),
            LoadedWorkflow {
                definition: wf_file.workflow,
                path: path.to_path_buf(),
            },
        );
        path_to_name.insert(path.to_path_buf(), name.clone());

        info!("Workflow loaded: {} (from {})", name, path.display());

        WorkflowEvent::Loaded {
            name,
            path: path.to_path_buf(),
        }
    }

    /// Handle a file being removed.
    async fn remove_file(&self, path: &Path) -> Option<WorkflowEvent> {
        let mut path_to_name = self.path_to_name.write().await;
        let mut workflows = self.workflows.write().await;

        if let Some(name) = path_to_name.remove(path) {
            workflows.remove(&name);
            info!("Workflow removed: {} (was {})", name, path.display());
            Some(WorkflowEvent::Removed {
                name,
                path: path.to_path_buf(),
            })
        } else {
            None
        }
    }

    /// Get a loaded workflow definition by name.
    pub async fn get(&self, name: &str) -> Option<crate::definition::WorkflowDefinition> {
        self.workflows
            .read()
            .await
            .get(name)
            .map(|lw| lw.definition.clone())
    }

    /// List all loaded workflow names.
    pub async fn list_names(&self) -> Vec<String> {
        self.workflows.read().await.keys().cloned().collect()
    }

    /// Get the number of loaded workflows.
    pub async fn len(&self) -> usize {
        self.workflows.read().await.len()
    }

    /// Check if any workflows are loaded.
    pub async fn is_empty(&self) -> bool {
        self.workflows.read().await.is_empty()
    }

    /// Start watching the workflow directory for changes.
    ///
    /// Returns a channel receiver that emits `WorkflowEvent`s when files change.
    /// The watcher runs in a background task and should be kept alive for the
    /// duration of the server.
    ///
    /// Debounces events with a 300ms window to handle editor save patterns.
    pub fn watch(
        &self,
    ) -> Result<(tokio::sync::mpsc::Receiver<WorkflowEvent>, WatcherHandle), PipelineError> {
        let (event_tx, event_rx) = tokio::sync::mpsc::channel(64);
        let (notify_tx, notify_rx) = std::sync::mpsc::channel();

        let mut debouncer = new_debouncer(Duration::from_millis(300), notify_tx).map_err(|e| {
            PipelineError::InitFailed(format!("Failed to create file watcher: {}", e))
        })?;

        debouncer
            .watcher()
            .watch(&self.workflow_dir, notify::RecursiveMode::NonRecursive)
            .map_err(|e| {
                PipelineError::InitFailed(format!(
                    "Failed to watch {}: {}",
                    self.workflow_dir.display(),
                    e
                ))
            })?;

        let workflows = self.workflows.clone();
        let path_to_name = self.path_to_name.clone();
        let workflow_dir = self.workflow_dir.clone();

        // Spawn blocking thread for the notify receiver (it's std::sync)
        let handle = std::thread::spawn(move || {
            // Keep debouncer alive in this thread
            let _debouncer = debouncer;

            while let Ok(Ok(events)) = notify_rx.recv() {
                for event in events {
                    let path = event.path;

                    // Skip non-TOML files
                    if !Self::is_workflow_file(&path) {
                        continue;
                    }

                    // Skip if path is outside our workflow dir
                    if !path.starts_with(&workflow_dir) {
                        continue;
                    }

                    let workflows = workflows.clone();
                    let path_to_name = path_to_name.clone();
                    let event_tx = event_tx.clone();

                    if event.kind == DebouncedEventKind::Any {
                        if path.exists() {
                            // File created or modified — load/reload
                            let loader_event = tokio::runtime::Handle::current().block_on(async {
                                let loader = WorkflowLoaderView {
                                    workflows,
                                    path_to_name,
                                };
                                loader.load_file(&path).await
                            });
                            if let Err(e) = event_tx.blocking_send(loader_event) {
                                tracing::warn!("Failed to send workflow event: {e}");
                            }
                        } else {
                            // File deleted
                            let maybe_event = tokio::runtime::Handle::current().block_on(async {
                                let loader = WorkflowLoaderView {
                                    workflows,
                                    path_to_name,
                                };
                                loader.remove_file(&path).await
                            });
                            if let Some(evt) = maybe_event {
                                if let Err(e) = event_tx.blocking_send(evt) {
                                    tracing::warn!("Failed to send workflow removal event: {e}");
                                }
                            }
                        }
                    }
                }
            }
        });

        Ok((event_rx, WatcherHandle { _thread: handle }))
    }

    /// Check if a path is a workflow TOML file.
    fn is_workflow_file(path: &Path) -> bool {
        path.extension().map(|ext| ext == "toml").unwrap_or(false)
    }
}

/// Internal view used by the watcher thread to update workflow state.
struct WorkflowLoaderView {
    workflows: Arc<RwLock<HashMap<String, LoadedWorkflow>>>,
    path_to_name: Arc<RwLock<HashMap<PathBuf, String>>>,
}

impl WorkflowLoaderView {
    async fn load_file(&self, path: &Path) -> WorkflowEvent {
        let wf_file = match WorkflowFile::from_file(path) {
            Ok(wf) => wf,
            Err(e) => {
                warn!("Failed to parse {}: {}", path.display(), e);
                return WorkflowEvent::Error {
                    path: path.to_path_buf(),
                    error: e.to_string(),
                };
            }
        };

        if let Err(e) = wf_file.workflow.validate() {
            warn!("Invalid workflow in {}: {}", path.display(), e);
            return WorkflowEvent::Error {
                path: path.to_path_buf(),
                error: e.to_string(),
            };
        }

        let name = wf_file.workflow.name.clone();

        let mut workflows = self.workflows.write().await;
        let mut path_to_name = self.path_to_name.write().await;

        workflows.insert(
            name.clone(),
            LoadedWorkflow {
                definition: wf_file.workflow,
                path: path.to_path_buf(),
            },
        );
        path_to_name.insert(path.to_path_buf(), name.clone());

        info!("Workflow reloaded: {} (from {})", name, path.display());

        WorkflowEvent::Loaded {
            name,
            path: path.to_path_buf(),
        }
    }

    async fn remove_file(&self, path: &Path) -> Option<WorkflowEvent> {
        let mut path_to_name = self.path_to_name.write().await;
        let mut workflows = self.workflows.write().await;

        if let Some(name) = path_to_name.remove(path) {
            workflows.remove(&name);
            info!("Workflow removed: {} (was {})", name, path.display());
            Some(WorkflowEvent::Removed {
                name,
                path: path.to_path_buf(),
            })
        } else {
            None
        }
    }
}

/// Handle that keeps the file watcher alive.
///
/// Drop this to stop watching.
pub struct WatcherHandle {
    _thread: std::thread::JoinHandle<()>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_workflow(dir: &Path, filename: &str, name: &str) {
        let content = format!(
            r#"
[workflow]
name = "{name}"
description = "Test workflow"

[[workflow.tasks]]
id = "task1"
action = {{ type = "tool", name = "echo" }}
"#,
        );
        std::fs::write(dir.join(filename), content).unwrap();
    }

    fn write_invalid(dir: &Path, filename: &str) {
        std::fs::write(dir.join(filename), "this is not valid toml {{{").unwrap();
    }

    #[tokio::test]
    async fn test_load_empty_directory() {
        let dir = tempfile::tempdir().unwrap();
        let loader = WorkflowLoader::new(dir.path()).unwrap();
        let events = loader.load_all().await;
        assert!(events.is_empty());
        assert!(loader.is_empty().await);
    }

    #[tokio::test]
    async fn test_load_single_workflow() {
        let dir = tempfile::tempdir().unwrap();
        write_workflow(dir.path(), "test.toml", "test_wf");

        let loader = WorkflowLoader::new(dir.path()).unwrap();
        let events = loader.load_all().await;

        assert_eq!(events.len(), 1);
        assert!(matches!(&events[0], WorkflowEvent::Loaded { name, .. } if name == "test_wf"));
        assert_eq!(loader.len().await, 1);

        let wf = loader.get("test_wf").await.unwrap();
        assert_eq!(wf.name, "test_wf");
    }

    #[tokio::test]
    async fn test_load_multiple_workflows() {
        let dir = tempfile::tempdir().unwrap();
        write_workflow(dir.path(), "a.toml", "workflow_a");
        write_workflow(dir.path(), "b.toml", "workflow_b");
        write_workflow(dir.path(), "c.toml", "workflow_c");

        let loader = WorkflowLoader::new(dir.path()).unwrap();
        loader.load_all().await;

        assert_eq!(loader.len().await, 3);
        let names = loader.list_names().await;
        assert!(names.contains(&"workflow_a".to_string()));
        assert!(names.contains(&"workflow_b".to_string()));
        assert!(names.contains(&"workflow_c".to_string()));
    }

    #[tokio::test]
    async fn test_invalid_file_doesnt_crash() {
        let dir = tempfile::tempdir().unwrap();
        write_workflow(dir.path(), "good.toml", "good");
        write_invalid(dir.path(), "bad.toml");

        let loader = WorkflowLoader::new(dir.path()).unwrap();
        let events = loader.load_all().await;

        // Should have 2 events: 1 loaded, 1 error
        assert_eq!(events.len(), 2);
        assert_eq!(loader.len().await, 1);
        assert!(loader.get("good").await.is_some());

        let error_events: Vec<_> = events
            .iter()
            .filter(|e| matches!(e, WorkflowEvent::Error { .. }))
            .collect();
        assert_eq!(error_events.len(), 1);
    }

    #[tokio::test]
    async fn test_skips_non_toml_files() {
        let dir = tempfile::tempdir().unwrap();
        write_workflow(dir.path(), "real.toml", "real");
        std::fs::write(dir.path().join("readme.md"), "# not a workflow").unwrap();
        std::fs::write(dir.path().join("data.json"), "{}").unwrap();

        let loader = WorkflowLoader::new(dir.path()).unwrap();
        let events = loader.load_all().await;

        assert_eq!(events.len(), 1);
        assert_eq!(loader.len().await, 1);
    }

    #[tokio::test]
    async fn test_creates_directory_if_missing() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("workflows").join("nested");

        let loader = WorkflowLoader::new(&nested).unwrap();
        assert!(nested.exists());
        assert!(loader.is_empty().await);
    }

    #[tokio::test]
    async fn test_reload_modified_file() {
        let dir = tempfile::tempdir().unwrap();
        write_workflow(dir.path(), "wf.toml", "original");

        let loader = WorkflowLoader::new(dir.path()).unwrap();
        loader.load_all().await;
        assert!(loader.get("original").await.is_some());

        // Overwrite with new name
        write_workflow(dir.path(), "wf.toml", "updated");
        let event = loader.load_file(&dir.path().join("wf.toml")).await;

        assert!(matches!(event, WorkflowEvent::Loaded { name, .. } if name == "updated"));
        assert!(loader.get("updated").await.is_some());
    }

    #[tokio::test]
    async fn test_remove_file() {
        let dir = tempfile::tempdir().unwrap();
        write_workflow(dir.path(), "wf.toml", "removable");

        let loader = WorkflowLoader::new(dir.path()).unwrap();
        loader.load_all().await;
        assert_eq!(loader.len().await, 1);

        let event = loader.remove_file(&dir.path().join("wf.toml")).await;
        assert!(matches!(event, Some(WorkflowEvent::Removed { name, .. }) if name == "removable"));
        assert!(loader.is_empty().await);
    }

    #[tokio::test]
    async fn test_get_nonexistent() {
        let dir = tempfile::tempdir().unwrap();
        let loader = WorkflowLoader::new(dir.path()).unwrap();
        assert!(loader.get("nope").await.is_none());
    }
}
