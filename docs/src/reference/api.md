# REST API Reference

HTTP endpoints for interacting with Arawn.

## Base URL

```
http://localhost:8080
```

## Authentication

When authentication is enabled, include a bearer token:

```bash
curl -H "Authorization: Bearer YOUR_TOKEN" http://localhost:8080/api/v1/chat
```

All `/api/v1/*` endpoints require authentication. Health and OpenAPI endpoints do not.

## Pagination

All list endpoints support pagination via query parameters:

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `limit` | integer | 50 | Maximum items to return |
| `offset` | integer | 0 | Number of items to skip |

## Health

### Health Check

```
GET /health
```

**Response:**
```json
{
  "status": "ok",
  "version": "0.1.0"
}
```

### Detailed Health

```
GET /health/detailed
```

Returns extended status including component health.

## OpenAPI Documentation

```
GET /swagger-ui    # Interactive Swagger UI
GET /swagger.json  # OpenAPI spec (JSON)
```

## Chat

### Synchronous Chat

```
POST /api/v1/chat
```

**Request:**
```json
{
  "message": "Hello, how are you?",
  "session_id": "optional-session-id",
  "workstream_id": "optional-workstream-id"
}
```

**Response:**
```json
{
  "session_id": "abc123",
  "response": "I'm doing well! How can I help you today?",
  "tool_calls": [],
  "usage": {
    "input_tokens": 42,
    "output_tokens": 18
  }
}
```

### Streaming Chat

```
POST /api/v1/chat/stream
```

**Request:** Same as synchronous.

**Response:** Server-Sent Events (SSE)

```
event: text
data: {"text": "I'm "}

event: text
data: {"text": "doing well!"}

event: tool_start
data: {"id": "t1", "name": "shell"}

event: tool_end
data: {"id": "t1", "result": "..."}

event: done
data: {"usage": {"input_tokens": 42, "output_tokens": 18}}
```

## Sessions

### Create Session

```
POST /api/v1/sessions
```

### List Sessions

```
GET /api/v1/sessions?limit=50&offset=0
```

### Get Session

```
GET /api/v1/sessions/{id}
```

### Update Session

```
PATCH /api/v1/sessions/{id}
```

### Delete Session

```
DELETE /api/v1/sessions/{id}
```

**Response:** `204 No Content`

Triggers background indexing.

### Get Session Messages

```
GET /api/v1/sessions/{id}/messages
```

## Memory

### Store Memory

```
POST /api/v1/memory
```

### Search Memory

```
GET /api/v1/memory/search?q=rust+project&limit=5
```

**Response:**
```json
{
  "results": [
    {
      "id": "mem123",
      "content": "Arawn is written in Rust",
      "confidence": 0.95,
      "source": "stated",
      "created_at": "2024-01-10T08:00:00Z"
    }
  ]
}
```

### Delete Memory Entry

```
DELETE /api/v1/memory/{id}
```

## Notes

### Create Note

```
POST /api/v1/notes
```

**Request:**
```json
{
  "title": "Research Notes",
  "content": "Key findings...",
  "session_id": "abc123"
}
```

**Response:** `201 Created` with `Note` object.

### List Notes

```
GET /api/v1/notes?limit=50&offset=0
```

### Get Note

```
GET /api/v1/notes/{id}
```

### Update Note

```
PUT /api/v1/notes/{id}
```

### Delete Note

```
DELETE /api/v1/notes/{id}
```

## Workstreams

### Create Workstream

```
POST /api/v1/workstreams
```

**Request:**
```json
{
  "name": "New Project"
}
```

### List Workstreams

```
GET /api/v1/workstreams?limit=50&offset=0
```

### Get Workstream

```
GET /api/v1/workstreams/{id}
```

### Update Workstream

```
PATCH /api/v1/workstreams/{id}
```

### Delete Workstream

```
DELETE /api/v1/workstreams/{id}
```

### Send Message

```
POST /api/v1/workstreams/{id}/messages
```

### List Messages

```
GET /api/v1/workstreams/{id}/messages
```

### List Workstream Sessions

```
GET /api/v1/workstreams/{id}/sessions
```

### Promote Content

```
POST /api/v1/workstreams/{id}/promote
```

### Promote File

```
POST /api/v1/workstreams/{id}/files/promote
```

### Export File

```
POST /api/v1/workstreams/{id}/files/export
```

### Clone Repository

```
POST /api/v1/workstreams/{id}/clone
```

### Get Usage

```
GET /api/v1/workstreams/{id}/usage
```

### Cleanup

```
POST /api/v1/workstreams/{id}/cleanup
```

## Agents

### List Agents

```
GET /api/v1/agents
```

### Get Agent

```
GET /api/v1/agents/{id}
```

## Tasks

### List Tasks

```
GET /api/v1/tasks
```

### Get Task

```
GET /api/v1/tasks/{id}
```

### Cancel Task

```
DELETE /api/v1/tasks/{id}
```

## MCP Servers

### Add Server

```
POST /api/v1/mcp/servers
```

### List Servers

```
GET /api/v1/mcp/servers
```

### Remove Server

```
DELETE /api/v1/mcp/servers/{name}
```

### List Server Tools

```
GET /api/v1/mcp/servers/{name}/tools
```

### Connect Server

```
POST /api/v1/mcp/servers/{name}/connect
```

### Disconnect Server

```
POST /api/v1/mcp/servers/{name}/disconnect
```

## Commands

### List Commands

```
GET /api/v1/commands
```

### Compact

```
POST /api/v1/commands/compact
```

### Compact (Streaming)

```
POST /api/v1/commands/compact/stream
```

## Config

### Get Configuration

```
GET /api/v1/config
```

## WebSocket

### Connection

```
ws://localhost:8080/ws
```

With workstream:
```
ws://localhost:8080/ws?workstream=ws_abc123
```

Authentication happens via the first message, not HTTP headers.

### Message Format

**Client → Server:**
```json
{
  "type": "message",
  "content": "Hello"
}
```

**Server → Client:**
```json
{
  "type": "text",
  "content": "Hi there!"
}

{
  "type": "tool_start",
  "id": "t1",
  "name": "shell"
}

{
  "type": "tool_end",
  "id": "t1",
  "result": "..."
}

{
  "type": "done",
  "usage": {...}
}
```

## Error Responses

```json
{
  "error": {
    "code": "invalid_request",
    "message": "Session not found",
    "details": {}
  }
}
```

### Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `invalid_request` | 400 | Malformed request |
| `unauthorized` | 401 | Missing/invalid auth |
| `not_found` | 404 | Resource not found |
| `rate_limited` | 429 | Too many requests |
| `internal_error` | 500 | Server error |

## Rate Limiting

When rate limited:

```
HTTP/1.1 429 Too Many Requests
Retry-After: 60
X-RateLimit-Limit: 60
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1705323600
```
