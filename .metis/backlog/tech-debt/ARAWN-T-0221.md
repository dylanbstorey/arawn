---
id: architecture-appstate-refactoring
level: task
title: "Architecture: AppState Refactoring"
short_code: "ARAWN-T-0221"
created_at: 2026-02-20T14:47:46.615517+00:00
updated_at: 2026-02-20T14:47:46.615517+00:00
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

- [ ] `SharedServices` struct holds immutable services (created at startup)
- [ ] `RuntimeState` struct holds mutable state (changes during operation)
- [ ] `AppState` composed of `SharedServices` + `RuntimeState`
- [ ] Lock contention reduced (finer-grained locks)
- [ ] All route handlers updated to use new structure
- [ ] All existing tests pass
- [ ] No functional changes to API behavior

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

*To be added during implementation*