//! Directory management for workstreams and sessions.
//!
//! Provides a convention-based directory structure separating production
//! artifacts from work-in-progress files.
//!
//! # Directory Layout
//!
//! ```text
//! ~/.arawn/workstreams/
//! ├── scratch/sessions/<session-id>/work/   # Isolated per-session
//! ├── <workstream>/production/              # Shared deliverables
//! └── <workstream>/work/                    # Shared working area
//! ```
//!
//! # Access Rules
//!
//! | Workstream | Session | Allowed Paths |
//! |------------|---------|---------------|
//! | scratch | S1 | `scratch/sessions/S1/work/` only |
//! | my-blog | any | `my-blog/production/`, `my-blog/work/` |

use std::fs;
use std::path::{Path, PathBuf};

use thiserror::Error;

/// Errors that can occur during directory operations.
#[derive(Debug, Error)]
pub enum DirectoryError {
    /// IO error during directory operations.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid workstream name.
    #[error("Invalid workstream name: {0}")]
    InvalidName(String),

    /// Invalid session ID.
    #[error("Invalid session ID: {0}")]
    InvalidSessionId(String),
}

/// Result type for directory operations.
pub type DirectoryResult<T> = std::result::Result<T, DirectoryError>;

/// Well-known scratch workstream ID (matches crate::scratch::SCRATCH_ID).
pub const SCRATCH_WORKSTREAM: &str = "scratch";

/// Subdirectory name for workstreams.
const WORKSTREAMS_DIR: &str = "workstreams";

/// Subdirectory for production artifacts.
const PRODUCTION_DIR: &str = "production";

/// Subdirectory for work-in-progress files.
const WORK_DIR: &str = "work";

/// Subdirectory for scratch sessions.
const SESSIONS_DIR: &str = "sessions";

/// Manages the convention-based directory structure for workstreams and sessions.
///
/// This struct is `Send + Sync` safe (only contains `PathBuf`).
///
/// # Example
///
/// ```no_run
/// use arawn_workstream::directory::DirectoryManager;
///
/// let manager = DirectoryManager::default();
/// let paths = manager.allowed_paths("my-project", "session-123");
/// // Returns [~/.arawn/workstreams/my-project/production/, ~/.arawn/workstreams/my-project/work/]
///
/// let scratch_paths = manager.allowed_paths("scratch", "session-456");
/// // Returns [~/.arawn/workstreams/scratch/sessions/session-456/work/]
/// ```
#[derive(Debug, Clone)]
pub struct DirectoryManager {
    base_path: PathBuf,
}

impl Default for DirectoryManager {
    /// Creates a DirectoryManager with the default base path `~/.arawn`.
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        Self {
            base_path: home.join(".arawn"),
        }
    }
}

impl DirectoryManager {
    /// Creates a new DirectoryManager with a custom base path.
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
        }
    }

    /// Returns the base path for all arawn data.
    pub fn base_path(&self) -> &Path {
        &self.base_path
    }

    /// Returns the root path for all workstreams.
    pub fn workstreams_root(&self) -> PathBuf {
        self.base_path.join(WORKSTREAMS_DIR)
    }

    /// Returns the path to a specific workstream's directory.
    pub fn workstream_path(&self, name: &str) -> PathBuf {
        self.workstreams_root().join(name)
    }

    /// Returns the production directory path for a workstream.
    pub fn production_path(&self, workstream: &str) -> PathBuf {
        self.workstream_path(workstream).join(PRODUCTION_DIR)
    }

    /// Returns the work directory path for a workstream.
    pub fn work_path(&self, workstream: &str) -> PathBuf {
        self.workstream_path(workstream).join(WORK_DIR)
    }

    /// Returns the path for a scratch session's isolated work directory.
    pub fn scratch_session_path(&self, session_id: &str) -> PathBuf {
        self.workstream_path(SCRATCH_WORKSTREAM)
            .join(SESSIONS_DIR)
            .join(session_id)
            .join(WORK_DIR)
    }

    /// Checks if a workstream name is valid.
    ///
    /// Valid names:
    /// - Are not empty
    /// - Contain only alphanumeric characters, hyphens, and underscores
    /// - Do not start with a hyphen or period
    /// - Do not contain path separators
    pub fn is_valid_name(name: &str) -> bool {
        if name.is_empty() {
            return false;
        }

        // Check first character
        let first = name.chars().next().unwrap();
        if first == '-' || first == '.' {
            return false;
        }

        // Check all characters
        name.chars().all(|c| {
            c.is_ascii_alphanumeric() || c == '-' || c == '_'
        })
    }

    /// Checks if a session ID is valid.
    ///
    /// Valid session IDs follow the same rules as workstream names.
    pub fn is_valid_session_id(id: &str) -> bool {
        Self::is_valid_name(id)
    }

    /// Checks if a workstream exists (has a directory).
    pub fn workstream_exists(&self, name: &str) -> bool {
        self.workstream_path(name).is_dir()
    }

    /// Returns the allowed paths for a session based on its workstream.
    ///
    /// # Access Rules
    ///
    /// - **Scratch workstream**: Sessions get isolated paths under
    ///   `scratch/sessions/<session-id>/work/`
    /// - **Named workstreams**: Sessions share `<workstream>/production/` and
    ///   `<workstream>/work/` directories
    ///
    /// # Returns
    ///
    /// A vector of `PathBuf` representing the allowed directories for the session.
    /// These paths may not exist yet; use `create_workstream` or `create_scratch_session`
    /// to ensure they exist.
    pub fn allowed_paths(&self, workstream: &str, session_id: &str) -> Vec<PathBuf> {
        if workstream == SCRATCH_WORKSTREAM {
            // Scratch sessions get isolated work directories
            vec![self.scratch_session_path(session_id)]
        } else {
            // Named workstreams share production and work directories
            vec![
                self.production_path(workstream),
                self.work_path(workstream),
            ]
        }
    }

    /// Creates a workstream directory structure.
    ///
    /// Creates:
    /// - `<workstream>/production/`
    /// - `<workstream>/work/`
    ///
    /// This operation is idempotent; calling it multiple times has no effect
    /// if the directories already exist.
    ///
    /// # Arguments
    ///
    /// * `name` - The workstream name. Must be valid per `is_valid_name`.
    ///
    /// # Returns
    ///
    /// The path to the workstream root directory.
    ///
    /// # Errors
    ///
    /// Returns `DirectoryError::InvalidName` if the name is invalid.
    /// Returns `DirectoryError::Io` if directory creation fails.
    pub fn create_workstream(&self, name: &str) -> DirectoryResult<PathBuf> {
        if !Self::is_valid_name(name) {
            return Err(DirectoryError::InvalidName(name.to_string()));
        }

        let ws_path = self.workstream_path(name);
        let production = self.production_path(name);
        let work = self.work_path(name);

        // Create directories atomically (create_dir_all is idempotent)
        fs::create_dir_all(&production)?;
        fs::create_dir_all(&work)?;

        tracing::debug!(
            workstream = %name,
            path = %ws_path.display(),
            "Created workstream directories"
        );

        Ok(ws_path)
    }

    /// Creates a scratch session's isolated work directory.
    ///
    /// Creates `scratch/sessions/<session-id>/work/`
    ///
    /// This operation is idempotent.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The session ID. Must be valid per `is_valid_session_id`.
    ///
    /// # Returns
    ///
    /// The path to the session's work directory.
    ///
    /// # Errors
    ///
    /// Returns `DirectoryError::InvalidSessionId` if the session ID is invalid.
    /// Returns `DirectoryError::Io` if directory creation fails.
    pub fn create_scratch_session(&self, session_id: &str) -> DirectoryResult<PathBuf> {
        if !Self::is_valid_session_id(session_id) {
            return Err(DirectoryError::InvalidSessionId(session_id.to_string()));
        }

        let session_work = self.scratch_session_path(session_id);

        fs::create_dir_all(&session_work)?;

        tracing::debug!(
            session_id = %session_id,
            path = %session_work.display(),
            "Created scratch session directory"
        );

        Ok(session_work)
    }

    /// Removes a scratch session's directory tree.
    ///
    /// # Safety
    ///
    /// This permanently deletes all files in the session's work directory.
    /// Use with caution.
    ///
    /// # Errors
    ///
    /// Returns `DirectoryError::Io` if deletion fails.
    pub fn remove_scratch_session(&self, session_id: &str) -> DirectoryResult<()> {
        if !Self::is_valid_session_id(session_id) {
            return Err(DirectoryError::InvalidSessionId(session_id.to_string()));
        }

        // Get the session directory (parent of work dir)
        let session_dir = self.workstream_path(SCRATCH_WORKSTREAM)
            .join(SESSIONS_DIR)
            .join(session_id);

        if session_dir.exists() {
            fs::remove_dir_all(&session_dir)?;
            tracing::debug!(
                session_id = %session_id,
                path = %session_dir.display(),
                "Removed scratch session directory"
            );
        }

        Ok(())
    }

    /// Lists all scratch session IDs that have directories.
    pub fn list_scratch_sessions(&self) -> DirectoryResult<Vec<String>> {
        let sessions_dir = self.workstream_path(SCRATCH_WORKSTREAM).join(SESSIONS_DIR);

        if !sessions_dir.exists() {
            return Ok(Vec::new());
        }

        let mut sessions = Vec::new();
        for entry in fs::read_dir(&sessions_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    sessions.push(name.to_string());
                }
            }
        }

        Ok(sessions)
    }

    /// Lists all workstream names that have directories (excluding scratch).
    pub fn list_workstreams(&self) -> DirectoryResult<Vec<String>> {
        let workstreams_dir = self.workstreams_root();

        if !workstreams_dir.exists() {
            return Ok(Vec::new());
        }

        let mut workstreams = Vec::new();
        for entry in fs::read_dir(&workstreams_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    // Skip scratch
                    if name != SCRATCH_WORKSTREAM {
                        workstreams.push(name.to_string());
                    }
                }
            }
        }

        Ok(workstreams)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (tempfile::TempDir, DirectoryManager) {
        let dir = tempfile::tempdir().unwrap();
        let manager = DirectoryManager::new(dir.path());
        (dir, manager)
    }

    #[test]
    fn test_default_base_path() {
        let manager = DirectoryManager::default();
        let home = dirs::home_dir().unwrap();
        assert_eq!(manager.base_path(), home.join(".arawn"));
    }

    #[test]
    fn test_custom_base_path() {
        let (_dir, manager) = setup();
        assert!(manager.base_path().exists());
    }

    #[test]
    fn test_is_valid_name() {
        // Valid names
        assert!(DirectoryManager::is_valid_name("my-project"));
        assert!(DirectoryManager::is_valid_name("project_123"));
        assert!(DirectoryManager::is_valid_name("A"));
        assert!(DirectoryManager::is_valid_name("scratch"));

        // Invalid names
        assert!(!DirectoryManager::is_valid_name(""));
        assert!(!DirectoryManager::is_valid_name("-starts-with-hyphen"));
        assert!(!DirectoryManager::is_valid_name(".hidden"));
        assert!(!DirectoryManager::is_valid_name("has/slash"));
        assert!(!DirectoryManager::is_valid_name("has spaces"));
        assert!(!DirectoryManager::is_valid_name("has.dot"));
    }

    #[test]
    fn test_workstream_paths() {
        let (_dir, manager) = setup();

        let ws_path = manager.workstream_path("my-project");
        assert!(ws_path.ends_with("workstreams/my-project"));

        let prod_path = manager.production_path("my-project");
        assert!(prod_path.ends_with("workstreams/my-project/production"));

        let work_path = manager.work_path("my-project");
        assert!(work_path.ends_with("workstreams/my-project/work"));
    }

    #[test]
    fn test_scratch_session_path() {
        let (_dir, manager) = setup();

        let session_path = manager.scratch_session_path("abc-123");
        assert!(session_path.ends_with("workstreams/scratch/sessions/abc-123/work"));
    }

    #[test]
    fn test_allowed_paths_named_workstream() {
        let (_dir, manager) = setup();

        let paths = manager.allowed_paths("my-blog", "session-1");
        assert_eq!(paths.len(), 2);
        assert!(paths[0].ends_with("workstreams/my-blog/production"));
        assert!(paths[1].ends_with("workstreams/my-blog/work"));

        // Different session, same paths
        let paths2 = manager.allowed_paths("my-blog", "session-2");
        assert_eq!(paths, paths2);
    }

    #[test]
    fn test_allowed_paths_scratch() {
        let (_dir, manager) = setup();

        let paths1 = manager.allowed_paths("scratch", "session-1");
        assert_eq!(paths1.len(), 1);
        assert!(paths1[0].ends_with("workstreams/scratch/sessions/session-1/work"));

        // Different session, different paths
        let paths2 = manager.allowed_paths("scratch", "session-2");
        assert_eq!(paths2.len(), 1);
        assert!(paths2[0].ends_with("workstreams/scratch/sessions/session-2/work"));
        assert_ne!(paths1, paths2);
    }

    #[test]
    fn test_create_workstream() {
        let (_dir, manager) = setup();

        let ws_path = manager.create_workstream("test-project").unwrap();
        assert!(ws_path.exists());
        assert!(manager.production_path("test-project").exists());
        assert!(manager.work_path("test-project").exists());
    }

    #[test]
    fn test_create_workstream_idempotent() {
        let (_dir, manager) = setup();

        let path1 = manager.create_workstream("test-project").unwrap();
        let path2 = manager.create_workstream("test-project").unwrap();
        assert_eq!(path1, path2);
    }

    #[test]
    fn test_create_workstream_invalid_name() {
        let (_dir, manager) = setup();

        let err = manager.create_workstream("../escape").unwrap_err();
        assert!(matches!(err, DirectoryError::InvalidName(_)));

        let err = manager.create_workstream("").unwrap_err();
        assert!(matches!(err, DirectoryError::InvalidName(_)));
    }

    #[test]
    fn test_create_scratch_session() {
        let (_dir, manager) = setup();

        let session_path = manager.create_scratch_session("session-abc").unwrap();
        assert!(session_path.exists());
        assert!(session_path.ends_with("sessions/session-abc/work"));
    }

    #[test]
    fn test_create_scratch_session_idempotent() {
        let (_dir, manager) = setup();

        let path1 = manager.create_scratch_session("session-abc").unwrap();
        let path2 = manager.create_scratch_session("session-abc").unwrap();
        assert_eq!(path1, path2);
    }

    #[test]
    fn test_create_scratch_session_invalid_id() {
        let (_dir, manager) = setup();

        let err = manager.create_scratch_session("has/slash").unwrap_err();
        assert!(matches!(err, DirectoryError::InvalidSessionId(_)));
    }

    #[test]
    fn test_remove_scratch_session() {
        let (_dir, manager) = setup();

        // Create then remove
        let session_path = manager.create_scratch_session("to-remove").unwrap();
        assert!(session_path.exists());

        manager.remove_scratch_session("to-remove").unwrap();

        // Work dir gone
        assert!(!session_path.exists());
        // Session dir also gone
        let session_dir = manager.workstream_path(SCRATCH_WORKSTREAM)
            .join(SESSIONS_DIR)
            .join("to-remove");
        assert!(!session_dir.exists());
    }

    #[test]
    fn test_remove_nonexistent_session_is_noop() {
        let (_dir, manager) = setup();

        // Should not error
        manager.remove_scratch_session("nonexistent").unwrap();
    }

    #[test]
    fn test_list_scratch_sessions() {
        let (_dir, manager) = setup();

        // Empty initially
        let sessions = manager.list_scratch_sessions().unwrap();
        assert!(sessions.is_empty());

        // Create some sessions
        manager.create_scratch_session("session-1").unwrap();
        manager.create_scratch_session("session-2").unwrap();

        let mut sessions = manager.list_scratch_sessions().unwrap();
        sessions.sort();
        assert_eq!(sessions, vec!["session-1", "session-2"]);
    }

    #[test]
    fn test_list_workstreams() {
        let (_dir, manager) = setup();

        // Empty initially
        let workstreams = manager.list_workstreams().unwrap();
        assert!(workstreams.is_empty());

        // Create some workstreams
        manager.create_workstream("alpha").unwrap();
        manager.create_workstream("beta").unwrap();

        // Create scratch session (should not appear in list)
        manager.create_scratch_session("session-1").unwrap();

        let mut workstreams = manager.list_workstreams().unwrap();
        workstreams.sort();
        assert_eq!(workstreams, vec!["alpha", "beta"]);
    }

    #[test]
    fn test_workstream_exists() {
        let (_dir, manager) = setup();

        assert!(!manager.workstream_exists("test-project"));

        manager.create_workstream("test-project").unwrap();

        assert!(manager.workstream_exists("test-project"));
    }

    #[test]
    fn test_thread_safety() {
        // Verify DirectoryManager is Send + Sync
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<DirectoryManager>();
    }
}
