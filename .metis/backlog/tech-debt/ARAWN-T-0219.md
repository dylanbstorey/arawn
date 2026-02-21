---
id: architecture-storage-abstraction
level: task
title: "Architecture: Storage Abstraction Traits"
short_code: "ARAWN-T-0219"
created_at: 2026-02-20T14:47:44.864726+00:00
updated_at: 2026-02-20T22:28:14.720070+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#tech-debt"
  - "#phase/active"


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

## Acceptance Criteria

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

### 2026-02-20 - Session 1: Analysis

**Current Implementations Reviewed:**

1. **arawn-memory/src/store/mod.rs** - `MemoryStore` struct
   - Uses `Mutex<Connection>` for SQLite access
   - Methods: `insert_memory`, `get_memory`, `search_memories`, `delete_memory`
   - Also has graph + vector capabilities
   - Key operations are synchronous (blocking)

2. **arawn-workstream/src/manager.rs** - `WorkstreamManager` struct
   - Composes `WorkstreamStore` (SQLite) + `MessageStore` (JSONL)
   - Methods: CRUD for workstreams, session management, messaging
   - Already has some abstraction with internal `WorkstreamStore`

3. **arawn-session/src/cache.rs** - `SessionCache<P>` generic struct
   - Already has `PersistenceHook` trait for pluggable persistence!
   - LRU + TTL eviction built in
   - Methods: `get_or_load`, `insert`, `update`, `save`, `remove`, `invalidate`

**Key Finding**: `arawn-session` already has a trait-based design (`PersistenceHook`). The session cache is already well-abstracted.

**Plan:**
1. Create `MemoryBackend` trait in arawn-memory
2. Create `WorkstreamBackend` trait in arawn-workstream (the inner store layer)
3. Verify `SessionStore` pattern already exists via `PersistenceHook`
4. Add mock implementations for testing

### 2026-02-20 - Session 1: Implementation

**Created Storage Abstraction Traits:**

1. **arawn-memory/src/backend.rs** - `MemoryBackend` trait
   - Core operations: `insert`, `get`, `update`, `delete`, `list`, `count`, `touch`
   - Extension trait `MemoryBackendExt` for: `find_contradictions`, `supersede`, `reinforce`, `update_last_accessed`
   - `MockMemoryBackend` implementation for testing
   - `MemoryStore` implements both traits (delegating to existing methods)
   - ✅ 142 tests pass

2. **arawn-workstream/src/storage.rs** - `WorkstreamStorage` + `MessageStorage` traits
   - `WorkstreamStorage`: workstream CRUD, tags, session operations
   - `MessageStorage`: append, read_all, read_range, move_messages, delete_all
   - `MockWorkstreamStorage` and `MockMessageStorage` for testing
   - `WorkstreamStore` implements `WorkstreamStorage`
   - `MessageStore` implements `MessageStorage`
   - Added `move_messages()` and `delete_all()` methods to MessageStore
   - ✅ 161 tests pass

3. **arawn-session already has `PersistenceHook` trait**
   - `PersistenceHook` trait in persistence.rs: `load`, `save`, `delete`, `on_evict`
   - `NoPersistence` implementation for in-memory-only caching
   - No changes needed - already trait-based!

**Acceptance Criteria:**
- [x] `MemoryBackend` trait defined in `arawn-memory`
- [x] `SqliteMemoryStore` implements `MemoryBackend` (via MemoryStore)
- [x] `WorkstreamStore` trait defined in `arawn-workstream` (as `WorkstreamStorage`)
- [x] Current JSONL+SQLite implementation wrapped as `WorkstreamStore` + `MessageStore`
- [x] `SessionStore` trait already exists via `PersistenceHook`
- [x] `LruSessionStore` implements via `SessionCache<P: PersistenceHook>`
- [x] All existing tests pass
- [x] Mock implementations for testing: `MockMemoryBackend`, `MockWorkstreamStorage`, `MockMessageStorage`

### Final Verification

**Workspace Build**: ✅ Success
**Test Results**:
- arawn-memory: 142 passed
- arawn-workstream: 161 passed  
- arawn-session: 14 passed
- arawn-domain: 5 passed

**Files Created/Modified**:
- `crates/arawn-memory/src/backend.rs` (new) - MemoryBackend trait + MockMemoryBackend
- `crates/arawn-memory/src/lib.rs` (modified) - export backend module
- `crates/arawn-memory/src/store/mod.rs` (modified) - implement MemoryBackend for MemoryStore
- `crates/arawn-workstream/src/storage.rs` (new) - WorkstreamStorage + MessageStorage traits + mocks
- `crates/arawn-workstream/src/lib.rs` (modified) - export storage module
- `crates/arawn-workstream/src/store.rs` (modified) - implement WorkstreamStorage for WorkstreamStore
- `crates/arawn-workstream/src/message_store.rs` (modified) - implement MessageStorage, add move_messages/delete_all

**Note**: arawn-agent has a pre-existing SIGABRT in shell/PTY tests (unrelated to these changes)