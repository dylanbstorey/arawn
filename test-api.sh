#!/bin/bash
# Arawn API Test Suite
# Generates a log of all API responses for analysis

set -e

SERVER="http://localhost:8080"
LOG_FILE="api-test-$(date +%Y%m%d-%H%M%S).log"

# Colors for terminal output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log() {
    echo -e "${GREEN}[TEST]${NC} $1"
    echo "=== $1 ===" >> "$LOG_FILE"
}

run_test() {
    local name="$1"
    shift
    log "$name"
    echo "Command: curl $@" >> "$LOG_FILE"
    echo "Response:" >> "$LOG_FILE"
    curl -s -w "\nHTTP_STATUS: %{http_code}\n" "$@" >> "$LOG_FILE" 2>&1
    echo "" >> "$LOG_FILE"
    echo "---" >> "$LOG_FILE"
    echo "" >> "$LOG_FILE"
}

echo "Arawn API Test Suite"
echo "===================="
echo "Server: $SERVER"
echo "Log file: $LOG_FILE"
echo ""

# Initialize log
echo "Arawn API Test Log - $(date)" > "$LOG_FILE"
echo "Server: $SERVER" >> "$LOG_FILE"
echo "" >> "$LOG_FILE"

# ============================================================================
# Phase 1: Health & Connectivity
# ============================================================================
echo -e "${YELLOW}Phase 1: Health & Connectivity${NC}"

run_test "1.1 Health Check" \
    "$SERVER/health"

run_test "1.2 Chat (basic - no session)" \
    -X POST "$SERVER/api/v1/chat" \
    -H "Content-Type: application/json" \
    -d '{"message": "What is 2+2? Reply with just the number."}'

# ============================================================================
# Phase 2: Chat & Sessions
# ============================================================================
echo -e "${YELLOW}Phase 2: Chat & Sessions${NC}"

# Store session ID from first chat
run_test "2.1 Chat - create session" \
    -X POST "$SERVER/api/v1/chat" \
    -H "Content-Type: application/json" \
    -d '{"message": "Remember that my favorite color is blue. Just acknowledge."}'

# Extract session_id from log for next test
SESSION_ID=$(grep -o '"session_id":"[^"]*"' "$LOG_FILE" | tail -1 | cut -d'"' -f4)
echo "Extracted session_id: $SESSION_ID" >> "$LOG_FILE"

if [ -n "$SESSION_ID" ]; then
    run_test "2.2 Chat - continue session" \
        -X POST "$SERVER/api/v1/chat" \
        -H "Content-Type: application/json" \
        -d "{\"message\": \"What is my favorite color?\", \"session_id\": \"$SESSION_ID\"}"
fi

run_test "2.3 Chat - trigger tool use" \
    -X POST "$SERVER/api/v1/chat" \
    -H "Content-Type: application/json" \
    -d '{"message": "Use the glob tool with pattern \"*.toml\" to find toml files in the current directory (not recursive)."}'

run_test "2.4 List sessions" \
    "$SERVER/api/v1/sessions"

if [ -n "$SESSION_ID" ]; then
    run_test "2.5 Get specific session" \
        "$SERVER/api/v1/sessions/$SESSION_ID"
fi

# Test streaming endpoint (just first few bytes to confirm it works)
run_test "2.6 Chat stream (SSE)" \
    -X POST "$SERVER/api/v1/chat/stream" \
    -H "Content-Type: application/json" \
    -H "Accept: text/event-stream" \
    -d '{"message": "Say hello"}' \
    --max-time 5 || true

# ============================================================================
# Phase 3: Notes & Memory
# ============================================================================
echo -e "${YELLOW}Phase 3: Notes & Memory${NC}"

run_test "3.1 Create note" \
    -X POST "$SERVER/api/v1/notes" \
    -H "Content-Type: application/json" \
    -d '{"content": "This is a test note about purple elephants created by the API test suite."}'

run_test "3.2 List notes" \
    "$SERVER/api/v1/notes"

# Delete session to trigger indexing (memories extracted from session)
if [ -n "$SESSION_ID" ]; then
    run_test "3.3 Delete session (triggers memory indexing)" \
        -X DELETE "$SERVER/api/v1/sessions/$SESSION_ID"

    # Brief pause for async indexing
    echo "Waiting for background indexing..." >> "$LOG_FILE"
    sleep 2
fi

# Memory search - should find the note content AND potentially indexed session facts
run_test "3.4 Memory search (note content)" \
    "$SERVER/api/v1/memory/search?q=purple%20elephants&limit=5"

run_test "3.5 Memory search (session content)" \
    "$SERVER/api/v1/memory/search?q=favorite%20color&limit=5"

# ============================================================================
# Phase 5: MCP Server Management
# ============================================================================
echo -e "${YELLOW}Phase 5: MCP Server Management${NC}"

# Use the mock MCP server for functional testing
MOCK_MCP_SERVER="./target/debug/mock-mcp-server"
if [ ! -f "$MOCK_MCP_SERVER" ]; then
    echo "Building mock-mcp-server..." >> "$LOG_FILE"
    cargo build -p arawn-mcp --bin mock-mcp-server 2>/dev/null
fi

run_test "5.1 List MCP servers (empty)" \
    "$SERVER/api/v1/mcp/servers"

run_test "5.2 Add MCP server (mock)" \
    -X POST "$SERVER/api/v1/mcp/servers" \
    -H "Content-Type: application/json" \
    -d "{\"name\": \"test-server\", \"command\": \"$MOCK_MCP_SERVER\", \"args\": []}"

run_test "5.3 List MCP servers (after add)" \
    "$SERVER/api/v1/mcp/servers"

run_test "5.4 Connect to MCP server" \
    -X POST "$SERVER/api/v1/mcp/servers/test-server/connect"

run_test "5.5 List MCP server tools" \
    "$SERVER/api/v1/mcp/servers/test-server/tools"

run_test "5.6 Disconnect MCP server" \
    -X POST "$SERVER/api/v1/mcp/servers/test-server/disconnect"

run_test "5.7 Remove MCP server" \
    -X DELETE "$SERVER/api/v1/mcp/servers/test-server"

run_test "5.8 List MCP servers (after remove)" \
    "$SERVER/api/v1/mcp/servers"

# ============================================================================
# Phase 6: Workstreams
# ============================================================================
echo -e "${YELLOW}Phase 6: Workstreams${NC}"

run_test "6.1 Create workstream" \
    -X POST "$SERVER/api/v1/workstreams" \
    -H "Content-Type: application/json" \
    -d '{"title": "test-workstream", "tags": ["test"]}'

run_test "6.2 List workstreams" \
    "$SERVER/api/v1/workstreams"

# Extract workstream ID
WORKSTREAM_ID=$(grep -o '"id":"[^"]*"' "$LOG_FILE" | tail -1 | cut -d'"' -f4)
echo "Extracted workstream_id: $WORKSTREAM_ID" >> "$LOG_FILE"

if [ -n "$WORKSTREAM_ID" ]; then
    run_test "6.3 Get workstream" \
        "$SERVER/api/v1/workstreams/$WORKSTREAM_ID"

    run_test "6.4 Send message to workstream" \
        -X POST "$SERVER/api/v1/workstreams/$WORKSTREAM_ID/messages" \
        -H "Content-Type: application/json" \
        -d '{"content": "Hello workstream!"}'

    run_test "6.5 List workstream messages" \
        "$SERVER/api/v1/workstreams/$WORKSTREAM_ID/messages"

    run_test "6.6 Delete workstream" \
        -X DELETE "$SERVER/api/v1/workstreams/$WORKSTREAM_ID"
fi

# ============================================================================
# Phase 7: Verify cleanup
# ============================================================================
echo -e "${YELLOW}Phase 7: Verify Cleanup${NC}"

# Session was already deleted in Phase 3 to trigger indexing
run_test "7.1 Verify sessions list (should not include deleted session)" \
    "$SERVER/api/v1/sessions"

# ============================================================================
# Summary
# ============================================================================
echo ""
echo "===================="
echo "Test suite complete!"
echo "Log file: $LOG_FILE"
echo ""

# Count results
TOTAL=$(grep -c "^=== " "$LOG_FILE" || echo 0)
SUCCESS=$(grep -c "HTTP_STATUS: 200\|HTTP_STATUS: 201\|HTTP_STATUS: 204" "$LOG_FILE" || echo 0)
ERRORS=$(grep -c "HTTP_STATUS: [45]" "$LOG_FILE" || echo 0)

echo "Summary:" | tee -a "$LOG_FILE"
echo "  Total tests: $TOTAL" | tee -a "$LOG_FILE"
echo "  Success (2xx): $SUCCESS" | tee -a "$LOG_FILE"
echo "  Errors (4xx/5xx): $ERRORS" | tee -a "$LOG_FILE"
