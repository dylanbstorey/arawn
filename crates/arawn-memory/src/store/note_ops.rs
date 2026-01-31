//! Note CRUD, search, and tag operations.

use chrono::{DateTime, Utc};
use rusqlite::params;
use tracing::debug;

use crate::error::{MemoryError, Result};
use crate::types::{Note, NoteId};

use super::MemoryStore;

impl MemoryStore {
    /// Insert a new note.
    pub fn insert_note(&self, note: &Note) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        let tags_json = serde_json::to_string(&note.tags)?;

        conn.execute(
            r#"
            INSERT INTO notes (id, title, content, tags, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
            params![
                note.id.to_string(),
                note.title,
                note.content,
                tags_json,
                note.created_at.to_rfc3339(),
                note.updated_at.to_rfc3339(),
            ],
        )?;

        debug!("Inserted note {}", note.id);
        Ok(())
    }

    /// Get a note by ID.
    pub fn get_note(&self, id: NoteId) -> Result<Option<Note>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT id, title, content, tags, created_at, updated_at FROM notes WHERE id = ?1",
        )?;

        let mut rows = stmt.query(params![id.to_string()])?;

        if let Some(row) = rows.next()? {
            let note = Self::row_to_note(row)?;
            Ok(Some(note))
        } else {
            Ok(None)
        }
    }

    /// Update a note.
    pub fn update_note(&self, note: &Note) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        let tags_json = serde_json::to_string(&note.tags)?;

        let rows_affected = conn.execute(
            r#"
            UPDATE notes
            SET title = ?2, content = ?3, tags = ?4, updated_at = ?5
            WHERE id = ?1
            "#,
            params![
                note.id.to_string(),
                note.title,
                note.content,
                tags_json,
                note.updated_at.to_rfc3339(),
            ],
        )?;

        if rows_affected == 0 {
            return Err(MemoryError::NotFound(format!("Note {}", note.id)));
        }

        Ok(())
    }

    /// Delete a note by ID.
    pub fn delete_note(&self, id: NoteId) -> Result<bool> {
        let conn = self.conn.lock().unwrap();

        let rows_affected =
            conn.execute("DELETE FROM notes WHERE id = ?1", params![id.to_string()])?;

        Ok(rows_affected > 0)
    }

    /// List notes ordered by updated_at descending.
    pub fn list_notes(&self, limit: usize, offset: usize) -> Result<Vec<Note>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            r#"
            SELECT id, title, content, tags, created_at, updated_at
            FROM notes
            ORDER BY updated_at DESC
            LIMIT ?1 OFFSET ?2
            "#,
        )?;

        let mut rows = stmt.query(params![limit as i64, offset as i64])?;

        let mut notes = Vec::new();
        while let Some(row) = rows.next()? {
            notes.push(Self::row_to_note(row)?);
        }

        Ok(notes)
    }

    /// Search notes by content or title.
    pub fn search_notes(&self, query: &str, limit: usize) -> Result<Vec<Note>> {
        let conn = self.conn.lock().unwrap();

        let pattern = format!("%{}%", query);

        let mut stmt = conn.prepare(
            r#"
            SELECT id, title, content, tags, created_at, updated_at
            FROM notes
            WHERE content LIKE ?1 OR title LIKE ?1
            ORDER BY updated_at DESC
            LIMIT ?2
            "#,
        )?;

        let mut rows = stmt.query(params![pattern, limit as i64])?;

        let mut notes = Vec::new();
        while let Some(row) = rows.next()? {
            notes.push(Self::row_to_note(row)?);
        }

        Ok(notes)
    }

    /// List notes that have a specific tag.
    pub fn list_notes_by_tag(&self, tag: &str, limit: usize) -> Result<Vec<Note>> {
        let conn = self.conn.lock().unwrap();

        let pattern = format!("%\"{}\"%", tag);

        let mut stmt = conn.prepare(
            r#"
            SELECT id, title, content, tags, created_at, updated_at
            FROM notes
            WHERE tags LIKE ?1
            ORDER BY updated_at DESC
            LIMIT ?2
            "#,
        )?;

        let mut rows = stmt.query(params![pattern, limit as i64])?;

        let mut notes = Vec::new();
        while let Some(row) = rows.next()? {
            notes.push(Self::row_to_note(row)?);
        }

        Ok(notes)
    }

    /// List notes that have all of the specified tags.
    pub fn list_notes_by_tags(&self, tags: &[&str], limit: usize) -> Result<Vec<Note>> {
        if tags.is_empty() {
            return self.list_notes(limit, 0);
        }

        let conn = self.conn.lock().unwrap();

        let conditions: Vec<String> = tags
            .iter()
            .enumerate()
            .map(|(i, _)| format!("tags LIKE ?{}", i + 1))
            .collect();
        let where_clause = conditions.join(" AND ");

        let sql = format!(
            r#"
            SELECT id, title, content, tags, created_at, updated_at
            FROM notes
            WHERE {}
            ORDER BY updated_at DESC
            LIMIT ?{}
            "#,
            where_clause,
            tags.len() + 1
        );

        let mut stmt = conn.prepare(&sql)?;

        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = tags
            .iter()
            .map(|tag| Box::new(format!("%\"{}\"%", tag)) as Box<dyn rusqlite::ToSql>)
            .collect();
        params_vec.push(Box::new(limit as i64));

        let params_refs: Vec<&dyn rusqlite::ToSql> =
            params_vec.iter().map(|b| b.as_ref()).collect();
        let mut rows = stmt.query(params_refs.as_slice())?;

        let mut notes = Vec::new();
        while let Some(row) = rows.next()? {
            notes.push(Self::row_to_note(row)?);
        }

        Ok(notes)
    }

    /// Count notes with a specific tag.
    pub fn count_notes_by_tag(&self, tag: &str) -> Result<usize> {
        let conn = self.conn.lock().unwrap();

        let pattern = format!("%\"{}\"%", tag);

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM notes WHERE tags LIKE ?1",
            params![pattern],
            |row| row.get(0),
        )?;

        Ok(count as usize)
    }

    /// Convert a database row to a Note struct.
    pub(crate) fn row_to_note(row: &rusqlite::Row) -> Result<Note> {
        let id_str: String = row.get(0)?;
        let title: Option<String> = row.get(1)?;
        let content: String = row.get(2)?;
        let tags_json: String = row.get(3)?;
        let created_at_str: String = row.get(4)?;
        let updated_at_str: String = row.get(5)?;

        let id = NoteId::parse(&id_str)?;
        let tags: Vec<String> = serde_json::from_str(&tags_json)?;
        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|e| MemoryError::InvalidData(e.to_string()))?
            .with_timezone(&Utc);
        let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
            .map_err(|e| MemoryError::InvalidData(e.to_string()))?
            .with_timezone(&Utc);

        Ok(Note {
            id,
            title,
            content,
            tags,
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

    #[test]
    fn test_note_crud() {
        let store = create_test_store();

        let note = Note::new("Test note content")
            .with_title("My Note")
            .with_tag("important");
        store.insert_note(&note).unwrap();

        let fetched = store.get_note(note.id).unwrap().unwrap();
        assert_eq!(fetched.content, "Test note content");
        assert_eq!(fetched.title, Some("My Note".to_string()));
        assert!(fetched.tags.contains(&"important".to_string()));

        let mut updated = fetched;
        updated.content = "Updated content".to_string();
        store.update_note(&updated).unwrap();

        let fetched = store.get_note(note.id).unwrap().unwrap();
        assert_eq!(fetched.content, "Updated content");

        assert!(store.delete_note(note.id).unwrap());
        assert!(store.get_note(note.id).unwrap().is_none());
    }

    #[test]
    fn test_note_search() {
        let store = create_test_store();

        store
            .insert_note(&Note::new("The quick brown fox"))
            .unwrap();
        store.insert_note(&Note::new("Lazy dog sleeping")).unwrap();
        store
            .insert_note(&Note::new("Another note").with_title("Fox title"))
            .unwrap();

        let results = store.search_notes("fox", 10).unwrap();
        assert_eq!(results.len(), 2);

        let results = store.search_notes("dog", 10).unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_list_notes_by_tag() {
        let store = create_test_store();

        store
            .insert_note(&Note::new("Note 1").with_tag("rust"))
            .unwrap();
        store
            .insert_note(&Note::new("Note 2").with_tag("python"))
            .unwrap();
        store
            .insert_note(&Note::new("Note 3").with_tag("rust").with_tag("async"))
            .unwrap();
        store
            .insert_note(&Note::new("Note 4").with_tag("rust"))
            .unwrap();

        let rust_notes = store.list_notes_by_tag("rust", 10).unwrap();
        assert_eq!(rust_notes.len(), 3);

        let python_notes = store.list_notes_by_tag("python", 10).unwrap();
        assert_eq!(python_notes.len(), 1);

        let async_notes = store.list_notes_by_tag("async", 10).unwrap();
        assert_eq!(async_notes.len(), 1);

        let missing_notes = store.list_notes_by_tag("nonexistent", 10).unwrap();
        assert_eq!(missing_notes.len(), 0);
    }

    #[test]
    fn test_list_notes_by_tags_multiple() {
        let store = create_test_store();

        store
            .insert_note(&Note::new("Note 1").with_tag("rust").with_tag("async"))
            .unwrap();
        store
            .insert_note(&Note::new("Note 2").with_tag("rust"))
            .unwrap();
        store
            .insert_note(&Note::new("Note 3").with_tag("async"))
            .unwrap();
        store
            .insert_note(
                &Note::new("Note 4")
                    .with_tag("rust")
                    .with_tag("async")
                    .with_tag("tokio"),
            )
            .unwrap();

        let both_tags = store.list_notes_by_tags(&["rust", "async"], 10).unwrap();
        assert_eq!(both_tags.len(), 2);

        let all_three = store
            .list_notes_by_tags(&["rust", "async", "tokio"], 10)
            .unwrap();
        assert_eq!(all_three.len(), 1);
    }

    #[test]
    fn test_count_notes_by_tag() {
        let store = create_test_store();

        store
            .insert_note(&Note::new("Note 1").with_tag("important"))
            .unwrap();
        store
            .insert_note(&Note::new("Note 2").with_tag("important"))
            .unwrap();
        store
            .insert_note(&Note::new("Note 3").with_tag("todo"))
            .unwrap();

        assert_eq!(store.count_notes_by_tag("important").unwrap(), 2);
        assert_eq!(store.count_notes_by_tag("todo").unwrap(), 1);
        assert_eq!(store.count_notes_by_tag("missing").unwrap(), 0);
    }
}
