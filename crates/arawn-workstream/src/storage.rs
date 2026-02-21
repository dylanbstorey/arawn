//! Storage abstraction traits for workstreams.
//!
//! This module defines traits for workstream storage backends, allowing
//! different implementations (SQLite, PostgreSQL, mock, etc.) to be used
//! interchangeably.
//!
//! # Architecture
//!
//! ```text
//! WorkstreamStorage (trait)       - Workstream CRUD operations
//!     └── SqliteWorkstreamStore   - Default SQLite implementation
//!     └── MockWorkstreamStorage   - In-memory mock for testing
//!
//! MessageStorage (trait)          - Message append/read operations
//!     └── JsonlMessageStore       - Default JSONL file implementation
//!     └── MockMessageStorage      - In-memory mock for testing
//! ```

use chrono::{DateTime, Utc};

use crate::store::{Session, Workstream};
use crate::types::WorkstreamMessage;
use crate::Result;

/// Trait for workstream metadata storage.
///
/// This trait defines operations for managing workstream records and their
/// associated sessions. The default implementation uses SQLite.
pub trait WorkstreamStorage: Send + Sync {
    // ── Workstream Operations ───────────────────────────────────────────

    /// Create a new workstream.
    fn create_workstream(
        &self,
        title: &str,
        default_model: Option<&str>,
        is_scratch: bool,
    ) -> Result<Workstream>;

    /// Get a workstream by ID.
    fn get_workstream(&self, id: &str) -> Result<Workstream>;

    /// List workstreams with optional state filter.
    ///
    /// Pass `Some("active")` to list only active workstreams,
    /// or `None` to list all workstreams.
    fn list_workstreams(&self, state_filter: Option<&str>) -> Result<Vec<Workstream>>;

    /// Update workstream fields.
    fn update_workstream(
        &self,
        id: &str,
        title: Option<&str>,
        summary: Option<&str>,
        state: Option<&str>,
        default_model: Option<&str>,
    ) -> Result<()>;

    // ── Tag Operations ──────────────────────────────────────────────────

    /// Set tags for a workstream (replaces existing tags).
    fn set_tags(&self, workstream_id: &str, tags: &[String]) -> Result<()>;

    /// Get tags for a workstream.
    fn get_tags(&self, workstream_id: &str) -> Result<Vec<String>>;

    // ── Session Operations ──────────────────────────────────────────────

    /// Create a new session for a workstream.
    fn create_session(&self, workstream_id: &str) -> Result<Session>;

    /// Create a session with a specific ID.
    fn create_session_with_id(&self, session_id: &str, workstream_id: &str) -> Result<Session>;

    /// Get active session for a workstream.
    fn get_active_session(&self, workstream_id: &str) -> Result<Option<Session>>;

    /// List all sessions for a workstream.
    fn list_sessions(&self, workstream_id: &str) -> Result<Vec<Session>>;

    /// End a session (set ended_at).
    fn end_session(&self, session_id: &str) -> Result<()>;

    /// Move a session to a different workstream.
    fn reassign_session(&self, session_id: &str, new_workstream_id: &str) -> Result<Session>;
}

/// Trait for message storage (conversation history).
///
/// Messages are typically stored separately from workstream metadata
/// for performance reasons (append-heavy workload).
pub trait MessageStorage: Send + Sync {
    /// Append a message to a workstream's message log.
    fn append(
        &self,
        workstream_id: &str,
        session_id: Option<&str>,
        role: crate::types::MessageRole,
        content: &str,
        metadata: Option<&str>,
    ) -> Result<WorkstreamMessage>;

    /// Read all messages for a workstream.
    fn read_all(&self, workstream_id: &str) -> Result<Vec<WorkstreamMessage>>;

    /// Read messages since a given timestamp.
    fn read_range(
        &self,
        workstream_id: &str,
        since: DateTime<Utc>,
    ) -> Result<Vec<WorkstreamMessage>>;

    /// Move messages from one workstream to another.
    fn move_messages(&self, from_workstream: &str, to_workstream: &str) -> Result<()>;

    /// Delete all messages for a workstream.
    fn delete_all(&self, workstream_id: &str) -> Result<()>;
}

/// Mock implementation of WorkstreamStorage for testing.
#[derive(Debug, Default)]
pub struct MockWorkstreamStorage {
    workstreams: std::sync::Mutex<std::collections::HashMap<String, Workstream>>,
    sessions: std::sync::Mutex<std::collections::HashMap<String, Session>>,
    tags: std::sync::Mutex<std::collections::HashMap<String, Vec<String>>>,
}

impl MockWorkstreamStorage {
    /// Create a new empty mock storage.
    pub fn new() -> Self {
        Self::default()
    }
}

impl WorkstreamStorage for MockWorkstreamStorage {
    fn create_workstream(
        &self,
        title: &str,
        default_model: Option<&str>,
        is_scratch: bool,
    ) -> Result<Workstream> {
        let id = if is_scratch {
            "scratch".to_string()
        } else {
            uuid::Uuid::new_v4().to_string()
        };
        let now = Utc::now();

        let workstream = Workstream {
            id: id.clone(),
            title: title.to_string(),
            summary: None,
            is_scratch,
            state: "active".to_string(),
            default_model: default_model.map(String::from),
            created_at: now,
            updated_at: now,
        };

        self.workstreams
            .lock()
            .unwrap()
            .insert(id, workstream.clone());
        Ok(workstream)
    }

    fn get_workstream(&self, id: &str) -> Result<Workstream> {
        self.workstreams
            .lock()
            .unwrap()
            .get(id)
            .cloned()
            .ok_or_else(|| crate::WorkstreamError::NotFound(id.to_string()))
    }

    fn list_workstreams(&self, state_filter: Option<&str>) -> Result<Vec<Workstream>> {
        let map = self.workstreams.lock().unwrap();
        let mut results: Vec<_> = map
            .values()
            .filter(|ws| state_filter.is_none() || Some(ws.state.as_str()) == state_filter)
            .cloned()
            .collect();
        results.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        Ok(results)
    }

    fn update_workstream(
        &self,
        id: &str,
        title: Option<&str>,
        summary: Option<&str>,
        state: Option<&str>,
        default_model: Option<&str>,
    ) -> Result<()> {
        let mut map = self.workstreams.lock().unwrap();
        if let Some(ws) = map.get_mut(id) {
            if let Some(t) = title {
                ws.title = t.to_string();
            }
            if let Some(s) = summary {
                ws.summary = Some(s.to_string());
            }
            if let Some(st) = state {
                ws.state = st.to_string();
            }
            if let Some(dm) = default_model {
                ws.default_model = Some(dm.to_string());
            }
            ws.updated_at = Utc::now();
            Ok(())
        } else {
            Err(crate::WorkstreamError::NotFound(id.to_string()))
        }
    }

    fn set_tags(&self, workstream_id: &str, tags: &[String]) -> Result<()> {
        // Verify workstream exists
        if !self.workstreams.lock().unwrap().contains_key(workstream_id) {
            return Err(crate::WorkstreamError::NotFound(workstream_id.to_string()));
        }
        self.tags
            .lock()
            .unwrap()
            .insert(workstream_id.to_string(), tags.to_vec());
        Ok(())
    }

    fn get_tags(&self, workstream_id: &str) -> Result<Vec<String>> {
        Ok(self
            .tags
            .lock()
            .unwrap()
            .get(workstream_id)
            .cloned()
            .unwrap_or_default())
    }

    fn create_session(&self, workstream_id: &str) -> Result<Session> {
        let session_id = uuid::Uuid::new_v4().to_string();
        self.create_session_with_id(&session_id, workstream_id)
    }

    fn create_session_with_id(&self, session_id: &str, workstream_id: &str) -> Result<Session> {
        // Verify workstream exists
        if !self.workstreams.lock().unwrap().contains_key(workstream_id) {
            return Err(crate::WorkstreamError::NotFound(workstream_id.to_string()));
        }

        let session = Session {
            id: session_id.to_string(),
            workstream_id: workstream_id.to_string(),
            started_at: Utc::now(),
            ended_at: None,
            turn_count: Some(0),
            summary: None,
            compressed: false,
        };

        self.sessions
            .lock()
            .unwrap()
            .insert(session_id.to_string(), session.clone());
        Ok(session)
    }

    fn get_active_session(&self, workstream_id: &str) -> Result<Option<Session>> {
        let sessions = self.sessions.lock().unwrap();
        Ok(sessions
            .values()
            .find(|s| s.workstream_id == workstream_id && s.ended_at.is_none())
            .cloned())
    }

    fn list_sessions(&self, workstream_id: &str) -> Result<Vec<Session>> {
        let sessions = self.sessions.lock().unwrap();
        let mut results: Vec<_> = sessions
            .values()
            .filter(|s| s.workstream_id == workstream_id)
            .cloned()
            .collect();
        results.sort_by(|a, b| b.started_at.cmp(&a.started_at));
        Ok(results)
    }

    fn end_session(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get_mut(session_id) {
            session.ended_at = Some(Utc::now());
            Ok(())
        } else {
            Err(crate::WorkstreamError::NotFound(session_id.to_string()))
        }
    }

    fn reassign_session(&self, session_id: &str, new_workstream_id: &str) -> Result<Session> {
        // Verify new workstream exists
        if !self
            .workstreams
            .lock()
            .unwrap()
            .contains_key(new_workstream_id)
        {
            return Err(crate::WorkstreamError::NotFound(
                new_workstream_id.to_string(),
            ));
        }

        let mut sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get_mut(session_id) {
            session.workstream_id = new_workstream_id.to_string();
            Ok(session.clone())
        } else {
            Err(crate::WorkstreamError::NotFound(session_id.to_string()))
        }
    }
}

/// Mock implementation of MessageStorage for testing.
#[derive(Debug, Default)]
pub struct MockMessageStorage {
    messages: std::sync::Mutex<std::collections::HashMap<String, Vec<WorkstreamMessage>>>,
}

impl MockMessageStorage {
    /// Create a new empty mock storage.
    pub fn new() -> Self {
        Self::default()
    }
}

impl MessageStorage for MockMessageStorage {
    fn append(
        &self,
        workstream_id: &str,
        session_id: Option<&str>,
        role: crate::types::MessageRole,
        content: &str,
        metadata: Option<&str>,
    ) -> Result<WorkstreamMessage> {
        let msg = WorkstreamMessage {
            id: uuid::Uuid::new_v4().to_string(),
            workstream_id: workstream_id.to_string(),
            session_id: session_id.map(String::from),
            role,
            content: content.to_string(),
            metadata: metadata.map(String::from),
            timestamp: Utc::now(),
        };

        self.messages
            .lock()
            .unwrap()
            .entry(workstream_id.to_string())
            .or_default()
            .push(msg.clone());

        Ok(msg)
    }

    fn read_all(&self, workstream_id: &str) -> Result<Vec<WorkstreamMessage>> {
        Ok(self
            .messages
            .lock()
            .unwrap()
            .get(workstream_id)
            .cloned()
            .unwrap_or_default())
    }

    fn read_range(
        &self,
        workstream_id: &str,
        since: DateTime<Utc>,
    ) -> Result<Vec<WorkstreamMessage>> {
        Ok(self
            .messages
            .lock()
            .unwrap()
            .get(workstream_id)
            .map(|msgs| {
                msgs.iter()
                    .filter(|m| m.timestamp >= since)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default())
    }

    fn move_messages(&self, from_workstream: &str, to_workstream: &str) -> Result<()> {
        let mut map = self.messages.lock().unwrap();
        if let Some(mut msgs) = map.remove(from_workstream) {
            for msg in &mut msgs {
                msg.workstream_id = to_workstream.to_string();
            }
            map.entry(to_workstream.to_string())
                .or_default()
                .extend(msgs);
        }
        Ok(())
    }

    fn delete_all(&self, workstream_id: &str) -> Result<()> {
        self.messages.lock().unwrap().remove(workstream_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::MessageRole;

    #[test]
    fn test_mock_workstream_storage_crud() {
        let storage = MockWorkstreamStorage::new();

        // Create
        let ws = storage
            .create_workstream("Test Project", Some("claude-sonnet"), false)
            .unwrap();
        assert_eq!(ws.title, "Test Project");
        assert_eq!(ws.default_model, Some("claude-sonnet".to_string()));

        // Get
        let fetched = storage.get_workstream(&ws.id).unwrap();
        assert_eq!(fetched.id, ws.id);

        // List
        let list = storage.list_workstreams(Some("active")).unwrap();
        assert_eq!(list.len(), 1);

        // Update
        storage
            .update_workstream(&ws.id, Some("New Title"), None, None, None)
            .unwrap();
        let updated = storage.get_workstream(&ws.id).unwrap();
        assert_eq!(updated.title, "New Title");
    }

    #[test]
    fn test_mock_workstream_storage_tags() {
        let storage = MockWorkstreamStorage::new();

        let ws = storage
            .create_workstream("Tagged Project", None, false)
            .unwrap();

        storage
            .set_tags(&ws.id, &["rust".into(), "backend".into()])
            .unwrap();

        let tags = storage.get_tags(&ws.id).unwrap();
        assert_eq!(tags, vec!["rust", "backend"]);
    }

    #[test]
    fn test_mock_workstream_storage_sessions() {
        let storage = MockWorkstreamStorage::new();

        let ws = storage.create_workstream("Session Test", None, false).unwrap();

        // Create session
        let session = storage.create_session(&ws.id).unwrap();
        assert_eq!(session.workstream_id, ws.id);
        assert!(session.ended_at.is_none());

        // Get active session
        let active = storage.get_active_session(&ws.id).unwrap();
        assert!(active.is_some());

        // End session
        storage.end_session(&session.id).unwrap();
        let ended = storage.get_active_session(&ws.id).unwrap();
        assert!(ended.is_none());
    }

    #[test]
    fn test_mock_message_storage() {
        let storage = MockMessageStorage::new();

        // Append messages
        let msg1 = storage
            .append("ws-1", Some("sess-1"), MessageRole::User, "Hello", None)
            .unwrap();
        let msg2 = storage
            .append(
                "ws-1",
                Some("sess-1"),
                MessageRole::Assistant,
                "Hi there!",
                None,
            )
            .unwrap();

        assert_eq!(msg1.role, MessageRole::User);
        assert_eq!(msg2.role, MessageRole::Assistant);

        // Read all
        let messages = storage.read_all("ws-1").unwrap();
        assert_eq!(messages.len(), 2);

        // Move messages
        storage.move_messages("ws-1", "ws-2").unwrap();
        assert!(storage.read_all("ws-1").unwrap().is_empty());
        assert_eq!(storage.read_all("ws-2").unwrap().len(), 2);
    }
}
