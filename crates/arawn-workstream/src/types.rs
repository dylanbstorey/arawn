use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Role of a message within a workstream.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    User,
    Assistant,
    System,
    /// A tool call made by the assistant.
    /// Metadata should contain: tool_id, name, arguments (JSON).
    ToolUse,
    /// Result of a tool execution.
    /// Metadata should contain: tool_call_id, success.
    ToolResult,
    AgentPush,
}

impl MessageRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::System => "system",
            Self::ToolUse => "tool_use",
            Self::ToolResult => "tool_result",
            Self::AgentPush => "agent_push",
        }
    }
}

impl std::fmt::Display for MessageRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A single message in a workstream's conversation history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkstreamMessage {
    pub id: String,
    pub workstream_id: String,
    pub session_id: Option<String>,
    pub role: MessageRole,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    /// Arbitrary metadata stored as a JSON string.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<String>,
}
