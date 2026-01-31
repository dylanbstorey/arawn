---
id: mcp-runtime-registration-api
level: task
title: "MCP Runtime Registration API"
short_code: "ARAWN-T-0158"
created_at: 2026-02-07T19:55:34.000923+00:00
updated_at: 2026-02-08T01:54:03.791775+00:00
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

# MCP Runtime Registration API

## Parent Initiative

[[ARAWN-I-0016]]

## Objective

Add REST API endpoints for runtime MCP server registration, enabling hot-reload when plugins change.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] `POST /api/v1/mcp/servers` - Add new MCP server
- [x] `DELETE /api/v1/mcp/servers/:name` - Remove MCP server
- [x] `GET /api/v1/mcp/servers` - List connected servers and tools
- [x] `GET /api/v1/mcp/servers/:name/tools` - List tools for specific server
- [x] Request validation and error responses
- [x] Auth required for mutating endpoints
- [x] `POST /api/v1/mcp/servers/:name/connect` - Connect to a server
- [x] `POST /api/v1/mcp/servers/:name/disconnect` - Disconnect from a server
- [ ] Integration with plugin watcher for auto-registration (deferred - requires further work)

## Implementation Notes

### Technical Approach

Added McpManager to AppState with thread-safe wrapper (`Arc<RwLock<McpManager>>`). Created new routes module with Axum handlers following existing patterns from sessions.rs. All endpoints require authentication via auth middleware.

### Dependencies

- ARAWN-T-0152 (MCP Client Core) - For McpClient, McpServerConfig
- ARAWN-T-0154 (MCP Manager) - For McpManager lifecycle management

## Status Updates

### 2026-02-08: Implementation Complete

Added REST API endpoints for runtime MCP server management:

**API Endpoints:**
| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/v1/mcp/servers` | GET | List all configured MCP servers and their tools |
| `/api/v1/mcp/servers` | POST | Add a new MCP server (stdio or HTTP transport) |
| `/api/v1/mcp/servers/:name` | DELETE | Remove an MCP server |
| `/api/v1/mcp/servers/:name/tools` | GET | List tools for a specific server |
| `/api/v1/mcp/servers/:name/connect` | POST | Connect to a configured server |
| `/api/v1/mcp/servers/:name/disconnect` | POST | Disconnect from a server |

**Files created/modified:**

1. `crates/arawn-server/Cargo.toml` - Added arawn-mcp dependency

2. `crates/arawn-server/src/state.rs`
   - Added `SharedMcpManager` type alias (`Arc<RwLock<McpManager>>`)
   - Added `mcp_manager` field to `AppState`
   - Added `with_mcp_manager()` builder method

3. `crates/arawn-server/src/routes/mcp.rs` (~450 lines)
   - Request/response types: `AddServerRequest`, `AddServerResponse`, `ServerInfo`, `ListServersResponse`, `ToolInfo`, `ListToolsResponse`, `RemoveServerResponse`
   - Handler functions for all 6 endpoints
   - Comprehensive test suite (17 tests)

4. `crates/arawn-server/src/routes/mod.rs` - Added mcp module and exports

5. `crates/arawn-server/src/lib.rs` - Wired MCP routes into api_routes()

6. `crates/arawn/src/commands/start.rs` - Wired MCP manager into AppState

**Test results:** 61 arawn-server tests pass, full workspace builds cleanly.

**Note:** Plugin watcher integration for auto-registration is deferred as it requires additional coordination between the plugin system and the MCP manager. The current implementation provides all the building blocks needed.