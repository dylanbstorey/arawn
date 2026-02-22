---
id: architecture-appstate-refactoring
level: task
title: "Architecture: AppState Refactoring"
short_code: "ARAWN-T-0221"
created_at: 2026-02-20T14:47:46.615517+00:00
updated_at: 2026-02-21T14:08:22.755382+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#tech-debt"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Architecture: AppState Refactoring

## Objective

Refactor `AppState` in `arawn-server` to separate immutable services from mutable runtime state.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P2 - Medium (nice to have)

### Technical Debt Impact
- **Current Problems**: `AppState` mixes 10+ concerns in one struct:
  - agent, config, session_cache, session_owners
  - pending_reconnects, mcp_manager, sandbox_manager
  - workstream_manager, task_registry, etc.
- **Benefits of Fixing**: Clearer separation of concerns, easier testing, better lock granularity
- **Risk Assessment**: MEDIUM - touches central state management

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] `SharedServices` struct holds immutable services (created at startup)
- [x] `RuntimeState` struct holds mutable state (changes during operation)
- [x] `AppState` composed of `SharedServices` + `RuntimeState`
- [x] Lock contention reduced (finer-grained locks)
- [x] All route handlers updated to use new structure
- [x] All existing tests pass
- [x] No functional changes to API behavior

## Implementation Notes

### Current Structure

```rust
// arawn-server/src/state.rs (current)
pub struct AppState {
    pub agent: Arc<AgentRuntime>,
    pub config: Arc<ServerConfig>,
    pub session_cache: Arc<SessionCache>,
    pub session_owners: Arc<RwLock<HashMap<SessionId, SocketAddr>>>,
    pub pending_reconnects: Arc<RwLock<HashMap<String, ReconnectInfo>>>,
    pub mcp_manager: Arc<McpManager>,
    pub sandbox_manager: Arc<SandboxManager>,
    pub workstream_manager: Arc<WorkstreamManager>,
    pub task_registry: Arc<RwLock<HashMap<TaskId, TaskHandle>>>,
    // ... more fields
}
```

### Target Structure

```rust
// arawn-server/src/state.rs (new)

/// Immutable services created at startup
pub struct SharedServices {
    pub domain: Arc<DomainFacade>,  // From ARAWN-T-0218
    pub config: Arc<ServerConfig>,
}

/// Mutable state that changes during operation
pub struct RuntimeState {
    pub session_owners: RwLock<HashMap<SessionId, SocketAddr>>,
    pub pending_reconnects: RwLock<HashMap<String, ReconnectInfo>>,
    pub active_tasks: RwLock<HashMap<TaskId, TaskHandle>>,
}

/// Combined application state
pub struct AppState {
    pub services: SharedServices,
    pub runtime: RuntimeState,
}

impl AppState {
    pub fn new(config: ServerConfig) -> Result<Self>;
    
    // Convenience accessors
    pub fn domain(&self) -> &DomainFacade { &self.services.domain }
    pub fn config(&self) -> &ServerConfig { &self.services.config }
}
```

### Migration Strategy

1. Create `SharedServices` and `RuntimeState` structs
2. Update `AppState` to use composition
3. Add convenience methods for common access patterns
4. Update all route handlers (search for `state.` usages)
5. Update tests

### Route Handler Changes

**Before**:
```rust
async fn handler(State(state): State<Arc<AppState>>) {
    let session = state.session_cache.get(&id).await;
    let owner = state.session_owners.read().await.get(&id);
}
```

**After**:
```rust
async fn handler(State(state): State<Arc<AppState>>) {
    let session = state.domain().sessions().get(&id).await;
    let owner = state.runtime.session_owners.read().get(&id);
}
```

## Dependencies

- **Blocked by**: ARAWN-T-0218 (Domain Facade) - needs DomainFacade to exist first

## Status Updates

### Session 1 - 2026-02-21

**Completed**:
1. Refactored `state.rs` to introduce `SharedServices` and `RuntimeState` structs
2. `SharedServices` holds immutable services: agent, config, rate_limiter
3. `RuntimeState` holds mutable state: session_cache, session_owners, pending_reconnects, tasks, etc.
4. `AppState` now composes these two structs with convenience accessor methods
5. Updated all route handlers to use method accessors instead of field access:
   - `state.config` → `state.config()`
   - `state.session_cache` → `state.session_cache()`
   - `state.workstreams` → `state.workstreams()`
   - etc.
6. Fixed borrowing issues with `Option<&T>` accessors (removed redundant `.as_ref()` calls)
7. All 127 unit tests pass
8. All 92 integration tests pass

**Files Modified**:
- `crates/arawn-server/src/state.rs` - Core refactoring
- `crates/arawn-server/src/ratelimit.rs`
- `crates/arawn-server/src/auth.rs`
- `crates/arawn-server/src/lib.rs`
- `crates/arawn-server/src/routes/agents.rs`
- `crates/arawn-server/src/routes/chat.rs`
- `crates/arawn-server/src/routes/commands.rs`
- `crates/arawn-server/src/routes/config.rs`
- `crates/arawn-server/src/routes/health.rs`
- `crates/arawn-server/src/routes/mcp.rs`
- `crates/arawn-server/src/routes/memory.rs`
- `crates/arawn-server/src/routes/sessions.rs`
- `crates/arawn-server/src/routes/tasks.rs`
- `crates/arawn-server/src/routes/workstreams.rs`
- `crates/arawn-server/src/routes/ws/connection.rs`
- `crates/arawn-server/src/routes/ws/handlers.rs`

**Notes**:
- Maintained backward compatibility via convenience accessor methods
- Builder pattern preserved for AppState construction
- Lock granularity improved - SharedServices is fully immutable
- No functional API changes - pure internal refactor