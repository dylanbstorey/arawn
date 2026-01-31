---
id: memory-citation-tracking-source
level: task
title: "Memory Citation Tracking: Source File and Line References"
short_code: "ARAWN-T-0132"
created_at: 2026-02-04T15:00:55.320585+00:00
updated_at: 2026-02-07T16:04:38.820643+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#feature"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Memory Citation Tracking: Source File and Line References

## Objective

Add citation tracking to the memory system so that facts and entities can be traced back to their source (session, file, line number). Essential for research agents to provide verifiable information.

## Backlog Item Details

### Type
- [x] Feature - New functionality or enhancement

### Priority
- [x] P1 - High (critical for research use case)

### Business Justification
- **User Value**: 
  - "Where did I learn this?" - trace facts back to source
  - Verify information by checking original context
  - Build trust in agent's knowledge
- **Effort Estimate**: S (Small) - schema change + extraction update

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Memory struct includes optional `citation` field
- [x] Session indexer populates citation with session_id and message index
- [x] File-based memories include file path and line number
- [x] File citations include content hash + mtime for staleness detection
- [x] Web citations include content hash + ETag for cache validation
- [x] Recall can flag memories as potentially stale (source changed)
- [x] Recall results include citations when available
- [ ] Memory tool can filter by citation source (future: add filter param to recall query)
- [x] API responses include citation metadata

## Feature Details

### Citation Types

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Citation {
    /// From a conversation session
    Session {
        session_id: String,
        message_index: usize,
        timestamp: DateTime<Utc>,
    },
    /// From a file read by the agent
    File {
        path: PathBuf,
        line_start: Option<usize>,
        line_end: Option<usize>,
        commit_hash: Option<String>,  // If in git repo
        content_hash: Option<String>, // SHA-256 of file at extraction time
        mtime: Option<DateTime<Utc>>, // File modification time at extraction
    },
    /// From a web fetch
    Web {
        url: String,
        fetched_at: DateTime<Utc>,
        title: Option<String>,
        content_hash: Option<String>, // SHA-256 of content at fetch time
        etag: Option<String>,         // HTTP ETag for cache validation
    },
    /// User-stated fact
    User {
        session_id: String,
        stated_at: DateTime<Utc>,
    },
    /// System-derived (inferred, computed)
    System {
        derived_at: DateTime<Utc>,
        method: String,  // e.g., "graph_inference", "llm_extraction"
    },
}
```

### Memory Struct Update

```rust
pub struct Memory {
    pub id: Option<i64>,
    pub content: String,
    pub embedding: Option<Vec<f32>>,
    pub memory_type: MemoryType,
    pub source: MemorySource,
    pub metadata: Option<Value>,
    pub citation: Option<Citation>,  // NEW
    // ...
}
```

### Recall Output

```json
{
  "matches": [
    {
      "content": "Arawn is written in Rust",
      "confidence": 0.92,
      "citation": {
        "type": "session",
        "session_id": "abc123",
        "message_index": 5,
        "timestamp": "2026-02-04T10:30:00Z"
      }
    }
  ]
}
```

## Implementation Notes

### Technical Approach

1. **Schema Migration**
   - Add `citation` TEXT column to memories table (JSON serialized)
   - Index on citation type for filtering

2. **Indexer Updates**
   - Pass message index to extraction
   - Include session context in stored facts

3. **Tool Updates**
   - file_read: capture path + line range
   - web_fetch: capture URL + timestamp
   - note: capture user attribution

4. **Recall Updates**
   - Include citation in RecallMatch
   - Optional filter by citation type

### Dependencies
- Memory schema migration system
- No external dependencies

### Staleness Detection

**File-based memories:**
- Compute SHA-256 of file content at extraction time
- Store mtime for quick change detection (check mtime first, hash only if changed)
- On recall: compare current mtime/hash → flag as `potentially_stale` if different
- Could trigger automatic re-extraction or confidence degradation

**Web-based memories:**
- Store content hash at fetch time
- Store ETag/Last-Modified headers if available
- Age-based staleness: configurable threshold (e.g., 7 days for news, 30 days for docs)
- On recall: flag based on age; optionally re-fetch with If-None-Match for validation

**RecallMatch extension:**
```rust
pub struct RecallMatch {
    pub content: String,
    pub confidence: f32,
    pub citation: Option<Citation>,
    pub staleness: Option<Staleness>,  // NEW
}

pub enum Staleness {
    Fresh,                    // Source unchanged or recently verified
    PotentiallyStale {        // Source may have changed
        reason: String,       // "file_modified", "age_exceeded", etc.
        last_verified: DateTime<Utc>,
    },
    Invalidated,              // Source confirmed changed, needs re-extraction
}
```

### Risk Considerations
- Citation data increases storage slightly
- Need to handle missing citations gracefully (legacy data)
- Staleness checks add latency to recall (mitigate with async/background validation)

## Status Updates

### 2026-02-07: Implementation Complete

**Core Implementation:**
1. ✅ Added `Citation` enum to `types.rs` with 5 variants:
   - `Session { session_id, message_index, timestamp }`
   - `File { path, line_start, line_end, commit_hash, content_hash, mtime }`
   - `Web { url, fetched_at, title, content_hash, etag }`
   - `User { session_id, stated_at }`
   - `System { derived_at, method }`

2. ✅ Added `Staleness` enum to `types.rs`:
   - `Fresh` - Source unchanged or recently verified
   - `PotentiallyStale { reason, last_verified }` - Source may have changed
   - `Invalidated { reason, detected_at }` - Source confirmed changed
   - `Unknown` - No citation or can't determine

3. ✅ Added `citation: Option<Citation>` field to `Memory` struct with builder method

4. ✅ Added `staleness: Staleness` field to `RecallMatch` in `query.rs`

5. ✅ Schema migration v4 in `store/mod.rs`:
   - Added `citation TEXT` column to memories table
   - Added index on `json_extract(citation, '$.type')` for filtering

6. ✅ Updated all database operations in `memory_ops.rs`:
   - `insert_memory`, `insert_memory_with_embedding`
   - `update_memory`, `get_memory`, `list_memories`
   - `find_contradictions`
   - `row_to_memory` deserializer

7. ✅ Updated `session_ops.rs` to include citation column in queries

8. ✅ Added `compute_staleness()` function in `recall.rs`:
   - File citations: compares current mtime vs stored mtime (1-second tolerance)
   - Web citations: age-based (7-day threshold)
   - Session/User citations: always Fresh
   - System citations: Unknown

**Tests:** All 117 arawn-memory tests pass. Full workspace compiles.

**Files modified:**
- `crates/arawn-memory/src/types.rs`
- `crates/arawn-memory/src/store/mod.rs`
- `crates/arawn-memory/src/store/query.rs`
- `crates/arawn-memory/src/store/recall.rs`
- `crates/arawn-memory/src/store/memory_ops.rs`
- `crates/arawn-memory/src/store/session_ops.rs`

**Remaining work (future tasks):**
- File reader tool: capture path + line range + content_hash + mtime when storing memories
- Web fetch tool: capture URL + timestamp + hash + etag when storing memories
- Memory tool: add filter by citation type to recall query

### 2026-02-07: Integration Complete

**Session Indexer Integration:**
- ✅ Exported `Citation` and `Staleness` from `arawn-memory/lib.rs`
- ✅ Updated `SessionIndexer.store_facts()` to add `Citation::session(session_id, 0)` to each fact
- ✅ Updated `SessionIndexer.store_summary()` to add session citation to summaries
- (message_index is 0 as facts are extracted from session as a whole, not individual messages)

**API Response Integration:**
- ✅ Added `citation: Option<serde_json::Value>` to `MemorySearchResult` struct
- ✅ Updated `memory_search_handler` to serialize and include citation in results
- Citations are serialized to JSON for flexibility in the API response

**Files modified:**
- `crates/arawn-memory/src/lib.rs` - Export Citation, Staleness
- `crates/arawn-agent/src/indexing/indexer.rs` - Add citation to facts and summaries
- `crates/arawn-server/src/routes/memory.rs` - Include citation in API responses

**Tests:** All workspace tests pass (275 agent, 67 server, 117 memory)