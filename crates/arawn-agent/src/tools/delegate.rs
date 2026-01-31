//! Delegate tool for subagent invocation.
//!
//! Allows the main agent to delegate tasks to specialized subagents
//! with constrained tool sets and custom system prompts.

use async_trait::async_trait;
use serde_json::{Value, json};

use arawn_types::{DelegationOutcome, SharedSubagentSpawner, SubagentInfo};

use crate::error::Result;
use crate::tool::{Tool, ToolContext, ToolResult};

// ─────────────────────────────────────────────────────────────────────────────
// Delegate Tool
// ─────────────────────────────────────────────────────────────────────────────

/// Tool for delegating tasks to subagents.
///
/// Subagents are specialized agents with constrained tool sets, defined
/// in plugin configurations. The delegate tool allows the main agent to
/// invoke these subagents for specialized tasks.
///
/// # Example Usage
///
/// ```json
/// {
///   "agent": "researcher",
///   "task": "Find recent papers on RAG architectures",
///   "context": "The user is building a knowledge retrieval system",
///   "background": false
/// }
/// ```
pub struct DelegateTool {
    spawner: SharedSubagentSpawner,
}

impl std::fmt::Debug for DelegateTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DelegateTool")
            .field("spawner", &"<SubagentSpawner>")
            .finish()
    }
}

impl DelegateTool {
    /// Create a new delegate tool with the given subagent spawner.
    pub fn new(spawner: SharedSubagentSpawner) -> Self {
        Self { spawner }
    }

    /// List available subagents.
    pub async fn available_agents(&self) -> Vec<SubagentInfo> {
        self.spawner.list_agents().await
    }

    /// Format a list of available agent names for error messages.
    fn format_available_agents(agents: &[String]) -> String {
        if agents.is_empty() {
            "none configured".to_string()
        } else {
            agents.join(", ")
        }
    }
}

#[async_trait]
impl Tool for DelegateTool {
    fn name(&self) -> &str {
        "delegate"
    }

    fn description(&self) -> &str {
        "Delegate a task to a specialized subagent. Subagents have constrained tool sets \
         and custom prompts optimized for specific tasks. Use this to invoke a subagent \
         by name with a task description. Set background=true to run asynchronously \
         and receive a notification when complete."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "agent": {
                    "type": "string",
                    "description": "Name of the subagent to delegate to (e.g., 'researcher', 'reviewer')"
                },
                "task": {
                    "type": "string",
                    "description": "Task description for the subagent to execute"
                },
                "context": {
                    "type": "string",
                    "description": "Additional context from the current conversation to pass to the subagent"
                },
                "background": {
                    "type": "boolean",
                    "default": false,
                    "description": "If true, run in background and return immediately. You'll be notified when complete."
                },
                "max_turns": {
                    "type": "integer",
                    "description": "Override maximum conversation turns for this delegation"
                }
            },
            "required": ["agent", "task"]
        })
    }

    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        if ctx.is_cancelled() {
            return Ok(ToolResult::error("Operation cancelled"));
        }

        // Parse required parameters
        let agent_name = params
            .get("agent")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::AgentError::Tool("Missing required 'agent' parameter".to_string())
            })?;

        let task = params.get("task").and_then(|v| v.as_str()).ok_or_else(|| {
            crate::error::AgentError::Tool("Missing required 'task' parameter".to_string())
        })?;

        // Parse optional parameters
        let context = params.get("context").and_then(|v| v.as_str());
        let background = params
            .get("background")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let max_turns = params
            .get("max_turns")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize);

        if background {
            // Background execution - spawn and return immediately
            let parent_session_id = ctx.session_id.to_string();

            match self
                .spawner
                .delegate_background(agent_name, task, context, &parent_session_id)
                .await
            {
                Ok(()) => Ok(ToolResult::text(format!(
                    "Delegated to '{}' in background. You'll be notified when complete.",
                    agent_name
                ))),
                Err(e) => Ok(ToolResult::error(e)),
            }
        } else {
            // Blocking execution - wait for result
            match self
                .spawner
                .delegate(agent_name, task, context, max_turns)
                .await
            {
                DelegationOutcome::Success(result) => Ok(ToolResult::text(format!(
                    "## Result from '{}'\n\n{}",
                    agent_name, result.text
                ))),
                DelegationOutcome::Error { message } => Ok(ToolResult::error(format!(
                    "Subagent '{}' failed: {}",
                    agent_name, message
                ))),
                DelegationOutcome::UnknownAgent { name, available } => {
                    Ok(ToolResult::error(format!(
                        "Unknown agent '{}'. Available agents: {}",
                        name,
                        Self::format_available_agents(&available)
                    )))
                }
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    /// Mock spawner for testing.
    struct MockSpawner {
        agents: Vec<SubagentInfo>,
    }

    impl MockSpawner {
        fn new() -> Self {
            Self {
                agents: vec![
                    SubagentInfo {
                        name: "researcher".to_string(),
                        description: "Web research agent".to_string(),
                        tools: vec!["web_fetch".to_string(), "web_search".to_string()],
                        source: Some("test-plugin".to_string()),
                    },
                    SubagentInfo {
                        name: "reviewer".to_string(),
                        description: "Code review agent".to_string(),
                        tools: vec!["file_read".to_string(), "grep".to_string()],
                        source: None,
                    },
                ],
            }
        }
    }

    #[async_trait]
    impl arawn_types::SubagentSpawner for MockSpawner {
        async fn list_agents(&self) -> Vec<SubagentInfo> {
            self.agents.clone()
        }

        async fn delegate(
            &self,
            agent_name: &str,
            task: &str,
            _context: Option<&str>,
            _max_turns: Option<usize>,
        ) -> DelegationOutcome {
            if !self.agents.iter().any(|a| a.name == agent_name) {
                return DelegationOutcome::UnknownAgent {
                    name: agent_name.to_string(),
                    available: self.agents.iter().map(|a| a.name.clone()).collect(),
                };
            }

            DelegationOutcome::Success(arawn_types::SubagentResult {
                text: format!("Completed task: {}", task),
                success: true,
                turns: 3,
                duration_ms: 1500,
                truncated: false,
                compacted: false,
                original_len: None,
            })
        }

        async fn delegate_background(
            &self,
            agent_name: &str,
            _task: &str,
            _context: Option<&str>,
            _parent_session_id: &str,
        ) -> std::result::Result<(), String> {
            if !self.agents.iter().any(|a| a.name == agent_name) {
                return Err(format!("Unknown agent: {}", agent_name));
            }
            Ok(())
        }
    }

    #[test]
    fn test_delegate_tool_metadata() {
        let spawner: SharedSubagentSpawner = Arc::new(MockSpawner::new());
        let tool = DelegateTool::new(spawner);

        assert_eq!(tool.name(), "delegate");
        assert!(!tool.description().is_empty());
        assert!(tool.description().contains("subagent"));

        let params = tool.parameters();
        assert_eq!(params["type"], "object");
        assert!(params["properties"].get("agent").is_some());
        assert!(params["properties"].get("task").is_some());
        assert!(params["properties"].get("context").is_some());
        assert!(params["properties"].get("background").is_some());
        assert!(params["properties"].get("max_turns").is_some());
        assert_eq!(params["required"], json!(["agent", "task"]));
    }

    #[tokio::test]
    async fn test_delegate_blocking_success() {
        let spawner: SharedSubagentSpawner = Arc::new(MockSpawner::new());
        let tool = DelegateTool::new(spawner);
        let ctx = ToolContext::default();

        let result = tool
            .execute(
                json!({
                    "agent": "researcher",
                    "task": "Find papers on RAG"
                }),
                &ctx,
            )
            .await
            .unwrap();

        assert!(result.is_success());
        let content = result.to_llm_content();
        assert!(content.contains("Result from 'researcher'"));
        assert!(content.contains("Completed task: Find papers on RAG"));
    }

    #[tokio::test]
    async fn test_delegate_unknown_agent() {
        let spawner: SharedSubagentSpawner = Arc::new(MockSpawner::new());
        let tool = DelegateTool::new(spawner);
        let ctx = ToolContext::default();

        let result = tool
            .execute(
                json!({
                    "agent": "nonexistent",
                    "task": "Do something"
                }),
                &ctx,
            )
            .await
            .unwrap();

        // Tool returns error as ToolResult::Error, which is still Ok() from execute()
        assert!(result.is_error());
        let content = result.to_llm_content();
        assert!(content.contains("Unknown agent"));
        assert!(content.contains("researcher"));
        assert!(content.contains("reviewer"));
    }

    #[tokio::test]
    async fn test_delegate_background() {
        let spawner: SharedSubagentSpawner = Arc::new(MockSpawner::new());
        let tool = DelegateTool::new(spawner);
        let ctx = ToolContext::default();

        let result = tool
            .execute(
                json!({
                    "agent": "researcher",
                    "task": "Long running research",
                    "background": true
                }),
                &ctx,
            )
            .await
            .unwrap();

        assert!(result.is_success());
        let content = result.to_llm_content();
        assert!(content.contains("background"));
        assert!(content.contains("notified"));
    }

    #[tokio::test]
    async fn test_delegate_missing_agent_param() {
        let spawner: SharedSubagentSpawner = Arc::new(MockSpawner::new());
        let tool = DelegateTool::new(spawner);
        let ctx = ToolContext::default();

        let result = tool
            .execute(
                json!({
                    "task": "Do something"
                }),
                &ctx,
            )
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delegate_missing_task_param() {
        let spawner: SharedSubagentSpawner = Arc::new(MockSpawner::new());
        let tool = DelegateTool::new(spawner);
        let ctx = ToolContext::default();

        let result = tool
            .execute(
                json!({
                    "agent": "researcher"
                }),
                &ctx,
            )
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delegate_with_context() {
        let spawner: SharedSubagentSpawner = Arc::new(MockSpawner::new());
        let tool = DelegateTool::new(spawner);
        let ctx = ToolContext::default();

        let result = tool
            .execute(
                json!({
                    "agent": "researcher",
                    "task": "Find papers",
                    "context": "User is interested in knowledge retrieval"
                }),
                &ctx,
            )
            .await
            .unwrap();

        assert!(result.is_success());
    }

    #[tokio::test]
    async fn test_list_available_agents() {
        let spawner: SharedSubagentSpawner = Arc::new(MockSpawner::new());
        let tool = DelegateTool::new(spawner);

        let agents = tool.available_agents().await;
        assert_eq!(agents.len(), 2);
        assert_eq!(agents[0].name, "researcher");
        assert_eq!(agents[1].name, "reviewer");
    }

    #[test]
    fn test_format_available_agents_empty() {
        let formatted = DelegateTool::format_available_agents(&[]);
        assert_eq!(formatted, "none configured");
    }

    #[test]
    fn test_format_available_agents() {
        let agents = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let formatted = DelegateTool::format_available_agents(&agents);
        assert_eq!(formatted, "a, b, c");
    }
}
