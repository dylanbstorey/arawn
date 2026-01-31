//! Message types for conversations and chat.

use serde::{Deserialize, Serialize};

use crate::{Id, Timestamp, new_id, now};

/// Role in a conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
    System,
    Tool,
}

/// A message in a conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Id,
    pub role: Role,
    pub content: String,
    pub timestamp: Timestamp,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
}

impl Message {
    /// Create a new user message.
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            id: new_id(),
            role: Role::User,
            content: content.into(),
            timestamp: now(),
            tool_call_id: None,
            tool_name: None,
        }
    }

    /// Create a new assistant message.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            id: new_id(),
            role: Role::Assistant,
            content: content.into(),
            timestamp: now(),
            tool_call_id: None,
            tool_name: None,
        }
    }

    /// Create a new system message.
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            id: new_id(),
            role: Role::System,
            content: content.into(),
            timestamp: now(),
            tool_call_id: None,
            tool_name: None,
        }
    }

    /// Create a tool result message.
    pub fn tool_result(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: new_id(),
            role: Role::Tool,
            content: content.into(),
            timestamp: now(),
            tool_call_id: Some(tool_call_id.into()),
            tool_name: None,
        }
    }
}

/// A conversation session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Id,
    pub title: Option<String>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub messages: Vec<Message>,
}

impl Session {
    /// Create a new empty session.
    pub fn new() -> Self {
        let now = now();
        Self {
            id: new_id(),
            title: None,
            created_at: now,
            updated_at: now,
            messages: Vec::new(),
        }
    }

    /// Add a message to the session.
    pub fn add_message(&mut self, message: Message) {
        self.updated_at = now();
        self.messages.push(message);
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}
