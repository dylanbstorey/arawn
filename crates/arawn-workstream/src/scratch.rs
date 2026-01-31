use std::fs;

use crate::message_store::MessageStore;
use crate::store::WorkstreamStore;
use crate::{Result, WorkstreamError};

/// Well-known scratch workstream ID.
pub const SCRATCH_ID: &str = "scratch";

/// Manages the scratch workstream and promotion to named workstreams.
pub struct ScratchManager<'a> {
    store: &'a WorkstreamStore,
    message_store: &'a MessageStore,
}

impl<'a> ScratchManager<'a> {
    pub fn new(store: &'a WorkstreamStore, message_store: &'a MessageStore) -> Self {
        Self {
            store,
            message_store,
        }
    }

    /// Ensure the scratch workstream exists, creating it if missing.
    pub fn ensure_scratch(&self) -> Result<crate::store::Workstream> {
        self.store.ensure_scratch()
    }

    /// Promote the scratch workstream to a named workstream.
    ///
    /// Creates a new workstream, moves JSONL history and SQLite records
    /// from scratch to the new workstream, then resets scratch to empty.
    ///
    /// Returns the new workstream.
    pub fn promote(
        &self,
        new_title: &str,
        tags: &[String],
        default_model: Option<&str>,
    ) -> Result<crate::store::Workstream> {
        // Ensure scratch exists
        self.ensure_scratch()?;

        // Check scratch has messages worth promoting
        let messages = self.message_store.read_all(SCRATCH_ID)?;
        if messages.is_empty() {
            return Err(WorkstreamError::Migration(
                "Scratch workstream has no messages to promote".to_string(),
            ));
        }

        // Create the new named workstream
        let new_ws = self
            .store
            .create_workstream(new_title, default_model, false)?;

        // Move JSONL: rename scratch dir to new workstream dir
        let scratch_dir = self.message_store.workstream_dir(SCRATCH_ID);
        let new_dir = self.message_store.workstream_dir(&new_ws.id);

        if scratch_dir.exists() {
            // Ensure parent of new dir exists
            if let Some(parent) = new_dir.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::rename(&scratch_dir, &new_dir)?;
        }

        // Reassign SQLite session records from scratch to new workstream
        self.store.reassign_sessions(SCRATCH_ID, &new_ws.id)?;

        // Reassign tags from scratch to new workstream, then set new tags
        self.store.reassign_tags(SCRATCH_ID, &new_ws.id)?;
        if !tags.is_empty() {
            self.store.set_tags(&new_ws.id, tags)?;
        }

        // Reset scratch summary
        self.store
            .update_workstream(SCRATCH_ID, None, None, None, None)?;

        Ok(new_ws)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::MessageRole;

    fn setup() -> (tempfile::TempDir, WorkstreamStore, MessageStore) {
        let dir = tempfile::tempdir().unwrap();
        let store = WorkstreamStore::open_in_memory().unwrap();
        let msg_store = MessageStore::new(dir.path());
        (dir, store, msg_store)
    }

    #[test]
    fn test_ensure_scratch_idempotent() {
        let (_dir, store, msg_store) = setup();
        let mgr = ScratchManager::new(&store, &msg_store);

        let s1 = mgr.ensure_scratch().unwrap();
        let s2 = mgr.ensure_scratch().unwrap();
        assert_eq!(s1.id, SCRATCH_ID);
        assert_eq!(s2.id, SCRATCH_ID);
        assert!(s1.is_scratch);
    }

    #[test]
    fn test_promote_moves_messages() {
        let (_dir, store, msg_store) = setup();
        let mgr = ScratchManager::new(&store, &msg_store);

        mgr.ensure_scratch().unwrap();

        // Add messages to scratch
        msg_store
            .append(SCRATCH_ID, None, MessageRole::User, "hello", None)
            .unwrap();
        msg_store
            .append(SCRATCH_ID, None, MessageRole::Assistant, "hi there", None)
            .unwrap();

        // Create a session on scratch
        store.create_session(SCRATCH_ID).unwrap();

        // Promote
        let new_ws = mgr
            .promote("My Project", &["rust".into()], Some("claude-sonnet"))
            .unwrap();

        assert_ne!(new_ws.id, SCRATCH_ID);
        assert_eq!(new_ws.title, "My Project");
        assert_eq!(new_ws.default_model.as_deref(), Some("claude-sonnet"));

        // Messages moved to new workstream
        let new_msgs = msg_store.read_all(&new_ws.id).unwrap();
        assert_eq!(new_msgs.len(), 2);

        // Scratch is empty (dir was renamed)
        let scratch_msgs = msg_store.read_all(SCRATCH_ID).unwrap();
        assert!(scratch_msgs.is_empty());

        // Sessions moved
        let new_sessions = store.list_sessions(&new_ws.id).unwrap();
        assert_eq!(new_sessions.len(), 1);
        let scratch_sessions = store.list_sessions(SCRATCH_ID).unwrap();
        assert!(scratch_sessions.is_empty());

        // Tags set
        let tags = store.get_tags(&new_ws.id).unwrap();
        assert_eq!(tags, vec!["rust"]);
    }

    #[test]
    fn test_promote_empty_scratch_fails() {
        let (_dir, store, msg_store) = setup();
        let mgr = ScratchManager::new(&store, &msg_store);

        mgr.ensure_scratch().unwrap();

        let err = mgr.promote("Empty", &[], None).unwrap_err();
        assert!(
            format!("{err}").contains("no messages"),
            "expected 'no messages' error, got: {err}"
        );
    }

    #[test]
    fn test_scratch_cannot_be_deleted() {
        let (_dir, store, msg_store) = setup();
        let mgr = ScratchManager::new(&store, &msg_store);

        mgr.ensure_scratch().unwrap();

        // Archive scratch via store — it succeeds at the store level,
        // but ensure_scratch will recreate it
        store
            .update_workstream(SCRATCH_ID, None, None, Some("archived"), None)
            .unwrap();

        // ensure_scratch still returns it (it exists, just archived)
        let scratch = mgr.ensure_scratch().unwrap();
        assert_eq!(scratch.id, SCRATCH_ID);
    }
}
