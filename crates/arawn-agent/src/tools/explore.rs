//! Explore tool for triggering RLM exploration.
//!
//! Wraps [`RlmSpawner`] as a standard [`Tool`] so the main agent can
//! delegate research tasks to an isolated exploration sub-agent.

use async_trait::async_trait;
use serde_json::{Value, json};
use std::sync::Arc;

use crate::error::Result;
use crate::rlm::RlmSpawner;
use crate::tool::{Tool, ToolContext, ToolResult};

// ─────────────────────────────────────────────────────────────────────────────
// Explore Tool
// ─────────────────────────────────────────────────────────────────────────────

/// Tool that spawns an RLM exploration agent to research a query.
///
/// The exploration agent has read-only access to tools (file_read, grep, glob,
/// web_search, etc.) and uses iterative compaction to work beyond context
/// limits. Results are returned as a summary with metadata.
pub struct ExploreTool {
    spawner: Arc<RlmSpawner>,
}

impl ExploreTool {
    /// Create a new explore tool backed by the given spawner.
    pub fn new(spawner: Arc<RlmSpawner>) -> Self {
        Self { spawner }
    }
}

#[async_trait]
impl Tool for ExploreTool {
    fn name(&self) -> &str {
        "explore"
    }

    fn description(&self) -> &str {
        "Explore and research to answer a question. Spawns an isolated \
         sub-agent with read-only tools that investigates the query and \
         returns compressed findings. Use this for complex research tasks \
         that require searching files, reading code, or gathering information \
         from multiple sources."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The research question or topic to explore"
                }
            },
            "required": ["query"]
        })
    }

    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        if ctx.is_cancelled() {
            return Ok(ToolResult::error("Operation cancelled"));
        }

        // Extract query parameter
        let query = match params.get("query").and_then(|v| v.as_str()) {
            Some(q) if !q.trim().is_empty() => q,
            Some(_) => return Ok(ToolResult::error("query cannot be empty")),
            None => return Ok(ToolResult::error("missing required parameter: query")),
        };

        // Run exploration
        match self.spawner.explore(query).await {
            Ok(result) => {
                let mut output = result.summary.clone();

                // Append metadata footer
                output.push_str("\n\n---\n");
                output.push_str(&format!(
                    "Exploration: {} iterations, {} tokens ({}in/{}out)",
                    result.metadata.iterations_used,
                    result.metadata.total_tokens(),
                    result.metadata.input_tokens,
                    result.metadata.output_tokens,
                ));
                if result.metadata.compactions_performed > 0 {
                    output.push_str(&format!(
                        ", {} compaction{}",
                        result.metadata.compactions_performed,
                        if result.metadata.compactions_performed == 1 {
                            ""
                        } else {
                            "s"
                        }
                    ));
                }
                if result.truncated {
                    output.push_str(" [truncated]");
                }

                Ok(ToolResult::text(output))
            }
            Err(e) => Ok(ToolResult::error(format!("Exploration failed: {}", e))),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool::{MockTool, ToolRegistry};
    use arawn_llm::{CompletionResponse, ContentBlock, MockBackend, StopReason, Usage};

    fn mock_text_response(text: &str) -> CompletionResponse {
        CompletionResponse::new(
            "msg_1",
            "test-model",
            vec![ContentBlock::Text {
                text: text.to_string(),
                cache_control: None,
            }],
            StopReason::EndTurn,
            Usage::new(100, 50),
        )
    }

    fn make_spawner(backend: MockBackend) -> Arc<RlmSpawner> {
        let mut tools = ToolRegistry::new();
        tools.register(MockTool::new("file_read"));
        tools.register(MockTool::new("grep"));
        Arc::new(RlmSpawner::new(Arc::new(backend), tools))
    }

    #[test]
    fn test_tool_definition() {
        let spawner = make_spawner(MockBackend::new(vec![]));
        let tool = ExploreTool::new(spawner);

        assert_eq!(tool.name(), "explore");
        assert!(!tool.description().is_empty());
        assert!(tool.description().contains("research"));

        let params = tool.parameters();
        assert!(params["properties"].get("query").is_some());
        assert_eq!(params["required"][0], "query");
    }

    #[tokio::test]
    async fn test_explore_returns_summary() {
        let backend = MockBackend::new(vec![mock_text_response(
            "Auth uses JWT tokens with 24h expiry.",
        )]);
        let spawner = make_spawner(backend);
        let tool = ExploreTool::new(spawner);
        let ctx = ToolContext::default();

        let result = tool
            .execute(json!({"query": "How does auth work?"}), &ctx)
            .await
            .unwrap();

        assert!(result.is_success());
        let content = result.to_llm_content();
        assert!(content.contains("Auth uses JWT tokens with 24h expiry."));
        // Metadata footer
        assert!(content.contains("Exploration:"));
        assert!(content.contains("iterations"));
        assert!(content.contains("tokens"));
    }

    #[tokio::test]
    async fn test_explore_missing_query() {
        let spawner = make_spawner(MockBackend::new(vec![]));
        let tool = ExploreTool::new(spawner);
        let ctx = ToolContext::default();

        let result = tool.execute(json!({}), &ctx).await.unwrap();
        assert!(result.is_error());
        assert!(
            result
                .to_llm_content()
                .contains("missing required parameter")
        );
    }

    #[tokio::test]
    async fn test_explore_empty_query() {
        let spawner = make_spawner(MockBackend::new(vec![]));
        let tool = ExploreTool::new(spawner);
        let ctx = ToolContext::default();

        let result = tool.execute(json!({"query": "  "}), &ctx).await.unwrap();
        assert!(result.is_error());
        assert!(result.to_llm_content().contains("cannot be empty"));
    }

    #[tokio::test]
    async fn test_explore_registerable() {
        let spawner = make_spawner(MockBackend::new(vec![]));
        let tool = ExploreTool::new(spawner);

        let mut registry = ToolRegistry::new();
        registry.register(tool);

        assert!(registry.names().contains(&&"explore"));
    }
}
