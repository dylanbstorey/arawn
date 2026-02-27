//! Memory search tool.
//!
//! Provides a tool for searching the agent's memory/knowledge store.
// TODO(ARAWN-T-0233): Wire to real arawn-memory backend instead of returning stubs.

use async_trait::async_trait;
use serde_json::{Value, json};

use crate::error::Result;
use crate::tool::{Tool, ToolContext, ToolResult};

// ─────────────────────────────────────────────────────────────────────────────
// Memory Search Tool
// ─────────────────────────────────────────────────────────────────────────────

/// Tool for searching the agent's memory/knowledge store.
///
/// This tool allows the agent to query its persistent memory for:
/// - Past conversations and context
/// - Stored facts and knowledge
/// - User preferences and information
/// - Research findings and notes
///
/// Currently a stub - full implementation requires arawn-memory crate.
#[derive(Debug, Clone, Default)]
pub struct MemorySearchTool {
    /// Whether the memory backend is connected.
    connected: bool,
}

impl MemorySearchTool {
    /// Create a new memory search tool (disconnected by default).
    pub fn new() -> Self {
        Self { connected: false }
    }

    /// Create a memory search tool marked as connected.
    ///
    /// In the future, this will take a reference to the memory store.
    pub fn connected() -> Self {
        Self { connected: true }
    }

    // Future implementation will include:
    // pub fn with_memory_store(store: Arc<MemoryStore>) -> Self { ... }
}

#[async_trait]
impl Tool for MemorySearchTool {
    fn name(&self) -> &str {
        "memory_search"
    }

    fn description(&self) -> &str {
        "Search the agent's memory for relevant information. Query past conversations, stored facts, user preferences, and research findings."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The search query to find relevant memories"
                },
                "type": {
                    "type": "string",
                    "enum": ["all", "conversation", "fact", "preference", "research"],
                    "description": "Type of memory to search. Defaults to 'all'.",
                    "default": "all"
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of results to return. Defaults to 10.",
                    "default": 10
                },
                "time_range": {
                    "type": "string",
                    "enum": ["all", "today", "week", "month"],
                    "description": "Time range to search within. Defaults to 'all'.",
                    "default": "all"
                }
            },
            "required": ["query"]
        })
    }

    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        if ctx.is_cancelled() {
            return Ok(ToolResult::error("Operation cancelled"));
        }

        let query = params
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::AgentError::Tool("Missing 'query' parameter".to_string())
            })?;

        let memory_type = params.get("type").and_then(|v| v.as_str()).unwrap_or("all");

        let _limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;

        let time_range = params
            .get("time_range")
            .and_then(|v| v.as_str())
            .unwrap_or("all");

        // Check if memory backend is connected
        if !self.connected {
            return Ok(ToolResult::json(json!({
                "status": "disconnected",
                "message": "Memory store not connected. Memory search is not available.",
                "query": query,
                "results": []
            })));
        }

        // Stub implementation - return placeholder results
        // In the future, this will:
        // 1. Generate embedding for the query
        // 2. Search vector store for similar memories
        // 3. Optionally query knowledge graph for related entities
        // 4. Rank and return results

        Ok(ToolResult::json(json!({
            "status": "ok",
            "query": query,
            "type": memory_type,
            "time_range": time_range,
            "count": 0,
            "results": [],
            "note": "Memory search is a stub. Full implementation pending arawn-memory crate."
        })))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Memory Types (for future implementation)
// ─────────────────────────────────────────────────────────────────────────────

/// Types of memories that can be stored and searched.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryType {
    /// Conversation history and context.
    Conversation,
    /// Factual information.
    Fact,
    /// User preferences and settings.
    Preference,
    /// Research findings and notes.
    Research,
}

/// A memory search result.
#[derive(Debug, Clone)]
pub struct MemoryResult {
    /// Unique identifier.
    pub id: String,
    /// Type of memory.
    pub memory_type: MemoryType,
    /// The content of the memory.
    pub content: String,
    /// Relevance score (0.0 to 1.0).
    pub score: f32,
    /// When this memory was created.
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Associated metadata.
    pub metadata: serde_json::Value,
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_search_tool_metadata() {
        let tool = MemorySearchTool::new();
        assert_eq!(tool.name(), "memory_search");
        assert!(!tool.description().is_empty());

        let params = tool.parameters();
        assert!(params["properties"].get("query").is_some());
        assert!(params["properties"].get("type").is_some());
        assert!(params["properties"].get("limit").is_some());
    }

    #[tokio::test]
    async fn test_memory_search_disconnected() {
        let tool = MemorySearchTool::new();
        let ctx = ToolContext::default();

        let result = tool
            .execute(json!({"query": "test query"}), &ctx)
            .await
            .unwrap();

        assert!(result.is_success());
        let content = result.to_llm_content();
        assert!(content.contains("disconnected") || content.contains("not connected"));
    }

    #[tokio::test]
    async fn test_memory_search_connected() {
        let tool = MemorySearchTool::connected();
        let ctx = ToolContext::default();

        let result = tool
            .execute(
                json!({
                    "query": "test query",
                    "type": "fact",
                    "limit": 5
                }),
                &ctx,
            )
            .await
            .unwrap();

        assert!(result.is_success());
        let content = result.to_llm_content();
        assert!(content.contains("ok") || content.contains("stub"));
    }

    #[tokio::test]
    async fn test_memory_search_missing_query() {
        let tool = MemorySearchTool::new();
        let ctx = ToolContext::default();

        let result = tool.execute(json!({}), &ctx).await;

        // Still returns Err for now - can be updated when we add typed params
        assert!(result.is_err());
    }
}
