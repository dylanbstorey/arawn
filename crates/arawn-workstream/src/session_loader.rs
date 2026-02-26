//! Session reconstruction from JSONL message history.
//!
//! This module provides the ability to reconstruct agent sessions from
//! persisted JSONL messages, making workstream sessions the single source
//! of truth for conversation history.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing;
use uuid::Uuid;

use crate::Result;
use crate::message_store::MessageStore;
use crate::types::{MessageRole, WorkstreamMessage};

/// Metadata for a tool use message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUseMetadata {
    /// Unique ID for this tool call.
    pub tool_id: String,
    /// Name of the tool being called.
    pub name: String,
    /// Arguments passed to the tool (as JSON).
    pub arguments: serde_json::Value,
}

/// Metadata for a tool result message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResultMetadata {
    /// ID of the tool call this is a result for.
    pub tool_call_id: String,
    /// Whether the tool execution succeeded.
    pub success: bool,
}

/// A reconstructed turn from JSONL messages.
#[derive(Debug, Clone)]
pub struct ReconstructedTurn {
    /// Turn ID (generated from first message timestamp if not available).
    pub id: String,
    /// The user's input message.
    pub user_message: String,
    /// The agent's response (if turn is complete).
    pub assistant_response: Option<String>,
    /// Tool calls made during this turn.
    pub tool_calls: Vec<ReconstructedToolCall>,
    /// Results from tool executions.
    pub tool_results: Vec<ReconstructedToolResult>,
    /// When this turn started.
    pub started_at: DateTime<Utc>,
    /// When this turn completed.
    pub completed_at: Option<DateTime<Utc>>,
}

/// A reconstructed tool call.
#[derive(Debug, Clone)]
pub struct ReconstructedToolCall {
    /// Unique ID for this tool call.
    pub id: String,
    /// Name of the tool being called.
    pub name: String,
    /// Arguments passed to the tool (JSON).
    pub arguments: serde_json::Value,
}

/// A reconstructed tool result.
#[derive(Debug, Clone)]
pub struct ReconstructedToolResult {
    /// ID of the tool call this is a result for.
    pub tool_call_id: String,
    /// Whether the tool execution succeeded.
    pub success: bool,
    /// Output content from the tool.
    pub content: String,
}

/// A fully reconstructed session from JSONL messages.
#[derive(Debug, Clone)]
pub struct ReconstructedSession {
    /// Session ID.
    pub session_id: String,
    /// Workstream ID this session belongs to.
    pub workstream_id: String,
    /// All reconstructed turns.
    pub turns: Vec<ReconstructedTurn>,
    /// When this session was created (first message timestamp).
    pub created_at: DateTime<Utc>,
    /// When this session was last updated (last message timestamp).
    pub updated_at: DateTime<Utc>,
}

/// Loads and reconstructs sessions from JSONL message history.
pub struct SessionLoader<'a> {
    message_store: &'a MessageStore,
}

impl<'a> SessionLoader<'a> {
    /// Create a new session loader.
    pub fn new(message_store: &'a MessageStore) -> Self {
        Self { message_store }
    }

    /// Load and reconstruct a session from JSONL messages.
    ///
    /// Returns `None` if no messages exist for this session.
    pub fn load_session(
        &self,
        workstream_id: &str,
        session_id: &str,
    ) -> Result<Option<ReconstructedSession>> {
        let messages = self
            .message_store
            .read_for_session(workstream_id, session_id)?;

        if messages.is_empty() {
            return Ok(None);
        }

        let turns = self.reconstruct_turns(&messages);
        let created_at = messages
            .first()
            .map(|m| m.timestamp)
            .unwrap_or_else(Utc::now);
        let updated_at = messages
            .last()
            .map(|m| m.timestamp)
            .unwrap_or_else(Utc::now);

        Ok(Some(ReconstructedSession {
            session_id: session_id.to_string(),
            workstream_id: workstream_id.to_string(),
            turns,
            created_at,
            updated_at,
        }))
    }

    /// Reconstruct turns from a list of messages.
    ///
    /// Messages are grouped into turns based on User messages starting a new turn.
    fn reconstruct_turns(&self, messages: &[WorkstreamMessage]) -> Vec<ReconstructedTurn> {
        let mut turns = Vec::new();
        let mut current_turn: Option<ReconstructedTurn> = None;

        for msg in messages {
            match msg.role {
                MessageRole::User => {
                    // Start a new turn; push the previous one if it exists
                    if let Some(turn) = current_turn.take() {
                        turns.push(turn);
                    }

                    current_turn = Some(ReconstructedTurn {
                        id: Uuid::new_v4().to_string(),
                        user_message: msg.content.clone(),
                        assistant_response: None,
                        tool_calls: Vec::new(),
                        tool_results: Vec::new(),
                        started_at: msg.timestamp,
                        completed_at: None,
                    });
                }
                MessageRole::Assistant => {
                    if let Some(ref mut turn) = current_turn {
                        turn.assistant_response = Some(msg.content.clone());
                        turn.completed_at = Some(msg.timestamp);
                    }
                }
                MessageRole::ToolUse => {
                    if let Some(ref mut turn) = current_turn {
                        if let Some(ref metadata_str) = msg.metadata {
                            match serde_json::from_str::<ToolUseMetadata>(metadata_str) {
                                Ok(meta) => {
                                    turn.tool_calls.push(ReconstructedToolCall {
                                        id: meta.tool_id,
                                        name: meta.name,
                                        arguments: meta.arguments,
                                    });
                                }
                                Err(e) => {
                                    tracing::warn!(
                                        message_id = %msg.id,
                                        error = %e,
                                        "Failed to parse ToolUse metadata, skipping tool call"
                                    );
                                }
                            }
                        } else {
                            tracing::warn!(
                                message_id = %msg.id,
                                "ToolUse message missing metadata, skipping tool call"
                            );
                        }
                    }
                }
                MessageRole::ToolResult => {
                    if let Some(ref mut turn) = current_turn {
                        let (tool_call_id, success) = if let Some(ref metadata_str) = msg.metadata {
                            match serde_json::from_str::<ToolResultMetadata>(metadata_str) {
                                Ok(meta) => (meta.tool_call_id, meta.success),
                                Err(e) => {
                                    tracing::warn!(
                                        message_id = %msg.id,
                                        error = %e,
                                        "Failed to parse ToolResult metadata, using defaults"
                                    );
                                    // Legacy format without proper metadata
                                    (String::new(), true)
                                }
                            }
                        } else {
                            tracing::debug!(
                                message_id = %msg.id,
                                "ToolResult message missing metadata (legacy format)"
                            );
                            (String::new(), true)
                        };

                        // Warn if tool_call_id is empty - makes correlation difficult
                        if tool_call_id.is_empty() {
                            tracing::debug!(
                                message_id = %msg.id,
                                "ToolResult has empty tool_call_id, correlation may fail"
                            );
                        }

                        turn.tool_results.push(ReconstructedToolResult {
                            tool_call_id,
                            success,
                            content: msg.content.clone(),
                        });
                    }
                }
                MessageRole::System | MessageRole::AgentPush => {
                    // System and agent push messages are context, not part of turns
                }
            }
        }

        // Don't forget the last turn
        if let Some(turn) = current_turn {
            turns.push(turn);
        }

        turns
    }

    /// Save a turn to JSONL storage.
    ///
    /// This persists all parts of a turn: user message, tool calls, tool results,
    /// and assistant response.
    pub fn save_turn(
        &self,
        workstream_id: &str,
        session_id: &str,
        user_message: &str,
        tool_calls: &[(String, String, serde_json::Value)], // (id, name, arguments)
        tool_results: &[(String, bool, String)],            // (tool_call_id, success, content)
        assistant_response: Option<&str>,
    ) -> Result<()> {
        // 1. Save user message
        self.message_store.append(
            workstream_id,
            Some(session_id),
            MessageRole::User,
            user_message,
            None,
        )?;

        // 2. Save tool calls
        for (id, name, arguments) in tool_calls {
            let metadata = ToolUseMetadata {
                tool_id: id.clone(),
                name: name.clone(),
                arguments: arguments.clone(),
            };
            let metadata_str = serde_json::to_string(&metadata)?;

            self.message_store.append(
                workstream_id,
                Some(session_id),
                MessageRole::ToolUse,
                "", // Tool use content is in metadata
                Some(&metadata_str),
            )?;
        }

        // 3. Save tool results
        for (tool_call_id, success, content) in tool_results {
            let metadata = ToolResultMetadata {
                tool_call_id: tool_call_id.clone(),
                success: *success,
            };
            let metadata_str = serde_json::to_string(&metadata)?;

            self.message_store.append(
                workstream_id,
                Some(session_id),
                MessageRole::ToolResult,
                content,
                Some(&metadata_str),
            )?;
        }

        // 4. Save assistant response
        if let Some(response) = assistant_response {
            self.message_store.append(
                workstream_id,
                Some(session_id),
                MessageRole::Assistant,
                response,
                None,
            )?;
        }

        Ok(())
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
    fn test_load_empty_session() {
        let (_dir, store) = temp_store();
        let loader = SessionLoader::new(&store);

        let result = loader.load_session("ws-1", "session-1").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_load_simple_session() {
        let (_dir, store) = temp_store();

        // Create a simple conversation
        store
            .append("ws-1", Some("session-1"), MessageRole::User, "Hello", None)
            .unwrap();
        store
            .append(
                "ws-1",
                Some("session-1"),
                MessageRole::Assistant,
                "Hi there!",
                None,
            )
            .unwrap();
        store
            .append(
                "ws-1",
                Some("session-1"),
                MessageRole::User,
                "How are you?",
                None,
            )
            .unwrap();
        store
            .append(
                "ws-1",
                Some("session-1"),
                MessageRole::Assistant,
                "I'm great!",
                None,
            )
            .unwrap();

        let loader = SessionLoader::new(&store);
        let session = loader.load_session("ws-1", "session-1").unwrap().unwrap();

        assert_eq!(session.session_id, "session-1");
        assert_eq!(session.workstream_id, "ws-1");
        assert_eq!(session.turns.len(), 2);

        assert_eq!(session.turns[0].user_message, "Hello");
        assert_eq!(
            session.turns[0].assistant_response,
            Some("Hi there!".to_string())
        );

        assert_eq!(session.turns[1].user_message, "How are you?");
        assert_eq!(
            session.turns[1].assistant_response,
            Some("I'm great!".to_string())
        );
    }

    #[test]
    fn test_load_session_with_tool_calls() {
        let (_dir, store) = temp_store();

        // User message
        store
            .append(
                "ws-1",
                Some("session-1"),
                MessageRole::User,
                "Read the file",
                None,
            )
            .unwrap();

        // Tool use
        let tool_meta = ToolUseMetadata {
            tool_id: "call_1".to_string(),
            name: "read_file".to_string(),
            arguments: serde_json::json!({"path": "/test.txt"}),
        };
        store
            .append(
                "ws-1",
                Some("session-1"),
                MessageRole::ToolUse,
                "",
                Some(&serde_json::to_string(&tool_meta).unwrap()),
            )
            .unwrap();

        // Tool result
        let result_meta = ToolResultMetadata {
            tool_call_id: "call_1".to_string(),
            success: true,
        };
        store
            .append(
                "ws-1",
                Some("session-1"),
                MessageRole::ToolResult,
                "file contents here",
                Some(&serde_json::to_string(&result_meta).unwrap()),
            )
            .unwrap();

        // Assistant response
        store
            .append(
                "ws-1",
                Some("session-1"),
                MessageRole::Assistant,
                "The file contains...",
                None,
            )
            .unwrap();

        let loader = SessionLoader::new(&store);
        let session = loader.load_session("ws-1", "session-1").unwrap().unwrap();

        assert_eq!(session.turns.len(), 1);
        let turn = &session.turns[0];

        assert_eq!(turn.user_message, "Read the file");
        assert_eq!(turn.tool_calls.len(), 1);
        assert_eq!(turn.tool_calls[0].id, "call_1");
        assert_eq!(turn.tool_calls[0].name, "read_file");

        assert_eq!(turn.tool_results.len(), 1);
        assert_eq!(turn.tool_results[0].tool_call_id, "call_1");
        assert!(turn.tool_results[0].success);
        assert_eq!(turn.tool_results[0].content, "file contents here");

        assert_eq!(
            turn.assistant_response,
            Some("The file contains...".to_string())
        );
    }

    #[test]
    fn test_save_turn() {
        let (_dir, store) = temp_store();
        let loader = SessionLoader::new(&store);

        let tool_calls = vec![(
            "call_1".to_string(),
            "search".to_string(),
            serde_json::json!({"query": "test"}),
        )];
        let tool_results = vec![("call_1".to_string(), true, "results...".to_string())];

        loader
            .save_turn(
                "ws-1",
                "session-1",
                "Search for test",
                &tool_calls,
                &tool_results,
                Some("Found results"),
            )
            .unwrap();

        // Verify by loading
        let session = loader.load_session("ws-1", "session-1").unwrap().unwrap();
        assert_eq!(session.turns.len(), 1);

        let turn = &session.turns[0];
        assert_eq!(turn.user_message, "Search for test");
        assert_eq!(turn.tool_calls.len(), 1);
        assert_eq!(turn.tool_results.len(), 1);
        assert_eq!(turn.assistant_response, Some("Found results".to_string()));
    }

    #[test]
    fn test_incomplete_turn() {
        let (_dir, store) = temp_store();

        // User message without assistant response
        store
            .append("ws-1", Some("session-1"), MessageRole::User, "Hello", None)
            .unwrap();

        let loader = SessionLoader::new(&store);
        let session = loader.load_session("ws-1", "session-1").unwrap().unwrap();

        assert_eq!(session.turns.len(), 1);
        assert_eq!(session.turns[0].user_message, "Hello");
        assert!(session.turns[0].assistant_response.is_none());
        assert!(session.turns[0].completed_at.is_none());
    }
}
