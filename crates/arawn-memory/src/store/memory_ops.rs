//! Memory CRUD operations.

use chrono::{DateTime, Utc};
use rusqlite::params;
use tracing::debug;

use crate::error::{MemoryError, Result};
use crate::types::{
    Citation, ConfidenceSource, ContentType, Memory, MemoryConfidence, MemoryId, Metadata,
};

use super::MemoryStore;

impl MemoryStore {
    /// Insert a new memory.
    pub fn insert_memory(&self, memory: &Memory) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        let metadata_json = serde_json::to_string(&memory.metadata)?;
        let citation_json = memory
            .citation
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;

        conn.execute(
            r#"
            INSERT INTO memories (id, session_id, content_type, content, metadata, created_at, accessed_at, access_count,
                                  confidence_source, reinforcement_count, superseded, superseded_by,
                                  last_accessed, confidence_score, citation)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
            "#,
            params![
                memory.id.to_string(),
                memory.session_id,
                memory.content_type.as_str(),
                memory.content,
                metadata_json,
                memory.created_at.to_rfc3339(),
                memory.accessed_at.to_rfc3339(),
                memory.access_count,
                memory.confidence.source.as_str(),
                memory.confidence.reinforcement_count,
                memory.confidence.superseded as i32,
                memory.confidence.superseded_by.map(|id| id.to_string()),
                memory.confidence.last_accessed.to_rfc3339(),
                memory.confidence.score,
                citation_json,
            ],
        )?;

        debug!("Inserted memory {}", memory.id);
        Ok(())
    }

    /// Get a memory by ID.
    pub fn get_memory(&self, id: MemoryId) -> Result<Option<Memory>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            r#"
            SELECT id, session_id, content_type, content, metadata, created_at, accessed_at, access_count,
                   confidence_source, reinforcement_count, superseded, superseded_by,
                   last_accessed, confidence_score, citation
            FROM memories
            WHERE id = ?1
            "#,
        )?;

        let mut rows = stmt.query(params![id.to_string()])?;

        if let Some(row) = rows.next()? {
            let memory = Self::row_to_memory(row)?;
            Ok(Some(memory))
        } else {
            Ok(None)
        }
    }

    /// Update an existing memory.
    pub fn update_memory(&self, memory: &Memory) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        let metadata_json = serde_json::to_string(&memory.metadata)?;
        let citation_json = memory
            .citation
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;

        let rows_affected = conn.execute(
            r#"
            UPDATE memories
            SET session_id = ?2, content_type = ?3, content = ?4, metadata = ?5,
                accessed_at = ?6, access_count = ?7,
                confidence_source = ?8, reinforcement_count = ?9,
                superseded = ?10, superseded_by = ?11,
                last_accessed = ?12, confidence_score = ?13, citation = ?14
            WHERE id = ?1
            "#,
            params![
                memory.id.to_string(),
                memory.session_id,
                memory.content_type.as_str(),
                memory.content,
                metadata_json,
                memory.accessed_at.to_rfc3339(),
                memory.access_count,
                memory.confidence.source.as_str(),
                memory.confidence.reinforcement_count,
                memory.confidence.superseded as i32,
                memory.confidence.superseded_by.map(|id| id.to_string()),
                memory.confidence.last_accessed.to_rfc3339(),
                memory.confidence.score,
                citation_json,
            ],
        )?;

        if rows_affected == 0 {
            return Err(MemoryError::NotFound(format!("Memory {}", memory.id)));
        }

        debug!("Updated memory {}", memory.id);
        Ok(())
    }

    /// Delete a memory by ID.
    pub fn delete_memory(&self, id: MemoryId) -> Result<bool> {
        let conn = self.conn.lock().unwrap();

        let rows_affected = conn.execute(
            "DELETE FROM memories WHERE id = ?1",
            params![id.to_string()],
        )?;

        if rows_affected > 0 {
            debug!("Deleted memory {}", id);
        }

        Ok(rows_affected > 0)
    }

    /// List memories with optional filtering.
    pub fn list_memories(
        &self,
        content_type: Option<ContentType>,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Memory>> {
        let conn = self.conn.lock().unwrap();

        let (sql, params_vec): (&str, Vec<Box<dyn rusqlite::ToSql>>) = if let Some(ct) =
            content_type
        {
            (
                r#"
                SELECT id, session_id, content_type, content, metadata, created_at, accessed_at, access_count,
                       confidence_source, reinforcement_count, superseded, superseded_by,
                       last_accessed, confidence_score, citation
                FROM memories
                WHERE content_type = ?1
                ORDER BY created_at DESC
                LIMIT ?2 OFFSET ?3
                "#,
                vec![
                    Box::new(ct.as_str().to_string()),
                    Box::new(limit as i64),
                    Box::new(offset as i64),
                ],
            )
        } else {
            (
                r#"
                SELECT id, session_id, content_type, content, metadata, created_at, accessed_at, access_count,
                       confidence_source, reinforcement_count, superseded, superseded_by,
                       last_accessed, confidence_score, citation
                FROM memories
                ORDER BY created_at DESC
                LIMIT ?1 OFFSET ?2
                "#,
                vec![Box::new(limit as i64), Box::new(offset as i64)],
            )
        };

        let mut stmt = conn.prepare(sql)?;
        let params_refs: Vec<&dyn rusqlite::ToSql> =
            params_vec.iter().map(|b| b.as_ref()).collect();
        let mut rows = stmt.query(params_refs.as_slice())?;

        let mut memories = Vec::new();
        while let Some(row) = rows.next()? {
            memories.push(Self::row_to_memory(row)?);
        }

        Ok(memories)
    }

    /// Count memories with optional filtering.
    pub fn count_memories(&self, content_type: Option<ContentType>) -> Result<usize> {
        let conn = self.conn.lock().unwrap();

        let count: i64 = if let Some(ct) = content_type {
            conn.query_row(
                "SELECT COUNT(*) FROM memories WHERE content_type = ?1",
                params![ct.as_str()],
                |row| row.get(0),
            )?
        } else {
            conn.query_row("SELECT COUNT(*) FROM memories", [], |row| row.get(0))?
        };

        Ok(count as usize)
    }

    /// Record access to a memory (updates accessed_at and access_count).
    pub fn touch_memory(&self, id: MemoryId) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        let rows_affected = conn.execute(
            r#"
            UPDATE memories
            SET accessed_at = ?2, access_count = access_count + 1
            WHERE id = ?1
            "#,
            params![id.to_string(), Utc::now().to_rfc3339()],
        )?;

        if rows_affected == 0 {
            return Err(MemoryError::NotFound(format!("Memory {}", id)));
        }

        Ok(())
    }

    /// Convert a database row to a Memory struct.
    ///
    /// Expected column order: id, session_id, content_type, content, metadata, created_at,
    /// accessed_at, access_count, confidence_source, reinforcement_count, superseded,
    /// superseded_by, last_accessed, confidence_score, citation
    pub(crate) fn row_to_memory(row: &rusqlite::Row) -> Result<Memory> {
        let id_str: String = row.get(0)?;
        let session_id: Option<String> = row.get(1)?;
        let content_type_str: String = row.get(2)?;
        let content: String = row.get(3)?;
        let metadata_json: String = row.get(4)?;
        let created_at_str: String = row.get(5)?;
        let accessed_at_str: String = row.get(6)?;
        let access_count: u32 = row.get(7)?;

        // Confidence columns (8-13)
        let confidence_source_str: String = row.get(8)?;
        let reinforcement_count: u32 = row.get(9)?;
        let superseded_int: i32 = row.get(10)?;
        let superseded_by_str: Option<String> = row.get(11)?;
        let last_accessed_str: String = row.get(12)?;
        let confidence_score: f32 = row.get(13)?;

        // Citation column (14)
        let citation_json: Option<String> = row.get(14)?;

        let id = MemoryId::parse(&id_str)?;
        let content_type = ContentType::parse(&content_type_str).ok_or_else(|| {
            MemoryError::InvalidData(format!("Unknown content type: {}", content_type_str))
        })?;
        let metadata: Metadata = serde_json::from_str(&metadata_json)?;
        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|e| MemoryError::InvalidData(e.to_string()))?
            .with_timezone(&Utc);
        let accessed_at = DateTime::parse_from_rfc3339(&accessed_at_str)
            .map_err(|e| MemoryError::InvalidData(e.to_string()))?
            .with_timezone(&Utc);
        let confidence_source = ConfidenceSource::from_db_str(&confidence_source_str)
            .unwrap_or(ConfidenceSource::Inferred);
        let superseded_by = superseded_by_str
            .as_deref()
            .map(MemoryId::parse)
            .transpose()?;
        let last_accessed = DateTime::parse_from_rfc3339(&last_accessed_str)
            .map_err(|e| MemoryError::InvalidData(e.to_string()))?
            .with_timezone(&Utc);
        let citation: Option<Citation> = citation_json
            .as_deref()
            .map(serde_json::from_str)
            .transpose()?;

        Ok(Memory {
            id,
            session_id,
            content_type,
            content,
            metadata,
            created_at,
            accessed_at,
            access_count,
            confidence: MemoryConfidence {
                source: confidence_source,
                reinforcement_count,
                superseded: superseded_int != 0,
                superseded_by,
                last_accessed,
                score: confidence_score,
            },
            citation,
        })
    }

    /// Find existing non-superseded memories that match the given subject and predicate.
    ///
    /// Used for contradiction detection: if a new fact has the same subject+predicate
    /// but a different value, the old memory should be superseded.
    pub fn find_contradictions(&self, subject: &str, predicate: &str) -> Result<Vec<Memory>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            r#"
            SELECT id, session_id, content_type, content, metadata, created_at, accessed_at, access_count,
                   confidence_source, reinforcement_count, superseded, superseded_by,
                   last_accessed, confidence_score, citation
            FROM memories
            WHERE json_extract(metadata, '$.subject') = ?1
              AND json_extract(metadata, '$.predicate') = ?2
              AND superseded = 0
            ORDER BY created_at DESC
            "#,
        )?;

        let mut rows = stmt.query(params![subject, predicate])?;
        let mut memories = Vec::new();
        while let Some(row) = rows.next()? {
            memories.push(Self::row_to_memory(row)?);
        }

        Ok(memories)
    }

    /// Mark a memory as superseded by another memory.
    ///
    /// Sets `superseded=true`, `superseded_by=new_id`, and `confidence_score=0.0`.
    pub fn supersede(&self, old_id: MemoryId, new_id: MemoryId) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        let rows_affected = conn.execute(
            r#"
            UPDATE memories
            SET superseded = 1, superseded_by = ?2, confidence_score = 0.0
            WHERE id = ?1
            "#,
            params![old_id.to_string(), new_id.to_string()],
        )?;

        if rows_affected == 0 {
            return Err(MemoryError::NotFound(format!("Memory {}", old_id)));
        }

        debug!("Superseded memory {} by {}", old_id, new_id);
        Ok(())
    }

    /// Reinforce a memory by incrementing its reinforcement count and updating last_accessed.
    ///
    /// This is called when a fact is confirmed (same subject+predicate+content).
    pub fn reinforce(&self, id: MemoryId) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        let rows_affected = conn.execute(
            r#"
            UPDATE memories
            SET reinforcement_count = reinforcement_count + 1,
                last_accessed = ?2
            WHERE id = ?1
            "#,
            params![id.to_string(), Utc::now().to_rfc3339()],
        )?;

        if rows_affected == 0 {
            return Err(MemoryError::NotFound(format!("Memory {}", id)));
        }

        debug!("Reinforced memory {}", id);
        Ok(())
    }

    /// Update the last_accessed timestamp on a memory (e.g., when recalled).
    pub fn update_last_accessed(&self, id: MemoryId) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        let rows_affected = conn.execute(
            "UPDATE memories SET last_accessed = ?2 WHERE id = ?1",
            params![id.to_string(), Utc::now().to_rfc3339()],
        )?;

        if rows_affected == 0 {
            return Err(MemoryError::NotFound(format!("Memory {}", id)));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_store() -> MemoryStore {
        MemoryStore::open_in_memory().unwrap()
    }

    #[test]
    fn test_memory_crud() {
        let store = create_test_store();

        // Create
        let memory = Memory::new(ContentType::Note, "Test content").with_tag("test");
        store.insert_memory(&memory).unwrap();

        // Read
        let fetched = store.get_memory(memory.id).unwrap().unwrap();
        assert_eq!(fetched.content, "Test content");
        assert_eq!(fetched.content_type, ContentType::Note);

        // Update
        let mut updated = fetched;
        updated.content = "Updated content".to_string();
        store.update_memory(&updated).unwrap();

        let fetched = store.get_memory(memory.id).unwrap().unwrap();
        assert_eq!(fetched.content, "Updated content");

        // Delete
        assert!(store.delete_memory(memory.id).unwrap());
        assert!(store.get_memory(memory.id).unwrap().is_none());
    }

    #[test]
    fn test_memory_list_and_count() {
        let store = create_test_store();

        for i in 0..5 {
            let memory = Memory::new(ContentType::Note, format!("Note {}", i));
            store.insert_memory(&memory).unwrap();
        }
        for i in 0..3 {
            let memory = Memory::new(ContentType::Fact, format!("Fact {}", i));
            store.insert_memory(&memory).unwrap();
        }

        assert_eq!(store.count_memories(None).unwrap(), 8);
        assert_eq!(store.count_memories(Some(ContentType::Note)).unwrap(), 5);
        assert_eq!(store.count_memories(Some(ContentType::Fact)).unwrap(), 3);

        let all = store.list_memories(None, 100, 0).unwrap();
        assert_eq!(all.len(), 8);

        let page = store.list_memories(None, 3, 0).unwrap();
        assert_eq!(page.len(), 3);

        let notes = store
            .list_memories(Some(ContentType::Note), 100, 0)
            .unwrap();
        assert_eq!(notes.len(), 5);
    }

    #[test]
    fn test_touch_memory() {
        let store = create_test_store();

        let memory = Memory::new(ContentType::Note, "Test");
        store.insert_memory(&memory).unwrap();

        let before = store.get_memory(memory.id).unwrap().unwrap();
        assert_eq!(before.access_count, 0);

        store.touch_memory(memory.id).unwrap();
        store.touch_memory(memory.id).unwrap();

        let after = store.get_memory(memory.id).unwrap().unwrap();
        assert_eq!(after.access_count, 2);
        assert!(after.accessed_at > before.accessed_at);
    }

    fn make_fact(subject: &str, predicate: &str, content: &str) -> Memory {
        let mut memory = Memory::new(ContentType::Fact, content);
        memory.metadata.subject = Some(subject.to_string());
        memory.metadata.predicate = Some(predicate.to_string());
        memory
    }

    #[test]
    fn test_find_contradictions() {
        let store = create_test_store();

        let m1 = make_fact("user.model", "is", "GPT-4");
        store.insert_memory(&m1).unwrap();

        let m2 = make_fact("user.model", "is", "Claude");
        store.insert_memory(&m2).unwrap();

        // Different predicate — not a contradiction
        let m3 = make_fact("user.model", "was", "GPT-3");
        store.insert_memory(&m3).unwrap();

        let results = store.find_contradictions("user.model", "is").unwrap();
        assert_eq!(results.len(), 2);

        let results = store.find_contradictions("user.model", "was").unwrap();
        assert_eq!(results.len(), 1);

        let results = store.find_contradictions("nonexistent", "is").unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_supersede() {
        let store = create_test_store();

        let m1 = make_fact("user.model", "is", "GPT-4");
        store.insert_memory(&m1).unwrap();

        let m2 = make_fact("user.model", "is", "Claude");
        store.insert_memory(&m2).unwrap();

        store.supersede(m1.id, m2.id).unwrap();

        let old = store.get_memory(m1.id).unwrap().unwrap();
        assert!(old.confidence.superseded);
        assert_eq!(old.confidence.superseded_by, Some(m2.id));
        assert_eq!(old.confidence.score, 0.0);

        // Superseded memories excluded from find_contradictions
        let results = store.find_contradictions("user.model", "is").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, m2.id);
    }

    #[test]
    fn test_supersede_not_found() {
        let store = create_test_store();
        let fake = MemoryId::new();
        let other = MemoryId::new();
        assert!(store.supersede(fake, other).is_err());
    }

    #[test]
    fn test_reinforce() {
        let store = create_test_store();

        let m = make_fact("user.model", "is", "Claude");
        store.insert_memory(&m).unwrap();

        let before = store.get_memory(m.id).unwrap().unwrap();
        assert_eq!(before.confidence.reinforcement_count, 0);

        store.reinforce(m.id).unwrap();
        store.reinforce(m.id).unwrap();

        let after = store.get_memory(m.id).unwrap().unwrap();
        assert_eq!(after.confidence.reinforcement_count, 2);
        assert!(after.confidence.last_accessed >= before.confidence.last_accessed);
    }

    #[test]
    fn test_reinforce_not_found() {
        let store = create_test_store();
        assert!(store.reinforce(MemoryId::new()).is_err());
    }

    #[test]
    fn test_update_last_accessed() {
        let store = create_test_store();

        let m = Memory::new(ContentType::Note, "Test");
        store.insert_memory(&m).unwrap();

        let before = store.get_memory(m.id).unwrap().unwrap();

        // Small delay to ensure timestamp differs
        std::thread::sleep(std::time::Duration::from_millis(10));
        store.update_last_accessed(m.id).unwrap();

        let after = store.get_memory(m.id).unwrap().unwrap();
        assert!(after.confidence.last_accessed >= before.confidence.last_accessed);
    }

    #[test]
    fn test_update_last_accessed_not_found() {
        let store = create_test_store();
        assert!(store.update_last_accessed(MemoryId::new()).is_err());
    }
}
