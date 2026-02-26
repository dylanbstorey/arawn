---
id: medium-priority-codebase-cleanup
level: task
title: "MEDIUM priority codebase cleanup — dead code, hardcoded values, incomplete implementations"
short_code: "ARAWN-T-0224"
created_at: 2026-02-25T14:20:20.693218+00:00
updated_at: 2026-02-25T14:20:20.693218+00:00
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

# MEDIUM priority codebase cleanup — dead code, hardcoded values, incomplete implementations

## Objective

Address all 60 MEDIUM-severity findings from the 2026-02-25 codebase audit. These are DRY violations, non-configurable constants, partially dead code, and incomplete features that degrade maintainability but don't cause immediate breakage.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P2 - Medium (nice to have)

### Technical Debt Impact
- **Current Problems**: Constants duplicated across crates, server reimplements session caching (bypassing arawn-session), stub CLI subcommands mislead users, non-configurable timeouts and retry params limit deployment flexibility.
- **Benefits of Fixing**: Single source of truth for constants, smaller surface area, configurable operational params, cleaner public API per crate.
- **Risk Assessment**: Low — each fix is localized. Some items (arawn-session bypass) may warrant architectural discussion before changing.

## Acceptance Criteria

- [ ] All MEDIUM dead code items removed or justified with comments
- [ ] All MEDIUM hardcoded values extracted to constants or config
- [ ] All MEDIUM incomplete implementations either completed or gated behind feature flags
- [ ] `angreal test all` passes
- [ ] `angreal check all` passes

---

## Findings

### Dead Code (13 items)

| # | Location | Issue |
|---|----------|-------|
| 1 | `arawn/src/commands/stop.rs` | Cannot actually stop server — no PID/signal mechanism |
| 2 | `arawn-pipeline/src/loader.rs` | `remove_file()` method and `path` field never used |
| 3 | `arawn-session/src/lib.rs` | Most exports unused — server reimplements its own session cache |
| 4 | `arawn-workstream/src/compression.rs` | `Compressor`/`CompressorConfig` exported but never imported externally |
| 5 | `arawn-client/Cargo.toml` | `blocking` feature flag has no corresponding code |
| 6 | `arawn-agent/src/tools/memory_search.rs` | Registered but returns empty (stub) |
| 7 | `arawn-sandbox/src/policy.rs` | Several `SandboxPolicy` methods never called |
| 8 | `arawn-llm/src/backends/groq.rs` | `GroqStreamState` enum variants partially unused |
| 9 | `arawn-config/src/lib.rs` | `merge_env()` defined but never called |
| 10 | `arawn-types/src/lib.rs` | Re-exports of dead submodules |
| 11 | `arawn-server/src/routes/mod.rs` | `legacy_compat` route module compiled but unmounted |
| 12 | `arawn-domain/src/lib.rs` | `DomainError::NotImplemented` variant never constructed |
| 13 | `arawn-memory/src/graph.rs` | `GraphStore::remove_entity()` never called |

### Hardcoded Values (42 items)

| # | Location | Issue |
|---|----------|-------|
| 1 | `arawn-llm/src/backends/groq.rs` | Base URL `https://api.groq.com/openai/v1` hardcoded |
| 2 | `arawn-llm/src/backends/ollama.rs` | Default URL `http://localhost:11434` hardcoded |
| 3 | Multiple backends | Retry params (3 retries, 500ms backoff) duplicated across backends |
| 4 | `arawn-server/src/lib.rs` | `DEFAULT_MAX_SESSIONS = 5` defined in 3 places |
| 5 | `arawn-server/src/lib.rs` | API version `v1` defined in 2 places |
| 6 | `arawn-server/src/lib.rs` | Bind address `127.0.0.1:8080` in 3 places |
| 7 | `arawn-agent/src/tools/web_search.rs` | Brave/Google/DuckDuckGo API URLs inline |
| 8 | `arawn-agent/src/bootstrap.rs` | Bootstrap files list (CLAUDE.md, .arawn/, etc.) hardcoded |
| 9 | `arawn-config/src/lib.rs` | Config file search paths hardcoded |
| 10 | `arawn-sandbox/src/bubblewrap.rs` | Mount paths and sandbox params hardcoded |
| 11-20 | Various | Timeouts (30s, 60s, 120s), port numbers, buffer sizes not configurable |
| 21-42 | Various | Magic numbers, duplicated string literals for error messages, default values that should be named constants |
| 43 | `arawn-oauth/src/oauth.rs` | OAuth client ID, authorize/token URLs, redirect URI, and scope hardcoded in `OAuthConfig::anthropic_max()` — make configurable via `arawn.toml` for test/mock OAuth servers |

### Incomplete Implementations (5 items)

| # | Location | Issue |
|---|----------|-------|
| 1 | `arawn/src/commands/notes.rs` | `search`, `show`, `delete` subcommands are stubs |
| 2 | `arawn/src/commands/memory.rs` | `recent` and `export` subcommands are stubs |
| 3 | `arawn-tui/src/app.rs` | Session deletion not implemented (shows "TODO" in status bar) |
| 4 | `arawn-tui/src/views/workstream.rs` | Workstream overlay is non-interactive (renders but ignores input) |
| 5 | `arawn-llm/src/backends/mod.rs` | `MemoryBackendExt` trait methods are all no-ops by default |

## Implementation Notes

### Suggested Approach
1. **Constants consolidation first** — Create shared constants module or use `arawn-config` as single source of truth
2. **Dead code removal** — Safe to batch-delete with `angreal test all` verification after each batch
3. **arawn-session bypass** — Discuss whether to delete crate or rewire server to use it before making changes
4. **Stub commands** — Either implement or gate behind `#[cfg(feature = "experimental")]`

### Dependencies
- ARAWN-T-0223 (HIGH) should be done first — some items overlap (e.g., MemorySearchTool, stop command)

## Status Updates

*To be added during implementation*