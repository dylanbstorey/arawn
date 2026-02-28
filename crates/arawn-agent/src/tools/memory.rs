//! Memory search tool.
//!
//! Provides a tool for searching the agent's memory/knowledge store.

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{Value, json};

use arawn_memory::{ContentType, MemoryStore, TimeRange};

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
#[derive(Debug, Clone)]
pub struct MemorySearchTool {
    /// The memory store backend (None = disconnected).
    store: Option<Arc<MemoryStore>>,
}

impl Default for MemorySearchTool {
    fn default() -> Self {
        Self::new()
    }
}

impl MemorySearchTool {
    /// Create a new memory search tool (disconnected).
    pub fn new() -> Self {
        Self { store: None }
    }

    /// Create a memory search tool backed by a real memory store.
    pub fn with_store(store: Arc<MemoryStore>) -> Self {
        Self { store: Some(store) }
    }
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
        let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
        let time_range_str = params
            .get("time_range")
            .and_then(|v| v.as_str())
            .unwrap_or("all");

        let Some(ref store) = self.store else {
            return Ok(ToolResult::json(json!({
                "status": "disconnected",
                "message": "Memory store not connected. Memory search is not available.",
                "query": query,
                "results": []
            })));
        };

        let time_range = parse_time_range(time_range_str);

        // Search memories with time range filter
        let memories = store
            .search_memories_in_range(query, time_range, limit * 2)
            .unwrap_or_default();

        // Filter by content type if requested
        let content_type_filter = parse_content_type_filter(memory_type);
        let mut results: Vec<Value> = Vec::new();

        for memory in &memories {
            if let Some(ref filter) = content_type_filter {
                if !filter.contains(&memory.content_type) {
                    continue;
                }
            }
            if results.len() >= limit {
                break;
            }

            results.push(json!({
                "id": memory.id.to_string(),
                "content_type": memory.content_type.as_str(),
                "content": memory.content,
                "score": memory.confidence.score,
                "created_at": memory.created_at.to_rfc3339(),
                "session_id": memory.session_id,
            }));
        }

        // Supplement with notes if we have room
        let remaining = limit.saturating_sub(results.len());
        if remaining > 0 {
            if let Ok(notes) = store.search_notes(query, remaining) {
                for note in &notes {
                    results.push(json!({
                        "id": note.id.to_string(),
                        "content_type": "note",
                        "content": note.content,
                        "score": 1.0,
                        "created_at": note.created_at.to_rfc3339(),
                        "title": note.title,
                    }));
                }
            }
        }

        results.truncate(limit);

        Ok(ToolResult::json(json!({
            "status": "ok",
            "query": query,
            "type": memory_type,
            "time_range": time_range_str,
            "count": results.len(),
            "results": results
        })))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

fn parse_time_range(s: &str) -> TimeRange {
    match s {
        "today" => TimeRange::Today,
        "week" => TimeRange::Week,
        "month" => TimeRange::Month,
        _ => TimeRange::All,
    }
}

fn parse_content_type_filter(memory_type: &str) -> Option<Vec<ContentType>> {
    match memory_type {
        "conversation" => Some(vec![
            ContentType::UserMessage,
            ContentType::AssistantMessage,
        ]),
        "fact" => Some(vec![ContentType::Fact]),
        "preference" => Some(vec![ContentType::Note]),
        "research" => Some(vec![ContentType::WebContent, ContentType::FileContent]),
        _ => None, // "all" — no filter
    }
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
        assert!(content.contains("disconnected"));
    }

    #[tokio::test]
    async fn test_memory_search_with_store() {
        let store = MemoryStore::open_in_memory().unwrap();

        // Insert a fact
        let memory = arawn_memory::Memory::new(ContentType::Fact, "Rust is a systems language");
        store.insert_memory(&memory).unwrap();

        let store = Arc::new(store);
        let tool = MemorySearchTool::with_store(store);
        let ctx = ToolContext::default();

        let result = tool
            .execute(
                json!({
                    "query": "Rust",
                    "type": "fact",
                    "limit": 5
                }),
                &ctx,
            )
            .await
            .unwrap();

        assert!(result.is_success());
        let content = result.to_llm_content();
        assert!(content.contains("ok"));
        assert!(content.contains("Rust is a systems language"));
    }

    #[tokio::test]
    async fn test_memory_search_with_time_range() {
        let store = MemoryStore::open_in_memory().unwrap();

        let memory = arawn_memory::Memory::new(ContentType::Fact, "Recent finding about X");
        store.insert_memory(&memory).unwrap();

        let store = Arc::new(store);
        let tool = MemorySearchTool::with_store(store);
        let ctx = ToolContext::default();

        let result = tool
            .execute(
                json!({
                    "query": "finding",
                    "time_range": "today"
                }),
                &ctx,
            )
            .await
            .unwrap();

        assert!(result.is_success());
        let content = result.to_llm_content();
        assert!(content.contains("Recent finding about X"));
    }

    #[tokio::test]
    async fn test_memory_search_empty_results() {
        let store = Arc::new(MemoryStore::open_in_memory().unwrap());
        let tool = MemorySearchTool::with_store(store);
        let ctx = ToolContext::default();

        let result = tool
            .execute(json!({"query": "nonexistent"}), &ctx)
            .await
            .unwrap();

        assert!(result.is_success());
        let content = result.to_llm_content();
        assert!(content.contains("\"count\":0") || content.contains("\"count\": 0"));
    }

    #[tokio::test]
    async fn test_memory_search_missing_query() {
        let tool = MemorySearchTool::new();
        let ctx = ToolContext::default();

        let result = tool.execute(json!({}), &ctx).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_time_range() {
        assert_eq!(parse_time_range("today"), TimeRange::Today);
        assert_eq!(parse_time_range("week"), TimeRange::Week);
        assert_eq!(parse_time_range("month"), TimeRange::Month);
        assert_eq!(parse_time_range("all"), TimeRange::All);
        assert_eq!(parse_time_range("unknown"), TimeRange::All);
    }

    #[test]
    fn test_parse_content_type_filter() {
        assert!(parse_content_type_filter("all").is_none());
        assert_eq!(
            parse_content_type_filter("fact"),
            Some(vec![ContentType::Fact])
        );
        assert_eq!(
            parse_content_type_filter("conversation"),
            Some(vec![
                ContentType::UserMessage,
                ContentType::AssistantMessage
            ])
        );
    }
}
