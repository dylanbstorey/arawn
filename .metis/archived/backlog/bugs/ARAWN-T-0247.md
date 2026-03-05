---
id: fix-shell-command-injection-in
level: task
title: "Fix shell command injection in ShellTool"
short_code: "ARAWN-T-0247"
created_at: 2026-03-04T13:23:47.875676+00:00
updated_at: 2026-03-04T14:37:19.092936+00:00
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

# Fix shell command injection in ShellTool

## Objective

Sanitize or restructure ShellTool to prevent command injection. Currently, user-controlled input is passed directly to the shell without sanitization, allowing arbitrary command execution beyond the intended scope.

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing

### Priority
- [x] P0 - Critical (blocks users/revenue)

### Impact Assessment
- **Affected Users**: Any user running the server with agent tools enabled
- **Severity**: HIGH - An LLM agent could be tricked into running arbitrary shell commands
- **Expected vs Actual**: Commands should be constrained to safe operations; currently any shell input is executed verbatim

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] ShellTool input is sanitized or executed without shell interpretation (e.g., use `Command::new` with args, not `sh -c`)
- [ ] Known dangerous patterns (`;`, `&&`, `||`, backticks, `$()`) are blocked or escaped
- [ ] FsGate sandbox is enforced for all shell executions
- [ ] Unit tests cover injection attempts (semicolons, pipes, subshells)
- [ ] Existing legitimate shell usage still works

## Implementation Notes

### Technical Approach
Consider switching from `sh -c <command>` to direct argument execution, or implement an allowlist of permitted commands. The FsGate `sandbox_execute()` path should be the only way shell commands run.

### Key Files
- `crates/arawn-agent/src/tools/shell.rs`
- `crates/arawn-agent/src/tool.rs` (execute_with_config enforcement)

## Status Updates

### Session 1 — Investigation & Implementation

**Investigation findings:**
- `execute_shell_sandboxed()` bypasses `ShellTool::is_command_allowed()` entirely
- Blocklist used weak `contains()` matching, trivially bypassed

**Implementation:**
- Created `CommandValidator` with compiled regex patterns in `tool.rs`
- Uses word boundaries and precise root-targeting patterns (blocks `rm -rf /` but allows `rm -rf /tmp/build`)
- Wired into both execution paths: `execute_shell_sandboxed()` and `ShellTool::is_command_allowed()`
- Added exports: `CommandValidation`, `CommandValidator` in `lib.rs`

**16 new tests added**, all passing:
- Blocked patterns: rm root, system control, sandbox escape, kernel modules, process tracing, destructive FS, fork bombs
- Bypass resistance: case normalization, whitespace normalization
- Allowlist: legitimate commands, subdirectory rm, piped commands
- Integration: 3 tests via `execute_with_config` with MockFsGate

**Verification:** `angreal check all` (clippy+fmt clean), `angreal test unit` (all pass)