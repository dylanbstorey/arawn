//! Subagent delegation types and traits.
//!
//! This module defines the interface for delegating tasks to subagents.
//! The actual implementation lives in `arawn-plugin`, but the trait is
//! defined here to avoid cyclic dependencies.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Information about an available subagent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubagentInfo {
    /// The name of the subagent.
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// List of tools this subagent has access to.
    pub tools: Vec<String>,
    /// Source plugin name (if from a plugin).
    pub source: Option<String>,
}

/// Result of a subagent execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubagentResult {
    /// The subagent's final response text (may be truncated or compacted).
    pub text: String,
    /// Whether execution was successful.
    pub success: bool,
    /// Number of turns the subagent took.
    pub turns: usize,
    /// Execution duration in milliseconds.
    pub duration_ms: u64,
    /// Whether the result text was truncated due to length.
    #[serde(default)]
    pub truncated: bool,
    /// Whether the result was compacted via LLM summarization.
    #[serde(default)]
    pub compacted: bool,
    /// Original text length before truncation/compaction (if modified).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_len: Option<usize>,
}

/// Outcome of a subagent delegation attempt.
#[derive(Debug, Clone)]
pub enum DelegationOutcome {
    /// Subagent executed successfully.
    Success(SubagentResult),
    /// Subagent execution failed.
    Error { message: String },
    /// Unknown subagent name.
    UnknownAgent {
        name: String,
        available: Vec<String>,
    },
}

/// Trait for spawning and executing subagents.
///
/// This trait abstracts over the actual subagent implementation,
/// allowing `DelegateTool` in `arawn-agent` to delegate without
/// depending on `arawn-plugin`.
#[async_trait]
pub trait SubagentSpawner: Send + Sync {
    /// List all available subagents.
    async fn list_agents(&self) -> Vec<SubagentInfo>;

    /// Execute a task with a named subagent (blocking).
    ///
    /// # Arguments
    /// * `agent_name` - Name of the subagent to use
    /// * `task` - The task description to execute
    /// * `context` - Optional context from the parent session
    /// * `max_turns` - Optional override for maximum turns
    ///
    /// # Returns
    /// The outcome of the delegation attempt.
    async fn delegate(
        &self,
        agent_name: &str,
        task: &str,
        context: Option<&str>,
        max_turns: Option<usize>,
    ) -> DelegationOutcome;

    /// Execute a task with a named subagent in the background.
    ///
    /// Returns immediately after spawning. The caller should use
    /// hook events (SubagentStarted/SubagentCompleted) to track progress.
    ///
    /// # Arguments
    /// * `agent_name` - Name of the subagent to use
    /// * `task` - The task description to execute
    /// * `context` - Optional context from the parent session
    /// * `parent_session_id` - Session ID for event correlation
    ///
    /// # Returns
    /// Ok if the subagent was spawned, Err if the agent doesn't exist.
    async fn delegate_background(
        &self,
        agent_name: &str,
        task: &str,
        context: Option<&str>,
        parent_session_id: &str,
    ) -> Result<(), String>;

    /// Check if a subagent with the given name exists.
    async fn has_agent(&self, name: &str) -> bool {
        self.list_agents().await.iter().any(|a| a.name == name)
    }
}

/// Shared subagent spawner type for use across crates.
pub type SharedSubagentSpawner = Arc<dyn SubagentSpawner>;
