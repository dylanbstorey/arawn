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
use std::process::Command;

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

    /// Source file does not exist.
    #[error("Source file does not exist: {0}")]
    SourceNotFound(PathBuf),

    /// Source path is not a file.
    #[error("Source path is not a file: {0}")]
    NotAFile(PathBuf),

    /// Workstream does not exist.
    #[error("Workstream does not exist: {0}")]
    WorkstreamNotFound(String),

    /// Destination path already exists.
    #[error("Destination already exists: {0}")]
    AlreadyExists(PathBuf),

    /// Git clone operation failed.
    #[error("Git clone failed for {url}: {stderr}")]
    CloneFailed {
        url: String,
        stderr: String,
    },

    /// Git command not found.
    #[error("Git is not installed or not in PATH")]
    GitNotFound,

    /// Session work directory does not exist.
    #[error("Session work directory does not exist: {0}")]
    SessionWorkNotFound(String),
}

/// Result type for directory operations.
pub type DirectoryResult<T> = std::result::Result<T, DirectoryError>;

/// Result of a file promotion operation.
#[derive(Debug, Clone)]
pub struct PromoteResult {
    /// The final path of the promoted file (in production/).
    pub path: PathBuf,
    /// Size of the promoted file in bytes.
    pub bytes: u64,
    /// Whether the file was renamed due to a conflict.
    pub renamed: bool,
    /// The original destination path (before rename, if any).
    pub original_destination: PathBuf,
}

/// Result of a file export operation.
#[derive(Debug, Clone)]
pub struct ExportResult {
    /// The final path of the exported file.
    pub path: PathBuf,
    /// Size of the exported file in bytes.
    pub bytes: u64,
}

/// Result of a git clone operation.
#[derive(Debug, Clone)]
pub struct CloneResult {
    /// The path where the repository was cloned.
    pub path: PathBuf,
    /// The HEAD commit hash after cloning.
    pub commit: String,
}

/// Result of attaching a scratch session to a named workstream.
#[derive(Debug, Clone)]
pub struct AttachResult {
    /// Number of files migrated.
    pub files_migrated: usize,
    /// The new work path for the session.
    pub new_work_path: PathBuf,
    /// The new allowed paths for the session.
    pub allowed_paths: Vec<PathBuf>,
}

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

    /// Promotes a file from `work/` to `production/`.
    ///
    /// This moves a file from the workstream's work directory to its production
    /// directory. If a file already exists at the destination, a conflict suffix
    /// is appended (e.g., `file(1).txt`, `file(2).txt`).
    ///
    /// # Arguments
    ///
    /// * `workstream` - The workstream name.
    /// * `source` - Path relative to the work directory.
    /// * `destination` - Path relative to the production directory.
    ///
    /// # Returns
    ///
    /// A `PromoteResult` containing the final path, file size, and whether
    /// the file was renamed due to a conflict.
    ///
    /// # Errors
    ///
    /// - `DirectoryError::InvalidName` if the workstream name is invalid.
    /// - `DirectoryError::WorkstreamNotFound` if the workstream doesn't exist.
    /// - `DirectoryError::SourceNotFound` if the source file doesn't exist.
    /// - `DirectoryError::NotAFile` if the source path is not a file.
    /// - `DirectoryError::Io` if the move operation fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use arawn_workstream::directory::DirectoryManager;
    ///
    /// let manager = DirectoryManager::default();
    /// let result = manager.promote(
    ///     "my-blog",
    ///     Path::new("draft.md"),
    ///     Path::new("posts/final.md"),
    /// ).unwrap();
    /// println!("Promoted to: {}", result.path.display());
    /// ```
    pub fn promote(
        &self,
        workstream: &str,
        source: &Path,
        destination: &Path,
    ) -> DirectoryResult<PromoteResult> {
        // Validate workstream name
        if !Self::is_valid_name(workstream) {
            return Err(DirectoryError::InvalidName(workstream.to_string()));
        }

        // Check workstream exists
        if !self.workstream_exists(workstream) {
            return Err(DirectoryError::WorkstreamNotFound(workstream.to_string()));
        }

        // Build full paths
        let work_path = self.work_path(workstream);
        let prod_path = self.production_path(workstream);

        let src_full = work_path.join(source);
        let original_dest = prod_path.join(destination);

        // Validate source exists and is a file
        if !src_full.exists() {
            return Err(DirectoryError::SourceNotFound(src_full));
        }
        if !src_full.is_file() {
            return Err(DirectoryError::NotAFile(src_full));
        }

        // Create destination directory if needed
        if let Some(parent) = original_dest.parent() {
            fs::create_dir_all(parent)?;
        }

        // Resolve conflicts
        let (final_dest, renamed) = if original_dest.exists() {
            (Self::resolve_conflict(&original_dest), true)
        } else {
            (original_dest.clone(), false)
        };

        // Get file size before moving
        let metadata = fs::metadata(&src_full)?;
        let bytes = metadata.len();

        // Move the file
        fs::rename(&src_full, &final_dest)?;

        tracing::info!(
            workstream = %workstream,
            source = %source.display(),
            destination = %final_dest.display(),
            renamed = %renamed,
            bytes = %bytes,
            "Promoted file from work to production"
        );

        Ok(PromoteResult {
            path: final_dest,
            bytes,
            renamed,
            original_destination: original_dest,
        })
    }

    /// Resolves a filename conflict by appending a suffix.
    ///
    /// Given `file.txt`, tries `file(1).txt`, `file(2).txt`, etc.
    /// until finding a path that doesn't exist.
    fn resolve_conflict(path: &Path) -> PathBuf {
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("file");
        let extension = path.extension().and_then(|s| s.to_str());
        let parent = path.parent().unwrap_or(Path::new(""));

        for i in 1..=1000 {
            let new_name = match extension {
                Some(ext) => format!("{}({}).{}", stem, i, ext),
                None => format!("{}({})", stem, i),
            };
            let candidate = parent.join(&new_name);
            if !candidate.exists() {
                return candidate;
            }
        }

        // Fallback: use timestamp (extremely unlikely to reach here)
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        let new_name = match extension {
            Some(ext) => format!("{}({}).{}", stem, timestamp, ext),
            None => format!("{}({})", stem, timestamp),
        };
        parent.join(&new_name)
    }

    /// Exports a file from `production/` to an external path.
    ///
    /// This copies a file from the workstream's production directory to an
    /// arbitrary external location. The source file is preserved (not moved).
    ///
    /// # Arguments
    ///
    /// * `workstream` - The workstream name.
    /// * `source` - Path relative to the production directory.
    /// * `destination` - Absolute external path. If a directory, the source
    ///   filename is appended.
    ///
    /// # Returns
    ///
    /// An `ExportResult` containing the final destination path and file size.
    ///
    /// # Errors
    ///
    /// - `DirectoryError::InvalidName` if the workstream name is invalid.
    /// - `DirectoryError::WorkstreamNotFound` if the workstream doesn't exist.
    /// - `DirectoryError::SourceNotFound` if the source file doesn't exist.
    /// - `DirectoryError::NotAFile` if the source path is not a file.
    /// - `DirectoryError::Io` if the copy operation fails.
    ///
    /// # Security Note
    ///
    /// The destination path is outside the sandbox. Users are responsible for
    /// choosing safe destinations.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use arawn_workstream::directory::DirectoryManager;
    ///
    /// let manager = DirectoryManager::default();
    /// let result = manager.export(
    ///     "my-blog",
    ///     Path::new("report.pdf"),
    ///     Path::new("/mnt/dropbox/reports/"),
    /// ).unwrap();
    /// println!("Exported to: {}", result.path.display());
    /// ```
    pub fn export(
        &self,
        workstream: &str,
        source: &Path,
        destination: &Path,
    ) -> DirectoryResult<ExportResult> {
        // Validate workstream name
        if !Self::is_valid_name(workstream) {
            return Err(DirectoryError::InvalidName(workstream.to_string()));
        }

        // Check workstream exists
        if !self.workstream_exists(workstream) {
            return Err(DirectoryError::WorkstreamNotFound(workstream.to_string()));
        }

        // Build full source path
        let prod_path = self.production_path(workstream);
        let src_full = prod_path.join(source);

        // Validate source exists and is a file
        if !src_full.exists() {
            return Err(DirectoryError::SourceNotFound(src_full));
        }
        if !src_full.is_file() {
            return Err(DirectoryError::NotAFile(src_full));
        }

        // Determine destination file path
        let dest_full = if destination.is_dir() {
            // If destination is a directory, append the source filename
            let filename = source
                .file_name()
                .ok_or_else(|| DirectoryError::SourceNotFound(src_full.clone()))?;
            destination.join(filename)
        } else {
            destination.to_path_buf()
        };

        // Create destination directory if needed
        if let Some(parent) = dest_full.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }

        // Copy file (preserving source)
        let bytes = fs::copy(&src_full, &dest_full)?;

        tracing::info!(
            workstream = %workstream,
            source = %source.display(),
            destination = %dest_full.display(),
            bytes = %bytes,
            "Exported file from production to external path"
        );

        Ok(ExportResult {
            path: dest_full,
            bytes,
        })
    }

    /// Clones a git repository into the workstream's `production/` directory.
    ///
    /// This uses the system `git` command, relying on the user's SSH keys
    /// and credential helpers for authentication.
    ///
    /// # Arguments
    ///
    /// * `workstream` - The workstream name.
    /// * `url` - The git repository URL (HTTPS or SSH).
    /// * `name` - Optional custom directory name. If not provided, derived from URL.
    ///
    /// # Returns
    ///
    /// A `CloneResult` containing the clone path and HEAD commit hash.
    ///
    /// # Errors
    ///
    /// - `DirectoryError::InvalidName` if the workstream name is invalid.
    /// - `DirectoryError::WorkstreamNotFound` if the workstream doesn't exist.
    /// - `DirectoryError::AlreadyExists` if the destination directory already exists.
    /// - `DirectoryError::GitNotFound` if git is not installed.
    /// - `DirectoryError::CloneFailed` if the clone operation fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arawn_workstream::directory::DirectoryManager;
    ///
    /// let manager = DirectoryManager::default();
    /// let result = manager.clone_repo(
    ///     "my-project",
    ///     "https://github.com/user/repo.git",
    ///     Some("my-repo"),
    /// ).unwrap();
    /// println!("Cloned to: {}, commit: {}", result.path.display(), result.commit);
    /// ```
    pub fn clone_repo(
        &self,
        workstream: &str,
        url: &str,
        name: Option<&str>,
    ) -> DirectoryResult<CloneResult> {
        // Validate workstream name
        if !Self::is_valid_name(workstream) {
            return Err(DirectoryError::InvalidName(workstream.to_string()));
        }

        // Check workstream exists
        if !self.workstream_exists(workstream) {
            return Err(DirectoryError::WorkstreamNotFound(workstream.to_string()));
        }

        // Derive directory name from URL if not provided
        let repo_name = name.unwrap_or_else(|| Self::repo_name_from_url(url));

        // Build destination path
        let prod_path = self.production_path(workstream);
        let dest = prod_path.join(repo_name);

        // Check if destination already exists
        if dest.exists() {
            return Err(DirectoryError::AlreadyExists(dest));
        }

        // Ensure production directory exists
        fs::create_dir_all(&prod_path)?;

        // Check git is available
        if !Self::is_git_available() {
            return Err(DirectoryError::GitNotFound);
        }

        // Run git clone
        let output = Command::new("git")
            .args(["clone", "--", url])
            .arg(&dest)
            .output()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    DirectoryError::GitNotFound
                } else {
                    DirectoryError::Io(e)
                }
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(DirectoryError::CloneFailed {
                url: url.to_string(),
                stderr,
            });
        }

        // Get HEAD commit
        let commit = Self::get_head_commit(&dest)?;

        tracing::info!(
            workstream = %workstream,
            url = %url,
            path = %dest.display(),
            commit = %commit,
            "Cloned git repository into production"
        );

        Ok(CloneResult { path: dest, commit })
    }

    /// Derive repository name from URL.
    ///
    /// Examples:
    /// - `https://github.com/user/repo.git` -> `repo`
    /// - `git@github.com:user/repo.git` -> `repo`
    /// - `https://github.com/user/repo` -> `repo`
    fn repo_name_from_url(url: &str) -> &str {
        url.rsplit('/')
            .next()
            .map(|s| s.strip_suffix(".git").unwrap_or(s))
            .filter(|s| !s.is_empty())
            .unwrap_or("repo")
    }

    /// Check if git is available on the system.
    fn is_git_available() -> bool {
        Command::new("git")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Get the HEAD commit hash for a repository.
    fn get_head_commit(repo_path: &Path) -> DirectoryResult<String> {
        let output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(repo_path)
            .output()?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            // If we can't get the commit, return "unknown"
            Ok("unknown".to_string())
        }
    }

    /// Attaches a scratch session to a named workstream by migrating its files.
    ///
    /// This moves all files from the scratch session's work directory to a
    /// session-named subfolder in the target workstream's work directory.
    /// The empty scratch session directory is cleaned up afterward.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The session ID being migrated.
    /// * `target_workstream` - The workstream to attach the session to.
    ///
    /// # Returns
    ///
    /// An `AttachResult` containing the count of migrated files and new paths.
    ///
    /// # Errors
    ///
    /// - `DirectoryError::InvalidSessionId` if the session ID is invalid.
    /// - `DirectoryError::InvalidName` if the workstream name is invalid.
    /// - `DirectoryError::WorkstreamNotFound` if the target workstream doesn't exist.
    /// - `DirectoryError::SessionWorkNotFound` if the scratch session has no work directory.
    /// - `DirectoryError::Io` if file operations fail.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arawn_workstream::directory::DirectoryManager;
    ///
    /// let manager = DirectoryManager::default();
    /// let result = manager.attach_session("session-123", "my-blog").unwrap();
    /// println!("Migrated {} files to {:?}", result.files_migrated, result.new_work_path);
    /// ```
    pub fn attach_session(
        &self,
        session_id: &str,
        target_workstream: &str,
    ) -> DirectoryResult<AttachResult> {
        // Validate session ID
        if !Self::is_valid_session_id(session_id) {
            return Err(DirectoryError::InvalidSessionId(session_id.to_string()));
        }

        // Validate target workstream
        if !Self::is_valid_name(target_workstream) {
            return Err(DirectoryError::InvalidName(target_workstream.to_string()));
        }

        // Check target workstream exists
        if !self.workstream_exists(target_workstream) {
            return Err(DirectoryError::WorkstreamNotFound(target_workstream.to_string()));
        }

        // Source: scratch session work directory
        let scratch_work = self.scratch_session_path(session_id);

        // Check if scratch work directory exists (may not if session had no files)
        if !scratch_work.exists() {
            // No files to migrate, just return empty result
            let allowed = self.allowed_paths(target_workstream, session_id);
            return Ok(AttachResult {
                files_migrated: 0,
                new_work_path: self.work_path(target_workstream).join(session_id),
                allowed_paths: allowed,
            });
        }

        // Destination: work/<session_id>/ in target workstream to avoid conflicts
        let dest_work = self.work_path(target_workstream).join(session_id);

        // Create destination directory
        fs::create_dir_all(&dest_work)?;

        // Move all files and directories from scratch to destination
        let mut count = 0;
        for entry in fs::read_dir(&scratch_work)? {
            let entry = entry?;
            let src_path = entry.path();
            let dest_path = dest_work.join(entry.file_name());

            // Use rename for atomic move (same filesystem) or fall back to copy+delete
            if let Err(_) = fs::rename(&src_path, &dest_path) {
                // Cross-filesystem move: copy and delete
                if src_path.is_dir() {
                    Self::copy_dir_recursive(&src_path, &dest_path)?;
                } else {
                    fs::copy(&src_path, &dest_path)?;
                }
                if src_path.is_dir() {
                    fs::remove_dir_all(&src_path)?;
                } else {
                    fs::remove_file(&src_path)?;
                }
            }
            count += 1;
        }

        // Clean up empty scratch session directory
        // The session dir is: scratch/sessions/<session_id>/
        // scratch_work is: scratch/sessions/<session_id>/work/
        if let Some(session_dir) = scratch_work.parent() {
            // Remove the empty work dir first, then the session dir
            let _ = fs::remove_dir(&scratch_work);
            let _ = fs::remove_dir(session_dir);
        }

        // Get the new allowed paths for the target workstream
        let allowed = self.allowed_paths(target_workstream, session_id);

        tracing::info!(
            session_id = %session_id,
            target_workstream = %target_workstream,
            files_migrated = %count,
            new_work_path = %dest_work.display(),
            "Attached scratch session to workstream"
        );

        Ok(AttachResult {
            files_migrated: count,
            new_work_path: dest_work,
            allowed_paths: allowed,
        })
    }

    /// Recursively copy a directory.
    fn copy_dir_recursive(src: &Path, dest: &Path) -> DirectoryResult<()> {
        fs::create_dir_all(dest)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let src_path = entry.path();
            let dest_path = dest.join(entry.file_name());
            if src_path.is_dir() {
                Self::copy_dir_recursive(&src_path, &dest_path)?;
            } else {
                fs::copy(&src_path, &dest_path)?;
            }
        }
        Ok(())
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

    #[test]
    fn test_promote_basic() {
        let (_dir, manager) = setup();

        // Create workstream
        manager.create_workstream("test-project").unwrap();

        // Create a file in work/
        let work_path = manager.work_path("test-project");
        let source_file = work_path.join("draft.txt");
        fs::write(&source_file, "Hello, world!").unwrap();

        // Promote to production/
        let result = manager
            .promote("test-project", Path::new("draft.txt"), Path::new("final.txt"))
            .unwrap();

        // Verify result
        assert!(!result.renamed);
        assert_eq!(result.bytes, 13); // "Hello, world!" is 13 bytes
        assert!(result.path.ends_with("production/final.txt"));

        // Verify file moved
        assert!(!source_file.exists());
        assert!(result.path.exists());

        // Verify content preserved
        let content = fs::read_to_string(&result.path).unwrap();
        assert_eq!(content, "Hello, world!");
    }

    #[test]
    fn test_promote_to_subdirectory() {
        let (_dir, manager) = setup();

        manager.create_workstream("test-project").unwrap();

        let work_path = manager.work_path("test-project");
        fs::write(work_path.join("article.md"), "# Article").unwrap();

        let result = manager
            .promote(
                "test-project",
                Path::new("article.md"),
                Path::new("blog/posts/2024/article.md"),
            )
            .unwrap();

        assert!(result.path.ends_with("production/blog/posts/2024/article.md"));
        assert!(result.path.exists());
    }

    #[test]
    fn test_promote_with_conflict() {
        let (_dir, manager) = setup();

        manager.create_workstream("test-project").unwrap();

        // Create file in work/
        let work_path = manager.work_path("test-project");
        fs::write(work_path.join("file.txt"), "new content").unwrap();

        // Create existing file in production/
        let prod_path = manager.production_path("test-project");
        fs::write(prod_path.join("file.txt"), "old content").unwrap();

        // Promote - should rename due to conflict
        let result = manager
            .promote("test-project", Path::new("file.txt"), Path::new("file.txt"))
            .unwrap();

        assert!(result.renamed);
        assert!(result.path.ends_with("file(1).txt"));
        assert!(result.path.exists());

        // Original should still exist
        let original = prod_path.join("file.txt");
        assert!(original.exists());
        assert_eq!(fs::read_to_string(&original).unwrap(), "old content");

        // New file has new content
        assert_eq!(fs::read_to_string(&result.path).unwrap(), "new content");
    }

    #[test]
    fn test_promote_with_multiple_conflicts() {
        let (_dir, manager) = setup();

        manager.create_workstream("test-project").unwrap();

        let prod_path = manager.production_path("test-project");

        // Create existing files with conflict names
        fs::write(prod_path.join("file.txt"), "v0").unwrap();
        fs::write(prod_path.join("file(1).txt"), "v1").unwrap();
        fs::write(prod_path.join("file(2).txt"), "v2").unwrap();

        // Create file to promote
        let work_path = manager.work_path("test-project");
        fs::write(work_path.join("file.txt"), "v3").unwrap();

        let result = manager
            .promote("test-project", Path::new("file.txt"), Path::new("file.txt"))
            .unwrap();

        assert!(result.renamed);
        assert!(result.path.ends_with("file(3).txt"));
    }

    #[test]
    fn test_promote_file_without_extension() {
        let (_dir, manager) = setup();

        manager.create_workstream("test-project").unwrap();

        let work_path = manager.work_path("test-project");
        let prod_path = manager.production_path("test-project");

        fs::write(work_path.join("Makefile"), "all:").unwrap();
        fs::write(prod_path.join("Makefile"), "existing").unwrap();

        let result = manager
            .promote("test-project", Path::new("Makefile"), Path::new("Makefile"))
            .unwrap();

        assert!(result.renamed);
        assert!(result.path.ends_with("Makefile(1)"));
    }

    #[test]
    fn test_promote_source_not_found() {
        let (_dir, manager) = setup();

        manager.create_workstream("test-project").unwrap();

        let err = manager
            .promote(
                "test-project",
                Path::new("nonexistent.txt"),
                Path::new("dest.txt"),
            )
            .unwrap_err();

        assert!(matches!(err, DirectoryError::SourceNotFound(_)));
    }

    #[test]
    fn test_promote_source_is_directory() {
        let (_dir, manager) = setup();

        manager.create_workstream("test-project").unwrap();

        // Create a directory in work/
        let work_path = manager.work_path("test-project");
        fs::create_dir_all(work_path.join("subdir")).unwrap();

        let err = manager
            .promote("test-project", Path::new("subdir"), Path::new("dest"))
            .unwrap_err();

        assert!(matches!(err, DirectoryError::NotAFile(_)));
    }

    #[test]
    fn test_promote_workstream_not_found() {
        let (_dir, manager) = setup();

        let err = manager
            .promote(
                "nonexistent",
                Path::new("file.txt"),
                Path::new("file.txt"),
            )
            .unwrap_err();

        assert!(matches!(err, DirectoryError::WorkstreamNotFound(_)));
    }

    #[test]
    fn test_promote_invalid_workstream_name() {
        let (_dir, manager) = setup();

        let err = manager
            .promote(
                "../escape",
                Path::new("file.txt"),
                Path::new("file.txt"),
            )
            .unwrap_err();

        assert!(matches!(err, DirectoryError::InvalidName(_)));
    }

    #[test]
    fn test_resolve_conflict_basic() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.txt");

        // No conflict yet
        let resolved = DirectoryManager::resolve_conflict(&path);
        assert!(resolved.ends_with("test(1).txt"));
    }

    #[test]
    fn test_resolve_conflict_finds_gap() {
        let dir = tempfile::tempdir().unwrap();
        let base_path = dir.path().join("test.txt");

        // Create test(1).txt and test(2).txt
        fs::write(dir.path().join("test(1).txt"), "").unwrap();
        fs::write(dir.path().join("test(2).txt"), "").unwrap();

        let resolved = DirectoryManager::resolve_conflict(&base_path);
        assert!(resolved.ends_with("test(3).txt"));
    }

    // ── Export tests ───────────────────────────────────────────────────

    #[test]
    fn test_export_basic() {
        let (_dir, manager) = setup();
        let export_dir = tempfile::tempdir().unwrap();

        // Create workstream with a file in production
        manager.create_workstream("test-project").unwrap();
        let prod_path = manager.production_path("test-project");
        fs::write(prod_path.join("report.pdf"), "PDF content here").unwrap();

        // Export to external directory
        let result = manager
            .export(
                "test-project",
                Path::new("report.pdf"),
                export_dir.path(),
            )
            .unwrap();

        // Verify result
        assert_eq!(result.bytes, 16); // "PDF content here" is 16 bytes
        assert!(result.path.ends_with("report.pdf"));
        assert!(result.path.exists());

        // Verify source still exists (copy, not move)
        assert!(prod_path.join("report.pdf").exists());

        // Verify content preserved
        let content = fs::read_to_string(&result.path).unwrap();
        assert_eq!(content, "PDF content here");
    }

    #[test]
    fn test_export_to_specific_file() {
        let (_dir, manager) = setup();
        let export_dir = tempfile::tempdir().unwrap();

        manager.create_workstream("test-project").unwrap();
        let prod_path = manager.production_path("test-project");
        fs::write(prod_path.join("data.json"), r#"{"key": "value"}"#).unwrap();

        // Export to specific filename
        let dest_file = export_dir.path().join("renamed.json");
        let result = manager
            .export("test-project", Path::new("data.json"), &dest_file)
            .unwrap();

        assert!(result.path.ends_with("renamed.json"));
        assert!(result.path.exists());
    }

    #[test]
    fn test_export_creates_destination_dirs() {
        let (_dir, manager) = setup();
        let export_dir = tempfile::tempdir().unwrap();

        manager.create_workstream("test-project").unwrap();
        let prod_path = manager.production_path("test-project");
        fs::write(prod_path.join("file.txt"), "content").unwrap();

        // Export to nested path that doesn't exist
        let dest_path = export_dir.path().join("a/b/c/file.txt");
        let result = manager
            .export("test-project", Path::new("file.txt"), &dest_path)
            .unwrap();

        assert!(result.path.exists());
        assert!(result.path.ends_with("a/b/c/file.txt"));
    }

    #[test]
    fn test_export_from_nested_source() {
        let (_dir, manager) = setup();
        let export_dir = tempfile::tempdir().unwrap();

        manager.create_workstream("test-project").unwrap();
        let prod_path = manager.production_path("test-project");
        fs::create_dir_all(prod_path.join("reports/2024")).unwrap();
        fs::write(prod_path.join("reports/2024/q1.pdf"), "Q1 Report").unwrap();

        let result = manager
            .export(
                "test-project",
                Path::new("reports/2024/q1.pdf"),
                export_dir.path(),
            )
            .unwrap();

        assert!(result.path.ends_with("q1.pdf"));
        assert!(result.path.exists());
    }

    #[test]
    fn test_export_source_not_found() {
        let (_dir, manager) = setup();
        let export_dir = tempfile::tempdir().unwrap();

        manager.create_workstream("test-project").unwrap();

        let err = manager
            .export(
                "test-project",
                Path::new("nonexistent.txt"),
                export_dir.path(),
            )
            .unwrap_err();

        assert!(matches!(err, DirectoryError::SourceNotFound(_)));
    }

    #[test]
    fn test_export_source_is_directory() {
        let (_dir, manager) = setup();
        let export_dir = tempfile::tempdir().unwrap();

        manager.create_workstream("test-project").unwrap();
        let prod_path = manager.production_path("test-project");
        fs::create_dir_all(prod_path.join("subdir")).unwrap();

        let err = manager
            .export("test-project", Path::new("subdir"), export_dir.path())
            .unwrap_err();

        assert!(matches!(err, DirectoryError::NotAFile(_)));
    }

    #[test]
    fn test_export_workstream_not_found() {
        let (_dir, manager) = setup();
        let export_dir = tempfile::tempdir().unwrap();

        let err = manager
            .export("nonexistent", Path::new("file.txt"), export_dir.path())
            .unwrap_err();

        assert!(matches!(err, DirectoryError::WorkstreamNotFound(_)));
    }

    #[test]
    fn test_export_invalid_workstream_name() {
        let (_dir, manager) = setup();
        let export_dir = tempfile::tempdir().unwrap();

        let err = manager
            .export("../escape", Path::new("file.txt"), export_dir.path())
            .unwrap_err();

        assert!(matches!(err, DirectoryError::InvalidName(_)));
    }

    #[test]
    fn test_export_preserves_source() {
        let (_dir, manager) = setup();
        let export_dir = tempfile::tempdir().unwrap();

        manager.create_workstream("test-project").unwrap();
        let prod_path = manager.production_path("test-project");
        let source_file = prod_path.join("keep-me.txt");
        fs::write(&source_file, "original content").unwrap();

        // Export twice to same destination (overwrites)
        manager
            .export("test-project", Path::new("keep-me.txt"), export_dir.path())
            .unwrap();
        manager
            .export("test-project", Path::new("keep-me.txt"), export_dir.path())
            .unwrap();

        // Source should still exist
        assert!(source_file.exists());
        assert_eq!(fs::read_to_string(&source_file).unwrap(), "original content");
    }

    // ── Clone tests ────────────────────────────────────────────────────

    #[test]
    fn test_repo_name_from_url_https() {
        assert_eq!(
            DirectoryManager::repo_name_from_url("https://github.com/user/repo.git"),
            "repo"
        );
        assert_eq!(
            DirectoryManager::repo_name_from_url("https://github.com/user/repo"),
            "repo"
        );
        assert_eq!(
            DirectoryManager::repo_name_from_url("https://gitlab.com/group/subgroup/project.git"),
            "project"
        );
    }

    #[test]
    fn test_repo_name_from_url_ssh() {
        // SSH URLs still work because we split on '/' which gets the last segment
        // "git@github.com:user/repo.git" -> splits on '/' -> "repo.git" -> strips ".git" -> "repo"
        assert_eq!(
            DirectoryManager::repo_name_from_url("git@github.com:user/repo.git"),
            "repo"
        );
    }

    #[test]
    fn test_repo_name_from_url_fallback() {
        assert_eq!(DirectoryManager::repo_name_from_url(""), "repo");
        assert_eq!(DirectoryManager::repo_name_from_url("/"), "repo");
    }

    #[test]
    fn test_clone_workstream_not_found() {
        let (_dir, manager) = setup();

        let err = manager
            .clone_repo("nonexistent", "https://github.com/user/repo.git", None)
            .unwrap_err();

        assert!(matches!(err, DirectoryError::WorkstreamNotFound(_)));
    }

    #[test]
    fn test_clone_invalid_workstream_name() {
        let (_dir, manager) = setup();

        let err = manager
            .clone_repo("../escape", "https://github.com/user/repo.git", None)
            .unwrap_err();

        assert!(matches!(err, DirectoryError::InvalidName(_)));
    }

    #[test]
    fn test_clone_destination_exists() {
        let (_dir, manager) = setup();

        manager.create_workstream("test-project").unwrap();
        let prod_path = manager.production_path("test-project");

        // Create a directory that would conflict
        fs::create_dir_all(prod_path.join("repo")).unwrap();

        let err = manager
            .clone_repo(
                "test-project",
                "https://github.com/user/repo.git",
                None, // Will derive "repo" from URL
            )
            .unwrap_err();

        assert!(matches!(err, DirectoryError::AlreadyExists(_)));
    }

    #[test]
    fn test_clone_custom_name_conflict() {
        let (_dir, manager) = setup();

        manager.create_workstream("test-project").unwrap();
        let prod_path = manager.production_path("test-project");

        // Create a directory that would conflict with custom name
        fs::create_dir_all(prod_path.join("my-clone")).unwrap();

        let err = manager
            .clone_repo(
                "test-project",
                "https://github.com/user/repo.git",
                Some("my-clone"),
            )
            .unwrap_err();

        assert!(matches!(err, DirectoryError::AlreadyExists(_)));
    }

    #[test]
    fn test_is_git_available() {
        // This should pass on any system with git installed
        // If git isn't installed, this test documents that behavior
        let available = DirectoryManager::is_git_available();
        // Don't assert - just verify it doesn't panic
        let _ = available;
    }

    // Integration test that actually clones a repo
    #[test]
    #[ignore] // Run with --ignored flag - requires network and git
    fn test_clone_public_repo() {
        let (_dir, manager) = setup();

        manager.create_workstream("test-project").unwrap();

        // Clone a small, stable public repo
        let result = manager
            .clone_repo(
                "test-project",
                "https://github.com/octocat/Hello-World.git",
                Some("hello"),
            )
            .unwrap();

        assert!(result.path.ends_with("production/hello"));
        assert!(result.path.exists());
        assert!(result.path.join(".git").is_dir());
        assert!(!result.commit.is_empty());
        assert_ne!(result.commit, "unknown");
    }

    #[test]
    #[ignore] // Run with --ignored flag - requires network and git
    fn test_clone_invalid_url() {
        let (_dir, manager) = setup();

        manager.create_workstream("test-project").unwrap();

        let err = manager
            .clone_repo(
                "test-project",
                "https://github.com/nonexistent-user-12345/nonexistent-repo-67890.git",
                None,
            )
            .unwrap_err();

        assert!(matches!(err, DirectoryError::CloneFailed { .. }));
    }

    // ── Attach session tests ───────────────────────────────────────────

    #[test]
    fn test_attach_session_basic() {
        let (_dir, manager) = setup();

        // Create scratch session with files
        let scratch_work = manager.create_scratch_session("session-123").unwrap();
        fs::write(scratch_work.join("file1.txt"), "content1").unwrap();
        fs::write(scratch_work.join("file2.txt"), "content2").unwrap();

        // Create target workstream
        manager.create_workstream("my-blog").unwrap();

        // Attach session
        let result = manager.attach_session("session-123", "my-blog").unwrap();

        // Verify result
        assert_eq!(result.files_migrated, 2);
        assert!(result.new_work_path.ends_with("my-blog/work/session-123"));
        assert_eq!(result.allowed_paths.len(), 2); // production/ and work/

        // Verify files were moved
        assert!(result.new_work_path.join("file1.txt").exists());
        assert!(result.new_work_path.join("file2.txt").exists());
        assert_eq!(
            fs::read_to_string(result.new_work_path.join("file1.txt")).unwrap(),
            "content1"
        );

        // Verify scratch session directory was cleaned up
        assert!(!scratch_work.exists());
    }

    #[test]
    fn test_attach_session_with_subdirectories() {
        let (_dir, manager) = setup();

        // Create scratch session with nested structure
        let scratch_work = manager.create_scratch_session("session-456").unwrap();
        fs::create_dir_all(scratch_work.join("subdir/nested")).unwrap();
        fs::write(scratch_work.join("root.txt"), "root").unwrap();
        fs::write(scratch_work.join("subdir/child.txt"), "child").unwrap();
        fs::write(scratch_work.join("subdir/nested/deep.txt"), "deep").unwrap();

        // Create target workstream
        manager.create_workstream("project").unwrap();

        // Attach session
        let result = manager.attach_session("session-456", "project").unwrap();

        // Verify files were moved including subdirectories
        assert_eq!(result.files_migrated, 2); // root.txt and subdir/
        assert!(result.new_work_path.join("root.txt").exists());
        assert!(result.new_work_path.join("subdir/child.txt").exists());
        assert!(result.new_work_path.join("subdir/nested/deep.txt").exists());
    }

    #[test]
    fn test_attach_session_no_files() {
        let (_dir, manager) = setup();

        // Create target workstream (but no scratch session files)
        manager.create_workstream("empty-target").unwrap();

        // Attach session that doesn't exist (no work dir)
        let result = manager.attach_session("nonexistent-session", "empty-target").unwrap();

        // Should succeed with 0 files migrated
        assert_eq!(result.files_migrated, 0);
        assert_eq!(result.allowed_paths.len(), 2);
    }

    #[test]
    fn test_attach_session_invalid_session_id() {
        let (_dir, manager) = setup();

        manager.create_workstream("target").unwrap();

        let err = manager.attach_session("../escape", "target").unwrap_err();
        assert!(matches!(err, DirectoryError::InvalidSessionId(_)));
    }

    #[test]
    fn test_attach_session_invalid_workstream_name() {
        let (_dir, manager) = setup();

        // Create scratch session
        let _ = manager.create_scratch_session("session-123").unwrap();

        let err = manager.attach_session("session-123", "../escape").unwrap_err();
        assert!(matches!(err, DirectoryError::InvalidName(_)));
    }

    #[test]
    fn test_attach_session_workstream_not_found() {
        let (_dir, manager) = setup();

        // Create scratch session
        let _ = manager.create_scratch_session("session-123").unwrap();

        let err = manager.attach_session("session-123", "nonexistent").unwrap_err();
        assert!(matches!(err, DirectoryError::WorkstreamNotFound(_)));
    }

    #[test]
    fn test_attach_session_preserves_content() {
        let (_dir, manager) = setup();

        // Create scratch session with various content
        let scratch_work = manager.create_scratch_session("session-789").unwrap();
        fs::write(scratch_work.join("binary.bin"), vec![0u8, 1, 2, 3, 255]).unwrap();
        fs::write(scratch_work.join("unicode.txt"), "Hello 世界 🌍").unwrap();

        manager.create_workstream("preserve-test").unwrap();

        let result = manager.attach_session("session-789", "preserve-test").unwrap();

        // Verify content is preserved exactly
        assert_eq!(
            fs::read(result.new_work_path.join("binary.bin")).unwrap(),
            vec![0u8, 1, 2, 3, 255]
        );
        assert_eq!(
            fs::read_to_string(result.new_work_path.join("unicode.txt")).unwrap(),
            "Hello 世界 🌍"
        );
    }
}
