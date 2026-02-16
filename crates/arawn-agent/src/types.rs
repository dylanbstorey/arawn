//! Core types for the agent crate.
//!
//! This module defines the fundamental types used throughout the agent:
//! - [`Session`]: Conversation state container
//! - [`Turn`]: Single exchange (user message + response)
//! - [`AgentConfig`]: Runtime configuration
//! - [`AgentResponse`]: Agent output from a turn

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use uuid::Uuid;

// ─────────────────────────────────────────────────────────────────────────────
// ID Types
// ─────────────────────────────────────────────────────────────────────────────

/// Unique identifier for a session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(Uuid);

impl SessionId {
    /// Create a new random session ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create from an existing UUID.
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Get the underlying UUID.
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for a turn within a session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TurnId(Uuid);

impl TurnId {
    /// Create a new random turn ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create from an existing UUID.
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Get the underlying UUID.
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for TurnId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for TurnId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tool Call/Result Types (for Turn history)
// ─────────────────────────────────────────────────────────────────────────────

/// A tool call made by the agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Unique ID for this tool call.
    pub id: String,
    /// Name of the tool being called.
    pub name: String,
    /// Arguments passed to the tool (JSON).
    pub arguments: serde_json::Value,
}

/// Result of a tool execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResultRecord {
    /// ID of the tool call this is a result for.
    pub tool_call_id: String,
    /// Whether the tool execution succeeded.
    pub success: bool,
    /// Output content from the tool.
    pub content: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// Turn
// ─────────────────────────────────────────────────────────────────────────────

/// A single conversation turn (user message + agent response).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Turn {
    /// Unique identifier for this turn.
    pub id: TurnId,
    /// The user's input message.
    pub user_message: String,
    /// The agent's response (None if turn is in progress).
    pub assistant_response: Option<String>,
    /// Tool calls made during this turn.
    pub tool_calls: Vec<ToolCall>,
    /// Results from tool executions.
    pub tool_results: Vec<ToolResultRecord>,
    /// When this turn started.
    pub started_at: DateTime<Utc>,
    /// When this turn completed (None if in progress).
    pub completed_at: Option<DateTime<Utc>>,
}

impl Turn {
    /// Create a new turn with the given user message.
    pub fn new(user_message: impl Into<String>) -> Self {
        Self {
            id: TurnId::new(),
            user_message: user_message.into(),
            assistant_response: None,
            tool_calls: Vec::new(),
            tool_results: Vec::new(),
            started_at: Utc::now(),
            completed_at: None,
        }
    }

    /// Set the assistant response and mark as completed.
    pub fn complete(&mut self, response: impl Into<String>) {
        self.assistant_response = Some(response.into());
        self.completed_at = Some(Utc::now());
    }

    /// Add a tool call to this turn.
    pub fn add_tool_call(&mut self, call: ToolCall) {
        self.tool_calls.push(call);
    }

    /// Add a tool result to this turn.
    pub fn add_tool_result(&mut self, result: ToolResultRecord) {
        self.tool_results.push(result);
    }

    /// Check if this turn is complete.
    pub fn is_complete(&self) -> bool {
        self.completed_at.is_some()
    }

    /// Check if this turn has any tool calls.
    pub fn has_tool_calls(&self) -> bool {
        !self.tool_calls.is_empty()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Session
// ─────────────────────────────────────────────────────────────────────────────

/// A conversation session containing multiple turns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Unique identifier for this session.
    pub id: SessionId,
    /// All turns in this session.
    pub turns: Vec<Turn>,
    /// When this session was created.
    pub created_at: DateTime<Utc>,
    /// When this session was last updated.
    pub updated_at: DateTime<Utc>,
    /// Arbitrary metadata for the session.
    pub metadata: HashMap<String, serde_json::Value>,
    /// Optional context preamble included in system prompts but not in turn history.
    ///
    /// Use this to inject dynamic context (e.g., from a parent agent) that the LLM
    /// should see but that shouldn't pollute the conversation history.
    pub context_preamble: Option<String>,
}

impl Session {
    /// Create a new empty session.
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            id: SessionId::new(),
            turns: Vec::new(),
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
            context_preamble: None,
        }
    }

    /// Create a session with a specific ID.
    pub fn with_id(id: SessionId) -> Self {
        let now = Utc::now();
        Self {
            id,
            turns: Vec::new(),
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
            context_preamble: None,
        }
    }

    /// Set a context preamble that's included in system prompts but not in turn history.
    ///
    /// Use this to inject dynamic context (e.g., from a parent agent) that the LLM
    /// should see but that shouldn't become part of the conversation record.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mut session = Session::new();
    /// session.set_context_preamble("[Context from parent]\nUser is researching RAG systems.");
    /// ```
    pub fn set_context_preamble(&mut self, preamble: impl Into<String>) {
        self.context_preamble = Some(preamble.into());
    }

    /// Clear the context preamble.
    pub fn clear_context_preamble(&mut self) {
        self.context_preamble = None;
    }

    /// Get the context preamble, if set.
    pub fn context_preamble(&self) -> Option<&str> {
        self.context_preamble.as_deref()
    }

    /// Start a new turn with the given user message.
    pub fn start_turn(&mut self, user_message: impl Into<String>) -> &mut Turn {
        let turn = Turn::new(user_message);
        self.turns.push(turn);
        self.updated_at = Utc::now();
        self.turns.last_mut().unwrap()
    }

    /// Get the current (most recent) turn, if any.
    pub fn current_turn(&self) -> Option<&Turn> {
        self.turns.last()
    }

    /// Get the current turn mutably.
    pub fn current_turn_mut(&mut self) -> Option<&mut Turn> {
        self.turns.last_mut()
    }

    /// Get the N most recent turns.
    pub fn recent_turns(&self, n: usize) -> &[Turn] {
        let start = self.turns.len().saturating_sub(n);
        &self.turns[start..]
    }

    /// Get all turns.
    pub fn all_turns(&self) -> &[Turn] {
        &self.turns
    }

    /// Get the number of turns.
    pub fn turn_count(&self) -> usize {
        self.turns.len()
    }

    /// Check if the session is empty (no turns).
    pub fn is_empty(&self) -> bool {
        self.turns.is_empty()
    }

    /// Set a metadata value.
    pub fn set_metadata(&mut self, key: impl Into<String>, value: serde_json::Value) {
        self.metadata.insert(key.into(), value);
        self.updated_at = Utc::now();
    }

    /// Get a metadata value.
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }

    /// Remove a metadata value.
    pub fn remove_metadata(&mut self, key: &str) -> Option<serde_json::Value> {
        let value = self.metadata.remove(key);
        if value.is_some() {
            self.updated_at = Utc::now();
        }
        value
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Agent Configuration
// ─────────────────────────────────────────────────────────────────────────────

/// Configuration for the agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Model identifier to use.
    pub model: String,
    /// Maximum tokens for LLM responses.
    pub max_tokens: u32,
    /// Temperature for sampling (0.0 - 1.0).
    pub temperature: Option<f32>,
    /// Maximum tool execution iterations per turn.
    pub max_iterations: u32,
    /// Timeout for the entire turn.
    #[serde(with = "humantime_serde")]
    pub timeout: Duration,
    /// System prompt to use.
    pub system_prompt: Option<String>,
    /// Workspace path for file operations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace_path: Option<PathBuf>,
}

impl AgentConfig {
    /// Create a new config with the specified model.
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            max_tokens: 4096,
            temperature: None,
            max_iterations: 25,
            timeout: Duration::from_secs(300),
            system_prompt: None,
            workspace_path: None,
        }
    }

    /// Set max tokens.
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    /// Set temperature.
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Set max iterations.
    pub fn with_max_iterations(mut self, max_iterations: u32) -> Self {
        self.max_iterations = max_iterations;
        self
    }

    /// Set timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set system prompt.
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    /// Set the workspace path.
    pub fn with_workspace(mut self, path: impl Into<PathBuf>) -> Self {
        self.workspace_path = Some(path.into());
        self
    }
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self::new("claude-sonnet-4-20250514")
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Agent Response
// ─────────────────────────────────────────────────────────────────────────────

/// Response from an agent turn.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    /// The text response from the agent.
    pub text: String,
    /// Tool calls that were made during this turn.
    pub tool_calls: Vec<ToolCall>,
    /// Tool results from executions.
    pub tool_results: Vec<ToolResultRecord>,
    /// Number of LLM iterations used.
    pub iterations: u32,
    /// Token usage for this turn.
    pub usage: ResponseUsage,
    /// Whether the response was truncated (max iterations hit).
    pub truncated: bool,
}

impl AgentResponse {
    /// Create a simple text response.
    pub fn text(content: impl Into<String>) -> Self {
        Self {
            text: content.into(),
            tool_calls: Vec::new(),
            tool_results: Vec::new(),
            iterations: 1,
            usage: ResponseUsage::default(),
            truncated: false,
        }
    }
}

/// Token usage statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResponseUsage {
    /// Input tokens used.
    pub input_tokens: u32,
    /// Output tokens generated.
    pub output_tokens: u32,
}

impl ResponseUsage {
    /// Create new usage stats.
    pub fn new(input_tokens: u32, output_tokens: u32) -> Self {
        Self {
            input_tokens,
            output_tokens,
        }
    }

    /// Total tokens used.
    pub fn total(&self) -> u32 {
        self.input_tokens + self.output_tokens
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_id() {
        let id1 = SessionId::new();
        let id2 = SessionId::new();
        assert_ne!(id1, id2);

        let uuid = Uuid::new_v4();
        let id3 = SessionId::from_uuid(uuid);
        assert_eq!(id3.as_uuid(), &uuid);
    }

    #[test]
    fn test_turn_id() {
        let id1 = TurnId::new();
        let id2 = TurnId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_turn_creation() {
        let turn = Turn::new("Hello");
        assert_eq!(turn.user_message, "Hello");
        assert!(turn.assistant_response.is_none());
        assert!(!turn.is_complete());
        assert!(!turn.has_tool_calls());
    }

    #[test]
    fn test_turn_completion() {
        let mut turn = Turn::new("Hello");
        turn.complete("Hi there!");

        assert!(turn.is_complete());
        assert_eq!(turn.assistant_response, Some("Hi there!".to_string()));
        assert!(turn.completed_at.is_some());
    }

    #[test]
    fn test_turn_tool_calls() {
        let mut turn = Turn::new("Read file");

        turn.add_tool_call(ToolCall {
            id: "call_1".to_string(),
            name: "read_file".to_string(),
            arguments: serde_json::json!({"path": "/test.txt"}),
        });

        assert!(turn.has_tool_calls());
        assert_eq!(turn.tool_calls.len(), 1);

        turn.add_tool_result(ToolResultRecord {
            tool_call_id: "call_1".to_string(),
            success: true,
            content: "file contents".to_string(),
        });

        assert_eq!(turn.tool_results.len(), 1);
    }

    #[test]
    fn test_session_creation() {
        let session = Session::new();
        assert!(session.is_empty());
        assert_eq!(session.turn_count(), 0);
    }

    #[test]
    fn test_session_turns() {
        let mut session = Session::new();

        let turn = session.start_turn("First message");
        turn.complete("First response");

        session.start_turn("Second message");

        assert_eq!(session.turn_count(), 2);
        assert!(!session.is_empty());

        // Recent turns
        let recent = session.recent_turns(1);
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].user_message, "Second message");

        let recent = session.recent_turns(10);
        assert_eq!(recent.len(), 2);
    }

    #[test]
    fn test_session_metadata() {
        let mut session = Session::new();

        session.set_metadata("key1", serde_json::json!("value1"));
        session.set_metadata("key2", serde_json::json!(42));

        assert_eq!(
            session.get_metadata("key1"),
            Some(&serde_json::json!("value1"))
        );
        assert_eq!(session.get_metadata("key2"), Some(&serde_json::json!(42)));
        assert_eq!(session.get_metadata("missing"), None);

        let removed = session.remove_metadata("key1");
        assert!(removed.is_some());
        assert_eq!(session.get_metadata("key1"), None);
    }

    #[test]
    fn test_agent_config() {
        let config = AgentConfig::new("claude-sonnet-4-20250514")
            .with_max_tokens(8192)
            .with_temperature(0.7)
            .with_max_iterations(5)
            .with_timeout(Duration::from_secs(60))
            .with_system_prompt("You are a helpful assistant.");

        assert_eq!(config.model, "claude-sonnet-4-20250514");
        assert_eq!(config.max_tokens, 8192);
        assert_eq!(config.temperature, Some(0.7));
        assert_eq!(config.max_iterations, 5);
        assert_eq!(config.timeout, Duration::from_secs(60));
        assert_eq!(
            config.system_prompt,
            Some("You are a helpful assistant.".to_string())
        );
    }

    #[test]
    fn test_agent_config_default() {
        let config = AgentConfig::default();
        assert_eq!(config.max_tokens, 4096);
        assert_eq!(config.max_iterations, 25);
        assert!(config.temperature.is_none());
    }

    #[test]
    fn test_agent_response() {
        let response = AgentResponse::text("Hello!");
        assert_eq!(response.text, "Hello!");
        assert!(response.tool_calls.is_empty());
        assert!(!response.truncated);
    }

    #[test]
    fn test_response_usage() {
        let usage = ResponseUsage::new(100, 50);
        assert_eq!(usage.input_tokens, 100);
        assert_eq!(usage.output_tokens, 50);
        assert_eq!(usage.total(), 150);
    }

    #[test]
    fn test_session_serialization() {
        let mut session = Session::new();
        session.start_turn("Hello").complete("Hi!");

        let json = serde_json::to_string(&session).unwrap();
        let restored: Session = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.id, session.id);
        assert_eq!(restored.turn_count(), 1);
        assert_eq!(restored.turns[0].user_message, "Hello");
    }

    #[test]
    fn test_session_context_preamble() {
        let mut session = Session::new();

        // Initially no preamble
        assert!(session.context_preamble().is_none());

        // Set a preamble
        session.set_context_preamble("Context from parent agent");
        assert_eq!(
            session.context_preamble(),
            Some("Context from parent agent")
        );

        // Clear the preamble
        session.clear_context_preamble();
        assert!(session.context_preamble().is_none());
    }

    #[test]
    fn test_session_context_preamble_not_in_turns() {
        let mut session = Session::new();
        session.set_context_preamble("This is context");
        session.start_turn("Hello").complete("Hi!");

        // Preamble should not appear in turns
        assert_eq!(session.turn_count(), 1);
        assert_eq!(session.turns[0].user_message, "Hello");
        assert!(!session.turns[0].user_message.contains("context"));
    }
}

mod humantime_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_secs())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}
