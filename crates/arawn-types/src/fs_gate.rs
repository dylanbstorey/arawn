//! Filesystem access gate for sandboxed tool execution.
//!
//! Defines the [`FsGate`] trait that enforces workstream-scoped filesystem
//! boundaries. The trait is defined here in `arawn-types` so both `arawn-agent`
//! (consumer) and `arawn-workstream` (implementor) can reference it without
//! circular dependencies.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use thiserror::Error;

/// Errors from filesystem gate operations.
#[derive(Debug, Error)]
pub enum FsGateError {
    /// Access denied — path is outside the workstream sandbox.
    #[error("Access denied: {path} — {reason}")]
    AccessDenied { path: PathBuf, reason: String },

    /// The path is invalid (empty, no parent, etc.).
    #[error("Invalid path: {0}")]
    InvalidPath(String),

    /// OS-level sandbox execution failed.
    #[error("Sandbox error: {0}")]
    SandboxError(String),
}

/// Output from a sandboxed shell command.
#[derive(Debug, Clone)]
pub struct SandboxOutput {
    /// Standard output.
    pub stdout: String,
    /// Standard error.
    pub stderr: String,
    /// Exit code (0 = success).
    pub exit_code: i32,
    /// Whether the command succeeded.
    pub success: bool,
}

/// Filesystem access gate that enforces workstream boundaries.
///
/// The sandbox boundary is the **workstream**, not the session:
/// - Named workstreams: tools access `production/` + `work/` (shared across sessions)
/// - Scratch: tools are isolated to `scratch/sessions/<id>/work/`
#[async_trait]
pub trait FsGate: Send + Sync {
    /// Validate read access to a path.
    ///
    /// Returns the canonicalized path if access is allowed.
    fn validate_read(&self, path: &Path) -> Result<PathBuf, FsGateError>;

    /// Validate write access to a path.
    ///
    /// For new files, validates the parent directory.
    fn validate_write(&self, path: &Path) -> Result<PathBuf, FsGateError>;

    /// The working directory for tool execution.
    ///
    /// For named workstreams: the `work/` directory.
    /// For scratch: the session's `work/` directory.
    fn working_dir(&self) -> &Path;

    /// Execute a shell command inside an OS-level sandbox.
    ///
    /// The sandbox restricts filesystem and network access to the
    /// workstream boundaries.
    async fn sandbox_execute(
        &self,
        command: &str,
        timeout: Option<Duration>,
    ) -> Result<SandboxOutput, FsGateError>;
}

/// Type alias for a shared filesystem gate.
pub type SharedFsGate = Arc<dyn FsGate>;

/// Resolver that creates an FsGate for a given session and workstream.
///
/// Takes `(session_id, workstream_id)` and returns an FsGate scoped to
/// that workstream's boundaries. Returns `None` if no gate can be created
/// (e.g., no directory manager configured).
pub type FsGateResolver = Arc<dyn Fn(&str, &str) -> Option<Arc<dyn FsGate>> + Send + Sync>;

/// Tool names that require filesystem gate enforcement.
pub const GATED_TOOLS: &[&str] = &["file_read", "file_write", "glob", "grep", "shell"];

/// Check if a tool name requires filesystem gate enforcement.
///
/// # Examples
///
/// ```rust,ignore
/// use arawn_types::is_gated_tool;
///
/// assert!(is_gated_tool("shell"));
/// assert!(is_gated_tool("file_read"));
/// assert!(!is_gated_tool("web_search"));
/// ```
pub fn is_gated_tool(name: &str) -> bool {
    GATED_TOOLS.contains(&name)
}
