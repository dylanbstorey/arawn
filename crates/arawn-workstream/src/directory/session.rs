//! Session attachment: migrating scratch sessions to named workstreams.

use std::fs;
use std::path::Path;

use super::{AttachResult, DirectoryError, DirectoryManager, DirectoryResult};

impl DirectoryManager {
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
            return Err(DirectoryError::WorkstreamNotFound(
                target_workstream.to_string(),
            ));
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
            if fs::rename(&src_path, &dest_path).is_err() {
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
    use std::fs;

    use super::super::DirectoryError;
    use super::DirectoryManager;

    fn setup() -> (tempfile::TempDir, DirectoryManager) {
        let dir = tempfile::tempdir().unwrap();
        let manager = DirectoryManager::new(dir.path());
        (dir, manager)
    }

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
        let result = manager
            .attach_session("nonexistent-session", "empty-target")
            .unwrap();

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

        let err = manager
            .attach_session("session-123", "../escape")
            .unwrap_err();
        assert!(matches!(err, DirectoryError::InvalidName(_)));
    }

    #[test]
    fn test_attach_session_workstream_not_found() {
        let (_dir, manager) = setup();

        // Create scratch session
        let _ = manager.create_scratch_session("session-123").unwrap();

        let err = manager
            .attach_session("session-123", "nonexistent")
            .unwrap_err();
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

        let result = manager
            .attach_session("session-789", "preserve-test")
            .unwrap();

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
