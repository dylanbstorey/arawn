//! Think tool for persisting internal reasoning.
//!
//! Allows the agent to record thoughts as memories that are available
//! for recall in subsequent turns but not shown to the user.

use async_trait::async_trait;
use serde_json::{Value, json};
use std::sync::Arc;

use arawn_memory::store::MemoryStore;
use arawn_memory::types::{ContentType, Memory, Metadata};

use crate::error::Result;
use crate::tool::{ThinkParams, Tool, ToolContext, ToolResult};

// ─────────────────────────────────────────────────────────────────────────────
// Think Tool
// ─────────────────────────────────────────────────────────────────────────────

/// Tool for persisting internal reasoning as Thought memories.
///
/// Thoughts are stored in the memory system and available for recall
/// in subsequent turns, but are not displayed to the user.
#[derive(Debug, Clone)]
pub struct ThinkTool {
    store: Arc<MemoryStore>,
}

impl ThinkTool {
    /// Create a new think tool backed by the given memory store.
    pub fn new(store: Arc<MemoryStore>) -> Self {
        Self { store }
    }
}

#[async_trait]
impl Tool for ThinkTool {
    fn name(&self) -> &str {
        "think"
    }

    fn description(&self) -> &str {
        "Record your internal reasoning as a persistent thought. Thoughts are available for recall in future turns but are not shown to the user. Use this to work through complex problems, plan multi-step actions, or note important observations."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "thought": {
                    "type": "string",
                    "description": "Your internal reasoning or observation to record"
                }
            },
            "required": ["thought"]
        })
    }

    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        if ctx.is_cancelled() {
            return Ok(ToolResult::error("Operation cancelled"));
        }

        // Parse and validate parameters using typed struct
        let think_params = match ThinkParams::try_from(params) {
            Ok(p) => p,
            Err(e) => return Ok(ToolResult::error(e.to_string())),
        };

        let mut memory = Memory::new(ContentType::Thought, &think_params.thought);
        memory.metadata = Metadata {
            session_id: Some(ctx.session_id.to_string()),
            ..Default::default()
        };

        self.store.insert_memory(&memory).map_err(|e| {
            crate::error::AgentError::Tool(format!("Failed to store thought: {}", e))
        })?;

        Ok(ToolResult::text("Thought recorded."))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_tool() -> ThinkTool {
        let store = MemoryStore::open_in_memory().unwrap();
        ThinkTool::new(Arc::new(store))
    }

    #[test]
    fn test_think_tool_metadata() {
        let tool = create_test_tool();
        assert_eq!(tool.name(), "think");
        assert!(!tool.description().is_empty());

        let params = tool.parameters();
        assert!(params["properties"].get("thought").is_some());
        assert_eq!(params["required"][0], "thought");
    }

    #[tokio::test]
    async fn test_think_stores_thought() {
        let tool = create_test_tool();
        let ctx = ToolContext::default();

        let result = tool
            .execute(
                json!({"thought": "The user prefers Rust over Python"}),
                &ctx,
            )
            .await
            .unwrap();

        assert!(result.is_success());
        assert!(result.to_llm_content().contains("Thought recorded"));

        // Verify it was stored
        let stats = tool.store.stats().unwrap();
        assert_eq!(stats.memory_count, 1);

        let memories = tool.store.list_memories(None, 10, 0).unwrap();
        assert_eq!(memories[0].content, "The user prefers Rust over Python");
        assert_eq!(memories[0].content_type, ContentType::Thought);
        assert_eq!(
            memories[0].metadata.session_id,
            Some(ctx.session_id.to_string())
        );
    }

    #[tokio::test]
    async fn test_think_missing_param() {
        let tool = create_test_tool();
        let ctx = ToolContext::default();

        let result = tool.execute(json!({}), &ctx).await.unwrap();
        // Parameter validation returns a ToolResult error, not Err
        assert!(result.is_error());
        assert!(
            result
                .to_llm_content()
                .contains("missing required parameter")
        );
    }

    #[tokio::test]
    async fn test_think_empty_thought() {
        let tool = create_test_tool();
        let ctx = ToolContext::default();

        let result = tool.execute(json!({"thought": "  "}), &ctx).await.unwrap();
        assert!(result.is_error());
        assert!(result.to_llm_content().contains("cannot be empty"));
    }
}
