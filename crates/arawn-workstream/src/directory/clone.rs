//! Git clone operations for workstreams.

use std::fs;
use std::path::Path;
use std::process::Command;

use super::{CloneResult, DirectoryError, DirectoryManager, DirectoryResult};

impl DirectoryManager {
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
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::super::DirectoryError;
    use super::DirectoryManager;

    fn setup() -> (tempfile::TempDir, DirectoryManager) {
        let dir = tempfile::tempdir().unwrap();
        let manager = DirectoryManager::new(dir.path());
        (dir, manager)
    }

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

    // ── get_head_commit Tests ──────────────────────────────────────────

    #[test]
    fn test_get_head_commit_on_local_repo() {
        if !DirectoryManager::is_git_available() {
            return;
        }

        let tmp = tempfile::tempdir().unwrap();
        let repo_path = tmp.path().join("test-repo");
        fs::create_dir_all(&repo_path).unwrap();

        // Initialize a git repo with a commit
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(&repo_path)
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(&repo_path)
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(&repo_path)
            .output()
            .unwrap();
        fs::write(repo_path.join("README.md"), "# Test\n").unwrap();
        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(&repo_path)
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["commit", "-m", "initial"])
            .current_dir(&repo_path)
            .output()
            .unwrap();

        let commit = DirectoryManager::get_head_commit(&repo_path).unwrap();
        assert!(!commit.is_empty());
        assert_ne!(commit, "unknown");
        // SHA-1 hex hash is 40 chars
        assert_eq!(commit.len(), 40);
        assert!(commit.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_get_head_commit_empty_repo() {
        if !DirectoryManager::is_git_available() {
            return;
        }

        let tmp = tempfile::tempdir().unwrap();
        let repo_path = tmp.path().join("empty-repo");
        fs::create_dir_all(&repo_path).unwrap();

        // Init but don't commit
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(&repo_path)
            .output()
            .unwrap();

        // HEAD doesn't exist yet, should return "unknown"
        let commit = DirectoryManager::get_head_commit(&repo_path).unwrap();
        assert_eq!(commit, "unknown");
    }

    // ── Local Clone Success Path ───────────────────────────────────────

    #[test]
    fn test_clone_local_repo_success() {
        if !DirectoryManager::is_git_available() {
            return;
        }

        // Create a local git repo to clone from
        let source_tmp = tempfile::tempdir().unwrap();
        let source_path = source_tmp.path().join("source-repo");
        fs::create_dir_all(&source_path).unwrap();

        std::process::Command::new("git")
            .args(["init"])
            .current_dir(&source_path)
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(&source_path)
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(&source_path)
            .output()
            .unwrap();
        fs::write(source_path.join("hello.txt"), "Hello, world!\n").unwrap();
        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(&source_path)
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["commit", "-m", "initial commit"])
            .current_dir(&source_path)
            .output()
            .unwrap();

        // Now clone it via the DirectoryManager
        let (_dir, manager) = setup();
        manager.create_workstream("my-ws").unwrap();

        let result = manager
            .clone_repo(
                "my-ws",
                source_path.to_str().unwrap(),
                Some("cloned"),
            )
            .unwrap();

        assert!(result.path.exists());
        assert!(result.path.join(".git").is_dir());
        assert!(result.path.join("hello.txt").exists());
        assert!(!result.commit.is_empty());
        assert_ne!(result.commit, "unknown");
        assert_eq!(result.commit.len(), 40);
    }

    #[test]
    fn test_clone_local_repo_derives_name_from_url() {
        if !DirectoryManager::is_git_available() {
            return;
        }

        // Create a local git repo
        let source_tmp = tempfile::tempdir().unwrap();
        let source_path = source_tmp.path().join("my-project");
        fs::create_dir_all(&source_path).unwrap();

        std::process::Command::new("git")
            .args(["init"])
            .current_dir(&source_path)
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(&source_path)
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(&source_path)
            .output()
            .unwrap();
        fs::write(source_path.join("f.txt"), "x").unwrap();
        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(&source_path)
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["commit", "-m", "init"])
            .current_dir(&source_path)
            .output()
            .unwrap();

        let (_dir, manager) = setup();
        manager.create_workstream("ws").unwrap();

        // Clone without custom name — should derive "my-project" from the path
        let result = manager
            .clone_repo("ws", source_path.to_str().unwrap(), None)
            .unwrap();

        assert!(result.path.ends_with("production/my-project"));
        assert!(result.path.exists());
    }

    // ── repo_name_from_url edge cases ──────────────────────────────────

    #[test]
    fn test_repo_name_from_url_trailing_slash() {
        assert_eq!(
            DirectoryManager::repo_name_from_url("https://github.com/user/repo/"),
            "repo" // last segment is empty, falls back; actually "/" splits to ["", "repo", ""]
        );
    }

    #[test]
    fn test_repo_name_from_url_bare_name() {
        assert_eq!(
            DirectoryManager::repo_name_from_url("my-repo.git"),
            "my-repo"
        );
        assert_eq!(
            DirectoryManager::repo_name_from_url("my-repo"),
            "my-repo"
        );
    }
}
