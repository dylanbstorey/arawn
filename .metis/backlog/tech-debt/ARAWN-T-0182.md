---
id: wire-missing-configuration
level: task
title: "Wire missing configuration parameters"
short_code: "ARAWN-T-0182"
created_at: 2026-02-14T14:58:06.514967+00:00
updated_at: 2026-02-15T19:44:00.566234+00:00
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

# Wire missing configuration parameters

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Objective

Wire existing configuration parameters that are defined but not used, and add missing config sections for hardcoded operational values discovered in configuration audit.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P1 - High (important for user experience)

### Technical Debt Impact
- **Current Problems**: 
  - Config fields exist but are ignored (rate_limiting, request_logging, max_iterations)
  - Operational parameters hardcoded with no way to tune (timeouts, cache sizes, retry config)
  - Rate limiter has mismatch between config defaults and actual implementation
- **Benefits of Fixing**: 
  - Users can tune behavior via config without code changes
  - Consistent behavior between config documentation and runtime
  - Better operational control in production environments
- **Risk Assessment**: 
  - Low risk of not fixing immediately, but causes confusion and limits operational flexibility

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

### Phase 1: Wire Existing Config (Must Have)
- [x] Wire `config.server.rate_limiting` in `start.rs:940` (currently hardcoded `true`)
- [x] Wire `config.server.request_logging` in `start.rs:940` (currently hardcoded `true`)
- [x] Wire `config.agent.max_iterations` through AgentBuilder in `start.rs:790` - cascade pattern: agent-specific → agent.default → hardcoded (25)
- [x] Fix rate limiter mismatch in `ratelimit.rs:174` (uses hardcoded 120, ignores config RPM)

### Phase 2: Add Missing Config Sections (Should Have)
- [x] Add `[session]` config section with `max_sessions` (default 10000) and `cleanup_interval_secs` (default 60)
- [x] Add `[tools.output]` config with `max_size_bytes` (default 102400)
- [x] Add `[tools.shell]` and `[tools.web]` config with `timeout_secs` (default 30)
- [x] Add `[llm]` retry config: `retry_max` (default 3), `retry_backoff_ms` (default 500)

### Phase 3: Wire New Config (Should Have)
- [x] Wire session config to `arawn-server::SessionCache` via config traits
- [x] Wire tool configs to respective tool implementations
- [x] Wire LLM retry config to `arawn-llm` backends

### Verification
- [x] All tests pass (`angreal test all`)
- [x] Config reference docs updated (docs/src/configuration/reference.md)
- [x] Example config file reviewed (arawn.toml uses defaults, no changes needed)

## Implementation Notes

### Files to Modify

**Phase 1 (Wire Existing):**
- `crates/arawn/src/commands/start.rs:940` - Use config values instead of hardcoded `true`
- `crates/arawn/src/commands/start.rs:790` - Add `.with_max_iterations()` from config
- `crates/arawn-server/src/ratelimit.rs:174` - Use config RPM values

**Phase 2 (Add Config):**
- `crates/arawn-config/src/types.rs` - Add `SessionConfig`, `ToolsConfig`, extend `LlmConfig`

**Phase 3 (Wire New):**
- `crates/arawn/src/commands/start.rs` - Wire new config sections
- `crates/arawn-session/src/config.rs` - Accept external config
- `crates/arawn-agent/src/tools/*.rs` - Accept timeout config
- `crates/arawn-llm/src/anthropic.rs` - Accept retry config

### Audit Reference

| Parameter | Current | Config Field | File:Line |
|-----------|---------|--------------|-----------|
| rate_limiting | hardcoded `true` | `server.rate_limiting` | start.rs:940 |
| request_logging | hardcoded `true` | `server.request_logging` | start.rs:940 |
| max_iterations | default 10 | `agent.max_iterations` | types.rs:350 |
| rate limiter RPM | hardcoded 120 | `server.rate_limiting.*` | ratelimit.rs:174 |
| max_sessions | 10000 | NEW `session.max_sessions` | config.rs:7 |
| cleanup_interval | 60s | NEW `session.cleanup_interval` | config.rs:36 |
| tool_output_max | 100KB | NEW `tools.output.max_size` | tool.rs:570 |
| shell_timeout | 30s | NEW `tools.shell.timeout` | shell.rs |
| web_timeout | 30s | NEW `tools.web.timeout` | web.rs |
| retry_max | 3 | NEW `llm.retry_max` | anthropic.rs:61 |
| retry_backoff | 500ms | NEW `llm.retry_backoff_ms` | anthropic.rs:62 |

### Risk Considerations
- Changing defaults could affect existing deployments - maintain backward compatibility
- Rate limiter fix may change behavior for users relying on current (buggy) behavior

## Status Updates

- **2026-02-14**: Created from configuration audit. Identified 4 critical wiring gaps and 7 missing config sections.
- **2026-02-14**: Started Phase 1. Wired `rate_limiting` and `request_logging` from config in start.rs:938-944.
- **2026-02-14**: **Phase 1 COMPLETE**. All items wired:
  - `rate_limiting` and `request_logging`: wired in start.rs:948-955
  - `max_iterations`: Changed default from 10→25 in arawn-agent/types.rs. Added cascade pattern in AgentSpawner and PluginSubagentSpawner: agent-specific config overrides agent.default which overrides hardcoded fallback
  - `api_rpm`: Added to arawn-config ServerConfig and arawn-server ServerConfig. Rate limiter now created from config in AppState::new() instead of using global static with hardcoded 120
  - Tests passing
- **2026-02-15**: **Phase 2 COMPLETE**. Added config sections to arawn-config/types.rs:
  - `SessionConfig`: max_sessions (10000), cleanup_interval_secs (60)
  - `ToolsConfig`: output.max_size_bytes (102400), shell.timeout_secs (30), web.timeout_secs (30)
  - `LlmConfig`: Added retry_max and retry_backoff_ms fields
  - Added to ArawnConfig, RawConfig, and merge logic
- **2026-02-15**: **Phase 3 PARTIAL**. LLM retry config wired:
  - Extended ResolvedLlm with retry_max and retry_backoff_ms
  - Added with_retry_backoff() to OpenAiConfig
  - Updated create_backend() in start.rs to apply retry config to all backend types
  - Session and tool configs deferred (require more extensive changes to AppState and ToolRegistry)
- **2026-02-15**: **Verification COMPLETE**. Updated docs/src/configuration/reference.md with:
  - LLM retry_max/retry_backoff_ms settings
  - Server rate_limiting/api_rpm/request_logging settings  
  - Session cache config section
  - Tools config section (output, shell, web)
  - All tests passing
- **2026-02-15**: **Session config wired via traits**:
  - Created config trait system in `arawn-types/src/config.rs`: `ConfigProvider`, `HasSessionConfig`, `HasToolConfig`, `HasAgentConfig`, `HasRateLimitConfig`
  - Implemented `HasSessionConfig` for `arawn-config::SessionConfig`
  - Implemented `HasToolConfig` for `arawn-config::ToolsConfig`
  - Added `from_session_config()` to `SessionCache` accepting `impl HasSessionConfig`
  - Added `with_session_config()` to `AppState` enabling decoupled config passing
  - Wired session config in start.rs after workstreams setup
- **2026-02-15**: **Tool config wired**:
  - Created `ShellConfig` with timeout from `tools.shell.timeout_secs`
  - Created `WebFetchConfig` with timeout from `tools.web.timeout_secs` and max_text_length from `tools.output.max_size_bytes`
  - Registered tools using `with_config()` instead of `new()`
- **2026-02-15**: **ALL PHASES COMPLETE**. Task ready for completion.