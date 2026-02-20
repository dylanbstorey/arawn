---
id: architecture-storage-abstraction
level: task
title: "Architecture: Storage Abstraction Traits"
short_code: "ARAWN-T-0219"
created_at: 2026-02-20T14:47:44.864726+00:00
updated_at: 2026-02-20T14:47:44.864726+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#tech-debt"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Architecture: Storage Abstraction Traits

## Objective

Add storage abstraction traits to enable swappable backends for memory, workstreams, and sessions.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P3 - Low (when time permits)

### Technical Debt Impact
- **Current Problems**: 
  - `MemoryStore` exposes SQLite + sqlite-vec internals (leaky abstraction)
  - `WorkstreamManager` tightly coupled to JSONL + SQLite (no storage trait)
  - `SessionCache` is concrete LRU only (can't swap to Redis)
- **Benefits of Fixing**: Swappable backends (Redis sessions, different DB engines), better testability with mock implementations
- **Risk Assessment**: LOW - additive change, existing implementations become first impl of traits

## Acceptance Criteria

- [ ] `MemoryBackend` trait defined in `arawn-memory`
- [ ] `SqliteMemoryStore` implements `MemoryBackend`
- [ ] `WorkstreamStore` trait defined in `arawn-workstream`
- [ ] Current JSONL+SQLite implementation wrapped as `LocalWorkstreamStore`
- [ ] `SessionStore` trait defined in `arawn-session` or `arawn-server`
- [ ] `LruSessionStore` implements `SessionStore`
- [ ] All existing tests pass
- [ ] At least one mock implementation for testing

## Implementation Notes

### MemoryBackend Trait

```rust
// arawn-memory/src/traits.rs
#[async_trait]
pub trait MemoryBackend: Send + Sync {
    async fn store(&self, memory: Memory) -> Result<MemoryId>;
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<Memory>>;
    async fn delete(&self, id: MemoryId) -> Result<()>;
    async fn get(&self, id: MemoryId) -> Result<Option<Memory>>;
}

// Existing implementation
pub struct SqliteMemoryStore { ... }
impl MemoryBackend for SqliteMemoryStore { ... }
```

### WorkstreamStore Trait

```rust
// arawn-workstream/src/traits.rs
#[async_trait]
pub trait WorkstreamStore: Send + Sync {
    async fn create(&self, config: WorkstreamConfig) -> Result<Workstream>;
    async fn get(&self, id: &str) -> Result<Option<Workstream>>;
    async fn list(&self, state: Option<&str>) -> Result<Vec<Workstream>>;
    async fn update(&self, id: &str, updates: WorkstreamUpdate) -> Result<()>;
    async fn delete(&self, id: &str) -> Result<()>;
    async fn store_message(&self, ws_id: &str, msg: Message) -> Result<()>;
    async fn load_messages(&self, ws_id: &str, limit: Option<usize>) -> Result<Vec<Message>>;
}

pub struct LocalWorkstreamStore { ... }  // Current JSONL + SQLite impl
impl WorkstreamStore for LocalWorkstreamStore { ... }
```

### SessionStore Trait

```rust
// arawn-session/src/traits.rs or arawn-server/src/session_store.rs
#[async_trait]
pub trait SessionStore: Send + Sync {
    async fn get(&self, id: SessionId) -> Result<Option<Session>>;
    async fn put(&self, id: SessionId, session: Session) -> Result<()>;
    async fn remove(&self, id: SessionId) -> Result<Option<Session>>;
    async fn contains(&self, id: SessionId) -> Result<bool>;
}

pub struct LruSessionStore { ... }  // Current impl
impl SessionStore for LruSessionStore { ... }

// Future: pub struct RedisSessionStore { ... }
```

## Dependencies

- Can be done in parallel with ARAWN-T-0218 (Domain Facade)
- Not blocking other tasks

## Status Updates

*To be added during implementation*