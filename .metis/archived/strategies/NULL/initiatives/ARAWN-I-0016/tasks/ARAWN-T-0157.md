---
id: mcp-cli-commands
level: task
title: "MCP CLI Commands"
short_code: "ARAWN-T-0157"
created_at: 2026-02-07T19:55:33.087555+00:00
updated_at: 2026-02-08T01:35:40.490524+00:00
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

# MCP CLI Commands

## Parent Initiative

[[ARAWN-I-0016]]

## Objective

Add `arawn mcp` CLI subcommands for managing MCP server connections.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] `arawn mcp list` - List connected MCP servers and their tools
- [x] `arawn mcp add <name> <command> [args...]` - Add stdio MCP server (with --http for HTTP transport)
- [x] `arawn mcp remove <name>` - Disconnect and remove server
- [x] `arawn mcp test <name>` - Connect, list tools, disconnect (verify works)
- [x] JSON and table output formats (--json flag from global CLI)
- [x] Persistent config updates (write to config.toml via save_config)
- [x] Helpful error messages for common issues

## Implementation Notes

### Technical Approach

Created a new command module following the existing plugin.rs pattern:
- Clap Args/Subcommand structs for CLI parsing
- Async run functions for each subcommand
- Table and JSON output formats using Context flags
- Uses `arawn_mcp::McpClient` for server connections
- Uses new `arawn_config::save_config` for persistent config updates

### Dependencies

- ARAWN-T-0152 (MCP Client Core) - For McpClient, McpServerConfig
- ARAWN-T-0154 (MCP Manager) - For McpManager (used in list --tools)

## Status Updates

### 2026-02-08: Implementation Complete

Created `crates/arawn/src/commands/mcp.rs` with full CLI implementation:

**Commands implemented:**
- `arawn mcp list [--tools]` - Lists configured servers with optional tool discovery
- `arawn mcp add <name> <target> [--http] [--env KEY=VAL] [--header KEY=VAL] [--timeout] [--retries] [--disabled]`
- `arawn mcp remove <name>` - Removes server from config
- `arawn mcp test <name> [--full]` - Tests connection, initialization, and tool listing

**Changes made:**
1. Created `crates/arawn/src/commands/mcp.rs` (~650 lines)
   - Clap-based subcommands following existing plugin.rs pattern
   - Table and JSON output formats
   - Stdio and HTTP transport support in add command
   - Test command with detailed connection/initialization/tool output

2. Updated `crates/arawn/src/commands/mod.rs` - Added mcp module export

3. Updated `crates/arawn/src/main.rs`
   - Added `Mcp(mcp::McpArgs)` to Commands enum
   - Added dispatch case in main match

4. Added `save_config` to arawn-config:
   - `crates/arawn-config/src/discovery.rs` - Added `save_config()` function
   - `crates/arawn-config/src/error.rs` - Added `WriteFile` error variant
   - `crates/arawn-config/src/lib.rs` - Export `save_config`
   - Added `source` field to `LoadedConfig` for primary config path tracking

5. Updated `crates/arawn/src/commands/start.rs` - Added source field to manual LoadedConfig

**All tests pass:** 91 config tests, 33 MCP tests, full workspace builds cleanly.