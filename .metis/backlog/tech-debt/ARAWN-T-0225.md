---
id: low-priority-codebase-cleanup-dead
level: task
title: "LOW priority codebase cleanup — dead code, hardcoded values, incomplete implementations"
short_code: "ARAWN-T-0225"
created_at: 2026-02-25T14:20:21.580871+00:00
updated_at: 2026-03-01T14:01:48.641217+00:00
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

# LOW priority codebase cleanup — dead code, hardcoded values, incomplete implementations

## Objective

Address 19 remaining findings from the 2026-02-25 codebase audit (re-audited 2026-02-28; 13 of original 25 resolved by prior work). Items span dead code (9), hardcoded values (8), and incomplete implementations (2).

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P3 - Low (when time permits)

### Technical Debt Impact
- **Current Problems**: Mock types in production builds, blanket `#[allow(dead_code)]` hiding dead methods, stale version strings in User-Agent/OpenAPI, duplicated magic numbers across 4+ files, health check that can't detect memory store failures.
- **Benefits of Fixing**: Smaller production binaries (mock types removed), accurate version reporting, single source of truth for thresholds/defaults, health endpoint that actually validates system components.
- **Risk Assessment**: Low — most changes are constant extraction or cfg-gating. I2 (plugin CLI wiring) is the only item with moderate scope.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] D1–D9: Dead code items cleaned up (blanket allow removed, mock types gated, unused fields/methods deleted) — committed d830d73
- [x] H2–H8: Hardcoded values replaced with constants or `env!()` macros (H1 skipped: proc macro limitation) — committed 515ce24
- [x] I1: Deleted fake detailed health check endpoint — stub returned hardcoded values, simple `/health` is sufficient
- [x] I2: Deleted `CliPluginTool`, `CliToolDef`, and `cli_tool` module — speculative infrastructure with no consumers
- [x] `angreal test unit` passes
- [x] `angreal check all` passes

---

## Findings (audited 2026-02-28)

Original audit had 25 items. After re-audit, 13 are resolved (code removed/rewritten since audit). 19 real items remain across 3 categories.

### Dead Code (9 items)

| # | Location | Issue | Severity |
|---|----------|-------|----------|
| D1 | `arawn/src/client/mod.rs:7` | `#![allow(dead_code)]` blanket suppression hides 4 dead methods: `with_token()`, `chat()`, `list_sessions()`, `delete_session()` | High |
| D2 | `arawn-agent/src/tool.rs:1228` | `pub MockTool` compiled unconditionally — not gated by `#[cfg(test)]` or feature flag, re-exported from crate public API | High |
| D3 | `arawn-llm/src/backend.rs:360` | `pub MockBackend` / `pub MockResponse` compiled unconditionally — same issue as D2 | High |
| D4 | `arawn-workstream/src/storage.rs:122,320` | `pub MockWorkstreamStorage` / `pub MockMessageStorage` compiled unconditionally, only used in own `#[cfg(test)]` blocks | Medium |
| D5 | `arawn-memory/src/backend.rs:130` | `pub MockMemoryBackend` compiled unconditionally, only used in own `#[cfg(test)]` blocks | Medium |
| D6 | `arawn-llm/src/anthropic.rs:257` | `ApiResponse::role` field deserialized but never read — remove field (serde ignores unknown fields by default) | Low |
| D7 | `arawn-llm/src/anthropic.rs:338,583` | `ApiErrorDetail::error_type` and `StreamErrorDetail::error_type` fields deserialized but never read | Low |
| D8 | `arawn-server/tests/common/mod.rs:107` | `start_with_memory()` stub never called — remove or implement | Low |
| D9 | `arawn-server/tests/common/mod.rs:134` | `#[allow(dead_code)]` on `delete()` is stale — method IS used now, remove annotation | Low |

**Note**: D2/D3 mock types are used by integration tests in other crates (e.g. `arawn-server/tests/common/mod.rs` imports `MockBackend`). Moving to `#[cfg(test)]` would break cross-crate usage. A `testing` feature flag is the correct fix.

### Hardcoded Values (8 items)

| # | Location | Issue | Severity |
|---|----------|-------|----------|
| H1 | `arawn-server/src/routes/openapi.rs:14` | OpenAPI version hardcoded `"1.0.0"` — use `env!("CARGO_PKG_VERSION")` | Medium |
| H2 | `arawn-agent/src/tools/web.rs:42` | User-Agent `"Arawn/0.1 (Research Agent)"` — stale version, use `env!("CARGO_PKG_VERSION")` | Medium |
| H3 | `arawn-server/src/routes/ws/protocol.rs:251` + `arawn-tui/src/ui/layout.rs:165,441` + `arawn-tui/src/ui/sidebar.rs:195` | Context thresholds `70`/`90` duplicated 4x — constants exist in `arawn-agent/src/context.rs` (`DEFAULT_WARNING_THRESHOLD`, `DEFAULT_CRITICAL_THRESHOLD`) but aren't referenced | Medium |
| H4 | `arawn/src/commands/start.rs:1187` | `api_rpm` fallback `120` — constant `defaults::REQUESTS_PER_MINUTE` exists but isn't used here | Low |
| H5 | `arawn/src/commands/start.rs:199,204` | Port `8080` / bind `"127.0.0.1"` fallbacks — same values defined in `ServerConfig` defaults but re-hardcoded | Low |
| H6 | `arawn-agent/src/context.rs:310` | `4096` reserved response tokens unnamed — define `const RESERVED_RESPONSE_TOKENS` | Low |
| H7 | `arawn-agent/src/context.rs:237` | `chars_per_token: 4` doesn't reference `CHARS_PER_TOKEN` constant defined on line 13 of same file | Low |
| H8 | `arawn-server/src/state.rs:219,254` | `Duration::from_secs(60)` WS rate window duplicated in `check_rate` and `cleanup` — extract constant | Low |

### Incomplete Implementations (2 items)

| # | Location | Issue | Severity |
|---|----------|-------|----------|
| I1 | `arawn-server/src/routes/health.rs:63` | Deep health check skips memory store validation — `agent_ready` hardcoded `true`, memory store never probed | Medium |
| I2 | `arawn-plugin/src/manager.rs` | Plugin CLI tool wiring absent — `CliPluginTool` executor works and is tested, but `load_plugin()` never discovers commands from plugin `commands/` directory or constructs `CliPluginTool` instances | Medium |

### Resolved (removed from audit)

These items from the original audit are no longer applicable:
- WS alert/notification TODO → fully implemented during ws module refactor
- `--daemon` flag → removed entirely from `agent.rs`
- `delete_workstream()` JSONL leak → no hard-delete exists; archive is non-destructive by design
- `traverse()` depth=1 → method removed; only `get_neighbors()` exists
- Summarization prompt placeholder → full production prompt with 10 tests
- Pipeline error recovery → delegated to Cloacina; indexer log-and-continue is intentional
- Keyring test coverage → resolution chain tested; real keyring path structurally untestable without mock injection (accepted limitation)
- Color codes → TUI uses `ratatui::Color` enum exclusively, no hardcoded hex/ANSI
- Unused Display impls → all found impls are either required by `std::error::Error` or serve public API ergonomics

## Status Updates

### 2026-02-28 — Dead code (D1–D9)
Committed d830d73. Removed blanket `#![allow(dead_code)]` from client, deleted unused `with_token()`/`chat()` methods, gated mock types (`MockBackend`, `MockResponse`, `MockTool`, `MockWorkstreamStorage`, `MockMessageStorage`, `MockMemoryBackend`) behind `#[cfg(test)]` or `testing` feature flag, removed unused serde fields (`role`, `error_type`), deleted `start_with_memory()` stub, removed stale `#[allow(dead_code)]` from `delete()`.

### 2026-02-28 — Hardcoded values (H2–H8)
Committed 515ce24. H1 skipped (proc macro limitation). Replaced hardcoded User-Agent version with `env!("CARGO_PKG_VERSION")`, extracted context thresholds to `arawn_types::config::defaults`, wired `REQUESTS_PER_MINUTE`/`DEFAULT_PORT`/`DEFAULT_BIND` constants into fallback sites, named `RESERVED_RESPONSE_TOKENS` and `WS_RATE_WINDOW` constants.

### 2026-03-01 — Incomplete implementations (I1–I2)
I1: Deleted the fake `/health/detailed` endpoint entirely — `agent_ready` was hardcoded `true`, memory store never probed. The simple `/health` endpoint is all that's needed for a local tool. Also removed `--detailed` flag from `arawn status` (printed "not yet implemented"), `DetailedHealthResponse`/`AgentHealth`/`HealthComponents` types from server and client, and OpenAPI registrations.

I2: Deleted `CliPluginTool` adapter, `CliToolDef` type, `default_parameters()` helper, and `cli_tool` module entirely. Plugins only need slash commands for now — the CLI tool executor was speculative infrastructure never wired into `load_plugin()`. Also removed from `lib.rs` re-exports and crate doc comment. Archived ARAWN-T-0236 (the backlog item created for wiring it up).