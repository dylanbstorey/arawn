---
id: mcp-runtime-model-context-protocol
level: initiative
title: "MCP Runtime: Model Context Protocol Server Integration"
short_code: "ARAWN-I-0016"
created_at: 2026-01-29T02:18:33.475868+00:00
updated_at: 2026-02-08T17:29:37.464258+00:00
parent: ARAWN-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
strategy_id: NULL
initiative_id: mcp-runtime-model-context-protocol
---

# MCP Runtime: Model Context Protocol Server Integration Initiative

## Context

MCP (Model Context Protocol) is becoming the standard for connecting LLM agents to external tools and data sources. It defines a JSON-RPC protocol over stdio or HTTP for tool discovery, invocation, and resource access. A growing ecosystem of MCP servers exists for databases, APIs, filesystems, and more.

Arawn's plugin system (ARAWN-I-0013) handles skills, hooks, agents, and CLI-wrapper tools. MCP is a complementary transport layer specifically for tool injection — an MCP server declares tools, and Arawn discovers and exposes them to the agent alongside built-in and plugin tools.

Plugins often guide MCP server usage: a plugin might provide skills and prompt fragments that teach the agent how to effectively use an MCP server's tools. The plugin manifest can reference MCP servers it depends on.

## Goals & Non-Goals

**Goals:**
- Implement an MCP client that connects to local MCP servers (stdio transport)
- Auto-discover tools from MCP servers via `tools/list`
- Register MCP tools in `ToolRegistry` alongside built-in and plugin tools
- Support MCP server configuration in `config.toml` and per-plugin
- Support HTTP/SSE transport for remote MCP servers
- Provide `arawn mcp` CLI subcommands for managing MCP server connections
- Hot-registration: API endpoint to add/remove MCP servers at runtime

**Non-Goals:**
- Arawn acting as an MCP *server* (exposing its own tools via MCP) — future work
- MCP resources and prompts — v1 focuses on tools only
- MCP sampling (letting MCP servers call the LLM) — security implications, defer

## Detailed Design

### MCP Client

```rust
// crates/arawn-mcp/src/client.rs
pub struct McpClient {
    transport: McpTransport,
    server_info: ServerInfo,
    available_tools: Vec<McpToolDef>,
}

pub enum McpTransport {
    Stdio { process: Child, stdin: ChildStdin, stdout: BufReader<ChildStdout> },
    Http { base_url: Url, client: reqwest::Client },
}

impl McpClient {
    /// Spawn stdio server or connect to HTTP endpoint
    pub async fn connect(config: &McpServerConfig) -> Result<Self>;
    
    /// JSON-RPC: tools/list
    pub async fn list_tools(&self) -> Result<Vec<McpToolDef>>;
    
    /// JSON-RPC: tools/call
    pub async fn call_tool(&self, name: &str, args: Value) -> Result<McpToolResult>;
    
    /// Graceful shutdown
    pub async fn disconnect(&mut self) -> Result<()>;
}
```

### MCP Tool Adapter

Each MCP tool is wrapped as an Arawn `Tool`:

```rust
pub struct McpToolAdapter {
    name: String,
    description: String,
    parameters: Value,       // JSON Schema from MCP server
    client: Arc<McpClient>,  // shared client for all tools from same server
}

#[async_trait]
impl Tool for McpToolAdapter {
    fn name(&self) -> &str { &self.name }
    fn description(&self) -> &str { &self.description }
    fn parameters(&self) -> Value { self.parameters.clone() }
    
    async fn execute(&self, _id: &str, params: Value, _ctx: &ToolContext) -> ToolResult {
        match self.client.call_tool(&self.name, params).await {
            Ok(result) => ToolResult::text(result.content),
            Err(e) => ToolResult::error(format!("MCP error: {e}")),
        }
    }
}
```

### Configuration

```toml
# config.toml
[[mcp.servers]]
name = "sqlite"
command = "mcp-server-sqlite"
args = ["--db", "/path/to/db.sqlite"]
transport = "stdio"   # default

[[mcp.servers]]
name = "github"
url = "http://localhost:3100"
transport = "http"
```

Plugin manifests can also declare MCP server dependencies:

```toml
# plugin.toml
[[mcp_servers]]
name = "sqlite"
command = "mcp-server-sqlite"
args = ["--db", "{workspace}/data.db"]
```

### MCP Manager

```rust
pub struct McpManager {
    clients: HashMap<String, Arc<McpClient>>,
}

impl McpManager {
    /// Connect to all configured MCP servers, discover tools
    pub async fn connect_all(configs: &[McpServerConfig]) -> Result<Self>;
    
    /// Register all discovered tools into a ToolRegistry
    pub fn register_tools(&self, registry: &mut ToolRegistry);
    
    /// Add a new MCP server at runtime
    pub async fn add_server(&mut self, config: McpServerConfig) -> Result<Vec<String>>;
    
    /// Remove an MCP server and its tools
    pub async fn remove_server(&mut self, name: &str) -> Result<()>;
}
```

### CLI Commands

```
arawn mcp list                         # list connected MCP servers and their tools
arawn mcp add <name> <command> [args]  # add a stdio MCP server
arawn mcp remove <name>                # disconnect and remove
arawn mcp test <name>                  # connect, list tools, disconnect (verify works)
```

### Runtime Registration Endpoint

```
POST /api/v1/mcp/servers    { "name": "sqlite", "command": "mcp-server-sqlite", "args": [...] }
DELETE /api/v1/mcp/servers/:name
GET /api/v1/mcp/servers      # list connected servers + tools
```

This enables the plugin hot-reload watcher (ARAWN-I-0013) to register/unregister MCP servers when plugins change.

## Alternatives Considered

- **Custom tool protocol instead of MCP**: Would need our own discovery, schema, invocation protocol. MCP already solves this and has ecosystem momentum.
- **MCP only (no plugin system)**: MCP handles tools well but doesn't cover skills, hooks, or agents. Both systems are needed.
- **Embedded MCP servers (in-process)**: Some MCP servers could run in-process as Rust libraries. But MCP's value is the subprocess isolation and language-agnostic protocol. Keep them external.
- **gRPC instead of JSON-RPC**: MCP uses JSON-RPC, which is the standard. No reason to deviate.

## Implementation Plan

1. Create `arawn-mcp` crate with `McpClient` (stdio transport)
2. Implement JSON-RPC message framing and `tools/list` / `tools/call`
3. Implement `McpToolAdapter` wrapping MCP tools as Arawn `Tool` trait
4. Implement `McpManager` for multi-server lifecycle
5. Add MCP server configuration to `arawn-config`
6. Wire `McpManager` into server startup (connect, discover, register)
7. Add HTTP/SSE transport support
8. Add `arawn mcp` CLI subcommands
9. Add runtime registration API endpoint
10. Integration tests with a simple test MCP server

## Decomposed Tasks

| Code | Title | Size |
|------|-------|------|
| [[ARAWN-T-0152]] | MCP Client Core: JSON-RPC and Stdio Transport | M |
| [[ARAWN-T-0153]] | MCP Tool Adapter: Wrap MCP Tools as Arawn Tools | S |
| [[ARAWN-T-0154]] | MCP Manager and Configuration | S |
| [[ARAWN-T-0155]] | MCP Server Startup Integration | S |
| [[ARAWN-T-0156]] | MCP HTTP/SSE Transport | S |
| [[ARAWN-T-0157]] | MCP CLI Commands | S |
| [[ARAWN-T-0158]] | MCP Runtime Registration API | S |
| [[ARAWN-T-0159]] | MCP Integration Tests | M |

**Recommended Order:** T-0152 → T-0153 → T-0154 → T-0155 → (T-0156, T-0157, T-0158) → T-0159