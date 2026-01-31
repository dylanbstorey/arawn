---
id: mcp-manager-and-configuration
level: task
title: "MCP Manager and Configuration"
short_code: "ARAWN-T-0154"
created_at: 2026-02-07T19:55:30.562629+00:00
updated_at: 2026-02-08T17:29:02.107584+00:00
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

# MCP Manager and Configuration

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[ARAWN-I-0016]]

## Objective

Implement `McpManager` for multi-server lifecycle management and add MCP configuration support to `arawn-config`.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Create `McpManager` struct to manage multiple MCP clients
- [x] Implement `connect_all()` to spawn all configured servers
- [x] Implement `register_tools()` to add all tools to ToolRegistry (via `list_all_tools()` + adapter integration)
- [x] Implement `add_server()` / `remove_server()` for dynamic management
- [x] Add `[mcp]` section to config schema in `arawn-config`
- [x] Support both global config and per-plugin MCP servers (via `McpServerEntry`)
- [x] Graceful shutdown of all servers
- [x] Unit tests for manager lifecycle (10 tests)



## Implementation Notes

### Technical Approach
- `McpManager` uses `HashMap<String, McpServerConfig>` for configs and `HashMap<String, Arc<McpClient>>` for connected clients
- Servers are connected lazily via `connect_all()` or `connect_server_by_name()`
- Tool discovery uses `list_all_tools()` returning `HashMap<String, Vec<ToolInfo>>`
- Integration with `ToolRegistry` is handled by `McpToolAdapter` (from ARAWN-T-0153)

### Dependencies
- ARAWN-T-0152: MCP Client Core (completed)
- ARAWN-T-0153: MCP Tool Adapter (completed)

## Status Updates **[REQUIRED]**

### 2026-02-07: Implementation Complete

**Files Created/Modified:**

1. **`crates/arawn-config/src/types.rs`** - Added MCP configuration types:
   - `McpConfig` struct with `enabled: bool` and `servers: Vec<McpServerEntry>`
   - `McpServerEntry` struct for individual server configs (name, command, args, env, enabled)
   - Updated `ArawnConfig`, `RawConfig`, `merge()`, and `From` implementations
   - 10 unit tests for MCP config parsing and manipulation

2. **`crates/arawn-mcp/src/manager.rs`** - NEW: Multi-server lifecycle manager:
   - `McpManager` with `configs` and `clients` HashMaps
   - `new()`, `with_configs()` constructors
   - `add_server()`, `remove_server()` for dynamic management
   - `connect_all()`, `connect_server_by_name()` for spawning servers
   - `list_all_tools()`, `all_tools_flat()`, `tool_count()` for tool discovery
   - `shutdown_all()`, `shutdown_server()` for graceful cleanup
   - `Drop` implementation for automatic cleanup
   - 10 unit tests for manager lifecycle

3. **`crates/arawn-mcp/src/lib.rs`** - Added module export:
   - `pub mod manager;`
   - `pub use manager::McpManager;`

**Test Results:**
- All 91 config tests passing
- All 31 MCP tests passing (includes manager + client tests)
- `angreal check all` passes cleanly

**TOML Configuration Example:**
```toml
[mcp]
enabled = true

[[mcp.servers]]
name = "sqlite"
command = "mcp-server-sqlite"
args = ["--db", "/path/to/db.sqlite"]
enabled = true

[[mcp.servers]]
name = "filesystem"
command = "mcp-server-filesystem"
args = ["/home/user/docs"]
env = [["MCP_LOG", "debug"]]
enabled = true
```