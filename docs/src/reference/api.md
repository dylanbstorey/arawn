# REST API Reference

HTTP endpoints for interacting with Arawn.

## Base URL

```
http://localhost:8080
```

## Authentication

When authentication is enabled:

```bash
curl -H "Authorization: Bearer YOUR_TOKEN" http://localhost:8080/api/v1/chat
```

## Endpoints

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

### Chat

#### Synchronous Chat

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

#### Streaming Chat

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

### Sessions

#### List Sessions

```
GET /api/v1/sessions
```

**Response:**
```json
{
  "sessions": [
    {
      "id": "abc123",
      "created_at": "2024-01-15T10:00:00Z",
      "message_count": 5
    }
  ]
}
```

#### Get Session

```
GET /api/v1/sessions/{id}
```

**Response:**
```json
{
  "id": "abc123",
  "created_at": "2024-01-15T10:00:00Z",
  "messages": [
    {"role": "user", "content": "Hello"},
    {"role": "assistant", "content": "Hi there!"}
  ]
}
```

#### Close Session

```
DELETE /api/v1/sessions/{id}
```

**Response:** `204 No Content`

Triggers background indexing.

### Memory

#### Search Memory

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

#### Get Memory Stats

```
GET /api/v1/memory/stats
```

**Response:**
```json
{
  "total_memories": 142,
  "total_entities": 47,
  "total_relationships": 89,
  "last_indexed": "2024-01-15T10:00:00Z"
}
```

### Notes

#### List Notes

```
GET /api/v1/notes?session_id=abc123
```

**Response:**
```json
{
  "notes": [
    {
      "id": "note1",
      "title": "TODO",
      "content": "- Fix auth bug",
      "session_id": "abc123"
    }
  ]
}
```

#### Create Note

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

### Workstreams

#### List Workstreams

```
GET /api/v1/workstreams
```

**Response:**
```json
{
  "workstreams": [
    {
      "id": "ws_abc123",
      "name": "Research Project",
      "message_count": 47,
      "updated_at": "2024-01-15T10:00:00Z"
    }
  ]
}
```

#### Create Workstream

```
POST /api/v1/workstreams
```

**Request:**
```json
{
  "name": "New Project"
}
```

#### Get Workstream

```
GET /api/v1/workstreams/{id}
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
