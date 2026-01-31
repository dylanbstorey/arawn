//! Session CRUD and entry operations.

use chrono::{DateTime, Utc};
use rusqlite::params;
use tracing::debug;

use crate::error::{MemoryError, Result};
use crate::types::{ContentType, Memory, MemoryId, Metadata, Session, SessionId};

use super::MemoryStore;

impl MemoryStore {
    /// Insert a new session.
    pub fn insert_session(&self, session: &Session) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            r#"
            INSERT INTO sessions (id, title, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4)
            "#,
            params![
                session.id.to_string(),
                session.title,
                session.created_at.to_rfc3339(),
                session.updated_at.to_rfc3339(),
            ],
        )?;

        debug!("Inserted session {}", session.id);
        Ok(())
    }

    /// Get a session by ID.
    pub fn get_session(&self, id: SessionId) -> Result<Option<Session>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt =
            conn.prepare("SELECT id, title, created_at, updated_at FROM sessions WHERE id = ?1")?;

        let mut rows = stmt.query(params![id.to_string()])?;

        if let Some(row) = rows.next()? {
            let session = Self::row_to_session(row)?;
            Ok(Some(session))
        } else {
            Ok(None)
        }
    }

    /// Update a session.
    pub fn update_session(&self, session: &Session) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        let rows_affected = conn.execute(
            "UPDATE sessions SET title = ?2, updated_at = ?3 WHERE id = ?1",
            params![
                session.id.to_string(),
                session.title,
                session.updated_at.to_rfc3339(),
            ],
        )?;

        if rows_affected == 0 {
            return Err(MemoryError::NotFound(format!("Session {}", session.id)));
        }

        Ok(())
    }

    /// Delete a session by ID.
    pub fn delete_session(&self, id: SessionId) -> Result<bool> {
        let conn = self.conn.lock().unwrap();

        let rows_affected = conn.execute(
            "DELETE FROM sessions WHERE id = ?1",
            params![id.to_string()],
        )?;

        Ok(rows_affected > 0)
    }

    /// List sessions ordered by updated_at descending.
    pub fn list_sessions(&self, limit: usize, offset: usize) -> Result<Vec<Session>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            r#"
            SELECT id, title, created_at, updated_at
            FROM sessions
            ORDER BY updated_at DESC
            LIMIT ?1 OFFSET ?2
            "#,
        )?;

        let mut rows = stmt.query(params![limit as i64, offset as i64])?;

        let mut sessions = Vec::new();
        while let Some(row) = rows.next()? {
            sessions.push(Self::row_to_session(row)?);
        }

        Ok(sessions)
    }

    /// Get or create a session by ID.
    ///
    /// If a session with the given ID exists, it is returned.
    /// Otherwise, a new session is created with that ID and returned.
    pub fn get_or_create_session(&self, id: SessionId) -> Result<Session> {
        if let Some(session) = self.get_session(id)? {
            return Ok(session);
        }

        let now = Utc::now();
        let session = Session {
            id,
            title: None,
            created_at: now,
            updated_at: now,
        };
        self.insert_session(&session)?;
        Ok(session)
    }

    /// Append an entry to a session.
    ///
    /// Creates a Memory with the session_id in its metadata.
    /// This allows session history to be searched via vector similarity.
    ///
    /// Returns the created memory's ID.
    pub fn append_to_session(
        &self,
        session_id: SessionId,
        content_type: ContentType,
        content: impl Into<String>,
    ) -> Result<MemoryId> {
        // Ensure session exists
        self.get_or_create_session(session_id)?;

        // Create memory with session reference
        let session_id_str = session_id.to_string();
        let mut metadata = Metadata::default();
        metadata.session_id = Some(session_id_str.clone());

        let now = Utc::now();
        let memory = Memory {
            id: MemoryId::new(),
            session_id: Some(session_id_str.clone()),
            content_type,
            content: content.into(),
            metadata,
            created_at: now,
            accessed_at: now,
            access_count: 0,
            confidence: crate::types::MemoryConfidence::default(),
            citation: None,
        };

        self.insert_memory(&memory)?;

        // Update session timestamp
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE sessions SET updated_at = ?2 WHERE id = ?1",
            params![session_id_str, Utc::now().to_rfc3339()],
        )?;

        debug!(
            "Appended {} to session {}: {}",
            content_type.as_str(),
            session_id,
            memory.id
        );

        Ok(memory.id)
    }

    /// Append an entry to a session with an optional embedding.
    ///
    /// Like `append_to_session`, but also stores the embedding for vector search.
    pub fn append_to_session_with_embedding(
        &self,
        session_id: SessionId,
        content_type: ContentType,
        content: impl Into<String>,
        embedding: &[f32],
    ) -> Result<MemoryId> {
        // Ensure session exists
        self.get_or_create_session(session_id)?;

        // Create memory with session reference
        let session_id_str = session_id.to_string();
        let mut metadata = Metadata::default();
        metadata.session_id = Some(session_id_str.clone());

        let now = Utc::now();
        let memory = Memory {
            id: MemoryId::new(),
            session_id: Some(session_id_str.clone()),
            content_type,
            content: content.into(),
            metadata,
            created_at: now,
            accessed_at: now,
            access_count: 0,
            confidence: crate::types::MemoryConfidence::default(),
            citation: None,
        };

        self.insert_memory_with_embedding(&memory, embedding)?;

        // Update session timestamp
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE sessions SET updated_at = ?2 WHERE id = ?1",
            params![session_id_str, Utc::now().to_rfc3339()],
        )?;

        Ok(memory.id)
    }

    /// Get session history (all memories associated with a session).
    ///
    /// Returns memories ordered by creation time (oldest first).
    pub fn get_session_history(
        &self,
        session_id: SessionId,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Memory>> {
        let conn = self.conn.lock().unwrap();

        let session_id_str = session_id.to_string();

        let mut stmt = conn.prepare(
            r#"
            SELECT id, session_id, content_type, content, metadata, created_at, accessed_at, access_count,
                   confidence_source, reinforcement_count, superseded, superseded_by,
                   last_accessed, confidence_score, citation
            FROM memories
            WHERE session_id = ?1
            ORDER BY created_at ASC
            LIMIT ?2 OFFSET ?3
            "#,
        )?;

        let mut rows = stmt.query(params![session_id_str, limit as i64, offset as i64])?;

        let mut memories = Vec::new();
        while let Some(row) = rows.next()? {
            memories.push(Self::row_to_memory(row)?);
        }

        Ok(memories)
    }

    /// Count entries in a session.
    pub fn count_session_entries(&self, session_id: SessionId) -> Result<usize> {
        let conn = self.conn.lock().unwrap();

        let session_id_str = session_id.to_string();

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM memories WHERE session_id = ?1",
            params![session_id_str],
            |row| row.get(0),
        )?;

        Ok(count as usize)
    }

    /// Convert a database row to a Session struct.
    pub(crate) fn row_to_session(row: &rusqlite::Row) -> Result<Session> {
        let id_str: String = row.get(0)?;
        let title: Option<String> = row.get(1)?;
        let created_at_str: String = row.get(2)?;
        let updated_at_str: String = row.get(3)?;

        let id = SessionId::parse(&id_str)?;
        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|e| MemoryError::InvalidData(e.to_string()))?
            .with_timezone(&Utc);
        let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
            .map_err(|e| MemoryError::InvalidData(e.to_string()))?
            .with_timezone(&Utc);

        Ok(Session {
            id,
            title,
            created_at,
            updated_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_store() -> MemoryStore {
        MemoryStore::open_in_memory().unwrap()
    }

    fn create_test_store_with_vectors() -> MemoryStore {
        crate::vector::init_vector_extension();
        let store = MemoryStore::open_in_memory().unwrap();
        store.init_vectors(4, "mock").unwrap();
        store
    }

    #[test]
    fn test_session_crud() {
        let store = create_test_store();

        let session = Session::new().with_title("Test Session");
        store.insert_session(&session).unwrap();

        let fetched = store.get_session(session.id).unwrap().unwrap();
        assert_eq!(fetched.title, Some("Test Session".to_string()));

        let mut updated = fetched;
        updated.title = Some("Updated Title".to_string());
        store.update_session(&updated).unwrap();

        let fetched = store.get_session(session.id).unwrap().unwrap();
        assert_eq!(fetched.title, Some("Updated Title".to_string()));

        assert!(store.delete_session(session.id).unwrap());
        assert!(store.get_session(session.id).unwrap().is_none());
    }

    #[test]
    fn test_get_or_create_session_existing() {
        let store = create_test_store();

        let session = Session::new().with_title("Existing Session");
        store.insert_session(&session).unwrap();

        let fetched = store.get_or_create_session(session.id).unwrap();
        assert_eq!(fetched.id, session.id);
        assert_eq!(fetched.title, Some("Existing Session".to_string()));
    }

    #[test]
    fn test_get_or_create_session_new() {
        let store = create_test_store();

        let new_id = SessionId::new();
        let created = store.get_or_create_session(new_id).unwrap();

        assert_eq!(created.id, new_id);
        assert!(created.title.is_none());

        let fetched = store.get_session(new_id).unwrap().unwrap();
        assert_eq!(fetched.id, new_id);
    }

    #[test]
    fn test_append_to_session() {
        let store = create_test_store();

        let session_id = SessionId::new();

        let m1 = store
            .append_to_session(session_id, ContentType::UserMessage, "Hello!")
            .unwrap();

        let m2 = store
            .append_to_session(session_id, ContentType::AssistantMessage, "Hi there!")
            .unwrap();

        let session = store.get_session(session_id).unwrap().unwrap();
        assert_eq!(session.id, session_id);

        let mem1 = store.get_memory(m1).unwrap().unwrap();
        assert_eq!(mem1.content, "Hello!");
        assert_eq!(mem1.content_type, ContentType::UserMessage);
        assert_eq!(mem1.metadata.session_id, Some(session_id.to_string()));

        let mem2 = store.get_memory(m2).unwrap().unwrap();
        assert_eq!(mem2.content, "Hi there!");
        assert_eq!(mem2.content_type, ContentType::AssistantMessage);
    }

    #[test]
    fn test_get_session_history() {
        let store = create_test_store();

        let session_id = SessionId::new();

        store
            .append_to_session(session_id, ContentType::UserMessage, "First")
            .unwrap();
        store
            .append_to_session(session_id, ContentType::AssistantMessage, "Second")
            .unwrap();
        store
            .append_to_session(session_id, ContentType::UserMessage, "Third")
            .unwrap();

        let history = store.get_session_history(session_id, 100, 0).unwrap();
        assert_eq!(history.len(), 3);

        assert_eq!(history[0].content, "First");
        assert_eq!(history[1].content, "Second");
        assert_eq!(history[2].content, "Third");
    }

    #[test]
    fn test_get_session_history_pagination() {
        let store = create_test_store();

        let session_id = SessionId::new();

        for i in 0..5 {
            store
                .append_to_session(
                    session_id,
                    ContentType::UserMessage,
                    format!("Message {}", i),
                )
                .unwrap();
        }

        let page1 = store.get_session_history(session_id, 2, 0).unwrap();
        assert_eq!(page1.len(), 2);
        assert_eq!(page1[0].content, "Message 0");
        assert_eq!(page1[1].content, "Message 1");

        let page2 = store.get_session_history(session_id, 2, 2).unwrap();
        assert_eq!(page2.len(), 2);
        assert_eq!(page2[0].content, "Message 2");
        assert_eq!(page2[1].content, "Message 3");
    }

    #[test]
    fn test_count_session_entries() {
        let store = create_test_store();

        let session_id = SessionId::new();

        assert_eq!(store.count_session_entries(session_id).unwrap(), 0);

        store
            .append_to_session(session_id, ContentType::UserMessage, "One")
            .unwrap();
        assert_eq!(store.count_session_entries(session_id).unwrap(), 1);

        store
            .append_to_session(session_id, ContentType::UserMessage, "Two")
            .unwrap();
        store
            .append_to_session(session_id, ContentType::UserMessage, "Three")
            .unwrap();
        assert_eq!(store.count_session_entries(session_id).unwrap(), 3);
    }

    #[test]
    fn test_append_to_session_with_embedding() {
        let store = create_test_store_with_vectors();

        let session_id = SessionId::new();

        let memory_id = store
            .append_to_session_with_embedding(
                session_id,
                ContentType::UserMessage,
                "Hello with embedding!",
                &[0.1, 0.2, 0.3, 0.4],
            )
            .unwrap();

        assert!(store.has_embedding(memory_id).unwrap());

        let history = store.get_session_history(session_id, 10, 0).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].content, "Hello with embedding!");
    }
}
