//! Note-taking tool.
//!
//! Provides a tool for creating and managing notes/memory during a session.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::error::Result;
use crate::tool::{Tool, ToolContext, ToolResult};

// ─────────────────────────────────────────────────────────────────────────────
// Note Storage
// ─────────────────────────────────────────────────────────────────────────────

/// A single note entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    /// Title of the note.
    pub title: String,
    /// Content of the note.
    pub content: String,
    /// When the note was created.
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// When the note was last updated.
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Note {
    /// Create a new note.
    pub fn new(title: impl Into<String>, content: impl Into<String>) -> Self {
        let now = chrono::Utc::now();
        Self {
            title: title.into(),
            content: content.into(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Update the note content.
    pub fn update(&mut self, content: impl Into<String>) {
        self.content = content.into();
        self.updated_at = chrono::Utc::now();
    }
}

/// Shared storage for notes.
pub type NoteStorage = Arc<RwLock<HashMap<String, Note>>>;

/// Create a new note storage.
pub fn new_note_storage() -> NoteStorage {
    Arc::new(RwLock::new(HashMap::new()))
}

// ─────────────────────────────────────────────────────────────────────────────
// Note Tool
// ─────────────────────────────────────────────────────────────────────────────

/// Tool for creating and managing notes.
#[derive(Debug, Clone)]
pub struct NoteTool {
    storage: NoteStorage,
}

impl NoteTool {
    /// Create a new note tool with its own storage.
    pub fn new() -> Self {
        Self {
            storage: new_note_storage(),
        }
    }

    /// Create a note tool with shared storage.
    pub fn with_storage(storage: NoteStorage) -> Self {
        Self { storage }
    }

    /// Get the underlying storage.
    pub fn storage(&self) -> &NoteStorage {
        &self.storage
    }

    /// Get all notes (for inspection/testing).
    pub fn get_all_notes(&self) -> HashMap<String, Note> {
        self.storage.read().unwrap().clone()
    }

    /// Get a specific note by title.
    pub fn get_note(&self, title: &str) -> Option<Note> {
        self.storage.read().unwrap().get(title).cloned()
    }
}

impl Default for NoteTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for NoteTool {
    fn name(&self) -> &str {
        "note"
    }

    fn description(&self) -> &str {
        "Create, update, or retrieve notes. Use this to remember important information during a session. Notes persist throughout the conversation."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["create", "update", "get", "list", "delete"],
                    "description": "The action to perform: create a new note, update an existing note, get a specific note, list all notes, or delete a note"
                },
                "title": {
                    "type": "string",
                    "description": "The title of the note (required for create, update, get, delete)"
                },
                "content": {
                    "type": "string",
                    "description": "The content of the note (required for create and update)"
                }
            },
            "required": ["action"]
        })
    }

    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        // Check cancellation
        if ctx.is_cancelled() {
            return Ok(ToolResult::error("Operation cancelled"));
        }

        // Extract action parameter
        let action = params
            .get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::AgentError::Tool("Missing 'action' parameter".to_string())
            })?;

        match action {
            "create" => self.create_note(&params),
            "update" => self.update_note(&params),
            "get" => self.get_note_action(&params),
            "list" => self.list_notes(),
            "delete" => self.delete_note(&params),
            _ => Ok(ToolResult::error(format!("Unknown action: {}", action))),
        }
    }
}

impl NoteTool {
    fn create_note(&self, params: &Value) -> Result<ToolResult> {
        let title = params
            .get("title")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::AgentError::Tool("Missing 'title' parameter for create".to_string())
            })?;

        let content = params
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::AgentError::Tool("Missing 'content' parameter for create".to_string())
            })?;

        let mut storage = self.storage.write().unwrap();

        if storage.contains_key(title) {
            return Ok(ToolResult::error(format!(
                "Note '{}' already exists. Use 'update' action to modify it.",
                title
            )));
        }

        let note = Note::new(title, content);
        storage.insert(title.to_string(), note);

        Ok(ToolResult::text(format!("Created note '{}'", title)))
    }

    fn update_note(&self, params: &Value) -> Result<ToolResult> {
        let title = params
            .get("title")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::AgentError::Tool("Missing 'title' parameter for update".to_string())
            })?;

        let content = params
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::AgentError::Tool("Missing 'content' parameter for update".to_string())
            })?;

        let mut storage = self.storage.write().unwrap();

        if let Some(note) = storage.get_mut(title) {
            note.update(content);
            Ok(ToolResult::text(format!("Updated note '{}'", title)))
        } else {
            Ok(ToolResult::error(format!(
                "Note '{}' not found. Use 'create' action to create it.",
                title
            )))
        }
    }

    fn get_note_action(&self, params: &Value) -> Result<ToolResult> {
        let title = params
            .get("title")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::AgentError::Tool("Missing 'title' parameter for get".to_string())
            })?;

        let storage = self.storage.read().unwrap();

        if let Some(note) = storage.get(title) {
            Ok(ToolResult::json(json!({
                "title": note.title,
                "content": note.content,
                "created_at": note.created_at.to_rfc3339(),
                "updated_at": note.updated_at.to_rfc3339()
            })))
        } else {
            Ok(ToolResult::error(format!("Note '{}' not found", title)))
        }
    }

    fn list_notes(&self) -> Result<ToolResult> {
        let storage = self.storage.read().unwrap();

        if storage.is_empty() {
            return Ok(ToolResult::text("No notes found"));
        }

        let notes: Vec<Value> = storage
            .values()
            .map(|note| {
                json!({
                    "title": note.title,
                    "created_at": note.created_at.to_rfc3339(),
                    "updated_at": note.updated_at.to_rfc3339(),
                    "content_preview": if note.content.len() > 50 {
                        format!("{}...", &note.content[..50])
                    } else {
                        note.content.clone()
                    }
                })
            })
            .collect();

        Ok(ToolResult::json(json!({
            "count": notes.len(),
            "notes": notes
        })))
    }

    fn delete_note(&self, params: &Value) -> Result<ToolResult> {
        let title = params
            .get("title")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::AgentError::Tool("Missing 'title' parameter for delete".to_string())
            })?;

        let mut storage = self.storage.write().unwrap();

        if storage.remove(title).is_some() {
            Ok(ToolResult::text(format!("Deleted note '{}'", title)))
        } else {
            Ok(ToolResult::error(format!("Note '{}' not found", title)))
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_tool_metadata() {
        let tool = NoteTool::new();
        assert_eq!(tool.name(), "note");
        assert!(!tool.description().is_empty());

        let params = tool.parameters();
        assert!(params.get("properties").is_some());
        assert!(params["properties"].get("action").is_some());
    }

    #[test]
    fn test_note_creation() {
        let note = Note::new("Test", "Content");
        assert_eq!(note.title, "Test");
        assert_eq!(note.content, "Content");
        assert!(note.created_at <= chrono::Utc::now());
    }

    #[test]
    fn test_note_update() {
        let mut note = Note::new("Test", "Original");
        let original_created = note.created_at;

        std::thread::sleep(std::time::Duration::from_millis(10));
        note.update("Updated");

        assert_eq!(note.content, "Updated");
        assert_eq!(note.created_at, original_created);
        assert!(note.updated_at > original_created);
    }

    #[tokio::test]
    async fn test_create_note() {
        let tool = NoteTool::new();
        let ctx = ToolContext::default();

        let result = tool
            .execute(
                json!({
                    "action": "create",
                    "title": "My Note",
                    "content": "This is the content"
                }),
                &ctx,
            )
            .await
            .unwrap();

        assert!(result.is_success());
        assert!(result.to_llm_content().contains("Created note"));

        // Verify note was stored
        let note = tool.get_note("My Note").unwrap();
        assert_eq!(note.content, "This is the content");
    }

    #[tokio::test]
    async fn test_create_duplicate_note() {
        let tool = NoteTool::new();
        let ctx = ToolContext::default();

        // Create first note
        tool.execute(
            json!({
                "action": "create",
                "title": "Duplicate",
                "content": "First"
            }),
            &ctx,
        )
        .await
        .unwrap();

        // Try to create duplicate
        let result = tool
            .execute(
                json!({
                    "action": "create",
                    "title": "Duplicate",
                    "content": "Second"
                }),
                &ctx,
            )
            .await
            .unwrap();

        assert!(result.is_error());
        assert!(result.to_llm_content().contains("already exists"));
    }

    #[tokio::test]
    async fn test_update_note() {
        let tool = NoteTool::new();
        let ctx = ToolContext::default();

        // Create note
        tool.execute(
            json!({
                "action": "create",
                "title": "To Update",
                "content": "Original"
            }),
            &ctx,
        )
        .await
        .unwrap();

        // Update note
        let result = tool
            .execute(
                json!({
                    "action": "update",
                    "title": "To Update",
                    "content": "Updated content"
                }),
                &ctx,
            )
            .await
            .unwrap();

        assert!(result.is_success());

        // Verify update
        let note = tool.get_note("To Update").unwrap();
        assert_eq!(note.content, "Updated content");
    }

    #[tokio::test]
    async fn test_update_nonexistent_note() {
        let tool = NoteTool::new();
        let ctx = ToolContext::default();

        let result = tool
            .execute(
                json!({
                    "action": "update",
                    "title": "Nonexistent",
                    "content": "Content"
                }),
                &ctx,
            )
            .await
            .unwrap();

        assert!(result.is_error());
        assert!(result.to_llm_content().contains("not found"));
    }

    #[tokio::test]
    async fn test_get_note() {
        let tool = NoteTool::new();
        let ctx = ToolContext::default();

        // Create note
        tool.execute(
            json!({
                "action": "create",
                "title": "To Get",
                "content": "Get this content"
            }),
            &ctx,
        )
        .await
        .unwrap();

        // Get note
        let result = tool
            .execute(
                json!({
                    "action": "get",
                    "title": "To Get"
                }),
                &ctx,
            )
            .await
            .unwrap();

        assert!(result.is_success());
        let content = result.to_llm_content();
        assert!(content.contains("Get this content"));
    }

    #[tokio::test]
    async fn test_list_notes() {
        let tool = NoteTool::new();
        let ctx = ToolContext::default();

        // Create some notes
        for i in 1..=3 {
            tool.execute(
                json!({
                    "action": "create",
                    "title": format!("Note {}", i),
                    "content": format!("Content {}", i)
                }),
                &ctx,
            )
            .await
            .unwrap();
        }

        // List notes
        let result = tool.execute(json!({"action": "list"}), &ctx).await.unwrap();

        assert!(result.is_success());
        let content = result.to_llm_content();
        assert!(content.contains("\"count\":3") || content.contains("\"count\": 3"));
    }

    #[tokio::test]
    async fn test_list_empty_notes() {
        let tool = NoteTool::new();
        let ctx = ToolContext::default();

        let result = tool.execute(json!({"action": "list"}), &ctx).await.unwrap();

        assert!(result.is_success());
        assert!(result.to_llm_content().contains("No notes found"));
    }

    #[tokio::test]
    async fn test_delete_note() {
        let tool = NoteTool::new();
        let ctx = ToolContext::default();

        // Create note
        tool.execute(
            json!({
                "action": "create",
                "title": "To Delete",
                "content": "Delete me"
            }),
            &ctx,
        )
        .await
        .unwrap();

        // Delete note
        let result = tool
            .execute(
                json!({
                    "action": "delete",
                    "title": "To Delete"
                }),
                &ctx,
            )
            .await
            .unwrap();

        assert!(result.is_success());
        assert!(result.to_llm_content().contains("Deleted"));

        // Verify deletion
        assert!(tool.get_note("To Delete").is_none());
    }

    #[tokio::test]
    async fn test_shared_storage() {
        let storage = new_note_storage();
        let tool1 = NoteTool::with_storage(storage.clone());
        let tool2 = NoteTool::with_storage(storage);
        let ctx = ToolContext::default();

        // Create note with tool1
        tool1
            .execute(
                json!({
                    "action": "create",
                    "title": "Shared",
                    "content": "Shared content"
                }),
                &ctx,
            )
            .await
            .unwrap();

        // Should be visible in tool2
        let note = tool2.get_note("Shared").unwrap();
        assert_eq!(note.content, "Shared content");
    }
}
