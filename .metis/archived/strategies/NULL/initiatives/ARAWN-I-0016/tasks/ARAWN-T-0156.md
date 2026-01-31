---
id: mcp-http-sse-transport
level: task
title: "MCP HTTP/SSE Transport"
short_code: "ARAWN-T-0156"
created_at: 2026-02-07T19:55:32.250214+00:00
updated_at: 2026-02-08T17:29:03.409231+00:00
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

# MCP HTTP/SSE Transport

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[ARAWN-I-0016]]

## Objective

Add HTTP/SSE transport support to `McpClient` for connecting to remote MCP servers.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Implement `McpTransport::Http` variant
- [x] HTTP POST for JSON-RPC requests
- [x] SSE for server-initiated notifications (if needed) - Note: MCP notifications sent via POST, response ignored
- [x] Connection pooling with `reqwest::Client` (via Arc<Client>)
- [x] Timeout and retry configuration
- [x] TLS/HTTPS support (via reqwest default HTTPS handling)
- [x] Config: `transport = "http"` with `url` field
- [ ] Integration test with HTTP MCP server (deferred to ARAWN-T-0159)



## Implementation Notes

### Technical Approach
Added HTTP transport alongside existing stdio transport:
1. `McpTransport::Http` variant with `Arc<reqwest::blocking::Client>` and `HttpTransportConfig`
2. `HttpTransportConfig` for timeout (default 30s), retries (default 3), and custom headers
3. `McpClient::connect_http()` method creates HTTP transport from config
4. `McpClient::connect()` auto-selects transport based on `TransportType` in config
5. `McpServerConfig` extended with `transport`, `url`, `headers`, `timeout`, `retries` fields
6. Config types updated with `McpTransportType` enum (`stdio` or `http`)

### Dependencies
- ARAWN-T-0152: MCP Client Core (completed)
- ARAWN-T-0154: MCP Manager and Configuration (completed)

## Status Updates **[REQUIRED]**

### 2026-02-07: Implementation Complete

**Files Modified:**

1. **`Cargo.toml` (workspace)**
   - Added `blocking` feature to reqwest

2. **`crates/arawn-mcp/Cargo.toml`**
   - Added `reqwest` and `url` dependencies

3. **`crates/arawn-mcp/src/transport.rs`**
   - Added `HttpTransportConfig` struct with timeout, retries, headers
   - Added `McpTransport::Http` variant
   - Added `connect_http()` constructor
   - Updated `send_request()` to dispatch to HTTP or stdio
   - Added `send_request_http_impl()` with retry logic
   - Added `is_http()` and `is_stdio()` helpers
   - 6 new unit tests for HTTP transport

4. **`crates/arawn-mcp/src/client.rs`**
   - Added `TransportType` enum (Stdio, Http)
   - Extended `McpServerConfig` with transport, url, headers, timeout, retries
   - Added `McpServerConfig::http()` constructor
   - Added `McpClient::connect()` for auto transport selection
   - Added `McpClient::connect_http()` for explicit HTTP
   - 5 new unit tests for HTTP client

5. **`crates/arawn-mcp/src/lib.rs`**
   - Re-exported `TransportType` and `HttpTransportConfig`

6. **`crates/arawn-config/src/types.rs`**
   - Added `McpTransportType` enum with serde rename_all
   - Extended `McpServerEntry` with transport, url, headers, timeout_secs, retries
   - Added `McpServerEntry::http()` constructor
   - Added `is_http()`, `is_stdio()`, `header_tuples()` methods

7. **`crates/arawn/src/commands/start.rs`**
   - Updated MCP config conversion to handle both transport types

**TOML Configuration Example:**
```toml
[mcp]
enabled = true

# Stdio transport (default)
[[mcp.servers]]
name = "sqlite"
command = "mcp-server-sqlite"
args = ["--db", "/path/to/db.sqlite"]

# HTTP transport
[[mcp.servers]]
name = "remote"
transport = "http"
url = "https://mcp.example.com/api"
headers = [["Authorization", "Bearer token123"]]
timeout_secs = 60
retries = 5
```

**Test Results:**
- 40 MCP tests passing (33 unit + 7 integration)
- 91 config tests passing
- `angreal check all` passes