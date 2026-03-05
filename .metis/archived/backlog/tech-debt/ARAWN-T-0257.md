---
id: reduce-server-crate-coupling-via
level: task
title: "Reduce server crate coupling via domain facade"
short_code: "ARAWN-T-0257"
created_at: 2026-03-04T13:24:01.847600+00:00
updated_at: 2026-03-05T00:50:34.748932+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#tech-debt"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
---

# Reduce server crate coupling via domain facade

## Objective

Reduce `arawn-server`'s direct dependency on 8+ internal crates by introducing a domain facade that aggregates the APIs it needs. This improves build times and reduces coupling.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P3 - Low (when time permits)

### Technical Debt Impact
- **Current Problems**: Server crate depends directly on arawn-agent, arawn-workstream, arawn-config, arawn-llm, arawn-types, arawn-memory, arawn-notes, arawn-oauth — changes in any trigger recompilation
- **Benefits of Fixing**: Faster incremental builds, clearer dependency graph, easier to test server in isolation
- **Risk Assessment**: Medium — requires careful API design to avoid leaky abstractions

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Server depends on at most 3-4 internal crates (domain facade + types + config)
- [ ] Domain facade re-exports only what server routes need
- [ ] Build time for server-only changes improves measurably
- [ ] All integration tests pass

## Status Updates

### Analysis Complete
**Current state:** arawn-server depends on 9 internal crates (arawn-agent, arawn-config, arawn-domain, arawn-llm, arawn-memory, arawn-mcp, arawn-sandbox, arawn-session, arawn-types, arawn-workstream).

**Plan:**
1. Add arawn-sandbox and arawn-session as dependencies to arawn-domain
2. Add comprehensive re-exports in arawn-domain lib.rs for all types server needs
3. Update all `use arawn_*` imports in arawn-server production code to use arawn-domain
4. Keep arawn-types as direct dep (foundation types + config::defaults module path)
5. Keep arawn-llm as dev-dep only (for MockBackend in tests)
6. Remove all other direct dependencies from arawn-server Cargo.toml
7. Target: arawn-server depends on arawn-domain + arawn-types (prod) + arawn-llm (test)

Starting implementation...

### Implementation Complete

**Result:** arawn-server production dependencies reduced from 9 internal crates to **2** (arawn-domain + arawn-types).

**Changes made:**

1. **arawn-domain/Cargo.toml**: Added arawn-config, arawn-sandbox, arawn-session as deps
2. **arawn-domain/src/lib.rs**: Added comprehensive re-exports:
   - Agent types: AgentError, Agent, Session, SessionId, Turn, TurnId, ToolCall, ToolRegistry, etc.
   - Config: ConfigError
   - Memory: MemoryStore, MemoryId, MemoryNote, NoteId, ContentType, Memory
   - Sandbox: SandboxManager
   - Session: SessionCacheImpl, CacheConfig, PersistenceHook, SessionStoreError, SessionStoreResult
   - Workstream: WorkstreamManager, DirectoryManager, WorkstreamError, DirectoryError, Workstream, WorkstreamMessage, SCRATCH_ID, PathValidator, AttachResult, MessageRole, etc.
3. **arawn-server production code**: Updated all imports across 15+ files from arawn_agent/arawn_workstream/arawn_memory/arawn_mcp/arawn_sandbox/arawn_session/arawn_config to arawn_domain
4. **arawn-server/Cargo.toml**: Removed 7 production deps (arawn-agent, arawn-config, arawn-memory, arawn-mcp, arawn-sandbox, arawn-session, arawn-workstream). Added arawn-agent + arawn-memory as dev-deps for integration test validation APIs.

**Verification:**
- `angreal check all` passes (clippy + fmt clean)
- `angreal test unit` passes (all tests green)
- Production deps: arawn-domain + arawn-types (2 crates)
- Dev deps: arawn-agent, arawn-llm, arawn-memory, arawn-plugin (testing only)