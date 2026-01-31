---
id: mcp-client-core-json-rpc-and-stdio
level: task
title: "MCP Client Core: JSON-RPC and Stdio Transport"
short_code: "ARAWN-T-0152"
created_at: 2026-02-07T19:55:28.769455+00:00
updated_at: 2026-02-07T20:10:24.852366+00:00
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

# MCP Client Core: JSON-RPC and Stdio Transport

## Parent Initiative

[[ARAWN-I-0016]] - MCP Runtime: Model Context Protocol Server Integration

## Objective

Create the `arawn-mcp` crate with core MCP client functionality: JSON-RPC 2.0 protocol implementation and stdio transport for spawning and communicating with local MCP servers.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Create `arawn-mcp` crate with proper workspace integration
- [x] Implement JSON-RPC 2.0 message framing (Content-Length header protocol)
- [x] Implement `McpTransport::Stdio` - spawn child process, manage stdin/stdout
- [x] Implement `initialize` handshake with MCP server
- [x] Implement `tools/list` request to discover available tools
- [x] Implement `tools/call` request to invoke tools
- [x] Proper error handling with `McpError` type
- [x] Graceful shutdown and process cleanup
- [x] Unit tests for JSON-RPC serialization/deserialization
- [x] Integration test with a mock MCP server

## Implementation Notes

### Technical Approach

```rust
// crates/arawn-mcp/src/client.rs
pub struct McpClient {
    transport: McpTransport,
    server_info: Option<ServerInfo>,
    request_id: AtomicU64,
}

pub enum McpTransport {
    Stdio {
        child: Child,
        stdin: BufWriter<ChildStdin>,
        stdout: BufReader<ChildStdout>,
    },
}

impl McpClient {
    pub async fn connect_stdio(command: &str, args: &[String]) -> Result<Self>;
    pub async fn initialize(&mut self) -> Result<ServerInfo>;
    pub async fn list_tools(&self) -> Result<Vec<ToolInfo>>;
    pub async fn call_tool(&self, name: &str, args: Value) -> Result<ToolResult>;
    pub async fn shutdown(&mut self) -> Result<()>;
}
```

### JSON-RPC Message Format

```
Content-Length: <length>\r\n
\r\n
{"jsonrpc": "2.0", "id": 1, "method": "...", "params": {...}}
```

### Files to Create

- `crates/arawn-mcp/Cargo.toml`
- `crates/arawn-mcp/src/lib.rs`
- `crates/arawn-mcp/src/client.rs`
- `crates/arawn-mcp/src/transport.rs`
- `crates/arawn-mcp/src/protocol.rs` (JSON-RPC types)
- `crates/arawn-mcp/src/error.rs`

### Dependencies

None - this is the foundational task

## Status Updates

### Session 2026-02-07

**Completed implementation of arawn-mcp crate:**

**Files created:**
- `crates/arawn-mcp/Cargo.toml` - Crate manifest with workspace integration
- `crates/arawn-mcp/src/lib.rs` - Module exports and crate documentation
- `crates/arawn-mcp/src/error.rs` - `McpError` enum with variants for all error cases
- `crates/arawn-mcp/src/protocol.rs` - JSON-RPC 2.0 and MCP protocol types
- `crates/arawn-mcp/src/transport.rs` - `McpTransport::Stdio` implementation
- `crates/arawn-mcp/src/client.rs` - `McpClient` and `McpServerConfig`
- `crates/arawn-mcp/tests/mock_server.rs` - Mock MCP server binary for testing
- `crates/arawn-mcp/tests/integration.rs` - 7 integration tests

**Key types:**
- `McpClient` - Main client with connect_stdio(), initialize(), list_tools(), call_tool(), shutdown()
- `McpServerConfig` - Server configuration with command, args, env
- `McpTransport` - Stdio transport with Content-Length framing
- `McpError` - Error types: SpawnFailed, Transport, Protocol, Json, Io, ServerError, etc.
- `ToolInfo`, `CallToolResult`, `ToolContent` - MCP tool types

**Test coverage:**
- 15 unit tests for protocol, error, client, and transport modules
- 7 integration tests with mock MCP server (connect, initialize, list_tools, call_tool, shutdown)
- All tests passing, workspace compiles cleanly