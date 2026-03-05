//! File operations: promote and export.

use std::fs;
use std::path::{Path, PathBuf};

use super::{DirectoryError, DirectoryManager, DirectoryResult, ExportResult, PromoteResult};

impl DirectoryManager {
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
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
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
        if let Some(parent) = dest_full.parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent)?;
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
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use super::super::DirectoryError;
    use super::DirectoryManager;

    fn setup() -> (tempfile::TempDir, DirectoryManager) {
        let dir = tempfile::tempdir().unwrap();
        let manager = DirectoryManager::new(dir.path());
        (dir, manager)
    }

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
            .promote(
                "test-project",
                Path::new("draft.txt"),
                Path::new("final.txt"),
            )
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

        assert!(
            result
                .path
                .ends_with("production/blog/posts/2024/article.md")
        );
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
            .promote("nonexistent", Path::new("file.txt"), Path::new("file.txt"))
            .unwrap_err();

        assert!(matches!(err, DirectoryError::WorkstreamNotFound(_)));
    }

    #[test]
    fn test_promote_invalid_workstream_name() {
        let (_dir, manager) = setup();

        let err = manager
            .promote("../escape", Path::new("file.txt"), Path::new("file.txt"))
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
}
