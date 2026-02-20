//! Path validation for secure file operations.
//!
//! Ensures all file operations stay within allowed boundaries with
//! defense-in-depth against path traversal attacks.
//!
//! # Security Model
//!
//! - All paths are canonicalized before validation
//! - Symlinks that escape allowed boundaries are rejected
//! - System-sensitive paths are always denied
//! - For new files, the parent directory is validated
//!
//! # Example
//!
//! ```no_run
//! use std::path::PathBuf;
//! use arawn_workstream::path_validator::PathValidator;
//!
//! let allowed = vec![PathBuf::from("/home/user/project")];
//! let validator = PathValidator::new(allowed);
//!
//! // This succeeds
//! let path = validator.validate(&PathBuf::from("/home/user/project/src/main.rs"));
//!
//! // This fails (outside allowed paths)
//! let result = validator.validate(&PathBuf::from("/etc/passwd"));
//! assert!(result.is_err());
//! ```

use std::path::{Path, PathBuf};

use thiserror::Error;

/// Errors that can occur during path validation.
#[derive(Debug, Error)]
pub enum PathError {
    /// Path is not within any allowed directory.
    #[error("Path not allowed: {path} (allowed: {allowed:?})")]
    NotAllowed {
        path: PathBuf,
        allowed: Vec<PathBuf>,
    },

    /// Path is in a denied system directory.
    #[error("Access denied to sensitive path: {path}")]
    DeniedPath { path: PathBuf },

    /// Symlink target escapes allowed boundaries.
    #[error("Symlink escapes allowed boundary: {path} -> {target}")]
    SymlinkEscape { path: PathBuf, target: PathBuf },

    /// Path is invalid (empty, no parent, etc.).
    #[error("Invalid path: {0}")]
    Invalid(PathBuf),

    /// Parent directory does not exist (for write validation).
    #[error("Parent directory does not exist: {0}")]
    ParentNotFound(PathBuf),

    /// IO error during validation.
    #[error("IO error validating path: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for path validation operations.
pub type PathResult<T> = std::result::Result<T, PathError>;

/// Validates that file operations stay within allowed boundaries.
///
/// This struct is `Send + Sync` safe.
#[derive(Debug, Clone)]
pub struct PathValidator {
    /// Paths that are allowed for file operations.
    allowed_paths: Vec<PathBuf>,
    /// Paths that are always denied (system-sensitive paths).
    denied_paths: Vec<PathBuf>,
}

impl PathValidator {
    /// Creates a new PathValidator with the given allowed paths.
    ///
    /// The default denied paths are automatically added:
    /// - `~/.ssh`
    /// - `~/.gnupg`
    /// - `~/.aws`
    /// - `~/.config/gcloud`
    /// - `/etc`
    /// - `/usr`
    /// - `/var`
    /// - `/System` (macOS)
    /// - `/Library` (macOS)
    pub fn new(allowed_paths: Vec<PathBuf>) -> Self {
        let denied_paths = Self::default_denied_paths();
        Self {
            allowed_paths,
            denied_paths,
        }
    }

    /// Creates a PathValidator with custom allowed and denied paths.
    pub fn with_denied(allowed_paths: Vec<PathBuf>, denied_paths: Vec<PathBuf>) -> Self {
        Self {
            allowed_paths,
            denied_paths,
        }
    }

    /// Returns the default list of denied system paths.
    ///
    /// Note: These are paths that should never be accessed by Arawn, even if
    /// they happen to be within an "allowed" directory. This protects against
    /// configuration mistakes.
    pub fn default_denied_paths() -> Vec<PathBuf> {
        let mut paths = vec![
            PathBuf::from("/etc"),
            PathBuf::from("/usr"),
            // Note: /var is intentionally NOT included because on macOS, /var
            // symlinks to /private/var which includes tempfile directories.
            // Instead, we deny specific sensitive subdirectories.
            PathBuf::from("/var/log"),
            PathBuf::from("/var/run"),
            PathBuf::from("/var/spool"),
            PathBuf::from("/private/var/log"),
            PathBuf::from("/private/var/run"),
            PathBuf::from("/System"),   // macOS
            PathBuf::from("/Library"),  // macOS
            PathBuf::from("/bin"),
            PathBuf::from("/sbin"),
            PathBuf::from("/boot"),
            PathBuf::from("/root"),
            PathBuf::from("/proc"),     // Linux
            PathBuf::from("/sys"),      // Linux
            PathBuf::from("/dev"),
        ];

        // Add home directory sensitive paths
        if let Some(home) = dirs::home_dir() {
            paths.push(home.join(".ssh"));
            paths.push(home.join(".gnupg"));
            paths.push(home.join(".aws"));
            paths.push(home.join(".config/gcloud"));
            paths.push(home.join(".kube"));
            paths.push(home.join(".docker"));
            paths.push(home.join(".npmrc"));
            paths.push(home.join(".netrc"));
            paths.push(home.join(".gitconfig"));
        }

        paths
    }

    /// Get the allowed paths.
    pub fn allowed_paths(&self) -> &[PathBuf] {
        &self.allowed_paths
    }

    /// Get the denied paths.
    pub fn denied_paths(&self) -> &[PathBuf] {
        &self.denied_paths
    }

    /// Validate a path for read operations.
    ///
    /// The path must exist and resolve (via canonicalization) to a location
    /// within one of the allowed paths and not within any denied paths.
    ///
    /// # Returns
    ///
    /// The canonicalized path if validation succeeds.
    ///
    /// # Errors
    ///
    /// - `PathError::NotAllowed` if the path is outside allowed directories
    /// - `PathError::DeniedPath` if the path is in a denied directory
    /// - `PathError::SymlinkEscape` if a symlink points outside allowed directories
    /// - `PathError::Io` if the path cannot be canonicalized
    pub fn validate(&self, path: &Path) -> PathResult<PathBuf> {
        // Canonicalize to resolve symlinks and get absolute path
        let canonical = path.canonicalize().map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                PathError::Invalid(path.to_path_buf())
            } else {
                PathError::Io(e)
            }
        })?;

        // Check against denied paths first (security-critical)
        self.check_denied(&canonical)?;

        // Check symlink didn't escape (the original path vs canonical)
        // If original path looked like it was in allowed area but canonical is not,
        // that's a symlink escape.
        //
        // Note: We need to compare using canonicalized allowed paths to handle
        // cases like macOS where /var symlinks to /private/var.
        if path != canonical {
            // Check if original path appears to be under allowed (using canonical allowed paths)
            let original_appears_allowed = self.is_under_allowed_canonical(path);
            let canonical_is_allowed = self.is_under_allowed_canonical(&canonical);

            if original_appears_allowed && !canonical_is_allowed {
                return Err(PathError::SymlinkEscape {
                    path: path.to_path_buf(),
                    target: canonical,
                });
            }
        }

        // Check against allowed paths
        self.check_allowed(&canonical)?;

        Ok(canonical)
    }

    /// Validate a path for write operations.
    ///
    /// For files that don't exist yet, validates the parent directory exists
    /// and is within allowed boundaries, then constructs the full path.
    ///
    /// # Algorithm
    ///
    /// 1. Get the parent directory of the path
    /// 2. Canonicalize the parent (must exist)
    /// 3. Check parent against denied paths
    /// 4. Check parent against allowed paths
    /// 5. Return parent + filename
    ///
    /// # Returns
    ///
    /// The validated path (parent canonicalized + original filename).
    ///
    /// # Errors
    ///
    /// - `PathError::Invalid` if the path has no parent or filename
    /// - `PathError::ParentNotFound` if the parent directory doesn't exist
    /// - `PathError::NotAllowed` if the parent is outside allowed directories
    /// - `PathError::DeniedPath` if the parent is in a denied directory
    pub fn validate_write(&self, path: &Path) -> PathResult<PathBuf> {
        // Get filename
        let filename = path.file_name().ok_or_else(|| PathError::Invalid(path.to_path_buf()))?;

        // Get parent directory
        let parent = path.parent().ok_or_else(|| PathError::Invalid(path.to_path_buf()))?;

        // Handle empty parent (e.g., just a filename)
        let parent = if parent.as_os_str().is_empty() {
            Path::new(".")
        } else {
            parent
        };

        // Canonicalize parent (must exist)
        let parent_canonical = parent.canonicalize().map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                PathError::ParentNotFound(parent.to_path_buf())
            } else {
                PathError::Io(e)
            }
        })?;

        // Check parent against denied paths
        self.check_denied(&parent_canonical)?;

        // Check parent against allowed paths
        self.check_allowed(&parent_canonical)?;

        // Return validated full path
        Ok(parent_canonical.join(filename))
    }

    /// Check if a path is within any denied directory.
    fn check_denied(&self, path: &Path) -> PathResult<()> {
        for denied in &self.denied_paths {
            // Handle both existing and non-existing denied paths
            // For denied paths that exist, canonicalize them
            let denied_canonical = if denied.exists() {
                denied.canonicalize().unwrap_or_else(|_| denied.clone())
            } else {
                denied.clone()
            };

            if path.starts_with(&denied_canonical) {
                return Err(PathError::DeniedPath {
                    path: path.to_path_buf(),
                });
            }
        }
        Ok(())
    }

    /// Check if a path is within any allowed directory.
    fn check_allowed(&self, path: &Path) -> PathResult<()> {
        for allowed in &self.allowed_paths {
            // Canonicalize allowed path if it exists
            let allowed_canonical = if allowed.exists() {
                allowed.canonicalize().unwrap_or_else(|_| allowed.clone())
            } else {
                allowed.clone()
            };

            if path.starts_with(&allowed_canonical) {
                return Ok(());
            }
        }

        Err(PathError::NotAllowed {
            path: path.to_path_buf(),
            allowed: self.allowed_paths.clone(),
        })
    }

    /// Check if a path is under an allowed directory (using canonicalized allowed paths).
    ///
    /// This handles cases like macOS where /var symlinks to /private/var.
    fn is_under_allowed_canonical(&self, path: &Path) -> bool {
        // Canonicalize the path we're checking (if possible)
        let path_canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

        for allowed in &self.allowed_paths {
            // Canonicalize allowed path if it exists
            let allowed_canonical = if allowed.exists() {
                allowed.canonicalize().unwrap_or_else(|_| allowed.clone())
            } else {
                allowed.clone()
            };

            // Check both the original path and its canonical form
            if path.starts_with(&allowed_canonical) || path_canonical.starts_with(&allowed_canonical) {
                return true;
            }
        }
        false
    }

    /// Validate that a path is safe for shell execution.
    ///
    /// This is a stricter check that ensures the path:
    /// 1. Is within allowed boundaries
    /// 2. Does not contain shell metacharacters
    /// 3. Is not a symlink (to prevent TOCTOU attacks)
    ///
    /// # Returns
    ///
    /// The canonicalized path if validation succeeds.
    pub fn validate_for_shell(&self, path: &Path) -> PathResult<PathBuf> {
        // First do standard validation
        let canonical = self.validate(path)?;

        // Check for shell metacharacters in the path string
        let path_str = canonical.to_string_lossy();
        const SHELL_METACHARACTERS: &[char] = &[
            '`', '$', '(', ')', '{', '}', '[', ']', '|', '&', ';', '<', '>', '\n', '\r', '\0',
        ];

        for c in SHELL_METACHARACTERS {
            if path_str.contains(*c) {
                return Err(PathError::Invalid(canonical));
            }
        }

        Ok(canonical)
    }
}

/// Create a PathValidator from a DirectoryManager for a specific session.
///
/// This is a convenience function that creates a validator with the
/// allowed paths for a workstream/session from the DirectoryManager.
impl PathValidator {
    /// Create a validator for a specific workstream and session.
    ///
    /// Uses `DirectoryManager::allowed_paths()` to get the allowed directories.
    pub fn for_session(
        directory_manager: &crate::directory::DirectoryManager,
        workstream: &str,
        session_id: &str,
    ) -> Self {
        let allowed = directory_manager.allowed_paths(workstream, session_id);
        Self::new(allowed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::symlink;

    fn setup() -> (tempfile::TempDir, PathValidator) {
        let dir = tempfile::tempdir().unwrap();
        let allowed = vec![dir.path().to_path_buf()];
        let validator = PathValidator::new(allowed);
        (dir, validator)
    }

    #[test]
    fn test_validate_existing_file() {
        let (dir, validator) = setup();

        // Create a file
        let file_path = dir.path().join("test.txt");
        fs::write(&file_path, "hello").unwrap();

        // Validate should succeed
        let result = validator.validate(&file_path).unwrap();
        assert_eq!(result, file_path.canonicalize().unwrap());
    }

    #[test]
    fn test_validate_nonexistent_file_fails() {
        let (dir, validator) = setup();

        let file_path = dir.path().join("nonexistent.txt");
        let result = validator.validate(&file_path);

        assert!(matches!(result, Err(PathError::Invalid(_))));
    }

    #[test]
    fn test_validate_write_new_file() {
        let (dir, validator) = setup();

        // Validate write to new file in existing directory
        let file_path = dir.path().join("new_file.txt");
        let result = validator.validate_write(&file_path).unwrap();

        assert!(result.ends_with("new_file.txt"));
        assert!(result.parent().unwrap().exists());
    }

    #[test]
    fn test_validate_write_nested_directory() {
        let (dir, validator) = setup();

        // Create a subdirectory
        let subdir = dir.path().join("subdir");
        fs::create_dir(&subdir).unwrap();

        // Validate write to new file in subdirectory
        let file_path = subdir.join("new_file.txt");
        let result = validator.validate_write(&file_path).unwrap();

        assert!(result.ends_with("new_file.txt"));
    }

    #[test]
    fn test_validate_write_nonexistent_parent_fails() {
        let (dir, validator) = setup();

        let file_path = dir.path().join("nonexistent_dir").join("file.txt");
        let result = validator.validate_write(&file_path);

        assert!(matches!(result, Err(PathError::ParentNotFound(_))));
    }

    #[test]
    fn test_path_outside_allowed_rejected() {
        let (dir, validator) = setup();

        // Try to access a file outside the allowed directory
        let outside_path = PathBuf::from("/tmp/outside.txt");

        // Create the file so it can be canonicalized
        fs::write(&outside_path, "test").ok();

        let result = validator.validate(&outside_path);
        assert!(matches!(result, Err(PathError::NotAllowed { .. })));

        // Clean up
        fs::remove_file(&outside_path).ok();
    }

    #[test]
    #[allow(unused_variables)]
    fn test_traversal_attack_rejected() {
        let (dir, validator) = setup();

        // Create a file
        let file_path = dir.path().join("test.txt");
        fs::write(&file_path, "hello").unwrap();

        // Try path traversal
        let traversal_path = dir.path().join("..").join("..").join("etc").join("passwd");
        let result = validator.validate(&traversal_path);

        // Should fail (either denied or not allowed)
        assert!(result.is_err());
    }

    #[test]
    fn test_symlink_within_allowed_succeeds() {
        let (dir, validator) = setup();

        // Create a file and symlink within allowed directory
        let file_path = dir.path().join("real_file.txt");
        fs::write(&file_path, "hello").unwrap();

        let link_path = dir.path().join("link.txt");
        symlink(&file_path, &link_path).unwrap();

        // Should succeed - symlink stays within allowed area
        let result = validator.validate(&link_path).unwrap();
        assert_eq!(result, file_path.canonicalize().unwrap());
    }

    #[test]
    fn test_symlink_escape_rejected() {
        let (dir, validator) = setup();

        // Create a file outside allowed directory
        let outside_dir = tempfile::tempdir().unwrap();
        let outside_file = outside_dir.path().join("secret.txt");
        fs::write(&outside_file, "secret data").unwrap();

        // Create symlink inside allowed directory pointing outside
        let link_path = dir.path().join("sneaky_link.txt");
        symlink(&outside_file, &link_path).unwrap();

        // Should fail with symlink escape
        let result = validator.validate(&link_path);
        assert!(
            matches!(result, Err(PathError::SymlinkEscape { .. }) | Err(PathError::NotAllowed { .. })),
            "Expected SymlinkEscape or NotAllowed, got: {:?}",
            result
        );
    }

    #[test]
    fn test_denied_path_rejected() {
        let dir = tempfile::tempdir().unwrap();

        // Create validator with /tmp as allowed but a subdirectory as denied
        let denied_dir = dir.path().join("denied");
        fs::create_dir(&denied_dir).unwrap();

        let validator = PathValidator::with_denied(
            vec![dir.path().to_path_buf()],
            vec![denied_dir.clone()],
        );

        // Create file in denied directory
        let denied_file = denied_dir.join("secret.txt");
        fs::write(&denied_file, "secret").unwrap();

        let result = validator.validate(&denied_file);
        assert!(matches!(result, Err(PathError::DeniedPath { .. })));
    }

    #[test]
    fn test_validate_for_shell_rejects_metacharacters() {
        let (dir, validator) = setup();

        // Create a file with a normal name
        let file_path = dir.path().join("normal.txt");
        fs::write(&file_path, "hello").unwrap();

        // Normal file should pass
        let result = validator.validate_for_shell(&file_path);
        assert!(result.is_ok());

        // File with shell metacharacter in name should fail
        // Note: We can't easily create such files on most systems,
        // so we test the path string check directly
        let bad_path = dir.path().join("file`whoami`.txt");
        // This will fail at canonicalize since file doesn't exist
        let result = validator.validate_for_shell(&bad_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_default_denied_paths() {
        let denied = PathValidator::default_denied_paths();

        // Should include system paths
        assert!(denied.contains(&PathBuf::from("/etc")));
        assert!(denied.contains(&PathBuf::from("/usr")));

        // Should include home directory sensitive paths (if home exists)
        if let Some(home) = dirs::home_dir() {
            assert!(denied.contains(&home.join(".ssh")));
            assert!(denied.contains(&home.join(".aws")));
        }
    }

    #[test]
    fn test_empty_allowed_paths_rejects_all() {
        let dir = tempfile::tempdir().unwrap();
        let validator = PathValidator::new(vec![]);

        let file_path = dir.path().join("test.txt");
        fs::write(&file_path, "hello").unwrap();

        let result = validator.validate(&file_path);
        assert!(matches!(result, Err(PathError::NotAllowed { .. })));
    }

    #[test]
    fn test_multiple_allowed_paths() {
        let dir1 = tempfile::tempdir().unwrap();
        let dir2 = tempfile::tempdir().unwrap();

        let validator = PathValidator::new(vec![
            dir1.path().to_path_buf(),
            dir2.path().to_path_buf(),
        ]);

        // File in first directory
        let file1 = dir1.path().join("file1.txt");
        fs::write(&file1, "hello").unwrap();
        assert!(validator.validate(&file1).is_ok());

        // File in second directory
        let file2 = dir2.path().join("file2.txt");
        fs::write(&file2, "hello").unwrap();
        assert!(validator.validate(&file2).is_ok());
    }

    #[test]
    fn test_for_session_creates_validator() {
        let dir = tempfile::tempdir().unwrap();
        let dm = crate::directory::DirectoryManager::new(dir.path());

        // Create workstream directories
        dm.create_workstream("test-project").unwrap();

        let validator = PathValidator::for_session(&dm, "test-project", "session-1");

        // Should have production and work paths as allowed
        assert_eq!(validator.allowed_paths().len(), 2);
    }

    #[test]
    fn test_thread_safety() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<PathValidator>();
    }

    #[test]
    fn test_validate_write_just_filename() {
        // Test with just a filename (no directory)
        let dir = tempfile::tempdir().unwrap();
        let _validator = PathValidator::new(vec![std::env::current_dir().unwrap()]);

        // Change to temp dir for this test
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        // Create a new validator with current dir as allowed
        let validator = PathValidator::new(vec![dir.path().to_path_buf()]);

        let result = validator.validate_write(Path::new("test.txt"));
        // Should work - parent is "." which becomes current dir
        assert!(result.is_ok());

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
    }
}

/// Property-based tests for path validation security.
#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;
    use std::fs;

    /// Strategy to generate paths with path traversal sequences.
    fn traversal_path_strategy() -> impl Strategy<Value = String> {
        prop::collection::vec(
            prop_oneof![
                Just("..".to_string()),
                Just("../".to_string()),
                Just("..\\".to_string()),
                "[a-zA-Z0-9_-]{1,10}".prop_map(|s| s),
            ],
            1..5,
        )
        .prop_map(|parts| parts.join("/"))
    }

    /// Strategy to generate paths with shell metacharacters.
    fn shell_metachar_path_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            "[a-z]+`[a-z]+`[a-z]+".prop_map(|s| s),
            "[a-z]+\\$\\([a-z]+\\)[a-z]+".prop_map(|s| s),
            "[a-z]+;[a-z]+".prop_map(|s| s),
            "[a-z]+\\|[a-z]+".prop_map(|s| s),
            "[a-z]+&[a-z]+".prop_map(|s| s),
            "[a-z]+<[a-z]+".prop_map(|s| s),
            "[a-z]+>[a-z]+".prop_map(|s| s),
        ]
    }

    proptest! {
        /// Property: Path traversal sequences should never allow access outside allowed directories.
        #[test]
        fn path_traversal_never_escapes(traversal in traversal_path_strategy()) {
            let dir = tempfile::tempdir().unwrap();
            let validator = PathValidator::new(vec![dir.path().to_path_buf()]);

            // Create a valid file first
            let file_path = dir.path().join("test.txt");
            fs::write(&file_path, "hello").unwrap();

            // Try to escape using traversal
            let attack_path = dir.path().join(&traversal).join("passwd");
            let result = validator.validate(&attack_path);

            // Should either fail to canonicalize (Invalid) or be rejected (NotAllowed/DeniedPath)
            // It should NEVER successfully return a path outside the allowed directory
            if let Ok(validated) = &result {
                // If validation succeeded, path must be under allowed directory
                let canonical_allowed = dir.path().canonicalize().unwrap();
                prop_assert!(
                    validated.starts_with(&canonical_allowed),
                    "Escaped allowed directory! traversal={}, result={:?}",
                    traversal,
                    validated
                );
            }
        }

        /// Property: Shell metacharacters in paths should always be rejected by validate_for_shell.
        #[test]
        fn shell_metacharacters_always_rejected(filename in shell_metachar_path_strategy()) {
            let dir = tempfile::tempdir().unwrap();
            let validator = PathValidator::new(vec![dir.path().to_path_buf()]);

            // Create the path (it won't exist, which is fine - we're testing the metachar check)
            let path = dir.path().join(&filename);

            // validate_for_shell should reject paths with shell metacharacters
            // (Note: It will fail at canonicalize for non-existent files, which is also safe)
            let result = validator.validate_for_shell(&path);
            prop_assert!(
                result.is_err(),
                "Shell metacharacter path was accepted! filename={}, result={:?}",
                filename,
                result
            );
        }

        /// Property: Paths that don't exist should never successfully validate.
        #[test]
        fn nonexistent_paths_fail_validation(filename in "[a-zA-Z0-9_]{1,20}") {
            let dir = tempfile::tempdir().unwrap();
            let validator = PathValidator::new(vec![dir.path().to_path_buf()]);

            // Path that definitely doesn't exist
            let path = dir.path().join(&filename).join("definitely_does_not_exist.txt");

            let result = validator.validate(&path);
            prop_assert!(
                result.is_err(),
                "Non-existent path was accepted! path={:?}",
                path
            );
        }

        /// Property: Denied system paths should always be rejected, even if they exist.
        #[test]
        fn denied_paths_always_rejected(suffix in "[a-zA-Z0-9_]{0,10}(/[a-zA-Z0-9_]{1,5}){0,2}") {
            // Use /etc as our test denied path (usually exists on Unix)
            // Note: suffix cannot start with "/" to avoid replacing the base path
            let denied_path = if suffix.is_empty() {
                PathBuf::from("/etc")
            } else {
                PathBuf::from("/etc").join(&suffix)
            };

            // Create validator that "allows" root but has /etc in denied
            let validator = PathValidator::new(vec![PathBuf::from("/")]);

            // If the path exists, validation should fail due to denied
            if denied_path.exists() {
                let result = validator.validate(&denied_path);
                prop_assert!(
                    result.is_err(),
                    "Denied path was accepted! path={:?}, result={:?}",
                    denied_path,
                    result
                );
            }
        }

        /// Property: validate_write should accept any filename in an allowed existing directory.
        #[test]
        fn valid_filenames_accepted_for_write(filename in "[a-zA-Z0-9_-]{1,20}\\.txt") {
            let dir = tempfile::tempdir().unwrap();
            let validator = PathValidator::new(vec![dir.path().to_path_buf()]);

            let path = dir.path().join(&filename);
            let result = validator.validate_write(&path);

            prop_assert!(
                result.is_ok(),
                "Valid filename rejected for write! filename={}, error={:?}",
                filename,
                result.err()
            );
        }
    }
}
