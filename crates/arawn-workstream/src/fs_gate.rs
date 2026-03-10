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
    sandbox_manager: Option<Arc<SandboxManager>>,
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
        Self::build(dm, Some(sandbox), workstream_id, session_id)
    }

    /// Create a path-only gate (no sandbox for shell execution).
    ///
    /// File tools (file_read, file_write, glob, grep) work normally with
    /// path validation. Shell commands return a clear error explaining that
    /// the sandbox is unavailable.
    pub fn path_only(dm: &DirectoryManager, workstream_id: &str, session_id: &str) -> Self {
        Self::build(dm, None, workstream_id, session_id)
    }

    fn build(
        dm: &DirectoryManager,
        sandbox: Option<Arc<SandboxManager>>,
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
        let manager = match &self.sandbox_manager {
            Some(m) => m.clone(),
            None => {
                return Err(FsGateError::SandboxError(
                    "Shell execution is unavailable: sandbox runtime not initialized. \
                     File tools (file_read, file_write, glob, grep) still work."
                        .to_string(),
                ));
            }
        };

        let mut config = arawn_sandbox::SandboxConfig::default()
            .with_write_paths(self.allowed_paths.clone())
            .with_working_dir(&self.working_dir);

        if let Some(t) = timeout {
            config = config.with_timeout(t);
        }

        // Run sandbox execution via block_in_place because the sandbox-runtime
        // crate holds a parking_lot RwLockWriteGuard (which is !Send) across
        // internal awaits. This prevents the future from being Send, which
        // async_trait requires. block_in_place lets us drive the future on the
        // current thread without requiring Send.
        let command = command.to_string();
        let output = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(manager.execute(&command, &config))
        })
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

    // ── WorkstreamFsGate (FsGate trait) Tests ──────────────────────────

    #[test]
    fn test_path_only_gate_validate_read_allowed() {
        let dir = tempfile::tempdir().unwrap();
        let dm = DirectoryManager::new(dir.path());
        dm.create_workstream("proj").unwrap();

        let work_dir = dm.work_path("proj");
        let test_file = work_dir.join("readme.md");
        fs::write(&test_file, "# Hello").unwrap();

        let gate = WorkstreamFsGate::path_only(&dm, "proj", "s1");

        let result = gate.validate_read(&test_file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_path_only_gate_validate_read_denied() {
        let dir = tempfile::tempdir().unwrap();
        let dm = DirectoryManager::new(dir.path());
        dm.create_workstream("proj").unwrap();

        let outside = dir.path().join("outside.txt");
        fs::write(&outside, "secret").unwrap();

        let gate = WorkstreamFsGate::path_only(&dm, "proj", "s1");

        let result = gate.validate_read(&outside);
        assert!(result.is_err());
        match result.unwrap_err() {
            FsGateError::AccessDenied { reason, .. } => {
                assert!(reason.contains("outside the workstream sandbox"));
            }
            other => panic!("Expected AccessDenied, got: {:?}", other),
        }
    }

    #[test]
    fn test_path_only_gate_validate_write_allowed() {
        let dir = tempfile::tempdir().unwrap();
        let dm = DirectoryManager::new(dir.path());
        dm.create_workstream("proj").unwrap();

        let work_dir = dm.work_path("proj");
        let new_file = work_dir.join("output.txt");

        let gate = WorkstreamFsGate::path_only(&dm, "proj", "s1");

        let result = gate.validate_write(&new_file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_path_only_gate_validate_write_denied() {
        let dir = tempfile::tempdir().unwrap();
        let dm = DirectoryManager::new(dir.path());
        dm.create_workstream("proj").unwrap();

        let outside = dir.path().join("outside.txt");

        let gate = WorkstreamFsGate::path_only(&dm, "proj", "s1");

        let result = gate.validate_write(&outside);
        assert!(result.is_err());
        match result.unwrap_err() {
            FsGateError::AccessDenied { reason, .. } => {
                assert!(reason.contains("outside the workstream sandbox"));
            }
            other => panic!("Expected AccessDenied, got: {:?}", other),
        }
    }

    #[test]
    fn test_path_only_gate_working_dir_named() {
        let dir = tempfile::tempdir().unwrap();
        let dm = DirectoryManager::new(dir.path());
        dm.create_workstream("proj").unwrap();

        let gate = WorkstreamFsGate::path_only(&dm, "proj", "s1");

        assert!(gate.working_dir().ends_with("proj/work"));
    }

    #[test]
    fn test_path_only_gate_working_dir_scratch() {
        let dir = tempfile::tempdir().unwrap();
        let dm = DirectoryManager::new(dir.path());
        dm.create_scratch_session("sess-x").unwrap();

        let gate = WorkstreamFsGate::path_only(&dm, "scratch", "sess-x");

        assert!(gate
            .working_dir()
            .to_string_lossy()
            .contains("scratch/sessions/sess-x"));
    }

    #[tokio::test]
    async fn test_path_only_gate_sandbox_execute_returns_error() {
        let dir = tempfile::tempdir().unwrap();
        let dm = DirectoryManager::new(dir.path());
        dm.create_workstream("proj").unwrap();

        let gate = WorkstreamFsGate::path_only(&dm, "proj", "s1");

        // sandbox_execute should fail with clear message when sandbox is None
        let result = gate.sandbox_execute("echo hello", None).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            FsGateError::SandboxError(msg) => {
                assert!(msg.contains("unavailable"));
                assert!(msg.contains("File tools"));
            }
            other => panic!("Expected SandboxError, got: {:?}", other),
        }
    }

    #[test]
    fn test_scratch_gate_validate_read_cross_session_denied() {
        let dir = tempfile::tempdir().unwrap();
        let dm = DirectoryManager::new(dir.path());
        dm.create_scratch_session("session-A").unwrap();
        dm.create_scratch_session("session-B").unwrap();

        let gate_a = WorkstreamFsGate::path_only(&dm, "scratch", "session-A");

        // Create file in session B
        let file_b = dm.scratch_session_path("session-B").join("b.txt");
        fs::write(&file_b, "b").unwrap();

        // Gate A should deny reading session B's file
        let result = gate_a.validate_read(&file_b);
        assert!(result.is_err());
    }

    #[test]
    fn test_scratch_gate_validate_write_own_session() {
        let dir = tempfile::tempdir().unwrap();
        let dm = DirectoryManager::new(dir.path());
        dm.create_scratch_session("sess-1").unwrap();

        let gate = WorkstreamFsGate::path_only(&dm, "scratch", "sess-1");

        let own_file = dm.scratch_session_path("sess-1").join("new.txt");
        assert!(gate.validate_write(&own_file).is_ok());
    }

    #[test]
    fn test_named_gate_allows_production_read() {
        let dir = tempfile::tempdir().unwrap();
        let dm = DirectoryManager::new(dir.path());
        dm.create_workstream("proj").unwrap();

        let prod_dir = dm.production_path("proj");
        let prod_file = prod_dir.join("data.csv");
        fs::write(&prod_file, "a,b,c").unwrap();

        let gate = WorkstreamFsGate::path_only(&dm, "proj", "s1");
        assert!(gate.validate_read(&prod_file).is_ok());
    }

    #[test]
    fn test_gate_allowed_paths_stored() {
        let dir = tempfile::tempdir().unwrap();
        let dm = DirectoryManager::new(dir.path());
        dm.create_workstream("proj").unwrap();

        let gate = WorkstreamFsGate::path_only(&dm, "proj", "s1");

        // Should have both production/ and work/ in allowed_paths
        assert!(!gate.allowed_paths.is_empty());
        let paths_str: Vec<String> = gate
            .allowed_paths
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();
        assert!(
            paths_str.iter().any(|p| p.contains("production")),
            "Should contain production path"
        );
        assert!(
            paths_str.iter().any(|p| p.contains("work")),
            "Should contain work path"
        );
    }
}
