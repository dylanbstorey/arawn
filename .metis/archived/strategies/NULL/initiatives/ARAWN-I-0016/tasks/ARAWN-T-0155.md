---
id: mcp-server-startup-integration
level: task
title: "MCP Server Startup Integration"
short_code: "ARAWN-T-0155"
created_at: 2026-02-07T19:55:31.405791+00:00
updated_at: 2026-02-08T17:29:02.873778+00:00
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

# MCP Server Startup Integration

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[ARAWN-I-0016]]

## Objective

Wire `McpManager` into the server startup flow so MCP servers are connected and their tools registered when Arawn starts.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Load MCP config from `config.toml` during startup
- [x] Create `McpManager` and connect to all configured servers
- [x] Register MCP tools in the agent's ToolRegistry
- [x] Log connected servers and discovered tools
- [x] Handle connection failures gracefully (warn, continue without)
- [x] Shutdown MCP servers on graceful server stop
- [ ] Integration test: start server with MCP config, verify tools available (deferred to ARAWN-T-0159)



## Implementation Notes

### Technical Approach
The MCP integration is wired into `start.rs` between the plugin system and the hook dispatcher. It:
1. Reads MCP config from the merged TOML configuration
2. Filters to enabled servers and converts `McpServerEntry` to `McpServerConfig`
3. Creates `McpManager` and calls `connect_all()` to spawn server processes
4. For each connected server, creates `McpToolAdapter` instances and registers them in the tool registry
5. Logs connection status and tool registration count
6. On shutdown, calls `manager.shutdown_all()` for graceful cleanup

### Dependencies
- ARAWN-T-0152: MCP Client Core (completed)
- ARAWN-T-0153: MCP Tool Adapter (completed)
- ARAWN-T-0154: MCP Manager and Configuration (completed)

## Status Updates **[REQUIRED]**

### 2026-02-07: Implementation Complete

**Files Modified:**

1. **`crates/arawn/Cargo.toml`**
   - Added `arawn-mcp = { workspace = true }` dependency

2. **`crates/arawn/src/commands/start.rs`**
   - Added imports: `McpToolAdapter`, `Tool`, `McpManager`, `McpServerConfig`
   - Added MCP initialization section (~90 lines) after plugin system:
     - Reads `config.mcp` and filters to enabled servers
     - Converts config entries to `McpServerConfig`
     - Creates `McpManager::with_configs()` and calls `connect_all()`
     - For each connected server, creates `McpToolAdapter` and registers tools
     - Logs connection status and tool count
   - Added MCP shutdown in graceful shutdown section

**Behavior:**

With `[mcp]` configuration in `arawn.toml`:
```toml
[mcp]
enabled = true

[[mcp.servers]]
name = "sqlite"
command = "mcp-server-sqlite"
args = ["--db", "/path/to/db.sqlite"]
enabled = true
```

On startup:
```
MCP: connecting to 1 server(s)...
  Registered: mcp:sqlite:query
  Registered: mcp:sqlite:execute
MCP: 1 server(s) connected, 2 tool(s) registered
```

On shutdown:
```
Shutting down MCP servers...
```

**Error Handling:**
- Connection failures log warnings but don't prevent startup
- Tool listing failures log warnings but continue with other servers
- Shutdown errors log warnings but don't fail the process

**Test Results:**
- All 24 MCP unit tests passing
- All 7 MCP integration tests passing
- `angreal check all` passes
- `angreal test unit` passes