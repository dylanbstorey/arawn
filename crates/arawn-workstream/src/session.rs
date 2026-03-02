use chrono::{Duration, Utc};

use crate::Result;
use crate::message_store::MessageStore;
use crate::store::{Session, WorkstreamStore};

/// Manages session lifecycle within workstreams.
///
/// Enforces one-active-session-per-workstream, handles timeout detection,
/// and counts messages on session end.
pub struct SessionManager<'a> {
    store: &'a WorkstreamStore,
    message_store: &'a MessageStore,
    timeout: Duration,
}

impl<'a> SessionManager<'a> {
    pub fn new(
        store: &'a WorkstreamStore,
        message_store: &'a MessageStore,
        timeout_minutes: i64,
    ) -> Self {
        Self {
            store,
            message_store,
            timeout: Duration::minutes(timeout_minutes),
        }
    }

    /// Get or start a session for the workstream.
    ///
    /// If an active session exists and is not timed out, returns it.
    /// If the active session is timed out, ends it and starts a new one.
    /// If no active session exists, starts a new one.
    pub fn get_or_start_session(&self, workstream_id: &str) -> Result<Session> {
        if let Some(active) = self.store.get_active_session(workstream_id)? {
            if self.is_timed_out(&active) {
                let turn_count = self.count_session_messages(&active)?;
                self.store.end_session(&active.id, turn_count)?;
                return self.store.create_session(workstream_id);
            }
            return Ok(active);
        }
        self.store.create_session(workstream_id)
    }

    /// Explicitly end a session, counting its messages from JSONL.
    pub fn end_session(&self, session_id: &str) -> Result<()> {
        let session = self.store.get_session(session_id)?;
        let turn_count = self.count_session_messages(&session)?;
        self.store.end_session(session_id, turn_count)
    }

    /// Scan for and end all timed-out sessions across all workstreams.
    /// Returns the number of sessions that were timed out.
    pub fn timeout_check(&self) -> Result<usize> {
        let workstreams = self.store.list_workstreams(Some("active"))?;
        let mut timed_out = 0;

        for ws in &workstreams {
            if let Some(active) = self.store.get_active_session(&ws.id)?
                && self.is_timed_out(&active)
            {
                let turn_count = self.count_session_messages(&active)?;
                self.store.end_session(&active.id, turn_count)?;
                timed_out += 1;
            }
        }

        Ok(timed_out)
    }

    fn is_timed_out(&self, session: &Session) -> bool {
        Utc::now() - session.started_at > self.timeout
    }

    fn count_session_messages(&self, session: &Session) -> Result<i32> {
        let messages = self
            .message_store
            .read_range(&session.workstream_id, session.started_at)?;

        let end = session.ended_at.unwrap_or_else(Utc::now);
        let count = messages.iter().filter(|m| m.timestamp <= end).count();

        Ok(count as i32)
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
    fn test_get_or_start_creates_session() {
        let (_dir, store, msg_store) = setup();
        let ws = store.create_workstream("Test", None, false).unwrap();
        let mgr = SessionManager::new(&store, &msg_store, 30);

        let session = mgr.get_or_start_session(&ws.id).unwrap();
        assert!(session.ended_at.is_none());

        // Calling again returns same session
        let same = mgr.get_or_start_session(&ws.id).unwrap();
        assert_eq!(same.id, session.id);
    }

    #[test]
    fn test_end_session_counts_messages() {
        let (_dir, store, msg_store) = setup();
        let ws = store.create_workstream("Test", None, false).unwrap();
        let mgr = SessionManager::new(&store, &msg_store, 30);

        let session = mgr.get_or_start_session(&ws.id).unwrap();

        msg_store
            .append(&ws.id, Some(&session.id), MessageRole::User, "hi", None)
            .unwrap();
        msg_store
            .append(
                &ws.id,
                Some(&session.id),
                MessageRole::Assistant,
                "hello",
                None,
            )
            .unwrap();

        mgr.end_session(&session.id).unwrap();

        let ended = store.get_session(&session.id).unwrap();
        assert!(ended.ended_at.is_some());
        assert_eq!(ended.turn_count, Some(2));
    }

    #[test]
    fn test_one_active_constraint() {
        let (_dir, store, msg_store) = setup();
        let ws = store.create_workstream("Test", None, false).unwrap();
        let mgr = SessionManager::new(&store, &msg_store, 30);

        let s1 = mgr.get_or_start_session(&ws.id).unwrap();
        let s2 = mgr.get_or_start_session(&ws.id).unwrap();
        assert_eq!(s1.id, s2.id, "should return same active session");

        mgr.end_session(&s1.id).unwrap();

        let s3 = mgr.get_or_start_session(&ws.id).unwrap();
        assert_ne!(
            s3.id, s1.id,
            "should create new session after ending previous"
        );
    }

    #[test]
    fn test_timeout_creates_new_session() {
        let (_dir, store, msg_store) = setup();
        let ws = store.create_workstream("Test", None, false).unwrap();
        // Use 0-minute timeout so any session is immediately timed out
        let mgr = SessionManager::new(&store, &msg_store, 0);

        let s1 = mgr.get_or_start_session(&ws.id).unwrap();
        // Sleep briefly so the new session has a later started_at
        std::thread::sleep(std::time::Duration::from_millis(5));
        let s2 = mgr.get_or_start_session(&ws.id).unwrap();

        assert_ne!(s2.id, s1.id, "timed out session should be replaced");

        let ended = store.get_session(&s1.id).unwrap();
        assert!(ended.ended_at.is_some());
    }

    #[test]
    fn test_timeout_check_bulk() {
        let (_dir, store, msg_store) = setup();
        let ws1 = store.create_workstream("WS1", None, false).unwrap();
        let ws2 = store.create_workstream("WS2", None, false).unwrap();

        // Create active sessions with 0-minute timeout
        let mgr_normal = SessionManager::new(&store, &msg_store, 9999);
        mgr_normal.get_or_start_session(&ws1.id).unwrap();
        mgr_normal.get_or_start_session(&ws2.id).unwrap();

        std::thread::sleep(std::time::Duration::from_millis(5));

        // Now check with 0-minute timeout
        let mgr_strict = SessionManager::new(&store, &msg_store, 0);
        let timed_out = mgr_strict.timeout_check().unwrap();
        assert_eq!(timed_out, 2);

        // No more active sessions
        assert!(store.get_active_session(&ws1.id).unwrap().is_none());
        assert!(store.get_active_session(&ws2.id).unwrap().is_none());
    }
}
