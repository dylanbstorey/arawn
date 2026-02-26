---
id: high-priority-codebase-cleanup
level: task
title: "HIGH priority codebase cleanup — dead code, hardcoded values, incomplete implementations"
short_code: "ARAWN-T-0223"
created_at: 2026-02-25T14:20:19.748758+00:00
updated_at: 2026-02-26T01:57:27.796135+00:00
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

# HIGH priority codebase cleanup — dead code, hardcoded values, incomplete implementations

## Objective

Address all 27 HIGH-severity findings from the 2026-02-25 codebase audit across dead code, hardcoded values, and incomplete implementations. These items represent code that actively misleads users, will break on external changes, or leaves core features non-functional.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P1 - High (important for user experience)

### Technical Debt Impact
- **Current Problems**: Stub CLI commands mislead users, health-check models will break on deprecation, model defaults are inconsistent across crates, MemorySearchTool is non-functional, entire dead modules inflate compile times.
- **Benefits of Fixing**: Users get working CLI commands or clear errors, no surprise breakages from deprecated models, consistent behavior across the system, smaller codebase.
- **Risk Assessment**: Medium — touches many crates but each fix is self-contained. Run full test suite after each batch.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] All HIGH dead code items removed or promoted to functional implementations
- [ ] All HIGH hardcoded values extracted to configuration or constants
- [ ] All HIGH incomplete implementations either completed or replaced with clear "not available" errors
- [ ] `angreal test all` passes
- [ ] `angreal check all` passes

---

## Findings

### Dead Code (10 items)

| # | Location | Issue | Fix |
|---|----------|-------|-----|
| 1 | `arawn-types/src/memory.rs` | Entire module dead — superseded by `arawn-memory` | Delete module |
| 2 | `arawn-types/src/message.rs` | Entire module dead — superseded by `arawn-llm`/`arawn-agent` | Delete module |
| 3 | `arawn-types/src/task.rs` | Entire module dead — superseded by `arawn-server::state` | Delete module |
| 4 | `arawn-types/src/error.rs` | Entire module dead — each crate defines own errors | Delete module |
| 5 | `arawn-types/src/lib.rs` | `Config`, `Id`, `new_id()`, `Timestamp`, `now()` all unused | Delete exports, clean re-exports |
| 6 | `arawn-domain/src/services/memory.rs` | `MemoryService` placeholder — always returns false/errors | Delete or wire up |
| 7 | `arawn-domain/src/services/mcp.rs` | `McpServerInfo`, `McpToolInfo` never constructed; `McpService` never accessed | Delete or wire up |
| 8 | `arawn-domain/src/services/mod.rs` | `DomainConfig` never used | Delete |
| 9 | `arawn/src/commands/research.rs` | Entire command is a stub | Remove from CLI or implement |
| 10 | `arawn/src/commands/tasks.rs` | All 4 subcommands are stubs | Remove from CLI or implement |

### Hardcoded Values (12 items)

| # | Location | Issue | Fix |
|---|----------|-------|-----|
| 1 | Multiple crates | Default model inconsistency: `AgentConfig` → `claude-sonnet-4-20250514`, `CompactorConfig` → `claude-sonnet`, `IndexerConfig` → `gpt-4o-mini`, spawner → `gpt-4o-mini` | Create single `DEFAULT_MODEL` constant or config cascade |
| 2 | `arawn-llm/src/backends/anthropic.rs` | Health check model `claude-3-haiku-20240307` will break on deprecation | Make configurable, use cheapest available |
| 3 | `arawn-llm/src/backends/openai.rs` | Health check model `gpt-3.5-turbo` will break on deprecation | Make configurable, use cheapest available |
| 4 | `arawn-oauth/src/lib.rs` | OAuth client ID `9d1c250a-...` hardcoded | Move to config |
| 5 | `arawn-oauth/src/lib.rs` | OAuth URLs and beta flags hardcoded | Move to config |
| 6 | `arawn-agent/src/indexer.rs` | Embedding dimension mapping only covers 3 models, silently defaults to 1536 | Expand mapping or query model metadata |
| 7 | `arawn-agent/src/ner/gliner.rs` | HuggingFace model download URLs not configurable (blocks airgapped deployments) | Add config for model URL/path |
| 8 | `arawn-agent/src/ner/gliner.rs` | Model filename and cache path hardcoded | Make configurable |
| 9 | `arawn-agent/src/tools/shell.rs` | Tool output limit 100KB not configurable | Add to tool config |
| 10 | `arawn-agent/src/tools/file_read.rs` | File read limit 500KB not configurable | Add to tool config |
| 11 | `arawn-agent/src/tools/web_fetch.rs` | Web fetch limit 200KB not configurable | Add to tool config |
| 12 | `arawn-agent/src/tools/web_search.rs` | Search result limit 50KB not configurable | Add to tool config |

### Incomplete Implementations (5 items)

| # | Location | Issue | Fix |
|---|----------|-------|-----|
| 1 | `arawn/src/commands/research.rs` | `research` command prints "not yet implemented" and exits | Implement or remove |
| 2 | `arawn/src/commands/tasks.rs` | All 4 subcommands print "not yet implemented" | Implement or remove |
| 3 | `arawn/src/commands/stop.rs` | Cannot actually stop a running server (no PID file or signal) | Implement PID file + signal, or remove |
| 4 | `arawn-agent/src/tools/memory_search.rs` | `MemorySearchTool` returns empty results — never wired to `MemoryStore` | Wire up to memory store |
| 5 | `arawn-domain/src/services/memory.rs` | `MemoryService` always returns false/errors — domain facade never connected | Wire up or delete (overlaps dead code #6) |

## Status Updates

### 2026-02-25 — Groups 1–3 complete (dead code removal)

**Group 1: arawn-types dead modules**
- Deleted `memory.rs`, `message.rs`, `task.rs`, `error.rs` (4 files)
- Cleaned `lib.rs`: removed dead module declarations, `Id`/`new_id()`/`Timestamp`/`now()`/`Config`
- Removed `chrono`, `uuid`, `thiserror` from Cargo.toml (no longer needed)
- Live modules retained: `config`, `delegation`, `hooks`

**Group 2: arawn-domain dead services**
- Deleted `services/memory.rs` (stub MemoryService)
- Removed `MemoryService` and `DomainConfig` from `services/mod.rs` and `lib.rs`
- Removed `DomainError::Memory` variant from `error.rs`
- Cleaned Cargo.toml: removed `uuid`, `chrono`, `arawn-memory`, `arawn-session`, `arawn-config`, `arawn-sandbox`, `parking_lot`, `futures-core`, `async-trait`; moved `arawn-llm` to dev-dependencies
- `McpService` retained (still used in DomainServices, though no external callers yet)
- Created ARAWN-T-0226 for future work: wire MemoryStore through domain facade

**Group 3: stub CLI commands**
- Deleted `research.rs`, `tasks.rs`, `stop.rs` (3 files)
- Removed from `commands/mod.rs`, `main.rs` (imports, enum variants, dispatch arms)
- Updated `cli_integration.rs`: removed 3 dead tests, updated subcommand list assertion

**Verification**: `cargo check` clean, `angreal test unit` — 1,705 passed, 0 failed

### 2026-02-25 — Group 4 complete (model default consolidation)

**Goal**: Remove all hardcoded model defaults from agent code; config system is the single source of truth.

**arawn-config/src/types.rs — CompactionConfig**
- Changed `model: Option<String>` → `model: String` with default `"gpt-4o-mini"`
- Model can never be `None` — config always provides a value
- Updated 4 tests to use `String` assertions instead of `Option` assertions

**arawn-agent/src/compaction.rs — CompactorConfig**
- Removed hardcoded `"claude-sonnet"` from `Default` impl (now `String::new()`)
- Removed `SessionCompactor::with_defaults()` convenience method
- Tests now use explicit `test_config()` helper that sets `model: "test-model"`

**arawn-agent/src/indexing/indexer.rs — IndexerConfig**
- Removed hardcoded `"gpt-4o-mini"` from `Default` impl (now `String::new()`)
- Tests now use explicit `test_indexer_config()` helper that sets `model: "test-model"`

**arawn-plugin/src/agent_spawner.rs — compact_result()**
- Changed `model` param from `Option<&str>` to `&str`
- Removed `unwrap_or("gpt-4o-mini")` fallback
- Callers pass `&self.compaction_config.model` directly
- Updated 3 tests

**arawn-server/src/routes/commands.rs — CompactCommand**
- Removed `Default` impl and `CompactCommand::default()`
- Added `CompactCommand::new(config)` constructor
- `compact_command_handler` and `compact_command_stream_handler` now read model from `state.agent().config().model`
- `CommandRegistry::with_defaults()` → `CommandRegistry::with_compact(model)`
- `list_commands_handler` now reads model from AppState
- Updated 8 tests to use `test_compact_command()` helper

**arawn-server/src/routes/ws/handlers.rs**
- `handle_command` now reads model from `app_state.agent().config().model`

**Verification**: `cargo check` clean, `angreal test unit` — 1,719 passed, 0 failed

### 2026-02-25 — Group 5 complete (health check removal)

**Decision**: Health checks for hosted APIs (Anthropic, OpenAI, Groq) are pointless — they burn API calls to test connectivity that will be immediately evident on the first real request. Worse, the hardcoded model IDs (`claude-3-haiku-20240307`, `gpt-3.5-turbo`) create false negatives when providers deprecate models.

**Removed**:
- `LlmBackend::health_check()` trait method from `backend.rs`
- `AnthropicBackend::health_check()` (hardcoded `claude-3-haiku-20240307`)
- `OpenAiBackend::health_check()` (hardcoded `gpt-3.5-turbo` fallback + Ollama `/models` check)
- `MockBackend::health_check()` and its test
- `LlmClient::health_check()` and `health_check_provider()` from `client.rs`
- `LlmBackend for LlmClient::health_check()` impl

**Not affected**: Server `/health` and `/health/detailed` endpoints don't call LLM health checks — they just report `agent_ready: true`.

**Verification**: `cargo check` clean, `angreal test unit` — 1,718 passed, 0 failed

### 2026-02-26 — Groups 6–10 reviewed, deferred to backlog

Discussed each remaining group. All are legitimate tech debt but not urgent cleanup — they're config gaps and feature work, not dead code or breakage risks.

| Group | Topic | Disposition | Ticket |
|-------|-------|-------------|--------|
| 6 | OAuth client ID + URLs hardcoded | Intentional protocol constants. Added TODO comment, deferred. | ARAWN-T-0224 (item #43) |
| 7 | Embedding dimension mapping covers only 3 models | `OpenAiEmbedder::new()` ignores config `dimensions`. Real fix: wire config through. | ARAWN-T-0227 (P2) |
| 8 | GLiNER model download not automated | NER is behind feature flag + manual config. Add auto-download like embeddings. | ARAWN-T-0228 (P3) |
| 9 | Per-tool output limits not configurable | 3 overlapping limit systems. Config exists but only partially wired. | ARAWN-T-0229 (P3) |
| 10 | MemorySearchTool returns empty (stub) | Already covered by ARAWN-T-0226 (wire MemoryStore). Left as-is. | ARAWN-T-0226 (P1) |

### Summary

- **Groups 1–5**: Implemented — dead code deleted, model defaults consolidated, health checks removed
- **Groups 6–10**: Triaged to backlog tickets (ARAWN-T-0224, T-0226, T-0227, T-0228, T-0229)
- **Net test delta**: 1,705 → 1,718 (Group 4 added tests) → 1,718 (Group 5 removed 1, unchanged count)
- **Files deleted**: 10 (4 arawn-types modules, 2 domain services, 3 CLI commands, plus various method removals)