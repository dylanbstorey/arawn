#!/bin/bash
# Arawn Conversational Test
# Simulates natural developer interactions with the system

set -e

SERVER="${ARAWN_SERVER:-http://localhost:8080}"
LOG_FILE="conversation-test-$(date +%Y%m%d-%H%M%S).log"

# Auth token (empty means no auth required, which is the default)
AUTH_TOKEN="${ARAWN_TOKEN:-}"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
DIM='\033[2m'
NC='\033[0m'

# Track state
SESSION_ID=""
WORKSTREAM_ID=""
ERROR_COUNT=0

# Build auth header if token is set
auth_header() {
    if [ -n "$AUTH_TOKEN" ]; then
        echo "-H" "Authorization: Bearer $AUTH_TOKEN"
    fi
}

log() {
    echo -e "${BLUE}►${NC} $1"
    echo ">>> $1" >> "$LOG_FILE"
}

# Make a curl request with error handling
# Usage: api_call METHOD URL [DATA]
api_call() {
    local method="$1"
    local url="$2"
    local data="$3"
    local auth_args=()

    if [ -n "$AUTH_TOKEN" ]; then
        auth_args=("-H" "Authorization: Bearer $AUTH_TOKEN")
    fi

    local curl_args=("-s" "-w" "\n__HTTP_STATUS__:%{http_code}")

    if [ "$method" != "GET" ]; then
        curl_args+=("-X" "$method")
    fi

    if [ -n "$data" ]; then
        curl_args+=("-H" "Content-Type: application/json" "-d" "$data")
    fi

    local raw_response
    raw_response=$(curl "${curl_args[@]}" "${auth_args[@]}" "$url" 2>&1)

    # Split response and status code
    local body="${raw_response%__HTTP_STATUS__:*}"
    local status="${raw_response##*__HTTP_STATUS__:}"

    # Log raw response
    echo "RAW RESPONSE ($method $url):" >> "$LOG_FILE"
    echo "Status: $status" >> "$LOG_FILE"
    echo "Body: $body" >> "$LOG_FILE"
    echo "---" >> "$LOG_FILE"

    # Check for errors
    if [[ ! "$status" =~ ^2 ]]; then
        echo -e "${RED}[HTTP $status]${NC}" >&2
        ((ERROR_COUNT++)) || true
    fi

    # Return just the body
    echo "$body"
}

chat() {
    local msg="$1"
    local session_arg=""

    echo -e "${GREEN}You:${NC} $msg"
    echo "USER: $msg" >> "$LOG_FILE"

    if [ -n "$SESSION_ID" ]; then
        session_arg="\"session_id\": \"$SESSION_ID\","
    fi

    local response
    response=$(api_call POST "$SERVER/api/v1/chat" "{$session_arg \"message\": \"$msg\"}")

    # Check if response is valid JSON
    if ! echo "$response" | jq -e . >/dev/null 2>&1; then
        echo -e "${RED}Invalid JSON response:${NC} ${response:0:200}"
        echo "INVALID RESPONSE: $response" >> "$LOG_FILE"
        ((ERROR_COUNT++)) || true
        return 1
    fi

    # Extract and store session_id
    SESSION_ID=$(echo "$response" | jq -r '.session_id // empty')

    # Check for error in response
    local error=$(echo "$response" | jq -r '.error // empty')
    if [ -n "$error" ]; then
        echo -e "${RED}Error:${NC} $error"
        echo "ERROR: $error" >> "$LOG_FILE"
        ((ERROR_COUNT++)) || true
        return 1
    fi

    # Display response
    local text=$(echo "$response" | jq -r '.response // "No response field"')
    local tokens=$(echo "$response" | jq -r '.usage.input_tokens // "?"')

    echo -e "${YELLOW}Assistant:${NC} ${text:0:500}"
    [ ${#text} -gt 500 ] && echo -e "${DIM}... (truncated)${NC}"
    echo -e "${DIM}[tokens: $tokens, session: ${SESSION_ID:0:8}...]${NC}"
    echo ""

    echo "ASSISTANT: $text" >> "$LOG_FILE"
    echo "SESSION: $SESSION_ID" >> "$LOG_FILE"
    echo "TOKENS: $tokens" >> "$LOG_FILE"

    sleep 1  # Be nice to the API
}

note() {
    local content="$1"
    echo -e "${BLUE}[Creating note]${NC}"

    local response
    response=$(api_call POST "$SERVER/api/v1/notes" "{\"content\": \"$content\"}")

    local note_id=$(echo "$response" | jq -r '.note.id // empty')
    if [ -n "$note_id" ]; then
        echo -e "${DIM}[Note created: ${note_id:0:8}...]${NC}"
    else
        echo -e "${RED}[Note creation may have failed]${NC}"
    fi
}

search_memory() {
    local query="$1"
    echo -e "${BLUE}[Searching: $query]${NC}"

    local encoded_query
    # Use printf to avoid trailing newline that echo adds
    encoded_query=$(printf '%s' "$query" | jq -sRr @uri)

    local response
    response=$(api_call GET "$SERVER/api/v1/memory/search?q=$encoded_query&limit=5")

    local count=$(echo "$response" | jq -r '.count // 0')
    local degraded=$(echo "$response" | jq -r '.degraded // false')

    if [ "$count" -gt 0 ]; then
        echo -e "${GREEN}Found $count results:${NC}"
        echo "$response" | jq -r '.results[]?.content // empty' | head -3 | while read -r line; do
            echo "  - ${line:0:80}..."
        done
    else
        echo -e "${DIM}No results found${NC}"
    fi

    [ "$degraded" = "true" ] && echo -e "${YELLOW}(search was degraded)${NC}"
}

new_session() {
    SESSION_ID=""
    echo -e "${DIM}[Starting fresh session]${NC}"
    echo "=== NEW SESSION ===" >> "$LOG_FILE"
}

# ============================================================================
echo "Arawn Conversational Test"
echo "========================="
echo -e "${DIM}Server: $SERVER | Log: $LOG_FILE${NC}"
[ -n "$AUTH_TOKEN" ] && echo -e "${DIM}Auth: token configured${NC}"
echo ""

# Initialize log
echo "Arawn Conversation Test - $(date)" > "$LOG_FILE"
echo "Server: $SERVER" >> "$LOG_FILE"
echo "" >> "$LOG_FILE"

# Quick health check
echo -e "${DIM}Checking server health...${NC}"
health_response=$(api_call GET "$SERVER/health")
if echo "$health_response" | jq -e '.status == "healthy"' >/dev/null 2>&1; then
    echo -e "${GREEN}Server is healthy${NC}"
else
    echo -e "${RED}Server health check failed - continuing anyway${NC}"
fi
echo ""

# ----------------------------------------------------------------------------
log "Scene 1: Getting oriented in a new codebase"
# ----------------------------------------------------------------------------

chat "Hey, I just cloned this repo. What kind of project is this?"

chat "What are the main components?"

# ----------------------------------------------------------------------------
log "Scene 2: Exploring specific functionality"
# ----------------------------------------------------------------------------

chat "I need to understand how sessions work here. Where should I look?"

chat "How does cleanup work? Want to make sure there's no memory leak."

# ----------------------------------------------------------------------------
log "Scene 3: Taking notes for later"
# ----------------------------------------------------------------------------

note "Session management: Uses LRU cache with configurable max_sessions. Cleanup runs on interval. Check session_cache.rs for implementation details."

chat "What patterns does the glob tool support? Just curious."

# ----------------------------------------------------------------------------
log "Scene 4: New session - different context"
# ----------------------------------------------------------------------------
new_session

chat "Working on a perf issue - file listing feels slow. What might cause that?"

chat "Any TODOs in the codebase mentioning performance?"

# ----------------------------------------------------------------------------
log "Scene 5: Workstream organization"
# ----------------------------------------------------------------------------

log "Creating a workstream for this investigation..."
ws_response=$(api_call POST "$SERVER/api/v1/workstreams" '{"title": "perf-investigation", "tags": ["debugging", "performance"]}')
WORKSTREAM_ID=$(echo "$ws_response" | jq -r '.id // empty')

if [ -n "$WORKSTREAM_ID" ]; then
    echo -e "${DIM}[Workstream created: ${WORKSTREAM_ID:0:8}...]${NC}"

    # Send a message to the workstream
    api_call POST "$SERVER/api/v1/workstreams/$WORKSTREAM_ID/messages" \
        '{"content": "Investigating slow file listing. Suspect glob tool depth issue."}' > /dev/null
fi

# ----------------------------------------------------------------------------
log "Scene 6: Memory recall"
# ----------------------------------------------------------------------------

echo -e "${BLUE}Can we find our earlier notes?${NC}"
search_memory "session"

echo ""
search_memory "LRU cache"

# ----------------------------------------------------------------------------
log "Scene 7: Continued conversation with context"
# ----------------------------------------------------------------------------
new_session

chat "Found the issue - glob was walking too deep. Fixed it to calculate depth from the pattern."

chat "What config options exist for tuning session behavior?"

# ----------------------------------------------------------------------------
log "Scene 8: Wrapping up"
# ----------------------------------------------------------------------------

chat "Quick summary of what we discussed today?"

# ----------------------------------------------------------------------------
# Cleanup & Summary
# ----------------------------------------------------------------------------
echo ""
echo "========================="
echo "Test complete!"
echo -e "${DIM}Log: $LOG_FILE${NC}"

# Stats
MESSAGES=$(grep -c "^USER:" "$LOG_FILE" 2>/dev/null || echo 0)
SESSIONS=$(grep -c "=== NEW SESSION ===" "$LOG_FILE" 2>/dev/null || echo 0)
echo ""
echo "Stats:"
echo "  Messages sent: $MESSAGES"
echo "  Sessions used: $((SESSIONS + 1))"
[ -n "$WORKSTREAM_ID" ] && echo "  Workstream: $WORKSTREAM_ID"

if [ "$ERROR_COUNT" -gt 0 ]; then
    echo -e "  ${RED}Errors: $ERROR_COUNT (check log for details)${NC}"
else
    echo -e "  ${GREEN}Errors: 0${NC}"
fi

# Token usage summary
echo ""
echo "Token usage by message:"
grep "^TOKENS:" "$LOG_FILE" | awk -F': ' '{print "  " NR ": " $2 " tokens"}' | head -10
