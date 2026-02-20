---
id: architecture-domain-facade-storage
level: task
title: "Architecture: Domain Facade, Storage Traits, Unified Config"
short_code: "ARAWN-T-0215"
created_at: 2026-02-20T13:41:44.507525+00:00
updated_at: 2026-02-20T13:41:44.507525+00:00
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

# Architecture: Domain Facade, Storage Traits, Unified Config

## Objective

Reduce coupling in arawn-server by introducing a domain facade crate, add storage abstraction traits for swappable backends, and unify configuration into a single source of truth.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P2 - Medium (nice to have)

### Technical Debt Impact
- **Current Problems**:
  - `arawn-server` has 8 internal dependencies - "god object" that's hard to test/maintain
  - `MemoryStore` exposes SQLite + sqlite-vec internals (leaky abstraction)
  - `WorkstreamManager` tightly coupled to JSONL + SQLite (no storage trait)
  - `SessionCache` is concrete LRU only (can't swap to Redis)
  - Configuration fragmented across 5+ config structs in different crates
  - `AppState` mixes 10+ concerns in one struct
- **Benefits of Fixing**: Testability, swappable backends, clearer architecture
- **Risk Assessment**: MEDIUM - large refactoring effort, but improves long-term maintainability

## Acceptance Criteria

- [ ] New `arawn-domain` (or `arawn-app`) crate acts as facade between server and infrastructure
- [ ] `arawn-server` depends only on `arawn-domain` + `arawn-config` (reduced from 8 deps)
- [ ] `MemoryBackend` trait in `arawn-memory` with SQLite implementation
- [ ] `WorkstreamStore` trait in `arawn-workstream` with JSONL+SQLite implementation
- [ ] `SessionStore` trait in `arawn-server` with LRU implementation
- [ ] All configuration unified in `ArawnConfig` TOML schema
- [ ] `AppState` split into `SharedServices` (immutable) and `RuntimeState` (mutable)

## Implementation Notes

### Phase 1: Domain Facade Crate

**Problem**: `arawn-server` imports agent, config, llm, memory, mcp, sandbox, session, workstream, types

**Solution**: Create `arawn-domain` that owns cross-component orchestration

```
arawn-domain/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── services/
│   │   ├── chat.rs      # Chat orchestration (agent + session + workstream)
│   │   ├── memory.rs    # Memory operations
│   │   └── mcp.rs       # MCP management
│   └── facade.rs        # DomainFacade struct
```

**New dependency graph**:
```
arawn-server → arawn-domain → {arawn-agent, arawn-memory, arawn-workstream, ...}
```

### Phase 2: Storage Traits

**MemoryBackend trait**:
```rust
#[async_trait]
pub trait MemoryBackend: Send + Sync {
    async fn store(&self, memory: Memory) -> Result<MemoryId>;
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<Memory>>;
    async fn delete(&self, id: MemoryId) -> Result<()>;
}

// Implementation
pub struct SqliteMemoryStore { ... }
impl MemoryBackend for SqliteMemoryStore { ... }
```

**WorkstreamStore trait**:
```rust
#[async_trait]
pub trait WorkstreamStore: Send + Sync {
    async fn create(&self, config: WorkstreamConfig) -> Result<Workstream>;
    async fn get(&self, id: &str) -> Result<Option<Workstream>>;
    async fn list(&self, state: Option<&str>) -> Result<Vec<Workstream>>;
    async fn store_message(&self, ws_id: &str, msg: Message) -> Result<()>;
    async fn load_messages(&self, ws_id: &str) -> Result<Vec<Message>>;
}
```

**SessionStore trait**:
```rust
#[async_trait]
pub trait SessionStore: Send + Sync {
    async fn get(&self, id: SessionId) -> Result<Option<Session>>;
    async fn put(&self, id: SessionId, session: Session) -> Result<()>;
    async fn remove(&self, id: SessionId) -> Result<()>;
}

// Implementations
pub struct LruSessionStore { ... }  // Current
pub struct RedisSessionStore { ... }  // Future
```

### Phase 3: Unified Configuration

**Current fragmentation**:
- `ArawnConfig` in `arawn-config`
- `ServerConfig` in `arawn-server`
- `TuiConfig` in `arawn-tui`
- `WorkstreamConfig` in `arawn-workstream`
- `AgentConfig` in `arawn-agent`

**Target**: Single `config.toml` with all sections:
```toml
[server]
bind_address = "127.0.0.1:8080"
auth_token = "..."

[agent]
max_turns = 50
memory_enabled = true

[workstreams]
base_dir = "~/.arawn/workstreams"

[tui]
theme = "dark"
```

### Phase 4: AppState Refactoring

**Current** (`arawn-server/src/state.rs`):
```rust
pub struct AppState {
    agent, config, session_cache, session_owners,
    pending_reconnects, mcp_manager, sandbox_manager,
    workstream_manager, task_registry, ...
}
```

**Target**:
```rust
pub struct SharedServices {
    pub domain: Arc<DomainFacade>,  // Immutable after init
    pub config: Arc<ServerConfig>,
}

pub struct RuntimeState {
    pub session_owners: RwLock<HashMap<...>>,
    pub pending_reconnects: RwLock<HashMap<...>>,
    pub tasks: RwLock<HashMap<...>>,
}

pub struct AppState {
    pub services: SharedServices,
    pub runtime: RuntimeState,
}
```

## Dependencies

- Should be done incrementally (one phase at a time)
- Phase 1 is prerequisite for Phases 2-4
- Consider ADR for architectural decisions

## Status Updates

*To be added during implementation*