# Workstreams

Persistent conversation contexts that span sessions.

## Overview

Workstreams provide:
- **Long-running context** — Conversations that persist across sessions
- **Message history** — JSONL-based storage of all messages
- **Metadata caching** — SQLite for fast lookups
- **Context continuity** — Resume conversations where you left off

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│ Workstream Manager                                               │
│                                                                  │
│  ┌──────────────────┐  ┌──────────────────────────────────────┐ │
│  │ SQLite Cache     │  │ JSONL Storage                        │ │
│  │                  │  │                                      │ │
│  │ • Workstream IDs │  │ ~/.arawn/data/workstreams/          │ │
│  │ • Metadata       │  │ ├── ws_abc123.jsonl                 │ │
│  │ • Last accessed  │  │ ├── ws_def456.jsonl                 │ │
│  │ • Message counts │  │ └── ws_ghi789.jsonl                 │ │
│  └──────────────────┘  └──────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

## Workstream Lifecycle

### Creation

```bash
# Create a new workstream
arawn workstream create "Research Project"

# Or implicitly by starting a workstream session
arawn chat --workstream "Research Project"
```

### Usage

```bash
# Continue an existing workstream
arawn chat --workstream ws_abc123

# Or by name
arawn chat --workstream "Research Project"
```

### Listing

```bash
arawn workstream list
```

Output:
```
ID          NAME                  MESSAGES  LAST USED
ws_abc123   Research Project      47        2 hours ago
ws_def456   Code Review           12        yesterday
ws_ghi789   Architecture Design   89        3 days ago
```

## Data Structure

### Workstream Record

```rust
pub struct Workstream {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub message_count: usize,
    pub metadata: HashMap<String, String>,
}
```

### Message Storage (JSONL)

Each workstream has a JSONL file with messages:

```jsonl
{"role":"user","content":"Let's start the research project","timestamp":"2024-01-15T10:00:00Z"}
{"role":"assistant","content":"I'll help you with the research...","timestamp":"2024-01-15T10:00:05Z"}
{"role":"user","content":"Focus on Rust async patterns","timestamp":"2024-01-15T10:01:00Z"}
```

## Integration with Sessions

When a workstream session starts:

1. Load workstream messages into session history
2. Apply context window limits (keep recent + important)
3. Session proceeds normally with full history
4. On close, new messages appended to JSONL

### Context Window Management

Large workstreams are summarized to fit context:

```
Total messages: 500
Context window: 100k tokens

Strategy:
1. Keep last 50 messages verbatim
2. Summarize earlier messages
3. Include all tool results from recent turns
```

## API Access

### REST API

```bash
# List workstreams
curl http://localhost:8080/api/v1/workstreams

# Get workstream details
curl http://localhost:8080/api/v1/workstreams/ws_abc123

# Create workstream
curl -X POST http://localhost:8080/api/v1/workstreams \
  -H "Content-Type: application/json" \
  -d '{"name": "New Project"}'

# Chat in workstream context
curl -X POST http://localhost:8080/api/v1/chat \
  -H "Content-Type: application/json" \
  -d '{"message": "Continue where we left off", "workstream_id": "ws_abc123"}'
```

### WebSocket

```javascript
const ws = new WebSocket('ws://localhost:8080/ws?workstream=ws_abc123');
```

## Use Cases

### Long-Running Projects

Track a multi-week project with persistent context:

```bash
arawn chat --workstream "Q4 Architecture Redesign"
# ... weeks later ...
arawn chat --workstream "Q4 Architecture Redesign"
# Agent remembers previous discussions
```

### Topic Separation

Keep different topics in separate workstreams:

```bash
arawn chat --workstream "Frontend Work"
arawn chat --workstream "Backend Work"
arawn chat --workstream "DevOps Tasks"
```

### Collaborative Context

Share workstream IDs for team collaboration:

```bash
# Person A creates
arawn workstream create "Shared Research" --id shared_research_001

# Person B continues
arawn chat --workstream shared_research_001
```

## Configuration

```toml
[workstreams]
enabled = true
storage_path = "~/.arawn/data/workstreams"
max_history = 1000          # Max messages before summarization
context_window = 50         # Recent messages to keep verbatim
```
