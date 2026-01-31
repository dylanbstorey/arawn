//! Memory and knowledge types.

use serde::{Deserialize, Serialize};

use crate::{Id, Timestamp, new_id, now};

/// A memory entry stored in the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: Id,
    pub content: String,
    pub source: MemorySource,
    pub created_at: Timestamp,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<Id>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<String>,
}

impl Memory {
    /// Create a new memory from a conversation.
    pub fn from_conversation(content: impl Into<String>, session_id: Id) -> Self {
        Self {
            id: new_id(),
            content: content.into(),
            source: MemorySource::Conversation,
            created_at: now(),
            session_id: Some(session_id),
            tags: Vec::new(),
        }
    }

    /// Create a new memory from a note.
    pub fn from_note(content: impl Into<String>) -> Self {
        Self {
            id: new_id(),
            content: content.into(),
            source: MemorySource::Note,
            created_at: now(),
            session_id: None,
            tags: Vec::new(),
        }
    }
}

/// Source of a memory entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemorySource {
    Conversation,
    Note,
    Document,
    Tool,
    System,
}

/// A note created by the user or agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: Id,
    pub content: String,
    pub title: Option<String>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<String>,
}

impl Note {
    /// Create a new note.
    pub fn new(content: impl Into<String>) -> Self {
        let now = now();
        Self {
            id: new_id(),
            content: content.into(),
            title: None,
            created_at: now,
            updated_at: now,
            tags: Vec::new(),
        }
    }

    /// Create a new note with a title.
    pub fn with_title(title: impl Into<String>, content: impl Into<String>) -> Self {
        let now = now();
        Self {
            id: new_id(),
            content: content.into(),
            title: Some(title.into()),
            created_at: now,
            updated_at: now,
            tags: Vec::new(),
        }
    }
}

/// A graph entity (node) in the knowledge graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: Id,
    pub name: String,
    pub entity_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub created_at: Timestamp,
}

/// A relationship (edge) between entities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub id: Id,
    pub from_entity: Id,
    pub to_entity: Id,
    pub relation_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<serde_json::Value>,
    pub created_at: Timestamp,
}

/// Result of a memory search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub memory: Memory,
    pub score: f32,
}
