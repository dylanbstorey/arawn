---
id: mcp-integration-tests
level: task
title: "MCP Integration Tests"
short_code: "ARAWN-T-0159"
created_at: 2026-02-07T19:55:34.757397+00:00
updated_at: 2026-02-08T02:25:23.054533+00:00
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

# MCP Integration Tests

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[ARAWN-I-0016]]

## Objective

Create integration tests for the MCP system using a simple test MCP server.

## Backlog Item Details **[CONDITIONAL: Backlog Item]**

{Delete this section when task is assigned to an initiative}

### Type
- [ ] Bug - Production issue that needs fixing
- [ ] Feature - New functionality or enhancement  
- [ ] Tech Debt - Code improvement or refactoring
- [ ] Chore - Maintenance or setup work

### Priority
- [ ] P0 - Critical (blocks users/revenue)
- [ ] P1 - High (important for user experience)
- [ ] P2 - Medium (nice to have)
- [ ] P3 - Low (when time permits)

### Impact Assessment **[CONDITIONAL: Bug]**
- **Affected Users**: {Number/percentage of users affected}
- **Reproduction Steps**: 
  1. {Step 1}
  2. {Step 2}
  3. {Step 3}
- **Expected vs Actual**: {What should happen vs what happens}

### Business Justification **[CONDITIONAL: Feature]**
- **User Value**: {Why users need this}
- **Business Value**: {Impact on metrics/revenue}
- **Effort Estimate**: {Rough size - S/M/L/XL}

### Technical Debt Impact **[CONDITIONAL: Tech Debt]**
- **Current Problems**: {What's difficult/slow/buggy now}
- **Benefits of Fixing**: {What improves after refactoring}
- **Risk Assessment**: {Risks of not addressing this}

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Create simple test MCP server (Rust or script)
- [x] Test: connect, initialize, list tools, disconnect
- [x] Test: call tool and receive result
- [x] Test: error handling for failed tool calls
- [x] Test: server crash recovery
- [x] Test: multiple concurrent MCP servers
- [x] Test: HTTP transport (if implemented)
- [x] Tests run in CI pipeline

## Test Cases **[CONDITIONAL: Testing Task]**

{Delete unless this is a testing task}

### Test Case 1: {Test Case Name}
- **Test ID**: TC-001
- **Preconditions**: {What must be true before testing}
- **Steps**: 
  1. {Step 1}
  2. {Step 2}
  3. {Step 3}
- **Expected Results**: {What should happen}
- **Actual Results**: {To be filled during execution}
- **Status**: {Pass/Fail/Blocked}

### Test Case 2: {Test Case Name}
- **Test ID**: TC-002
- **Preconditions**: {What must be true before testing}
- **Steps**: 
  1. {Step 1}
  2. {Step 2}
- **Expected Results**: {What should happen}
- **Actual Results**: {To be filled during execution}
- **Status**: {Pass/Fail/Blocked}

## Documentation Sections **[CONDITIONAL: Documentation Task]**

{Delete unless this is a documentation task}

### User Guide Content
- **Feature Description**: {What this feature does and why it's useful}
- **Prerequisites**: {What users need before using this feature}
- **Step-by-Step Instructions**:
  1. {Step 1 with screenshots/examples}
  2. {Step 2 with screenshots/examples}
  3. {Step 3 with screenshots/examples}

### Troubleshooting Guide
- **Common Issue 1**: {Problem description and solution}
- **Common Issue 2**: {Problem description and solution}
- **Error Messages**: {List of error messages and what they mean}

### API Documentation **[CONDITIONAL: API Documentation]**
- **Endpoint**: {API endpoint description}
- **Parameters**: {Required and optional parameters}
- **Example Request**: {Code example}
- **Example Response**: {Expected response format}

## Implementation Notes **[CONDITIONAL: Technical Task]**

{Keep for technical tasks, delete for non-technical. Technical details, approach, or important considerations}

### Technical Approach
{How this will be implemented}

### Dependencies
{Other tasks or systems this depends on}

### Risk Considerations
{Technical risks and mitigation strategies}

## Status Updates **[REQUIRED]**

### 2026-02-07: Task Completed

**Implementation Summary:**

Enhanced the existing mock MCP server (`crates/arawn-mcp/tests/mock_server.rs`) with:
- Command line arguments: `--delay-ms N`, `--crash-on TOOL`, `--slow-tool T:MS`
- New tools: `slow` (configurable delay) and `crash` (exits process for testing)
- `ServerConfig` struct for parsing CLI args

Expanded integration tests (`crates/arawn-mcp/tests/integration.rs`) with 19 total tests:

**Basic Client Tests:**
- `test_connect_and_initialize` - Connect and handshake
- `test_list_tools` - List all 4 tools (echo, add, slow, crash)
- `test_call_echo_tool` - Echo tool invocation
- `test_call_add_tool` - Add tool with numeric args
- `test_call_unknown_tool` - Error handling for unknown tools
- `test_call_before_initialize_fails` - Enforce initialization
- `test_shutdown` - Clean shutdown

**Crash Recovery Tests:**
- `test_server_crash_detection` - Detect server crash via `--crash-on`
- `test_connection_closed_detection` - Detect connection loss after shutdown

**Multiple Server Tests:**
- `test_multiple_servers` - 3 concurrent servers via McpManager
- `test_manager_connect_and_disconnect_individual` - Individual server control
- `test_manager_remove_server` - Remove server from manager
- `test_manager_tool_count` - Count tools across servers
- `test_all_tools_flat` - Flat list of all tools with server names

**HTTP Transport Tests:**
- `test_http_transport_config` - Config builder with headers/timeout/retries
- `test_http_transport_creation` - Create HTTP transport
- `test_http_transport_invalid_url` - Reject invalid URLs
- `test_server_config_http_builder` - HTTP server config builder
- `test_client_connect_auto_selects_transport` - Auto-select stdio vs HTTP

**Code Changes:**
- Added `is_http()` and `is_stdio()` convenience methods to `McpClient`
- All 19 integration tests pass
- All 700+ workspace tests pass