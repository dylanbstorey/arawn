use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::Result;
use crate::types::{MessageRole, WorkstreamMessage};

/// Append-only JSONL message store. One file per workstream.
///
/// Layout: `{data_dir}/workstreams/{workstream_id}/messages.jsonl`
pub struct MessageStore {
    data_dir: PathBuf,
}

impl MessageStore {
    pub fn new(data_dir: &Path) -> Self {
        Self {
            data_dir: data_dir.to_path_buf(),
        }
    }

    /// Append a message to the workstream's JSONL file. Returns the message with generated id/timestamp.
    pub fn append(
        &self,
        workstream_id: &str,
        session_id: Option<&str>,
        role: MessageRole,
        content: &str,
        metadata: Option<&str>,
    ) -> Result<WorkstreamMessage> {
        let msg = WorkstreamMessage {
            id: Uuid::new_v4().to_string(),
            workstream_id: workstream_id.to_string(),
            session_id: session_id.map(String::from),
            role,
            content: content.to_string(),
            timestamp: Utc::now(),
            metadata: metadata.map(String::from),
        };

        let dir = self.workstream_dir(workstream_id);
        fs::create_dir_all(&dir)?;

        let path = dir.join("messages.jsonl");
        let mut file = OpenOptions::new().create(true).append(true).open(&path)?;

        let mut line = serde_json::to_string(&msg)?;
        line.push('\n');
        file.write_all(line.as_bytes())?;
        // Ensure data is persisted to disk before returning success
        file.sync_all()?;

        Ok(msg)
    }

    /// Read all messages for a workstream.
    pub fn read_all(&self, workstream_id: &str) -> Result<Vec<WorkstreamMessage>> {
        let path = self.jsonl_path(workstream_id);
        if !path.exists() {
            return Ok(Vec::new());
        }

        let file = fs::File::open(&path)?;
        let reader = BufReader::new(file);
        let mut messages = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let msg: WorkstreamMessage = serde_json::from_str(&line)?;
            messages.push(msg);
        }

        Ok(messages)
    }

    /// Read messages after a given timestamp.
    pub fn read_range(
        &self,
        workstream_id: &str,
        since: DateTime<Utc>,
    ) -> Result<Vec<WorkstreamMessage>> {
        let all = self.read_all(workstream_id)?;
        Ok(all.into_iter().filter(|m| m.timestamp >= since).collect())
    }

    /// Read all messages for a specific session.
    pub fn read_for_session(
        &self,
        workstream_id: &str,
        session_id: &str,
    ) -> Result<Vec<WorkstreamMessage>> {
        let all = self.read_all(workstream_id)?;
        Ok(all
            .into_iter()
            .filter(|m| m.session_id.as_deref() == Some(session_id))
            .collect())
    }

    /// Path to a workstream's data directory.
    pub fn workstream_dir(&self, workstream_id: &str) -> PathBuf {
        self.data_dir.join("workstreams").join(workstream_id)
    }

    /// Path to a workstream's JSONL file.
    pub fn jsonl_path(&self, workstream_id: &str) -> PathBuf {
        self.workstream_dir(workstream_id).join("messages.jsonl")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_store() -> (tempfile::TempDir, MessageStore) {
        let dir = tempfile::tempdir().unwrap();
        let store = MessageStore::new(dir.path());
        (dir, store)
    }

    #[test]
    fn test_append_and_read_all() {
        let (_dir, store) = temp_store();

        let msg = store
            .append("ws-1", Some("s-1"), MessageRole::User, "hello", None)
            .unwrap();
        assert_eq!(msg.workstream_id, "ws-1");
        assert_eq!(msg.role, MessageRole::User);
        assert_eq!(msg.content, "hello");

        let msgs = store.read_all("ws-1").unwrap();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].id, msg.id);
    }

    #[test]
    fn test_multi_message_append() {
        let (_dir, store) = temp_store();

        store
            .append("ws-1", None, MessageRole::User, "hi", None)
            .unwrap();
        store
            .append("ws-1", None, MessageRole::Assistant, "hello!", None)
            .unwrap();
        store
            .append("ws-1", None, MessageRole::User, "how are you?", None)
            .unwrap();

        let msgs = store.read_all("ws-1").unwrap();
        assert_eq!(msgs.len(), 3);
        assert_eq!(msgs[0].role, MessageRole::User);
        assert_eq!(msgs[1].role, MessageRole::Assistant);
        assert_eq!(msgs[2].role, MessageRole::User);
    }

    #[test]
    fn test_read_range() {
        let (_dir, store) = temp_store();

        let m1 = store
            .append("ws-1", None, MessageRole::User, "first", None)
            .unwrap();
        // All messages written nearly simultaneously, so use m1's timestamp + 1ms as cutoff
        let cutoff = m1.timestamp + chrono::Duration::milliseconds(1);
        std::thread::sleep(std::time::Duration::from_millis(5));
        store
            .append("ws-1", None, MessageRole::Assistant, "second", None)
            .unwrap();

        let range = store.read_range("ws-1", cutoff).unwrap();
        assert_eq!(range.len(), 1);
        assert_eq!(range[0].content, "second");
    }

    #[test]
    fn test_missing_workstream_returns_empty() {
        let (_dir, store) = temp_store();
        let msgs = store.read_all("nonexistent").unwrap();
        assert!(msgs.is_empty());
    }

    #[test]
    fn test_metadata_roundtrip() {
        let (_dir, store) = temp_store();

        let meta = r#"{"tool":"search","query":"rust"}"#;
        store
            .append(
                "ws-1",
                Some("s-1"),
                MessageRole::ToolResult,
                "results...",
                Some(meta),
            )
            .unwrap();

        let msgs = store.read_all("ws-1").unwrap();
        assert_eq!(msgs[0].metadata.as_deref(), Some(meta));
    }

    #[test]
    fn test_separate_workstreams() {
        let (_dir, store) = temp_store();

        store
            .append("ws-1", None, MessageRole::User, "a", None)
            .unwrap();
        store
            .append("ws-2", None, MessageRole::User, "b", None)
            .unwrap();

        assert_eq!(store.read_all("ws-1").unwrap().len(), 1);
        assert_eq!(store.read_all("ws-2").unwrap().len(), 1);
    }

    #[test]
    fn test_read_for_session() {
        let (_dir, store) = temp_store();

        // Messages in session-1
        store
            .append("ws-1", Some("session-1"), MessageRole::User, "hello", None)
            .unwrap();
        store
            .append(
                "ws-1",
                Some("session-1"),
                MessageRole::Assistant,
                "hi there",
                None,
            )
            .unwrap();

        // Messages in session-2
        store
            .append("ws-1", Some("session-2"), MessageRole::User, "different", None)
            .unwrap();

        // Messages with no session
        store
            .append("ws-1", None, MessageRole::User, "orphan", None)
            .unwrap();

        let session1_msgs = store.read_for_session("ws-1", "session-1").unwrap();
        assert_eq!(session1_msgs.len(), 2);
        assert_eq!(session1_msgs[0].content, "hello");
        assert_eq!(session1_msgs[1].content, "hi there");

        let session2_msgs = store.read_for_session("ws-1", "session-2").unwrap();
        assert_eq!(session2_msgs.len(), 1);
        assert_eq!(session2_msgs[0].content, "different");

        let empty_msgs = store.read_for_session("ws-1", "nonexistent").unwrap();
        assert!(empty_msgs.is_empty());
    }
}
