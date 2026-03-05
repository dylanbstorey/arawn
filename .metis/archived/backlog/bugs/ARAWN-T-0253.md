---
id: prevent-secret-values-from-leaking
level: task
title: "Prevent secret values from leaking into trace logs"
short_code: "ARAWN-T-0253"
created_at: 2026-03-04T13:23:53.615602+00:00
updated_at: 2026-03-04T18:20:11.297219+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#bug"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
---

# Prevent secret values from leaking into trace logs

## Objective

Ensure secret values resolved from `${{secrets.<name>}}` handles are never written to trace/log output. Currently, tool parameters are logged after secret resolution, potentially exposing plaintext secrets in log files.

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing

### Priority
- [x] P2 - Medium (nice to have)

### Impact Assessment
- **Severity**: MEDIUM - Secrets could end up in rotating log files on disk
- **Expected vs Actual**: Resolved secret values should be redacted in logs; currently the resolved params may be traced

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Tool params are logged *before* secret resolution, not after (confirmed — architecture already correct)
- [x] Or: resolved secrets are redacted (replaced with `***`) in any log output (added redacted Debug impls as defense-in-depth)
- [x] Log files do not contain plaintext secret values (confirmed via comprehensive audit)
- [x] Unit test verifying redaction (3 tests added)

## Implementation Notes

### Key Files
- `crates/arawn-agent/src/tool.rs` — `execute_with_config()` and `execute_raw()`
- Tracing spans/events that log tool parameters

## Status Updates

### Investigation Complete — Code Already Safe (Architecture Review)

**Comprehensive audit of all tracing paths:**

1. **agent.rs lines 624-632**: Tool input debug log only records `input_bytes` and `input_tokens` — NOT the actual parameter string. The `input_str` variable is created from `tool_use.input` (original params with `${{secrets.*}}` handles, not resolved values), and only its `.len()` is logged. **SAFE**.

2. **agent.rs line 288**: Only logs tool names (comma-joined), not params. **SAFE**.

3. **agent.rs lines 605-608**: Hook block reason only logs tool name and reason. **SAFE**.

4. **stream.rs line 296**: Only logs `tool_use.name` and error message. **SAFE**.

5. **tool.rs `resolve_secret_handles()`**: Called INSIDE `execute_with_config()` (line 1222) and `execute_raw()` (line 1270), AFTER the caller has captured logging params. Comment at line 1404-1405 explicitly states "The original params (with handles, not values) are what get logged by the caller." **SAFE**.

6. **interaction_log.rs**: `ToolCallRecord` only contains `tool_name` and `call_id` — no input params. **SAFE**.

7. **Hook dispatchers (agent.rs lines 601, 669)**: Receive `tool_use.input` which is original (handle-containing) params. **SAFE**.

8. **ToolCall struct (agent.rs line 591)**: Stores `tool_use.input.clone()` — original, pre-resolution. **SAFE**.

### Defense-in-Depth Hardening Applied

While the architecture is already safe, added redacted `Debug` impls to prevent accidental future leaks:

1. **`ResolvedSecret`** (`crates/arawn-config/src/secrets.rs`): Replaced `#[derive(Debug)]` with custom `Debug` impl that shows `[REDACTED]` instead of the secret value.

2. **`ResolvedLlm`** (`crates/arawn-config/src/resolver.rs`): Replaced `#[derive(Debug)]` with custom `Debug` impl that shows `[REDACTED]` instead of `api_key` when present, `None` when absent.

3. **`AgeSecretStore`** (`crates/arawn-config/src/secret_store.rs`): Already had a custom `Debug` impl that only shows `secret_count` — no changes needed.

### Tests Added (3 new, all passing)
- `test_resolved_secret_debug_redacts_value` — verifies secret value absent from Debug output
- `test_resolved_llm_debug_redacts_api_key` — verifies API key absent from Debug output  
- `test_resolved_llm_debug_no_key` — verifies Debug works correctly when no key is present

### Verification
- `angreal check all` — clippy + fmt clean
- `angreal test unit` — all tests pass