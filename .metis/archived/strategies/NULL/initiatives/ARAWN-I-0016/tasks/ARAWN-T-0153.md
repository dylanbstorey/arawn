---
id: mcp-tool-adapter-wrap-mcp-tools-as
level: task
title: "MCP Tool Adapter: Wrap MCP Tools as Arawn Tools"
short_code: "ARAWN-T-0153"
created_at: 2026-02-07T19:55:29.737327+00:00
updated_at: 2026-02-08T17:28:51.349923+00:00
parent: ARAWN-I-0016
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0016
---

# MCP Tool Adapter: Wrap MCP Tools as Arawn Tools

## Parent Initiative

[[ARAWN-I-0016]] - MCP Runtime: Model Context Protocol Server Integration

## Objective

Create an adapter that wraps MCP tools as Arawn `Tool` trait implementations, enabling seamless integration with the existing ToolRegistry.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Create `McpToolAdapter` struct implementing `Tool` trait
- [x] Map MCP tool schema to Arawn's JSON Schema format
- [x] Delegate `execute()` to `McpClient::call_tool()`
- [x] Handle MCP error responses gracefully
- [x] Support tool namespacing (e.g., `mcp:sqlite:query`)
- [x] Preserve MCP tool metadata (description, schema)
- [x] Unit tests for adapter logic

## Implementation Notes

### Technical Approach

```rust
// crates/arawn-mcp/src/adapter.rs
pub struct McpToolAdapter {
    server_name: String,
    tool_name: String,
    description: String,
    parameters: Value,
    client: Arc<McpClient>,
}

#[async_trait]
impl Tool for McpToolAdapter {
    fn name(&self) -> &str {
        // e.g., "mcp:sqlite:query"
        &self.full_name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn parameters(&self) -> Value {
        self.parameters.clone()
    }
    
    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        match self.client.call_tool(&self.tool_name, params).await {
            Ok(result) => ToolResult::from_mcp(result),
            Err(e) => ToolResult::error(format!("MCP error: {}", e)),
        }
    }
}
```

### Files to Create/Modify

- `crates/arawn-mcp/src/adapter.rs` (new)
- `crates/arawn-mcp/src/lib.rs` (export adapter)

### Dependencies

- ARAWN-T-0152 (MCP Client Core)

## Status Updates

### Session 2026-02-07

**Completed implementation of MCP Tool Adapter:**

**Architecture Decision**: The adapter was placed in `arawn-agent` rather than `arawn-mcp` to avoid circular dependencies. Since the `Tool` trait is defined in `arawn-agent`, and `arawn-agent` now depends on `arawn-mcp`, the adapter lives where the trait is.

**Files created/modified:**
- `crates/arawn-agent/src/mcp.rs` - New MCP adapter module
- `crates/arawn-agent/src/lib.rs` - Added module and exports
- `crates/arawn-agent/Cargo.toml` - Added arawn-mcp dependency

**Key types:**
- `McpToolAdapter` - Wraps MCP tools as Arawn `Tool` implementations
  - `new(client, tool_info)` - Create from client and tool info
  - `from_client(client)` - Create adapters for all tools from a client
  - Implements `Tool` trait: `name()`, `description()`, `parameters()`, `execute()`
- `parse_namespaced_name()` - Parse "mcp:server:tool" format
- `is_mcp_tool()` - Check if a tool name is an MCP tool
- `MCP_PREFIX` ("mcp") and `NAMESPACE_DELIMITER` (":")

**Features:**
- Tool namespacing: `mcp:sqlite:query` format
- Flexible name matching: full name, server:tool, or just tool name
- MCP schema passthrough (inputSchema → parameters)
- Content type conversion: Text, Image (placeholder), Resource
- Error handling: MCP errors → recoverable ToolResult::error

**Test coverage:**
- 16 unit tests for adapter logic
- Tests for name parsing, content conversion, error handling
- All tests passing, workspace compiles cleanly