//! Workstream-scoped filesystem gate implementation.
//!
//! Wraps [`PathValidator`] and [`SandboxManager`] to enforce workstream
//! boundaries for all agent tool execution.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;

use arawn_sandbox::SandboxManager;
use arawn_types::{FsGate, FsGateError, SandboxOutput};

use crate::directory::DirectoryManager;
use crate::path_validator::PathValidator;

/// Filesystem gate scoped to a workstream.
///
/// For named workstreams, tools get access to the full workstream
/// (`production/` + `work/`), shared across all sessions.
/// For scratch, tools are isolated to `scratch/sessions/<id>/work/`.
pub struct WorkstreamFsGate {
    path_validator: PathValidator,
    sandbox_manager: Arc<SandboxManager>,
    working_dir: PathBuf,
    /// The allowed paths for sandbox write access.
    allowed_paths: Vec<PathBuf>,
}

impl WorkstreamFsGate {
    /// Create a gate for a specific workstream and session.
    ///
    /// Uses `DirectoryManager::allowed_paths()` to determine the sandbox boundary:
    /// - Named workstreams → `[production/, work/]` (workstream-scoped)
    /// - Scratch → `[scratch/sessions/<id>/work/]` (session-scoped)
    pub fn new(
        dm: &DirectoryManager,
        sandbox: Arc<SandboxManager>,
        workstream_id: &str,
        session_id: &str,
    ) -> Self {
        let allowed_paths = dm.allowed_paths(workstream_id, session_id);
        let path_validator = PathValidator::new(allowed_paths.clone());

        // Working directory: work/ for named, session work/ for scratch
        let working_dir = if workstream_id == crate::directory::SCRATCH_WORKSTREAM {
            dm.scratch_session_path(session_id)
        } else {
            dm.work_path(workstream_id)
        };

        Self {
            path_validator,
            sandbox_manager: sandbox,
            working_dir,
            allowed_paths,
        }
    }
}

#[async_trait]
impl FsGate for WorkstreamFsGate {
    fn validate_read(&self, path: &Path) -> Result<PathBuf, FsGateError> {
        self.path_validator.validate(path).map_err(|e| match e {
            crate::path_validator::PathError::NotAllowed { path, .. } => {
                FsGateError::AccessDenied {
                    path,
                    reason: "path is outside the workstream sandbox".to_string(),
                }
            }
            crate::path_validator::PathError::DeniedPath { path } => FsGateError::AccessDenied {
                path,
                reason: "access to sensitive system path is denied".to_string(),
            },
            crate::path_validator::PathError::SymlinkEscape { path, target } => {
                FsGateError::AccessDenied {
                    path,
                    reason: format!("symlink escapes sandbox (target: {})", target.display()),
                }
            }
            other => FsGateError::InvalidPath(other.to_string()),
        })
    }

    fn validate_write(&self, path: &Path) -> Result<PathBuf, FsGateError> {
        self.path_validator
            .validate_write(path)
            .map_err(|e| match e {
                crate::path_validator::PathError::NotAllowed { path, .. } => {
                    FsGateError::AccessDenied {
                        path,
                        reason: "write path is outside the workstream sandbox".to_string(),
                    }
                }
                crate::path_validator::PathError::DeniedPath { path } => {
                    FsGateError::AccessDenied {
                        path,
                        reason: "write to sensitive system path is denied".to_string(),
                    }
                }
                other => FsGateError::InvalidPath(other.to_string()),
            })
    }

    fn working_dir(&self) -> &Path {
        &self.working_dir
    }

    async fn sandbox_execute(
        &self,
        command: &str,
        timeout: Option<Duration>,
    ) -> Result<SandboxOutput, FsGateError> {
        let mut config = arawn_sandbox::SandboxConfig::default()
            .with_write_paths(self.allowed_paths.clone())
            .with_working_dir(&self.working_dir);

        if let Some(t) = timeout {
            config = config.with_timeout(t);
        }

        let output = self
            .sandbox_manager
            .execute(command, &config)
            .await
            .map_err(|e| FsGateError::SandboxError(e.to_string()))?;

        Ok(SandboxOutput {
            stdout: output.stdout,
            stderr: output.stderr,
            exit_code: output.exit_code,
            success: output.success,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_named_workstream_gate_allows_workstream_paths() {
        let dir = tempfile::tempdir().unwrap();
        let dm = DirectoryManager::new(dir.path());
        dm.create_workstream("my-project").unwrap();

        // Create a file inside the work directory
        let work_dir = dm.work_path("my-project");
        let test_file = work_dir.join("test.txt");
        fs::write(&test_file, "hello").unwrap();

        // Create gate — we can't create a real SandboxManager in tests easily,
        // but we can test the path validation parts
        let allowed_paths = dm.allowed_paths("my-project", "session-1");
        let validator = PathValidator::new(allowed_paths);

        // Should allow reading inside work/
        assert!(validator.validate(&test_file).is_ok());

        // Should allow writing inside work/
        let new_file = work_dir.join("new.txt");
        assert!(validator.validate_write(&new_file).is_ok());
    }

    #[test]
    fn test_named_workstream_gate_allows_production_paths() {
        let dir = tempfile::tempdir().unwrap();
        let dm = DirectoryManager::new(dir.path());
        dm.create_workstream("my-project").unwrap();

        let prod_dir = dm.production_path("my-project");
        let test_file = prod_dir.join("output.txt");
        fs::write(&test_file, "result").unwrap();

        let allowed_paths = dm.allowed_paths("my-project", "session-1");
        let validator = PathValidator::new(allowed_paths);

        assert!(validator.validate(&test_file).is_ok());
    }

    #[test]
    fn test_named_workstream_gate_denies_outside_paths() {
        let dir = tempfile::tempdir().unwrap();
        let dm = DirectoryManager::new(dir.path());
        dm.create_workstream("my-project").unwrap();

        // Create a file outside the workstream
        let outside = dir.path().join("outside.txt");
        fs::write(&outside, "secret").unwrap();

        let allowed_paths = dm.allowed_paths("my-project", "session-1");
        let validator = PathValidator::new(allowed_paths);

        assert!(validator.validate(&outside).is_err());
    }

    #[test]
    fn test_scratch_gate_isolates_sessions() {
        let dir = tempfile::tempdir().unwrap();
        let dm = DirectoryManager::new(dir.path());
        dm.create_scratch_session("session-A").unwrap();
        dm.create_scratch_session("session-B").unwrap();

        // Session A's paths
        let allowed_a = dm.allowed_paths("scratch", "session-A");
        let validator_a = PathValidator::new(allowed_a);

        // Session B's paths
        let allowed_b = dm.allowed_paths("scratch", "session-B");
        let validator_b = PathValidator::new(allowed_b);

        // Create files in each session
        let file_a = dm.scratch_session_path("session-A").join("a.txt");
        let file_b = dm.scratch_session_path("session-B").join("b.txt");
        fs::write(&file_a, "a").unwrap();
        fs::write(&file_b, "b").unwrap();

        // A can read its own file
        assert!(validator_a.validate(&file_a).is_ok());
        // A cannot read B's file
        assert!(validator_a.validate(&file_b).is_err());

        // B can read its own file
        assert!(validator_b.validate(&file_b).is_ok());
        // B cannot read A's file
        assert!(validator_b.validate(&file_a).is_err());
    }

    #[test]
    fn test_working_dir_named_workstream() {
        let dir = tempfile::tempdir().unwrap();
        let dm = DirectoryManager::new(dir.path());
        dm.create_workstream("my-project").unwrap();

        // For named workstreams, working_dir should be the work/ directory
        let working_dir = dm.work_path("my-project");
        assert!(working_dir.ends_with("my-project/work"));
    }

    #[test]
    fn test_working_dir_scratch() {
        let dir = tempfile::tempdir().unwrap();
        let dm = DirectoryManager::new(dir.path());
        dm.create_scratch_session("sess-1").unwrap();

        let working_dir = dm.scratch_session_path("sess-1");
        assert!(
            working_dir
                .to_string_lossy()
                .contains("scratch/sessions/sess-1")
        );
    }
}
