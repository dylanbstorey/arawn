use std::path::Path;

use chrono::{DateTime, Utc};
use parking_lot::Mutex;
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use tracing;
use uuid::Uuid;

use crate::{Result, WorkstreamError};

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("migrations");
}

/// A persistent conversational context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workstream {
    pub id: String,
    pub title: String,
    pub summary: Option<String>,
    pub is_scratch: bool,
    pub state: String,
    pub default_model: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A turn batch within a workstream.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub workstream_id: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub turn_count: Option<i32>,
    pub summary: Option<String>,
    pub compressed: bool,
}

/// Thin repository over SQLite for workstream operational data.
///
/// Thread-safe via internal `Mutex<Connection>`.
pub struct WorkstreamStore {
    conn: Mutex<Connection>,
}

impl WorkstreamStore {
    /// Open (or create) the database at `path` and run pending migrations.
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;

        let mut store = Self {
            conn: Mutex::new(conn),
        };
        store.run_migrations()?;
        Ok(store)
    }

    /// Open an in-memory database (for testing).
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;

        let mut store = Self {
            conn: Mutex::new(conn),
        };
        store.run_migrations()?;
        Ok(store)
    }

    fn run_migrations(&mut self) -> Result<()> {
        let conn = self.conn.get_mut();
        embedded::migrations::runner()
            .run(conn)
            .map_err(|e| WorkstreamError::Migration(e.to_string()))?;
        Ok(())
    }

    /// Lock the connection for use.
    fn conn(&self) -> parking_lot::MutexGuard<'_, Connection> {
        self.conn.lock()
    }

    // ── Workstream CRUD ─────────────────────────────────────────────

    pub fn create_workstream(
        &self,
        title: &str,
        default_model: Option<&str>,
        is_scratch: bool,
    ) -> Result<Workstream> {
        let id = if is_scratch {
            "scratch".to_string()
        } else {
            Uuid::new_v4().to_string()
        };
        let now = Utc::now();
        let now_str = now.to_rfc3339();

        self.conn().execute(
            "INSERT INTO workstreams (id, title, is_scratch, state, default_model, created_at, updated_at)
             VALUES (?1, ?2, ?3, 'active', ?4, ?5, ?6)",
            params![id, title, is_scratch as i32, default_model, now_str, now_str],
        )?;

        Ok(Workstream {
            id,
            title: title.to_string(),
            summary: None,
            is_scratch,
            state: "active".to_string(),
            default_model: default_model.map(String::from),
            created_at: now,
            updated_at: now,
        })
    }

    pub fn get_workstream(&self, id: &str) -> Result<Workstream> {
        self.conn()
            .query_row(
                "SELECT id, title, summary, is_scratch, state, default_model, created_at, updated_at
                 FROM workstreams WHERE id = ?1",
                params![id],
                |row| {
                    Ok(Workstream {
                        id: row.get(0)?,
                        title: row.get(1)?,
                        summary: row.get(2)?,
                        is_scratch: row.get::<_, i32>(3)? != 0,
                        state: row.get(4)?,
                        default_model: row.get(5)?,
                        created_at: parse_dt(&row.get::<_, String>(6)?),
                        updated_at: parse_dt(&row.get::<_, String>(7)?),
                    })
                },
            )
            .optional()?
            .ok_or_else(|| WorkstreamError::NotFound(id.to_string()))
    }

    pub fn list_workstreams(&self, state_filter: Option<&str>) -> Result<Vec<Workstream>> {
        let mut rows = Vec::new();

        if let Some(state) = state_filter {
            let conn = self.conn();
            let mut stmt = conn.prepare(
                "SELECT id, title, summary, is_scratch, state, default_model, created_at, updated_at
                 FROM workstreams WHERE state = ?1 ORDER BY updated_at DESC",
            )?;
            let iter = stmt.query_map(params![state], row_to_workstream)?;
            for r in iter {
                rows.push(r?);
            }
        } else {
            let conn = self.conn();
            let mut stmt = conn.prepare(
                "SELECT id, title, summary, is_scratch, state, default_model, created_at, updated_at
                 FROM workstreams ORDER BY updated_at DESC",
            )?;
            let iter = stmt.query_map([], row_to_workstream)?;
            for r in iter {
                rows.push(r?);
            }
        }

        Ok(rows)
    }

    pub fn update_workstream(
        &self,
        id: &str,
        title: Option<&str>,
        summary: Option<&str>,
        state: Option<&str>,
        default_model: Option<&str>,
    ) -> Result<()> {
        let now_str = Utc::now().to_rfc3339();

        // Build dynamic SET clause without leaking memory
        let mut set_parts: Vec<String> = vec!["updated_at = ?1".to_string()];
        let mut param_idx = 2u32;
        let mut values: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(now_str)];

        if let Some(t) = title {
            set_parts.push(format!("title = ?{}", param_idx));
            values.push(Box::new(t.to_string()));
            param_idx += 1;
        }
        if let Some(s) = summary {
            set_parts.push(format!("summary = ?{}", param_idx));
            values.push(Box::new(s.to_string()));
            param_idx += 1;
        }
        if let Some(st) = state {
            set_parts.push(format!("state = ?{}", param_idx));
            values.push(Box::new(st.to_string()));
            param_idx += 1;
        }
        if let Some(m) = default_model {
            set_parts.push(format!("default_model = ?{}", param_idx));
            values.push(Box::new(m.to_string()));
            param_idx += 1;
        }

        let sql = format!(
            "UPDATE workstreams SET {} WHERE id = ?{}",
            set_parts.join(", "),
            param_idx
        );
        values.push(Box::new(id.to_string()));

        let params: Vec<&dyn rusqlite::types::ToSql> = values.iter().map(|v| v.as_ref()).collect();
        let updated = self.conn().execute(&sql, params.as_slice())?;

        if updated == 0 {
            return Err(WorkstreamError::NotFound(id.to_string()));
        }
        Ok(())
    }

    // ── Bulk reassignment (for scratch promotion) ─────────────────

    /// Move all sessions from one workstream to another.
    pub fn reassign_sessions(&self, from_id: &str, to_id: &str) -> Result<()> {
        self.conn().execute(
            "UPDATE sessions SET workstream_id = ?1 WHERE workstream_id = ?2",
            params![to_id, from_id],
        )?;
        Ok(())
    }

    /// Move all tags from one workstream to another.
    pub fn reassign_tags(&self, from_id: &str, to_id: &str) -> Result<()> {
        self.conn().execute(
            "UPDATE workstream_tags SET workstream_id = ?1 WHERE workstream_id = ?2",
            params![to_id, from_id],
        )?;
        Ok(())
    }

    // ── Tags ────────────────────────────────────────────────────────

    pub fn set_tags(&self, workstream_id: &str, tags: &[String]) -> Result<()> {
        let conn = self.conn();
        conn.execute(
            "DELETE FROM workstream_tags WHERE workstream_id = ?1",
            params![workstream_id],
        )?;

        let mut stmt =
            conn.prepare("INSERT INTO workstream_tags (workstream_id, tag) VALUES (?1, ?2)")?;
        for tag in tags {
            stmt.execute(params![workstream_id, tag])?;
        }
        Ok(())
    }

    pub fn get_tags(&self, workstream_id: &str) -> Result<Vec<String>> {
        let conn = self.conn();
        let mut stmt =
            conn.prepare("SELECT tag FROM workstream_tags WHERE workstream_id = ?1 ORDER BY tag")?;
        let iter = stmt.query_map(params![workstream_id], |row| row.get(0))?;
        let mut tags = Vec::new();
        for t in iter {
            tags.push(t?);
        }
        Ok(tags)
    }

    // ── Sessions ────────────────────────────────────────────────────

    pub fn create_session(&self, workstream_id: &str) -> Result<Session> {
        let id = Uuid::new_v4().to_string();
        self.create_session_with_id(&id, workstream_id)
    }

    /// Create a session with a specific ID, or return existing if already exists.
    pub fn create_session_with_id(&self, id: &str, workstream_id: &str) -> Result<Session> {
        tracing::debug!(
            session_id = %id,
            workstream_id = %workstream_id,
            "WorkstreamStore::create_session_with_id called"
        );

        // Check if session already exists
        if let Ok(session) = self.get_session(id) {
            tracing::debug!(
                session_id = %id,
                existing_workstream_id = %session.workstream_id,
                "Session already exists, returning existing"
            );
            return Ok(session);
        }

        let now = Utc::now();
        let now_str = now.to_rfc3339();

        self.conn().execute(
            "INSERT INTO sessions (id, workstream_id, started_at) VALUES (?1, ?2, ?3)",
            params![id, workstream_id, now_str],
        )?;

        tracing::info!(
            session_id = %id,
            workstream_id = %workstream_id,
            "Created new session record in database"
        );

        Ok(Session {
            id: id.to_string(),
            workstream_id: workstream_id.to_string(),
            started_at: now,
            ended_at: None,
            turn_count: None,
            summary: None,
            compressed: false,
        })
    }

    pub fn get_session(&self, id: &str) -> Result<Session> {
        self.conn()
            .query_row(
                "SELECT id, workstream_id, started_at, ended_at, turn_count, summary, compressed
                 FROM sessions WHERE id = ?1",
                params![id],
                row_to_session,
            )
            .optional()?
            .ok_or_else(|| WorkstreamError::NotFound(id.to_string()))
    }

    pub fn get_active_session(&self, workstream_id: &str) -> Result<Option<Session>> {
        Ok(self
            .conn()
            .query_row(
                "SELECT id, workstream_id, started_at, ended_at, turn_count, summary, compressed
                 FROM sessions WHERE workstream_id = ?1 AND ended_at IS NULL
                 ORDER BY started_at DESC LIMIT 1",
                params![workstream_id],
                row_to_session,
            )
            .optional()?)
    }

    pub fn end_session(&self, id: &str, turn_count: i32) -> Result<()> {
        let now_str = Utc::now().to_rfc3339();
        let updated = self.conn().execute(
            "UPDATE sessions SET ended_at = ?1, turn_count = ?2 WHERE id = ?3 AND ended_at IS NULL",
            params![now_str, turn_count, id],
        )?;
        if updated == 0 {
            return Err(WorkstreamError::NotFound(id.to_string()));
        }
        Ok(())
    }

    pub fn update_session_summary(&self, id: &str, summary: &str) -> Result<()> {
        let updated = self.conn().execute(
            "UPDATE sessions SET summary = ?1, compressed = 1 WHERE id = ?2",
            params![summary, id],
        )?;
        if updated == 0 {
            return Err(WorkstreamError::NotFound(id.to_string()));
        }
        Ok(())
    }

    /// Move a session to a different workstream.
    pub fn reassign_session(&self, session_id: &str, new_workstream_id: &str) -> Result<Session> {
        tracing::info!(
            session_id = %session_id,
            new_workstream_id = %new_workstream_id,
            "WorkstreamStore::reassign_session called"
        );

        // Verify target workstream exists
        match self.get_workstream(new_workstream_id) {
            Ok(ws) => {
                tracing::debug!(
                    workstream_id = %ws.id,
                    workstream_title = %ws.title,
                    "Target workstream found"
                );
            }
            Err(e) => {
                tracing::error!(
                    new_workstream_id = %new_workstream_id,
                    error = %e,
                    "Target workstream not found"
                );
                return Err(e);
            }
        }

        // Update the session's workstream_id
        let updated = self.conn().execute(
            "UPDATE sessions SET workstream_id = ?1 WHERE id = ?2",
            params![new_workstream_id, session_id],
        )?;

        tracing::info!(
            session_id = %session_id,
            rows_updated = %updated,
            "UPDATE sessions executed"
        );

        if updated == 0 {
            tracing::error!(
                session_id = %session_id,
                "No session found with this ID - cannot reassign"
            );
            return Err(WorkstreamError::NotFound(session_id.to_string()));
        }

        // Return the updated session
        let session = self.get_session(session_id)?;
        tracing::info!(
            session_id = %session.id,
            workstream_id = %session.workstream_id,
            "Session reassigned successfully"
        );
        Ok(session)
    }

    pub fn list_sessions(&self, workstream_id: &str) -> Result<Vec<Session>> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT id, workstream_id, started_at, ended_at, turn_count, summary, compressed
             FROM sessions WHERE workstream_id = ?1 ORDER BY started_at DESC",
        )?;
        let iter = stmt.query_map(params![workstream_id], row_to_session)?;
        let mut sessions = Vec::new();
        for s in iter {
            sessions.push(s?);
        }
        Ok(sessions)
    }

    // ── Scratch ─────────────────────────────────────────────────────

    /// Ensure the well-known scratch workstream exists, creating it if missing.
    pub fn ensure_scratch(&self) -> Result<Workstream> {
        match self.get_workstream("scratch") {
            Ok(w) => Ok(w),
            Err(WorkstreamError::NotFound(_)) => self.create_workstream("Scratch", None, true),
            Err(e) => Err(e),
        }
    }
}

// ── Helpers ─────────────────────────────────────────────────────────

fn parse_dt(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|e| {
            tracing::warn!(
                input = %s,
                error = %e,
                "Failed to parse datetime, using current time as fallback"
            );
            Utc::now()
        })
}

fn row_to_workstream(row: &rusqlite::Row<'_>) -> rusqlite::Result<Workstream> {
    Ok(Workstream {
        id: row.get(0)?,
        title: row.get(1)?,
        summary: row.get(2)?,
        is_scratch: row.get::<_, i32>(3)? != 0,
        state: row.get(4)?,
        default_model: row.get(5)?,
        created_at: parse_dt(&row.get::<_, String>(6)?),
        updated_at: parse_dt(&row.get::<_, String>(7)?),
    })
}

fn row_to_session(row: &rusqlite::Row<'_>) -> rusqlite::Result<Session> {
    Ok(Session {
        id: row.get(0)?,
        workstream_id: row.get(1)?,
        started_at: parse_dt(&row.get::<_, String>(2)?),
        ended_at: row.get::<_, Option<String>>(3)?.map(|s| parse_dt(&s)),
        turn_count: row.get(4)?,
        summary: row.get(5)?,
        compressed: row.get::<_, i32>(6)? != 0,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// WorkstreamStorage Trait Implementation
// ─────────────────────────────────────────────────────────────────────────────

impl crate::storage::WorkstreamStorage for WorkstreamStore {
    fn create_workstream(
        &self,
        title: &str,
        default_model: Option<&str>,
        is_scratch: bool,
    ) -> Result<Workstream> {
        WorkstreamStore::create_workstream(self, title, default_model, is_scratch)
    }

    fn get_workstream(&self, id: &str) -> Result<Workstream> {
        WorkstreamStore::get_workstream(self, id)
    }

    fn list_workstreams(&self, state_filter: Option<&str>) -> Result<Vec<Workstream>> {
        WorkstreamStore::list_workstreams(self, state_filter)
    }

    fn update_workstream(
        &self,
        id: &str,
        title: Option<&str>,
        summary: Option<&str>,
        state: Option<&str>,
        default_model: Option<&str>,
    ) -> Result<()> {
        WorkstreamStore::update_workstream(self, id, title, summary, state, default_model)
    }

    fn set_tags(&self, workstream_id: &str, tags: &[String]) -> Result<()> {
        WorkstreamStore::set_tags(self, workstream_id, tags)
    }

    fn get_tags(&self, workstream_id: &str) -> Result<Vec<String>> {
        WorkstreamStore::get_tags(self, workstream_id)
    }

    fn create_session(&self, workstream_id: &str) -> Result<Session> {
        WorkstreamStore::create_session(self, workstream_id)
    }

    fn create_session_with_id(&self, session_id: &str, workstream_id: &str) -> Result<Session> {
        WorkstreamStore::create_session_with_id(self, session_id, workstream_id)
    }

    fn get_active_session(&self, workstream_id: &str) -> Result<Option<Session>> {
        WorkstreamStore::get_active_session(self, workstream_id)
    }

    fn list_sessions(&self, workstream_id: &str) -> Result<Vec<Session>> {
        WorkstreamStore::list_sessions(self, workstream_id)
    }

    fn end_session(&self, session_id: &str) -> Result<()> {
        // The trait doesn't take turn_count, so we use 0 as default
        WorkstreamStore::end_session(self, session_id, 0)
    }

    fn reassign_session(&self, session_id: &str, new_workstream_id: &str) -> Result<Session> {
        WorkstreamStore::reassign_session(self, session_id, new_workstream_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_store() -> WorkstreamStore {
        WorkstreamStore::open_in_memory().expect("failed to open in-memory store")
    }

    #[test]
    fn test_migrations_run() {
        let _store = test_store();
    }

    #[test]
    fn test_workstream_crud() {
        let store = test_store();

        let ws = store
            .create_workstream("Test Project", Some("claude-sonnet"), false)
            .unwrap();
        assert_eq!(ws.title, "Test Project");
        assert_eq!(ws.default_model.as_deref(), Some("claude-sonnet"));
        assert!(!ws.is_scratch);

        let fetched = store.get_workstream(&ws.id).unwrap();
        assert_eq!(fetched.title, "Test Project");

        store
            .update_workstream(&ws.id, Some("Renamed"), None, None, None)
            .unwrap();
        let updated = store.get_workstream(&ws.id).unwrap();
        assert_eq!(updated.title, "Renamed");

        let all = store.list_workstreams(None).unwrap();
        assert_eq!(all.len(), 1);

        let active = store.list_workstreams(Some("active")).unwrap();
        assert_eq!(active.len(), 1);

        let archived = store.list_workstreams(Some("archived")).unwrap();
        assert!(archived.is_empty());
    }

    #[test]
    fn test_tags() {
        let store = test_store();
        let ws = store.create_workstream("Tagged", None, false).unwrap();

        store
            .set_tags(&ws.id, &["rust".into(), "ai".into()])
            .unwrap();
        let tags = store.get_tags(&ws.id).unwrap();
        assert_eq!(tags, vec!["ai", "rust"]); // sorted

        // Replace tags
        store.set_tags(&ws.id, &["python".into()]).unwrap();
        let tags = store.get_tags(&ws.id).unwrap();
        assert_eq!(tags, vec!["python"]);
    }

    #[test]
    fn test_session_lifecycle() {
        let store = test_store();
        let ws = store.create_workstream("Sessions", None, false).unwrap();

        let session = store.create_session(&ws.id).unwrap();
        assert!(session.ended_at.is_none());

        let active = store.get_active_session(&ws.id).unwrap();
        assert!(active.is_some());
        assert_eq!(active.unwrap().id, session.id);

        store.end_session(&session.id, 5).unwrap();

        let ended = store.get_session(&session.id).unwrap();
        assert!(ended.ended_at.is_some());
        assert_eq!(ended.turn_count, Some(5));

        let active = store.get_active_session(&ws.id).unwrap();
        assert!(active.is_none());

        let all = store.list_sessions(&ws.id).unwrap();
        assert_eq!(all.len(), 1);
    }

    #[test]
    fn test_scratch_auto_creation() {
        let store = test_store();

        let scratch = store.ensure_scratch().unwrap();
        assert_eq!(scratch.id, "scratch");
        assert!(scratch.is_scratch);

        // Idempotent
        let scratch2 = store.ensure_scratch().unwrap();
        assert_eq!(scratch2.id, "scratch");
    }

    #[test]
    fn test_not_found() {
        let store = test_store();
        let err = store.get_workstream("nonexistent").unwrap_err();
        assert!(matches!(err, WorkstreamError::NotFound(_)));
    }
}
