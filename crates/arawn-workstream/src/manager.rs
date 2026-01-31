use std::path::PathBuf;

use crate::message_store::MessageStore;
use crate::scratch::{SCRATCH_ID, ScratchManager};
use crate::session::SessionManager;
use crate::store::{Session, Workstream, WorkstreamStore};
use crate::types::{MessageRole, WorkstreamMessage};
use crate::{Result, WorkstreamError};

/// Configuration for the workstream manager.
#[derive(Debug, Clone)]
pub struct WorkstreamConfig {
    /// Path to the SQLite database file.
    pub db_path: PathBuf,
    /// Root directory for JSONL message files.
    pub data_dir: PathBuf,
    /// Session timeout in minutes.
    pub session_timeout_minutes: i64,
}

/// High-level facade coordinating message store, session manager,
/// workstream store, and scratch logic.
///
/// This is the primary entry point for other crates interacting
/// with workstreams.
pub struct WorkstreamManager {
    store: WorkstreamStore,
    message_store: MessageStore,
    session_timeout_minutes: i64,
}

impl WorkstreamManager {
    /// Initialize the manager: opens SQLite, runs migrations, sets up data dirs.
    pub fn new(config: &WorkstreamConfig) -> Result<Self> {
        // Ensure parent dirs exist
        if let Some(parent) = config.db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::create_dir_all(&config.data_dir)?;

        let store = WorkstreamStore::open(&config.db_path)?;
        let message_store = MessageStore::new(&config.data_dir);

        Ok(Self {
            store,
            message_store,
            session_timeout_minutes: config.session_timeout_minutes,
        })
    }

    /// Create from pre-built components (for testing).
    pub fn from_parts(
        store: WorkstreamStore,
        message_store: MessageStore,
        session_timeout_minutes: i64,
    ) -> Self {
        Self {
            store,
            message_store,
            session_timeout_minutes,
        }
    }

    // ── Workstream CRUD ─────────────────────────────────────────────

    pub fn create_workstream(
        &self,
        title: &str,
        default_model: Option<&str>,
        tags: &[String],
    ) -> Result<Workstream> {
        let ws = self.store.create_workstream(title, default_model, false)?;
        if !tags.is_empty() {
            self.store.set_tags(&ws.id, tags)?;
        }
        // Create the JSONL directory
        std::fs::create_dir_all(self.message_store.workstream_dir(&ws.id))?;
        Ok(ws)
    }

    pub fn get_workstream(&self, id: &str) -> Result<Workstream> {
        self.store.get_workstream(id)
    }

    pub fn list_workstreams(&self) -> Result<Vec<Workstream>> {
        self.store.list_workstreams(Some("active"))
    }

    pub fn archive_workstream(&self, id: &str) -> Result<()> {
        if id == SCRATCH_ID {
            return Err(WorkstreamError::Migration(
                "Cannot archive the scratch workstream".to_string(),
            ));
        }
        self.store
            .update_workstream(id, None, None, Some("archived"), None)
    }

    pub fn get_tags(&self, workstream_id: &str) -> Result<Vec<String>> {
        self.store.get_tags(workstream_id)
    }

    // ── Messaging ───────────────────────────────────────────────────

    /// Send a message to a workstream. If `workstream_id` is None, routes to scratch.
    /// Ensures an active session exists.
    pub fn send_message(
        &self,
        workstream_id: Option<&str>,
        role: MessageRole,
        content: &str,
        metadata: Option<&str>,
    ) -> Result<WorkstreamMessage> {
        let ws_id = self.resolve_workstream(workstream_id)?;
        let session = self.session_manager().get_or_start_session(&ws_id)?;

        self.message_store
            .append(&ws_id, Some(&session.id), role, content, metadata)
    }

    /// Push a message from a background agent/process into a workstream.
    pub fn push_agent_message(
        &self,
        workstream_id: &str,
        content: &str,
        metadata: Option<&str>,
    ) -> Result<WorkstreamMessage> {
        // Ensure workstream exists
        self.store.get_workstream(workstream_id)?;
        let session = self.session_manager().get_or_start_session(workstream_id)?;

        self.message_store.append(
            workstream_id,
            Some(&session.id),
            MessageRole::AgentPush,
            content,
            metadata,
        )
    }

    /// Read all messages for a workstream.
    pub fn get_messages(&self, workstream_id: &str) -> Result<Vec<WorkstreamMessage>> {
        self.message_store.read_all(workstream_id)
    }

    /// Read messages since a given timestamp.
    pub fn get_messages_since(
        &self,
        workstream_id: &str,
        since: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<WorkstreamMessage>> {
        self.message_store.read_range(workstream_id, since)
    }

    // ── Sessions ────────────────────────────────────────────────────

    pub fn get_active_session(&self, workstream_id: &str) -> Result<Option<Session>> {
        self.store.get_active_session(workstream_id)
    }

    pub fn end_session(&self, session_id: &str) -> Result<()> {
        self.session_manager().end_session(session_id)
    }

    pub fn list_sessions(&self, workstream_id: &str) -> Result<Vec<Session>> {
        self.store.list_sessions(workstream_id)
    }

    /// Run a timeout check across all workstreams. Returns count of timed-out sessions.
    pub fn timeout_check(&self) -> Result<usize> {
        self.session_manager().timeout_check()
    }

    // ── Scratch / Promotion ─────────────────────────────────────────

    pub fn promote_scratch(
        &self,
        new_title: &str,
        tags: &[String],
        default_model: Option<&str>,
    ) -> Result<Workstream> {
        self.scratch_manager()
            .promote(new_title, tags, default_model)
    }

    // ── Internals ───────────────────────────────────────────────────

    /// Resolve workstream_id, defaulting to scratch.
    fn resolve_workstream(&self, workstream_id: Option<&str>) -> Result<String> {
        match workstream_id {
            Some(id) => {
                // Verify it exists
                self.store.get_workstream(id)?;
                Ok(id.to_string())
            }
            None => {
                self.scratch_manager().ensure_scratch()?;
                Ok(SCRATCH_ID.to_string())
            }
        }
    }

    fn session_manager(&self) -> SessionManager<'_> {
        SessionManager::new(
            &self.store,
            &self.message_store,
            self.session_timeout_minutes,
        )
    }

    fn scratch_manager(&self) -> ScratchManager<'_> {
        ScratchManager::new(&self.store, &self.message_store)
    }

    /// Access the underlying store (for advanced operations).
    pub fn store(&self) -> &WorkstreamStore {
        &self.store
    }

    /// Access the underlying message store.
    pub fn message_store(&self) -> &MessageStore {
        &self.message_store
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_manager() -> (tempfile::TempDir, WorkstreamManager) {
        let dir = tempfile::tempdir().unwrap();
        let store = WorkstreamStore::open_in_memory().unwrap();
        let msg_store = MessageStore::new(dir.path());
        let mgr = WorkstreamManager::from_parts(store, msg_store, 30);
        (dir, mgr)
    }

    #[test]
    fn test_create_and_list_workstreams() {
        let (_dir, mgr) = test_manager();

        let ws = mgr
            .create_workstream("Project Alpha", Some("claude-sonnet"), &["rust".into()])
            .unwrap();
        assert_eq!(ws.title, "Project Alpha");

        let list = mgr.list_workstreams().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, ws.id);

        let tags = mgr.get_tags(&ws.id).unwrap();
        assert_eq!(tags, vec!["rust"]);
    }

    #[test]
    fn test_send_message_full_cycle() {
        let (_dir, mgr) = test_manager();

        let ws = mgr.create_workstream("Chat", None, &[]).unwrap();

        let m1 = mgr
            .send_message(Some(&ws.id), MessageRole::User, "hello", None)
            .unwrap();
        assert_eq!(m1.role, MessageRole::User);
        assert_eq!(m1.workstream_id, ws.id);

        let m2 = mgr
            .send_message(Some(&ws.id), MessageRole::Assistant, "hi there!", None)
            .unwrap();
        assert_eq!(m2.role, MessageRole::Assistant);

        let messages = mgr.get_messages(&ws.id).unwrap();
        assert_eq!(messages.len(), 2);

        // Session was auto-created
        let session = mgr.get_active_session(&ws.id).unwrap();
        assert!(session.is_some());
    }

    #[test]
    fn test_scratch_auto_create_on_send() {
        let (_dir, mgr) = test_manager();

        // Send with no workstream_id → goes to scratch
        let msg = mgr
            .send_message(None, MessageRole::User, "quick question", None)
            .unwrap();
        assert_eq!(msg.workstream_id, SCRATCH_ID);

        let messages = mgr.get_messages(SCRATCH_ID).unwrap();
        assert_eq!(messages.len(), 1);
    }

    #[test]
    fn test_agent_push() {
        let (_dir, mgr) = test_manager();

        let ws = mgr.create_workstream("Background Work", None, &[]).unwrap();

        let msg = mgr
            .push_agent_message(&ws.id, "Task completed successfully", None)
            .unwrap();
        assert_eq!(msg.role, MessageRole::AgentPush);

        let messages = mgr.get_messages(&ws.id).unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].role, MessageRole::AgentPush);
    }

    #[test]
    fn test_archive_workstream() {
        let (_dir, mgr) = test_manager();

        let ws = mgr.create_workstream("Temp", None, &[]).unwrap();
        mgr.archive_workstream(&ws.id).unwrap();

        // No longer in active list
        let list = mgr.list_workstreams().unwrap();
        assert!(list.is_empty());

        // Still fetchable by ID
        let archived = mgr.get_workstream(&ws.id).unwrap();
        assert_eq!(archived.state, "archived");
    }

    #[test]
    fn test_cannot_archive_scratch() {
        let (_dir, mgr) = test_manager();

        // Ensure scratch exists
        mgr.send_message(None, MessageRole::User, "hi", None)
            .unwrap();

        let err = mgr.archive_workstream(SCRATCH_ID).unwrap_err();
        assert!(format!("{err}").contains("scratch"));
    }

    #[test]
    fn test_send_to_nonexistent_workstream_fails() {
        let (_dir, mgr) = test_manager();

        let err = mgr
            .send_message(Some("nonexistent"), MessageRole::User, "hello", None)
            .unwrap_err();
        assert!(matches!(err, WorkstreamError::NotFound(_)));
    }

    #[test]
    fn test_promote_scratch_via_manager() {
        let (_dir, mgr) = test_manager();

        // Send messages to scratch
        mgr.send_message(None, MessageRole::User, "idea", None)
            .unwrap();
        mgr.send_message(None, MessageRole::Assistant, "tell me more", None)
            .unwrap();

        // Promote
        let new_ws = mgr
            .promote_scratch("New Project", &["important".into()], None)
            .unwrap();

        // Messages moved
        let new_msgs = mgr.get_messages(&new_ws.id).unwrap();
        assert_eq!(new_msgs.len(), 2);

        let scratch_msgs = mgr.get_messages(SCRATCH_ID).unwrap();
        assert!(scratch_msgs.is_empty());
    }
}
