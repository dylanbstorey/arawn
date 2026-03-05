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

use std::path::PathBuf;

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
    CloneFailed { url: String, stderr: String },

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

/// Usage statistics for a single session.
#[derive(Debug, Clone)]
pub struct SessionUsage {
    /// Session ID.
    pub id: String,
    /// Disk usage in bytes.
    pub bytes: u64,
}

/// Result of a manual cleanup operation.
#[derive(Debug, Clone)]
pub struct ManualCleanupResult {
    /// Number of files deleted.
    pub deleted_files: usize,
    /// Total bytes freed.
    pub freed_bytes: u64,
    /// Number of files that would be deleted (for dry run / confirmation).
    pub pending_files: usize,
    /// Whether confirmation is required (>100 files).
    pub requires_confirmation: bool,
}

impl ManualCleanupResult {
    /// Convert freed bytes to megabytes.
    pub fn freed_mb(&self) -> f64 {
        self.freed_bytes as f64 / 1_048_576.0
    }
}

/// Disk usage statistics for a workstream.
#[derive(Debug, Clone)]
pub struct UsageStats {
    /// Disk usage of the production/ directory in bytes.
    pub production_bytes: u64,
    /// Disk usage of the work/ directory in bytes.
    pub work_bytes: u64,
    /// Per-session usage (only populated for scratch workstream).
    pub sessions: Vec<SessionUsage>,
    /// Total disk usage in bytes.
    pub total_bytes: u64,
    /// Any warnings based on configured thresholds.
    pub warnings: Vec<String>,
}

impl UsageStats {
    /// Convert production bytes to megabytes.
    pub fn production_mb(&self) -> f64 {
        self.production_bytes as f64 / 1_048_576.0
    }

    /// Convert work bytes to megabytes.
    pub fn work_mb(&self) -> f64 {
        self.work_bytes as f64 / 1_048_576.0
    }

    /// Convert total bytes to megabytes.
    pub fn total_mb(&self) -> f64 {
        self.total_bytes as f64 / 1_048_576.0
    }
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

mod clone;
mod manager;
mod operations;
mod session;
mod usage;

// Re-export the main manager type and its associated types
pub use manager::DirectoryManager;
