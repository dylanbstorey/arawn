# MCP Integration

Model Context Protocol (MCP) enables Arawn to bridge external tool servers.

## Overview

MCP allows Arawn to:
- Connect to external tool servers
- Expose their tools to the agent
- Handle tool execution transparently

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│ arawn-mcp                                                        │
│                                                                  │
│  ┌───────────────────────┐    ┌───────────────────────────────┐ │
│  │ McpManager            │    │ McpClient                      │ │
│  │                       │    │                                │ │
│  │ servers: HashMap<     │    │ transport: StdioTransport      │ │
│  │   String, McpClient>  │───▶│ capabilities: ServerCaps      │ │
│  │                       │    │                                │ │
│  │ list_tools() ────────▶│    │ list_tools() → Vec<Tool>      │ │
│  │ call_tool()  ────────▶│    │ call_tool(name, params)       │ │
│  └───────────────────────┘    └───────────────────────────────┘ │
│                                                                  │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │ McpToolAdapter                                             │  │
│  │                                                            │  │
│  │ Wraps MCP tools as Arawn Tool trait:                      │  │
│  │ • name() → "mcp__{server}__{tool}"                        │  │
│  │ • parameters() → JSON Schema from MCP                     │  │
│  │ • execute() → McpClient.call_tool()                       │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

## Configuration

### Basic Setup

```toml
[mcp]
enabled = true

[mcp.servers.sqlite]
command = "sqlite-mcp"
args = ["--db", "memory.db"]
```

### Transport Types

| Transport | Config | Use Case |
|-----------|--------|----------|
| **stdio** | `command`, `args` | Local CLI tools |
| **sse** | `url` | HTTP Server-Sent Events |

### Stdio Transport

```toml
[mcp.servers.sqlite]
command = "sqlite-mcp"
args = ["--db", "/path/to/database.db"]
env = { "DEBUG" = "true" }
```

### SSE Transport

```toml
[mcp.servers.remote]
url = "http://localhost:3000/mcp"
headers = { "Authorization" = "Bearer $env:MCP_TOKEN" }
```

## Tool Namespacing

MCP tools are namespaced to avoid collisions:

- **Pattern:** `mcp__{server_name}__{tool_name}`
- **Example:** `mcp__sqlite__query` for the `query` tool from `sqlite` server

When the LLM calls `mcp__sqlite__query`, Arawn:
1. Extracts server name: `sqlite`
2. Extracts tool name: `query`
3. Routes to the correct MCP client
4. Executes and returns result

## Available MCP Servers

### sqlite-mcp

SQLite database access:

```toml
[mcp.servers.sqlite]
command = "sqlite-mcp"
args = ["--db", "data.db"]
```

Tools provided:
- `query` — Execute SQL queries
- `schema` — Get table schemas
- `insert` — Insert rows

### filesystem-mcp

Enhanced file operations:

```toml
[mcp.servers.filesystem]
command = "filesystem-mcp"
args = ["--root", "/allowed/path"]
```

### Custom Servers

Any MCP-compatible server can be added:

```toml
[mcp.servers.my-server]
command = "/path/to/my-mcp-server"
args = ["--config", "config.json"]
```

## Server Lifecycle

1. **Startup** — MCP servers started with Arawn
2. **Discovery** — Server capabilities and tools listed
3. **Registration** — Tools added to agent's registry
4. **Execution** — Tools called via MCP protocol
5. **Shutdown** — Servers stopped when Arawn exits

## Error Handling

| Error | Behavior |
|-------|----------|
| Server crash | Reconnect attempt, then disable |
| Tool timeout | Return timeout error to LLM |
| Invalid response | Parse error returned to LLM |
| Connection lost | Attempt reconnection |

## Debugging

Enable MCP debug logging:

```bash
RUST_LOG=arawn_mcp=debug arawn start
```

List connected MCP servers:

```bash
arawn mcp list
```

Test a specific tool:

```bash
arawn mcp call sqlite query '{"sql": "SELECT 1"}'
```
