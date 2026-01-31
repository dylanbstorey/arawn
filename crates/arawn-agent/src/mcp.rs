//! MCP tool adapter for integrating MCP tools with the Arawn Tool framework.
//!
//! This module provides [`McpToolAdapter`], which wraps MCP tools as Arawn [`Tool`]
//! implementations, enabling seamless integration with the [`ToolRegistry`].
//!
//! # Example
//!
//! ```rust,ignore
//! use arawn_agent::{ToolRegistry, mcp::McpToolAdapter};
//! use arawn_mcp::{McpClient, McpServerConfig};
//! use std::sync::Arc;
//!
//! // Connect to an MCP server
//! let config = McpServerConfig::new("sqlite", "mcp-server-sqlite")
//!     .with_arg("--db")
//!     .with_arg("/path/to/db.sqlite");
//! let mut client = McpClient::connect_stdio(config)?;
//! client.initialize()?;
//!
//! // Create adapters for all tools
//! let client = Arc::new(client);
//! let adapters = McpToolAdapter::from_client(client)?;
//!
//! // Register with tool registry
//! let mut registry = ToolRegistry::new();
//! for adapter in adapters {
//!     registry.register(adapter);
//! }
//! ```

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;

use arawn_mcp::{CallToolResult, McpClient, McpError, ToolContent, ToolInfo};

use crate::error::Result;
use crate::tool::{Tool, ToolContext, ToolResult};

/// Delimiter used in namespaced tool names.
pub const NAMESPACE_DELIMITER: &str = ":";

/// Prefix for all MCP tool names.
pub const MCP_PREFIX: &str = "mcp";

/// Adapter that wraps an MCP tool as an Arawn [`Tool`].
///
/// This adapter:
/// - Maps MCP tool schemas to Arawn's JSON Schema format
/// - Delegates execution to the underlying [`McpClient`]
/// - Handles MCP error responses gracefully
/// - Supports tool namespacing (e.g., `mcp:sqlite:query`)
pub struct McpToolAdapter {
    /// The fully qualified tool name (e.g., "mcp:sqlite:query").
    full_name: String,
    /// The server name this tool belongs to.
    server_name: String,
    /// The original tool name from the MCP server.
    tool_name: String,
    /// Human-readable description.
    description: String,
    /// JSON Schema for tool parameters.
    parameters: Value,
    /// Shared reference to the MCP client.
    client: Arc<McpClient>,
}

impl McpToolAdapter {
    /// Create a new MCP tool adapter.
    ///
    /// # Arguments
    /// * `client` - Shared reference to the MCP client
    /// * `tool_info` - Tool information from the MCP server
    pub fn new(client: Arc<McpClient>, tool_info: &ToolInfo) -> Self {
        let server_name = client.name().to_string();
        let full_name = format!(
            "{}{}{}{}{}",
            MCP_PREFIX, NAMESPACE_DELIMITER, server_name, NAMESPACE_DELIMITER, tool_info.name
        );

        let description = tool_info
            .description
            .clone()
            .unwrap_or_else(|| format!("MCP tool: {}", tool_info.name));

        // MCP uses inputSchema, we need to pass it as-is since it's already JSON Schema
        let parameters = tool_info.input_schema.clone().unwrap_or_else(|| {
            serde_json::json!({
                "type": "object",
                "properties": {}
            })
        });

        Self {
            full_name,
            server_name,
            tool_name: tool_info.name.clone(),
            description,
            parameters,
            client,
        }
    }

    /// Create adapters for all tools available from an MCP client.
    ///
    /// This is a convenience method that calls `list_tools()` on the client
    /// and creates an adapter for each tool.
    ///
    /// # Errors
    /// Returns an error if listing tools fails.
    pub fn from_client(client: Arc<McpClient>) -> std::result::Result<Vec<Self>, McpError> {
        // We need to use a non-Arc reference to call list_tools
        // This is safe because we're only reading
        let tools = client.list_tools()?;

        Ok(tools
            .iter()
            .map(|tool_info| Self::new(Arc::clone(&client), tool_info))
            .collect())
    }

    /// Get the server name this tool belongs to.
    pub fn server_name(&self) -> &str {
        &self.server_name
    }

    /// Get the original tool name (without namespace).
    pub fn tool_name(&self) -> &str {
        &self.tool_name
    }

    /// Check if a tool name matches this adapter's namespaced name.
    ///
    /// Supports matching by:
    /// - Full name: `mcp:sqlite:query`
    /// - Server-qualified: `sqlite:query`
    /// - Original name: `query` (only if unambiguous)
    pub fn matches_name(&self, name: &str) -> bool {
        name == self.full_name
            || name
                == format!(
                    "{}{}{}",
                    self.server_name, NAMESPACE_DELIMITER, self.tool_name
                )
            || name == self.tool_name
    }
}

impl std::fmt::Debug for McpToolAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpToolAdapter")
            .field("full_name", &self.full_name)
            .field("server_name", &self.server_name)
            .field("tool_name", &self.tool_name)
            .field("description", &self.description)
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl Tool for McpToolAdapter {
    fn name(&self) -> &str {
        &self.full_name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn parameters(&self) -> Value {
        self.parameters.clone()
    }

    async fn execute(&self, params: Value, _ctx: &ToolContext) -> Result<ToolResult> {
        tracing::debug!(
            server = %self.server_name,
            tool = %self.tool_name,
            "executing MCP tool"
        );

        // Call the MCP tool
        let mcp_result = match self.client.call_tool(&self.tool_name, Some(params)) {
            Ok(result) => result,
            Err(e) => {
                tracing::warn!(
                    server = %self.server_name,
                    tool = %self.tool_name,
                    error = %e,
                    "MCP tool call failed"
                );
                return Ok(ToolResult::error(format!("MCP error: {}", e)));
            }
        };

        // Convert MCP result to Arawn ToolResult
        Ok(convert_mcp_result(mcp_result))
    }
}

/// Convert an MCP [`CallToolResult`] to an Arawn [`ToolResult`].
fn convert_mcp_result(mcp_result: CallToolResult) -> ToolResult {
    // Check if it's an error response
    if mcp_result.is_error() {
        let error_text = mcp_result
            .text()
            .unwrap_or_else(|| "Unknown MCP error".to_string());
        return ToolResult::error(error_text);
    }

    // Convert content to ToolResult
    // MCP can return multiple content items; we'll concatenate text content
    let mut text_parts: Vec<String> = Vec::new();
    let mut has_non_text = false;

    for content in &mcp_result.content {
        match content {
            ToolContent::Text { text } => {
                text_parts.push(text.clone());
            }
            ToolContent::Image { .. } => {
                // For now, indicate that images were returned but not included
                text_parts.push("[Image content not displayed]".to_string());
                has_non_text = true;
            }
            ToolContent::Resource { uri, text, .. } => {
                if let Some(t) = text {
                    text_parts.push(t.clone());
                } else {
                    text_parts.push(format!("[Resource: {}]", uri));
                    has_non_text = true;
                }
            }
        }
    }

    if text_parts.is_empty() {
        if has_non_text {
            ToolResult::text("[Non-text content returned]")
        } else {
            ToolResult::text("")
        }
    } else {
        ToolResult::text(text_parts.join("\n"))
    }
}

/// Parse a namespaced tool name into its components.
///
/// # Returns
/// A tuple of (prefix, server_name, tool_name) if the name is fully qualified,
/// or None if the format is invalid.
///
/// # Examples
/// ```rust,ignore
/// assert_eq!(
///     parse_namespaced_name("mcp:sqlite:query"),
///     Some(("mcp", "sqlite", "query"))
/// );
/// ```
pub fn parse_namespaced_name(name: &str) -> Option<(&str, &str, &str)> {
    let parts: Vec<&str> = name.split(NAMESPACE_DELIMITER).collect();
    if parts.len() == 3 && parts[0] == MCP_PREFIX {
        Some((parts[0], parts[1], parts[2]))
    } else {
        None
    }
}

/// Check if a tool name is an MCP tool (starts with "mcp:").
pub fn is_mcp_tool(name: &str) -> bool {
    name.starts_with(MCP_PREFIX) && name.contains(NAMESPACE_DELIMITER)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_namespace_delimiter() {
        assert_eq!(NAMESPACE_DELIMITER, ":");
    }

    #[test]
    fn test_mcp_prefix() {
        assert_eq!(MCP_PREFIX, "mcp");
    }

    #[test]
    fn test_parse_namespaced_name_valid() {
        let result = parse_namespaced_name("mcp:sqlite:query");
        assert_eq!(result, Some(("mcp", "sqlite", "query")));
    }

    #[test]
    fn test_parse_namespaced_name_long_tool_name() {
        let result = parse_namespaced_name("mcp:filesystem:read_file");
        assert_eq!(result, Some(("mcp", "filesystem", "read_file")));
    }

    #[test]
    fn test_parse_namespaced_name_invalid_prefix() {
        let result = parse_namespaced_name("tool:sqlite:query");
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_namespaced_name_too_few_parts() {
        let result = parse_namespaced_name("mcp:sqlite");
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_namespaced_name_no_delimiter() {
        let result = parse_namespaced_name("mcp_sqlite_query");
        assert_eq!(result, None);
    }

    #[test]
    fn test_is_mcp_tool_valid() {
        assert!(is_mcp_tool("mcp:sqlite:query"));
        assert!(is_mcp_tool("mcp:filesystem:read"));
    }

    #[test]
    fn test_is_mcp_tool_invalid() {
        assert!(!is_mcp_tool("sqlite:query"));
        assert!(!is_mcp_tool("mcp"));
        assert!(!is_mcp_tool("shell"));
    }

    #[test]
    fn test_convert_mcp_result_text() {
        let mcp_result = CallToolResult {
            content: vec![ToolContent::Text {
                text: "Hello, world!".to_string(),
            }],
            is_error: None,
        };

        let result = convert_mcp_result(mcp_result);
        match result {
            ToolResult::Text { content } => assert_eq!(content, "Hello, world!"),
            _ => panic!("Expected Text result"),
        }
    }

    #[test]
    fn test_convert_mcp_result_multiple_text() {
        let mcp_result = CallToolResult {
            content: vec![
                ToolContent::Text {
                    text: "Line 1".to_string(),
                },
                ToolContent::Text {
                    text: "Line 2".to_string(),
                },
            ],
            is_error: None,
        };

        let result = convert_mcp_result(mcp_result);
        match result {
            ToolResult::Text { content } => assert_eq!(content, "Line 1\nLine 2"),
            _ => panic!("Expected Text result"),
        }
    }

    #[test]
    fn test_convert_mcp_result_error() {
        let mcp_result = CallToolResult {
            content: vec![ToolContent::Text {
                text: "Something went wrong".to_string(),
            }],
            is_error: Some(true),
        };

        let result = convert_mcp_result(mcp_result);
        assert!(result.is_error());
    }

    #[test]
    fn test_convert_mcp_result_empty() {
        let mcp_result = CallToolResult {
            content: vec![],
            is_error: None,
        };

        let result = convert_mcp_result(mcp_result);
        match result {
            ToolResult::Text { content } => assert_eq!(content, ""),
            _ => panic!("Expected Text result"),
        }
    }

    #[test]
    fn test_convert_mcp_result_image() {
        let mcp_result = CallToolResult {
            content: vec![ToolContent::Image {
                data: "base64data".to_string(),
                mime_type: "image/png".to_string(),
            }],
            is_error: None,
        };

        let result = convert_mcp_result(mcp_result);
        match result {
            ToolResult::Text { content } => {
                assert!(content.contains("Image content not displayed"))
            }
            _ => panic!("Expected Text result"),
        }
    }

    #[test]
    fn test_convert_mcp_result_resource_with_text() {
        let mcp_result = CallToolResult {
            content: vec![ToolContent::Resource {
                uri: "file:///path/to/file".to_string(),
                text: Some("File contents".to_string()),
                mime_type: None,
            }],
            is_error: None,
        };

        let result = convert_mcp_result(mcp_result);
        match result {
            ToolResult::Text { content } => assert_eq!(content, "File contents"),
            _ => panic!("Expected Text result"),
        }
    }

    #[test]
    fn test_convert_mcp_result_resource_without_text() {
        let mcp_result = CallToolResult {
            content: vec![ToolContent::Resource {
                uri: "file:///path/to/file".to_string(),
                text: None,
                mime_type: None,
            }],
            is_error: None,
        };

        let result = convert_mcp_result(mcp_result);
        match result {
            ToolResult::Text { content } => {
                assert!(content.contains("file:///path/to/file"));
            }
            _ => panic!("Expected Text result"),
        }
    }
}
