---
id: 001-fsgate-centralized-filesystem
level: adr
title: "FsGate Centralized Filesystem Enforcement"
number: 1
short_code: "ARAWN-A-0006"
created_at: 2026-03-05T05:16:02.167739+00:00
updated_at: 2026-03-05T05:19:00.628737+00:00
decision_date: 
decision_maker: 
parent: 
archived: false

tags:
  - "#adr"
  - "#phase/decided"


exit_criteria_met: false
initiative_id: NULL
---

# ADR-6: FsGate Centralized Filesystem Enforcement

## Context

Agent tools (`file_read`, `file_write`, `glob`, `grep`, `shell`) have direct filesystem access. Without enforcement, an agent can read or write any file the user's process can access — including `~/.ssh/`, `/etc/`, other workstreams' data, or system files. This is a security problem:

- **Prompt injection**: A malicious prompt could instruct the agent to read sensitive files and exfiltrate them via a tool call
- **Accidental damage**: An agent could overwrite files outside its intended working directory
- **Cross-workstream leakage**: Sessions in one workstream could read another workstream's data

Arawn already has the building blocks for enforcement — `PathValidator` validates paths against allowed directories, `SandboxManager` executes shell commands in OS-level sandboxes, and `DirectoryManager` knows the allowed paths for each workstream/session. What's missing is a **centralized enforcement point** that gates every filesystem-touching tool call through these validators.

The sandbox boundary must be the **workstream**, not the session:
- **Named workstreams**: All sessions share access to the workstream's `production/` and `work/` directories
- **Scratch workstreams**: Each session is isolated to its own `scratch/sessions/<id>/work/` directory

## Decision

Introduce a **`FsGate` trait** as a centralized filesystem access gate, enforced at a single chokepoint in `ToolRegistry::execute_with_config()`. No filesystem-touching tool can bypass this gate.

### Architecture

```
Agent turn
  → ToolRegistry::execute_with_config()
    → is_gated_tool("file_read")? Yes
    → ctx.fs_gate present?
      → Yes: validate paths, then execute tool
      → No: DENY (return error)
    → is_gated_tool("shell")? Yes
    → Route through gate.sandbox_execute() instead of direct execution
```

### FsGate Trait (`arawn-types`)

```rust
pub trait FsGate: Send + Sync {
    fn validate_read(&self, path: &Path) -> Result<PathBuf, FsGateError>;
    fn validate_write(&self, path: &Path) -> Result<PathBuf, FsGateError>;
    fn working_dir(&self) -> &Path;
    async fn sandbox_execute(&self, command: &str, timeout: Option<Duration>)
        -> Result<SandboxOutput, FsGateError>;
}
```

### Gated Tools

Five tools are classified as gated: `file_read`, `file_write`, `glob`, `grep`, `shell`. This list is defined in `arawn-types` via `is_gated_tool()`.

### Enforcement Rules

1. **File tools** (`file_read`, `glob`, `grep`): Path parameter is validated via `gate.validate_read()` before execution. Returns canonicalized path or access denied error.
2. **Write tools** (`file_write`): Path validated via `gate.validate_write()` with symlink escape detection.
3. **Shell** (`shell`): Command is first checked against a blocklist (fork bombs, `rm -rf /`, sandbox escapes), then routed through `gate.sandbox_execute()` which uses OS-level sandboxing.
4. **Deny by default**: If a gated tool is called without a gate configured, execution is denied with a clear error message. No silent fallthrough to unrestricted access.

### Defense in Depth

Three layers of protection for shell commands:
1. **Command validator**: Regex-based blocklist catches clearly dangerous patterns (fork bombs, `rm -rf /`, `mkfs`, `shutdown`, sandbox escape tools)
2. **FsGate path validation**: `PathValidator` checks against allowed directories with symlink escape detection and denied system paths
3. **OS-level sandbox**: `SandboxManager` enforces filesystem boundaries at the OS level (macOS `sandbox-exec`, Linux namespaces)

### Gate Resolution

A `FsGateResolver` closure is set on the `Agent` at startup:

```rust
pub type FsGateResolver = Arc<dyn Fn(&str, &str) -> Option<Arc<dyn FsGate>> + Send + Sync>;
```

Takes `(session_id, workstream_id)` and returns a gate configured for that context. The `DirectoryManager::allowed_paths()` method determines the sandbox boundary based on workstream type.

## Alternatives Analysis

| Option | Pros | Cons | Risk Level |
|--------|------|------|------------|
| **Centralized gate (chosen)** | Single enforcement point, deny-by-default, easy to audit | All tools go through one path, potential bottleneck | Low |
| **Per-tool validation** | Each tool handles its own security | Tools can forget to validate, no deny-by-default, scattered logic | High |
| **Middleware/decorator pattern** | Clean separation | Harder to implement for async tool trait, tool wrapping complexity | Medium |
| **OS-only sandbox (no app-level)** | Strongest isolation | No path validation feedback to agent, platform-specific, coarse-grained | Medium |
| **Container-per-session** | Complete isolation | Heavy startup cost, Docker dependency, not laptop-friendly | High |

## Rationale

- **Single point of control**: Security reviews need to examine one function (`execute_with_config`), not five separate tool implementations. If the gate is correct, all tools are protected.
- **Deny by default**: The most critical property. If someone adds a new gated tool and forgets to configure the gate, execution fails safely rather than silently allowing unrestricted access.
- **Tools stay simple**: `FileReadTool`, `GlobTool`, etc. don't need to know about workstreams, sandboxes, or path validation. They receive pre-validated parameters and execute.
- **Defense in depth**: Three independent layers (command blocklist → path validation → OS sandbox) means a bug in one layer doesn't compromise security.
- **Workstream-scoped boundaries**: The gate carries the correct allowed paths for each workstream type, enforced consistently across all tools in that session.

## Consequences

### Positive
- All filesystem-touching tools are protected by a single, auditable enforcement point
- Deny-by-default prevents silent security regressions when adding new tools
- Tools remain simple and focused on their functionality
- Defense in depth — three independent layers of protection
- Clear error messages when access is denied (tool, path, reason)

### Negative
- All gated tool calls require a gate to be configured, adding setup complexity at startup
- The centralized path adds slight overhead to every gated tool call (path validation + canonicalization)
- Shell commands take a different execution path (sandbox) than non-gated tools, which could cause subtle behavioral differences

### Neutral
- Non-gated tools (e.g., `memory_search`, `web_fetch`) are unaffected — they pass through without gate checks
- CLI commands that run outside workstream context (e.g., `arawn chat`) don't have a gate, so gated tools are unavailable in that context
- The denied system paths list is hardcoded and may need updating for new platforms