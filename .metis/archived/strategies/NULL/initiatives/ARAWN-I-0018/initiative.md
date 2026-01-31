---
id: workstreams-persistent
level: initiative
title: "Workstreams: Persistent Conversational Contexts"
short_code: "ARAWN-I-0018"
created_at: 2026-01-29T03:21:23.502333+00:00
updated_at: 2026-01-29T04:23:12.199781+00:00
parent: ARAWN-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: XL
strategy_id: NULL
initiative_id: workstreams-persistent
---

# Workstreams: Persistent Conversational Contexts Initiative

## Context

Arawn's current `Session` is an in-memory, ephemeral conversation that dies with the server process. There's no persistence, no way to resume, no way to have multiple concurrent conversations, and no identity for a conversation that the agent or external systems can reference.

A **workstream** is a persistent, long-lived unit of work — closer to a forum thread than a chat session. Workstreams:
- Survive server restarts (persisted to disk)
- Can be resumed after hours or days
- Have their own message history, context, and metadata
- Can have attached processes (e.g. Cloacina scheduled workflows reporting back)
- Support agent-initiated messages (the agent pushes updates without a user request pending)

This is foundational infrastructure. Routing decisions, memory scoping, tool context, and multi-channel support (including SMS where there's no thread picker UI) all depend on workstreams.

## Goals & Non-Goals

**Goals:**
- Persistent, named workstream with ID, title, summary, and metadata
- Default "scratch" workstream for messages without an explicit workstream ID
- Agent-prompted promotion: scratch conversations that develop substance get promoted to named workstreams
- JSONL message history per workstream (source of truth, append-only)
- SQLite operational layer for workstream metadata indexing, state queries, and caching
- Workstream-level default model config (selects from configured `llm_profiles`)
- Resume a workstream by ID via API
- Agent can push messages into a workstream asynchronously
- Map-reduce context compression: session summaries → workstream summary, with memory extraction
- Session = turn batch within a workstream (many sessions per workstream lifetime)
- Workstream lifecycle: active, paused, archived
- New `arawn-workstream` crate

**Non-Goals (deferred):**
- Automatic inbound message classification to workstreams (needed for SMS, but deferred — requires embedding similarity)
- Cross-workstream knowledge sharing (memory scoping is I-0014/I-0017)
- RAG + knowledge subgraph per workstream (future extension once memory initiatives land)
- Cloacina workflow attachment (future, depends on I-0012 tool ecosystem)
- Multi-user workstreams / permissions

## Detailed Design

### Workstream Data Model

```rust
pub struct Workstream {
    pub id: WorkstreamId,          // UUID
    pub title: String,             // user-assigned or auto-generated
    pub summary: String,           // compressed objective + current state
    pub is_scratch: bool,          // true for the default scratch workstream
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub state: WorkstreamState,    // Active, Paused, Archived
    pub metadata: WorkstreamMetadata,
}

pub struct WorkstreamMetadata {
    pub default_model: Option<String>,  // profile name from llm_profiles
    pub tags: Vec<String>,
}

pub enum WorkstreamState {
    Active,
    Paused,
    Archived,
}
```

### Session Data Model

A session is a turn batch — a period of activity within a workstream. Many sessions per workstream lifetime.

```rust
pub struct WorkstreamSession {
    pub id: SessionId,             // UUID
    pub workstream_id: WorkstreamId,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub turn_count: u32,
    pub summary: Option<String>,   // generated on session end (map step)
}
```

Sessions are the unit of compression. When a session ends, it gets summarized. The workstream summary is the reduction of all session summaries.

### Storage: Dual Layer

**JSONL — source of truth** (append-only, never mutated):
```
~/.config/arawn/workstreams/
  {workstream-id}/
    messages.jsonl           # Full message history
    sessions.jsonl           # Session open/close/summary records
```

**SQLite — operational layer** (derived, queryable, rebuildable from JSONL):
```sql
-- Workstream metadata and state
CREATE TABLE workstreams (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    summary TEXT,
    is_scratch BOOLEAN DEFAULT FALSE,
    state TEXT DEFAULT 'active',
    default_model TEXT,
    created_at TEXT,
    updated_at TEXT
);

-- Session index
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    workstream_id TEXT REFERENCES workstreams(id),
    started_at TEXT,
    ended_at TEXT,
    turn_count INTEGER,
    summary TEXT
);

-- For future: message search, token counts, etc.
```

SQLite is a cache. If it's lost, rebuild from JSONL. This keeps the operational layer fast (list workstreams, query by state, search summaries) without making JSONL less portable.

### Scratch Workstream and Promotion

A default **scratch** workstream always exists. Messages without a workstream ID route there.

**Promotion flow:**
1. User sends messages to scratch without specifying a workstream
2. After the conversation develops substance (a few turns, a clear topic), the agent recognizes this
3. Agent asks: "This conversation about X is developing. Want me to create a workstream for it?"
4. User says yes → messages migrate from scratch to a new named workstream
5. Scratch resets (or continues accumulating unrelated messages)

Promotion is agent-driven, not algorithmic. The agent uses its judgment based on conversation depth and topic coherence. No embedding classifier needed for v1.

### Message History

Each message in `messages.jsonl`:

```rust
pub struct WorkstreamMessage {
    pub id: String,
    pub session_id: SessionId,
    pub timestamp: DateTime<Utc>,
    pub role: MessageRole,         // User, Assistant, System, AgentPush
    pub content: String,
    pub interaction_id: Option<String>,  // links to InteractionRecord (I-0011)
    pub metadata: HashMap<String, Value>,
}

pub enum MessageRole {
    User,
    Assistant,
    System,
    AgentPush,  // agent-initiated, no user request pending
}
```

### Context Compression: Map-Reduce Model

Compression operates at two levels with three triggers.

**Triggers:**
1. **Session end** — when a session closes (user disconnects, explicit end, timeout), compress that session
2. **Token threshold** — if context exceeds the window mid-session, compress earlier turns within the current session
3. **Startup / manual** — on server start, check for uncompressed sessions; manual trigger via API

**Map step (session → session summary):**
- Input: all turns in a completed session
- Output: session summary focused on what was discussed, decisions made, current state
- Uses configured LLM backend (batch API where available for cost savings)
- Extracts facts/entities for memory system (I-0014/I-0017) as a side effect

**Reduce step (session summaries → workstream summary):**
- Input: all session summaries + previous workstream summary
- Output: updated workstream summary: what is this workstream about, what's been accomplished, where are we now
- Runs after each session compression

**Context assembly for a turn:**
```
[System prompt]
[Workstream summary — compressed history of all prior sessions]
[Current session messages — recent turns, verbatim]
[User's new message]
```

If the current session itself gets too large, compress its earlier turns into a session-level summary and keep only the recent "hot" window verbatim.

### API Surface

```
POST   /api/v1/workstreams                    # Create workstream
GET    /api/v1/workstreams                    # List workstreams (query by state)
GET    /api/v1/workstreams/:id                # Get workstream details + summary
PATCH  /api/v1/workstreams/:id                # Update metadata (title, model, state)
DELETE /api/v1/workstreams/:id                # Archive workstream

POST   /api/v1/workstreams/:id/messages       # Send message (starts/continues session)
GET    /api/v1/workstreams/:id/messages        # Get message history (paginated)
POST   /api/v1/workstreams/:id/compress        # Trigger manual compression
POST   /api/v1/workstreams/:id/promote         # Promote scratch messages to this workstream

# Scratch shortcut
POST   /api/v1/chat                           # Send to scratch workstream (backwards compat)
```

### Session Lifecycle

1. **Start**: first message to a workstream after no active session (or explicit session start)
2. **Active**: turns accumulate, messages appended to JSONL
3. **End**: triggered by timeout (no message for N minutes), explicit close, or server shutdown
4. **Compression**: session summary generated (map step), workstream summary updated (reduce step), facts extracted to memory

### Agent-Initiated Messages

The agent can push messages into a workstream without a user request:

```rust
impl WorkstreamManager {
    pub async fn push_agent_message(
        &self,
        workstream_id: &WorkstreamId,
        content: &str,
    ) -> Result<()>;
}
```

This enables:
- Long-running background tasks reporting progress
- Scheduled workflows (Cloacina) posting results
- Proactive agent notifications ("that build finished")

### Crate Structure

New `arawn-workstream` crate with:
- `workstream.rs` — Workstream, WorkstreamState, WorkstreamMetadata
- `session.rs` — WorkstreamSession lifecycle
- `message.rs` — WorkstreamMessage, MessageRole, JSONL read/write
- `store.rs` — SQLite operational layer + JSONL persistence
- `compression.rs` — Map-reduce compression pipeline
- `manager.rs` — WorkstreamManager (high-level API used by server/agent)

Dependencies: `arawn-llm` (for compression LLM calls), `arawn-config`, `rusqlite`, `chrono`, `uuid`, `serde`, `serde_json`

## Alternatives Considered

- **Extend Session with persistence**: Simpler but Session is tightly coupled to the turn loop. Workstreams are a higher-level concept that sessions serve, not replace. Session becomes a turn batch within a workstream.
- **SQLite only**: More queryable but loses the portability and greppability of JSONL for training pipelines. Hybrid approach: JSONL as source of truth, SQLite as operational cache.
- **Filesystem only (no SQLite)**: Works for small workstream counts but listing/querying degrades as workstreams grow. SQLite handles metadata indexing efficiently.
- **Thread naming**: Considered "thread" but workstream better conveys persistent, work-oriented nature vs. casual chat threads.
- **Per-request routing**: Originally planned (I-0011) but routing decisions should bind to workstreams, not individual requests. Workstream-level model config is simpler and more predictable.
- **Algorithmic scratch promotion**: Could auto-detect when scratch needs promotion, but agent-driven judgment is more natural and avoids a hard classification problem.

## Implementation Plan

1. Workstream data model, types, and `arawn-workstream` crate scaffold
2. JSONL message store: append, read, pagination
3. SQLite operational layer: workstream CRUD, session index
4. Scratch workstream: auto-creation, message routing, promotion to named workstream
5. Session lifecycle: start, active, end, timeout detection
6. Session hydration: load workstream summary + current session messages into agent context
7. Server API: workstream management endpoints, replace `/chat` with workstream-scoped messaging
8. Agent-initiated message push
9. Context compression: session summary (map), workstream summary (reduce)
10. Workstream-level model selection (lookup backend from `llm_profiles`)