//! Usage statistics and cleanup operations.

use std::fs;
use std::path::{Path, PathBuf};

use super::{
    DirectoryError, DirectoryManager, DirectoryResult, ManualCleanupResult, PRODUCTION_DIR,
    SCRATCH_WORKSTREAM, SESSIONS_DIR, SessionUsage, UsageStats, WORK_DIR,
};

impl DirectoryManager {
    // ── Usage Statistics ──────────────────────────────────────────────────────

    /// Default warning threshold for work directory (500MB).
    const WORK_WARNING_THRESHOLD: u64 = 500 * 1024 * 1024;
    /// Default warning threshold for production directory (1GB).
    const PRODUCTION_WARNING_THRESHOLD: u64 = 1024 * 1024 * 1024;
    /// Default warning threshold for session work directory (100MB).
    const SESSION_WARNING_THRESHOLD: u64 = 100 * 1024 * 1024;

    /// Calculate disk usage statistics for a workstream.
    ///
    /// Returns detailed usage information including:
    /// - Production directory size
    /// - Work directory size
    /// - Per-session breakdown (for scratch workstream only)
    /// - Warnings if thresholds are exceeded
    ///
    /// # Arguments
    ///
    /// * `workstream` - The workstream name to analyze
    ///
    /// # Returns
    ///
    /// * `Ok(UsageStats)` with usage breakdown and any warnings
    /// * `Err(DirectoryError::InvalidName)` if the workstream name is invalid
    /// * `Err(DirectoryError::WorkstreamNotFound)` if the workstream doesn't exist
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arawn_workstream::directory::DirectoryManager;
    ///
    /// let manager = DirectoryManager::default();
    /// let stats = manager.get_usage("my-blog").unwrap();
    /// println!("Production: {:.2} MB", stats.production_mb());
    /// println!("Work: {:.2} MB", stats.work_mb());
    /// if !stats.warnings.is_empty() {
    ///     println!("Warnings: {:?}", stats.warnings);
    /// }
    /// ```
    pub fn get_usage(&self, workstream: &str) -> DirectoryResult<UsageStats> {
        // Validate workstream name
        if !Self::is_valid_name(workstream) {
            return Err(DirectoryError::InvalidName(workstream.to_string()));
        }

        // Check workstream exists
        let ws_path = self.workstream_path(workstream);
        if !ws_path.exists() {
            return Err(DirectoryError::WorkstreamNotFound(workstream.to_string()));
        }

        let mut warnings = Vec::new();

        // Calculate production directory size
        let production_path = ws_path.join(PRODUCTION_DIR);
        let production_bytes = Self::dir_size(&production_path).unwrap_or(0);
        if production_bytes > Self::PRODUCTION_WARNING_THRESHOLD {
            warnings.push(format!(
                "production directory exceeds {:.0}MB limit ({:.2}MB)",
                Self::PRODUCTION_WARNING_THRESHOLD as f64 / 1_048_576.0,
                production_bytes as f64 / 1_048_576.0
            ));
        }

        // Calculate work directory size and session breakdown
        let (work_bytes, sessions) = if workstream == SCRATCH_WORKSTREAM {
            // For scratch, enumerate sessions
            let sessions_path = ws_path.join(SESSIONS_DIR);
            let (total, sessions) = self.get_session_usages(&sessions_path)?;

            // Check individual session thresholds
            for session in &sessions {
                if session.bytes > Self::SESSION_WARNING_THRESHOLD {
                    warnings.push(format!(
                        "session {} exceeds {:.0}MB limit ({:.2}MB)",
                        session.id,
                        Self::SESSION_WARNING_THRESHOLD as f64 / 1_048_576.0,
                        session.bytes as f64 / 1_048_576.0
                    ));
                }
            }

            (total, sessions)
        } else {
            // For named workstreams, just calculate work directory size
            let work_path = ws_path.join(WORK_DIR);
            let work_bytes = Self::dir_size(&work_path).unwrap_or(0);
            if work_bytes > Self::WORK_WARNING_THRESHOLD {
                warnings.push(format!(
                    "work directory exceeds {:.0}MB limit ({:.2}MB)",
                    Self::WORK_WARNING_THRESHOLD as f64 / 1_048_576.0,
                    work_bytes as f64 / 1_048_576.0
                ));
            }
            (work_bytes, Vec::new())
        };

        let total_bytes = production_bytes + work_bytes;

        Ok(UsageStats {
            production_bytes,
            work_bytes,
            sessions,
            total_bytes,
            warnings,
        })
    }

    /// Calculate disk usage for all sessions in a directory.
    ///
    /// Returns the total bytes and per-session breakdown.
    fn get_session_usages(
        &self,
        sessions_path: &Path,
    ) -> DirectoryResult<(u64, Vec<SessionUsage>)> {
        let mut sessions = Vec::new();
        let mut total = 0u64;

        if !sessions_path.exists() {
            return Ok((0, sessions));
        }

        for entry in fs::read_dir(sessions_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let session_id = entry.file_name().to_string_lossy().to_string();

                let work_path = path.join(WORK_DIR);
                let bytes = Self::dir_size(&work_path).unwrap_or(0);

                total += bytes;
                sessions.push(SessionUsage {
                    id: session_id,
                    bytes,
                });
            }
        }

        // Sort by bytes descending (largest first)
        sessions.sort_by(|a, b| b.bytes.cmp(&a.bytes));

        Ok((total, sessions))
    }

    /// Calculate the total size of a directory recursively.
    fn dir_size(path: &Path) -> DirectoryResult<u64> {
        use walkdir::WalkDir;

        if !path.exists() {
            return Ok(0);
        }

        let mut size = 0u64;
        for entry in WalkDir::new(path).follow_links(false) {
            let entry =
                entry.map_err(|e| DirectoryError::Io(std::io::Error::other(e.to_string())))?;

            if entry.file_type().is_file() {
                size += entry.metadata().map(|m| m.len()).unwrap_or(0);
            }
        }

        Ok(size)
    }

    // ── Manual Cleanup ───────────────────────────────────────────────────────

    /// Threshold for requiring confirmation (>100 files).
    const CLEANUP_CONFIRMATION_THRESHOLD: usize = 100;

    /// Clean up files in the work directory.
    ///
    /// This method only cleans the `work/` directory, never `production/`.
    /// For scratch workstreams, it cleans session work directories.
    ///
    /// # Arguments
    ///
    /// * `workstream` - The workstream name.
    /// * `older_than_days` - Optional: only delete files older than this many days.
    /// * `confirmed` - Whether the user has confirmed large deletions.
    ///
    /// # Returns
    ///
    /// A `ManualCleanupResult` containing:
    /// - `deleted_files`: Number of files actually deleted.
    /// - `freed_bytes`: Total bytes freed.
    /// - `pending_files`: Files that would be deleted if confirmed.
    /// - `requires_confirmation`: True if >100 files would be deleted and not confirmed.
    ///
    /// If `requires_confirmation` is true and `confirmed` is false, no files are deleted.
    /// Call again with `confirmed=true` to proceed with the deletion.
    ///
    /// # Errors
    ///
    /// - `DirectoryError::InvalidName` if the workstream name is invalid.
    /// - `DirectoryError::WorkstreamNotFound` if the workstream doesn't exist.
    /// - `DirectoryError::Io` if file operations fail.
    ///
    /// # Safety
    ///
    /// This method NEVER deletes from `production/` - only from `work/` directories.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use arawn_workstream::directory::DirectoryManager;
    ///
    /// let manager = DirectoryManager::default();
    ///
    /// // First call - check if confirmation is needed
    /// let result = manager.cleanup_work("my-project", Some(7), false).unwrap();
    /// if result.requires_confirmation {
    ///     println!("Would delete {} files ({:.2} MB)", result.pending_files, result.freed_mb());
    ///     // Get user confirmation, then:
    ///     let result = manager.cleanup_work("my-project", Some(7), true).unwrap();
    /// }
    /// ```
    pub fn cleanup_work(
        &self,
        workstream: &str,
        older_than_days: Option<u32>,
        confirmed: bool,
    ) -> DirectoryResult<ManualCleanupResult> {
        use std::time::{Duration, SystemTime};
        use walkdir::WalkDir;

        // Validate workstream name
        if !Self::is_valid_name(workstream) {
            return Err(DirectoryError::InvalidName(workstream.to_string()));
        }

        // Check workstream exists
        if !self.workstream_exists(workstream) {
            return Err(DirectoryError::WorkstreamNotFound(workstream.to_string()));
        }

        // Calculate cutoff time if age filter specified
        let cutoff = older_than_days
            .map(|days| SystemTime::now() - Duration::from_secs(days as u64 * 86400));

        // Determine which directories to clean
        let paths_to_clean: Vec<PathBuf> = if workstream == SCRATCH_WORKSTREAM {
            // For scratch, clean all session work directories
            let sessions_path = self.workstream_path(SCRATCH_WORKSTREAM).join(SESSIONS_DIR);
            if sessions_path.exists() {
                fs::read_dir(&sessions_path)?
                    .filter_map(|e| e.ok())
                    .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
                    .map(|e| e.path().join(WORK_DIR))
                    .filter(|p| p.exists())
                    .collect()
            } else {
                Vec::new()
            }
        } else {
            // For named workstreams, clean the work/ directory
            let work_path = self.work_path(workstream);
            if work_path.exists() {
                vec![work_path]
            } else {
                Vec::new()
            }
        };

        // Collect files to delete
        let mut files_to_delete: Vec<(PathBuf, u64)> = Vec::new();

        for dir in &paths_to_clean {
            for entry in WalkDir::new(dir).follow_links(false) {
                let entry = match entry {
                    Ok(e) => e,
                    Err(_) => continue,
                };

                // Only process files
                if !entry.file_type().is_file() {
                    continue;
                }

                let path = entry.path().to_path_buf();
                let metadata = match entry.metadata() {
                    Ok(m) => m,
                    Err(_) => continue,
                };

                // Check age filter
                if let Some(cutoff_time) = cutoff
                    && let Ok(modified) = metadata.modified()
                    && modified > cutoff_time
                {
                    continue; // File is too new, skip
                }

                files_to_delete.push((path, metadata.len()));
            }
        }

        let pending_count = files_to_delete.len();
        let total_bytes: u64 = files_to_delete.iter().map(|(_, size)| size).sum();
        let requires_confirmation = pending_count > Self::CLEANUP_CONFIRMATION_THRESHOLD;

        // If confirmation required but not given, return without deleting
        if requires_confirmation && !confirmed {
            tracing::info!(
                workstream = %workstream,
                pending_files = %pending_count,
                bytes = %total_bytes,
                "Cleanup requires confirmation (>{} files)",
                Self::CLEANUP_CONFIRMATION_THRESHOLD
            );

            return Ok(ManualCleanupResult {
                deleted_files: 0,
                freed_bytes: 0,
                pending_files: pending_count,
                requires_confirmation: true,
            });
        }

        // Delete the files
        let mut deleted_count = 0usize;
        let mut freed_bytes = 0u64;

        for (path, size) in files_to_delete {
            match fs::remove_file(&path) {
                Ok(()) => {
                    deleted_count += 1;
                    freed_bytes += size;
                }
                Err(e) => {
                    tracing::warn!(
                        path = %path.display(),
                        error = %e,
                        "Failed to delete file during cleanup"
                    );
                }
            }
        }

        // Clean up empty directories
        for dir in &paths_to_clean {
            Self::remove_empty_dirs(dir);
        }

        tracing::info!(
            workstream = %workstream,
            deleted_files = %deleted_count,
            freed_bytes = %freed_bytes,
            older_than_days = ?older_than_days,
            "Work directory cleanup completed"
        );

        Ok(ManualCleanupResult {
            deleted_files: deleted_count,
            freed_bytes,
            pending_files: 0,
            requires_confirmation: false,
        })
    }

    /// Remove empty directories recursively (bottom-up).
    fn remove_empty_dirs(path: &Path) {
        use walkdir::WalkDir;

        // Collect all directories, deepest first
        let mut dirs: Vec<PathBuf> = WalkDir::new(path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_dir())
            .map(|e| e.path().to_path_buf())
            .collect();

        // Sort by depth descending (deepest first)
        dirs.sort_by_key(|b| std::cmp::Reverse(b.components().count()));

        for dir in dirs {
            // Try to remove - will fail if not empty, which is fine
            let _ = fs::remove_dir(&dir);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::super::{DirectoryError, ManualCleanupResult, SCRATCH_WORKSTREAM, UsageStats};
    use super::DirectoryManager;

    fn setup() -> (tempfile::TempDir, DirectoryManager) {
        let dir = tempfile::tempdir().unwrap();
        let manager = DirectoryManager::new(dir.path());
        (dir, manager)
    }

    fn test_get_usage_basic() {
        let (_dir, manager) = setup();

        // Create a workstream with some files
        manager.create_workstream("my-project").unwrap();

        // Add files to production
        let prod_path = manager.production_path("my-project");
        fs::write(prod_path.join("file1.txt"), "hello world").unwrap();
        fs::write(prod_path.join("file2.txt"), "test content").unwrap();

        // Add files to work
        let work_path = manager.work_path("my-project");
        fs::write(work_path.join("wip.txt"), "work in progress").unwrap();

        let stats = manager.get_usage("my-project").unwrap();

        // Verify production size (11 + 12 = 23 bytes)
        assert_eq!(stats.production_bytes, 23);
        // Verify work size (16 bytes)
        assert_eq!(stats.work_bytes, 16);
        // Verify total
        assert_eq!(stats.total_bytes, 39);
        // No sessions for named workstream
        assert!(stats.sessions.is_empty());
        // No warnings for small files
        assert!(stats.warnings.is_empty());
    }

    #[test]
    fn test_get_usage_scratch_with_sessions() {
        let (_dir, manager) = setup();

        // Create scratch workstream structure manually
        let scratch_path = manager.workstream_path(SCRATCH_WORKSTREAM);
        fs::create_dir_all(&scratch_path).unwrap();

        // Create session 1 with some files
        let session1_work = manager.create_scratch_session("session-001").unwrap();
        fs::write(session1_work.join("large.txt"), vec![b'a'; 1000]).unwrap();
        fs::write(session1_work.join("small.txt"), "tiny").unwrap();

        // Create session 2 with fewer files
        let session2_work = manager.create_scratch_session("session-002").unwrap();
        fs::write(session2_work.join("medium.txt"), vec![b'b'; 500]).unwrap();

        let stats = manager.get_usage(SCRATCH_WORKSTREAM).unwrap();

        // Verify work_bytes is the sum of all session work directories
        assert_eq!(stats.work_bytes, 1004 + 500);
        // Production should be empty for scratch
        assert_eq!(stats.production_bytes, 0);
        // Should have 2 sessions
        assert_eq!(stats.sessions.len(), 2);
        // Sessions should be sorted by size descending
        assert_eq!(stats.sessions[0].id, "session-001");
        assert_eq!(stats.sessions[0].bytes, 1004);
        assert_eq!(stats.sessions[1].id, "session-002");
        assert_eq!(stats.sessions[1].bytes, 500);
    }

    #[test]
    fn test_get_usage_empty_workstream() {
        let (_dir, manager) = setup();

        // Create a workstream with no files
        manager.create_workstream("empty-project").unwrap();

        let stats = manager.get_usage("empty-project").unwrap();

        assert_eq!(stats.production_bytes, 0);
        assert_eq!(stats.work_bytes, 0);
        assert_eq!(stats.total_bytes, 0);
        assert!(stats.sessions.is_empty());
        assert!(stats.warnings.is_empty());
    }

    #[test]
    fn test_get_usage_nonexistent_workstream() {
        let (_dir, manager) = setup();

        let err = manager.get_usage("nonexistent").unwrap_err();
        assert!(matches!(err, DirectoryError::WorkstreamNotFound(_)));
    }

    #[test]
    fn test_get_usage_invalid_name() {
        let (_dir, manager) = setup();

        let err = manager.get_usage("invalid/name").unwrap_err();
        assert!(matches!(err, DirectoryError::InvalidName(_)));
    }

    #[test]
    fn test_get_usage_nested_directories() {
        let (_dir, manager) = setup();

        manager.create_workstream("nested-project").unwrap();

        // Create nested directory structure in production
        let prod_path = manager.production_path("nested-project");
        fs::create_dir_all(prod_path.join("deep/nested/path")).unwrap();
        fs::write(prod_path.join("root.txt"), "root").unwrap();
        fs::write(prod_path.join("deep/level1.txt"), "level1").unwrap();
        fs::write(prod_path.join("deep/nested/level2.txt"), "level2").unwrap();
        fs::write(prod_path.join("deep/nested/path/level3.txt"), "level3").unwrap();

        let stats = manager.get_usage("nested-project").unwrap();

        // Should calculate total of all files recursively
        // root(4) + level1(6) + level2(6) + level3(6) = 22 bytes
        assert_eq!(stats.production_bytes, 22);
    }

    #[test]
    fn test_usage_stats_mb_conversions() {
        // Test the helper methods for MB conversion
        let stats = UsageStats {
            production_bytes: 1_048_576, // 1 MB
            work_bytes: 524_288,         // 0.5 MB
            sessions: vec![],
            total_bytes: 1_572_864, // 1.5 MB
            warnings: vec![],
        };

        assert!((stats.production_mb() - 1.0).abs() < 0.001);
        assert!((stats.work_mb() - 0.5).abs() < 0.001);
        assert!((stats.total_mb() - 1.5).abs() < 0.001);
    }

    #[test]
    fn test_dir_size_nonexistent() {
        // dir_size should return 0 for non-existent paths
        let result = DirectoryManager::dir_size(std::path::Path::new("/nonexistent/path"));
        assert_eq!(result.unwrap(), 0);
    }

    // ── Cleanup tests ─────────────────────────────────────────────────

    #[test]
    fn test_cleanup_work_basic() {
        let (_dir, manager) = setup();

        // Create workstream with files in work/
        manager.create_workstream("cleanup-test").unwrap();
        let work_path = manager.work_path("cleanup-test");
        fs::write(work_path.join("file1.txt"), "content1").unwrap();
        fs::write(work_path.join("file2.txt"), "content2").unwrap();

        // Cleanup without age filter
        let result = manager.cleanup_work("cleanup-test", None, false).unwrap();

        // Files deleted
        assert_eq!(result.deleted_files, 2);
        assert_eq!(result.freed_bytes, 16); // content1 + content2 = 8 + 8 bytes
        assert!(!result.requires_confirmation);

        // Verify files are gone
        assert!(!work_path.join("file1.txt").exists());
        assert!(!work_path.join("file2.txt").exists());
    }

    #[test]
    fn test_cleanup_work_with_age_filter() {
        let (_dir, manager) = setup();

        manager.create_workstream("age-test").unwrap();
        let work_path = manager.work_path("age-test");

        // Create a file
        fs::write(work_path.join("recent.txt"), "recent").unwrap();

        // Cleanup files older than 1 day - should skip the file we just created
        let result = manager.cleanup_work("age-test", Some(1), false).unwrap();

        assert_eq!(result.deleted_files, 0);
        assert!(work_path.join("recent.txt").exists());
    }

    #[test]
    fn test_cleanup_work_requires_confirmation() {
        let (_dir, manager) = setup();

        manager.create_workstream("large-cleanup").unwrap();
        let work_path = manager.work_path("large-cleanup");

        // Create more than 100 files
        for i in 0..105 {
            fs::write(work_path.join(format!("file{i}.txt")), "x").unwrap();
        }

        // First call without confirmation
        let result = manager.cleanup_work("large-cleanup", None, false).unwrap();

        assert_eq!(result.deleted_files, 0); // Nothing deleted yet
        assert_eq!(result.pending_files, 105);
        assert!(result.requires_confirmation);

        // Files should still exist
        assert!(work_path.join("file0.txt").exists());

        // Second call with confirmation
        let result = manager.cleanup_work("large-cleanup", None, true).unwrap();

        assert_eq!(result.deleted_files, 105);
        assert!(!result.requires_confirmation);
        assert!(!work_path.join("file0.txt").exists());
    }

    #[test]
    fn test_cleanup_work_nested_directories() {
        let (_dir, manager) = setup();

        manager.create_workstream("nested-cleanup").unwrap();
        let work_path = manager.work_path("nested-cleanup");

        // Create nested structure
        fs::create_dir_all(work_path.join("a/b/c")).unwrap();
        fs::write(work_path.join("root.txt"), "root").unwrap();
        fs::write(work_path.join("a/level1.txt"), "l1").unwrap();
        fs::write(work_path.join("a/b/level2.txt"), "l2").unwrap();
        fs::write(work_path.join("a/b/c/level3.txt"), "l3").unwrap();

        let result = manager.cleanup_work("nested-cleanup", None, false).unwrap();

        // All 4 files deleted
        assert_eq!(result.deleted_files, 4);

        // Empty directories should be cleaned up
        assert!(!work_path.join("a/b/c").exists());
    }

    #[test]
    fn test_cleanup_work_scratch_sessions() {
        let (_dir, manager) = setup();

        // Create scratch sessions with files
        let s1_work = manager.create_scratch_session("session-a").unwrap();
        let s2_work = manager.create_scratch_session("session-b").unwrap();

        fs::write(s1_work.join("file1.txt"), "s1f1").unwrap();
        fs::write(s2_work.join("file2.txt"), "s2f2").unwrap();
        fs::write(s2_work.join("file3.txt"), "s2f3").unwrap();

        // Cleanup scratch workstream
        let result = manager
            .cleanup_work(SCRATCH_WORKSTREAM, None, false)
            .unwrap();

        // All 3 files from both sessions deleted
        assert_eq!(result.deleted_files, 3);
        assert!(!s1_work.join("file1.txt").exists());
        assert!(!s2_work.join("file2.txt").exists());
    }

    #[test]
    fn test_cleanup_work_preserves_production() {
        let (_dir, manager) = setup();

        manager.create_workstream("preserve-prod").unwrap();
        let prod_path = manager.production_path("preserve-prod");
        let work_path = manager.work_path("preserve-prod");

        // Create files in both directories
        fs::write(prod_path.join("important.txt"), "must keep").unwrap();
        fs::write(work_path.join("temp.txt"), "can delete").unwrap();

        let result = manager.cleanup_work("preserve-prod", None, false).unwrap();

        // Only work file deleted
        assert_eq!(result.deleted_files, 1);
        assert!(prod_path.join("important.txt").exists());
        assert!(!work_path.join("temp.txt").exists());
    }

    #[test]
    fn test_cleanup_work_empty_workstream() {
        let (_dir, manager) = setup();

        manager.create_workstream("empty-cleanup").unwrap();

        let result = manager.cleanup_work("empty-cleanup", None, false).unwrap();

        assert_eq!(result.deleted_files, 0);
        assert_eq!(result.freed_bytes, 0);
        assert!(!result.requires_confirmation);
    }

    #[test]
    fn test_cleanup_work_workstream_not_found() {
        let (_dir, manager) = setup();

        let err = manager
            .cleanup_work("nonexistent", None, false)
            .unwrap_err();
        assert!(matches!(err, DirectoryError::WorkstreamNotFound(_)));
    }

    #[test]
    fn test_cleanup_work_invalid_name() {
        let (_dir, manager) = setup();

        let err = manager
            .cleanup_work("invalid/name", None, false)
            .unwrap_err();
        assert!(matches!(err, DirectoryError::InvalidName(_)));
    }

    #[test]
    fn test_manual_cleanup_result_freed_mb() {
        let result = ManualCleanupResult {
            deleted_files: 10,
            freed_bytes: 1_048_576, // 1 MB
            pending_files: 0,
            requires_confirmation: false,
        };

        assert!((result.freed_mb() - 1.0).abs() < 0.001);
    }
}
