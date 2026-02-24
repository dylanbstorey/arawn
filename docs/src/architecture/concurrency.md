# Concurrency Model

Arawn uses `tokio` for async I/O and `tokio::sync::RwLock` for shared mutable state. This page documents the lock ordering invariants and patterns that prevent deadlocks.

## Lock Inventory

All mutable state lives in `RuntimeState` or `SharedServices`. Each lock is assigned a **lock order number**; when acquiring multiple locks, always acquire lower numbers first.

| Order | Field | Type | Location |
|-------|-------|------|----------|
| 1 | `pending_reconnects` | `RwLock<HashMap<SessionId, PendingReconnect>>` | `RuntimeState` |
| 2 | `session_owners` | `RwLock<HashMap<SessionId, ConnectionId>>` | `RuntimeState` |
| 3 | `session_cache.inner` | `RwLock<LruCache<SessionId, Session>>` | `RuntimeState` |
| 4 | `mcp_manager` | `RwLock<McpManager>` | `SharedServices` |
| 5 | `tasks` | `RwLock<HashMap<String, TrackedTask>>` | `RuntimeState` |
| — | `ws_connection_tracker` | `RwLock<HashMap<IpAddr, Vec<Instant>>>` | `RuntimeState` |

The `ws_connection_tracker` is independent — it never nests with other locks, so it has no ordering constraint.

## Lock Ordering Rule

> **Never hold a higher-numbered lock while acquiring a lower-numbered one.**

For example, if you hold `session_owners` (2), you may acquire `mcp_manager` (4) but **not** `pending_reconnects` (1).

### Correct

```rust
// Acquire in order: 1 → 2
let pending = state.runtime.pending_reconnects.read().await;
let owners = state.runtime.session_owners.write().await;
```

### Wrong

```rust
// DEADLOCK RISK: acquiring 1 while holding 2
let owners = state.runtime.session_owners.write().await;
let pending = state.runtime.pending_reconnects.read().await; // BAD
```

## Patterns

### Release Before Spawn

When spawning a `tokio::spawn` task that will acquire locks, release all currently held locks first:

```rust
let data = {
    let store = state.runtime.tasks.read().await;
    store.get("id").cloned()
    // guard dropped here
};

tokio::spawn(async move {
    // Safe to acquire any lock now
    let mut store = state.runtime.tasks.write().await;
    // ...
});
```

### Clone and Release

Keep critical sections short by cloning data out of the lock:

```rust
let sessions: Vec<SessionId> = {
    let owners = state.runtime.session_owners.read().await;
    owners.keys().copied().collect()
    // guard dropped here
};

// Process sessions without holding the lock
for sid in sessions {
    // ...
}
```

### Prefer Read Locks

Use `read()` over `write()` whenever possible. Multiple readers can proceed concurrently; a write lock is exclusive:

```rust
// Good: read lock for lookups
let owners = state.runtime.session_owners.read().await;
let is_owner = owners.get(&session_id) == Some(&connection_id);

// Only use write() when you actually need to mutate
let mut owners = state.runtime.session_owners.write().await;
owners.insert(session_id, connection_id);
```

### Drop Guards Explicitly

When you need to acquire multiple locks in sequence (not nested), explicitly drop earlier guards:

```rust
let token_valid = {
    let pending = state.runtime.pending_reconnects.write().await;
    // ... validate and remove token ...
    true
}; // pending guard dropped

if token_valid {
    let mut owners = state.runtime.session_owners.write().await;
    owners.insert(session_id, connection_id);
}
```

## Adding New Locks

When introducing a new `RwLock` or `Mutex`:

1. **Assign a lock order number.** Place it in the table above relative to its expected nesting. If it never nests with existing locks, mark it independent.
2. **Document the ordering** in the field's doc comment with `/// Lock order: N`.
3. **Audit existing call sites** to verify no inverted acquisitions.
4. **Prefer `tokio::sync::RwLock`** over `std::sync::RwLock` — Arawn's handlers are async and must not block the executor.

## Immutable Services

Fields in `SharedServices` that are wrapped in `Arc<T>` (not `Arc<RwLock<T>>`) are immutable after startup and require no locking:

- `agent: Arc<Agent>`
- `config: Arc<ServerConfig>`
- `workstreams: Option<Arc<WorkstreamManager>>`
- `indexer: Option<Arc<SessionIndexer>>`
- `directory_manager: Option<Arc<DirectoryManager>>`
- `sandbox_manager: Option<Arc<SandboxManager>>`

These can be read freely from any handler without lock ordering concerns.
