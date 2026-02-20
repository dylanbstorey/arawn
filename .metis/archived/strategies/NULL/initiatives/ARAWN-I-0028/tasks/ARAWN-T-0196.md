---
id: shell-sandbox-integration
level: task
title: "Shell sandbox integration"
short_code: "ARAWN-T-0196"
created_at: 2026-02-18T19:03:13.063620+00:00
updated_at: 2026-02-18T19:40:57.730500+00:00
parent: ARAWN-I-0028
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0028
---

# Shell sandbox integration

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[ARAWN-I-0028]]

## Objective

Integrate the `sandbox-runtime` crate for shell command sandboxing, making sandbox required for all shell execution with no fallback to unsandboxed mode.

## Backlog Item Details **[CONDITIONAL: Backlog Item]**

{Delete this section when task is assigned to an initiative}

### Type
- [ ] Bug - Production issue that needs fixing
- [ ] Feature - New functionality or enhancement  
- [ ] Tech Debt - Code improvement or refactoring
- [ ] Chore - Maintenance or setup work

### Priority
- [ ] P0 - Critical (blocks users/revenue)
- [ ] P1 - High (important for user experience)
- [ ] P2 - Medium (nice to have)
- [ ] P3 - Low (when time permits)

### Impact Assessment **[CONDITIONAL: Bug]**
- **Affected Users**: {Number/percentage of users affected}
- **Reproduction Steps**: 
  1. {Step 1}
  2. {Step 2}
  3. {Step 3}
- **Expected vs Actual**: {What should happen vs what happens}

### Business Justification **[CONDITIONAL: Feature]**
- **User Value**: {Why users need this}
- **Business Value**: {Impact on metrics/revenue}
- **Effort Estimate**: {Rough size - S/M/L/XL}

### Technical Debt Impact **[CONDITIONAL: Tech Debt]**
- **Current Problems**: {What's difficult/slow/buggy now}
- **Benefits of Fixing**: {What improves after refactoring}
- **Risk Assessment**: {Risks of not addressing this}

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Add `sandbox-runtime` crate as dependency
- [x] `SandboxManager` struct wrapping sandbox-runtime functionality
- [x] `check_availability()` detects if sandbox deps are installed
- [x] `execute(command, allowed_paths)` runs command in sandbox
- [x] Shell execution fails with clear error if sandbox unavailable
- [x] Platform detection (macOS: sandbox-exec, Linux: bubblewrap)
- [x] Helpful error messages with installation instructions
- [x] Integration tests on both macOS and Linux

## Implementation Notes

### Location
`crates/arawn-agent/src/sandbox.rs` (new file) or new `arawn-sandbox` crate

### Core API

```rust
pub struct SandboxManager {
    available: bool,
    platform: Platform,
}

pub enum Platform {
    MacOS,      // Uses sandbox-exec (built-in)
    Linux,      // Uses bubblewrap + socat
    Unsupported,
}

pub enum SandboxStatus {
    Available,
    MissingDependency { missing: Vec<String>, install_hint: String },
    Unsupported { platform: String },
}

impl SandboxManager {
    pub fn new() -> Result<Self, SandboxError>;
    
    pub fn check_availability() -> SandboxStatus;
    
    pub fn execute(
        &self,
        command: &str,
        working_dir: &Path,
        allowed_write_paths: &[PathBuf],
        allowed_read_paths: &[PathBuf],
    ) -> Result<CommandOutput, SandboxError>;
}
```

### Filesystem Model (per sandbox-runtime)

| Access | Pattern | Description |
|--------|---------|-------------|
| Read | Deny-only | Everything allowed except blocked paths |
| Write | Allow-only | Nothing allowed except explicit paths |

### Blocked Read Paths (default)
- `~/.ssh`, `~/.gnupg`, `~/.aws`
- `/etc/shadow`, `/etc/passwd`
- `~/.arawn/config`

### Error Messages

```
Sandbox unavailable: bubblewrap not found

Shell commands require sandboxing for security.
Install dependencies:
  Ubuntu/Debian: sudo apt-get install bubblewrap socat
  Fedora:        sudo dnf install bubblewrap socat
  Arch:          sudo pacman -S bubblewrap socat
```

### Dependencies
- ARAWN-T-0194 (DirectoryManager)
- ARAWN-T-0195 (PathValidator)
- `sandbox-runtime` crate

### Risk Considerations
- Must test on both macOS and Linux CI
- Ensure child processes inherit sandbox restrictions
- Handle sandbox-runtime crate API changes

## Status Updates

### 2026-02-18 - Implementation Complete

**Created**: New crate `crates/arawn-sandbox/`

**Files**:
- `Cargo.toml` - Dependencies: sandbox-runtime, tokio, serde, thiserror, dirs
- `src/lib.rs` - Crate exports
- `src/error.rs` - `SandboxError` enum with clear messages
- `src/platform.rs` - `Platform` enum and `SandboxStatus` detection
- `src/config.rs` - `SandboxConfig` for execution options
- `src/manager.rs` - `SandboxManager` with execute method

**Public API**:
```rust
// Check availability
let status = SandboxManager::check_availability();
if !status.is_available() {
    // status.install_hint() returns installation instructions
}

// Create manager (fails if sandbox unavailable)
let manager = SandboxManager::new().await?;

// Configure and execute
let config = SandboxConfig::default()
    .with_write_paths(vec![work_dir])
    .with_timeout(Duration::from_secs(30));

let output = manager.execute("echo hello", &config).await?;
```

**Platform Support**:
| Platform | Backend | Detection |
|----------|---------|-----------|
| macOS | sandbox-exec | Built-in, checks with `which` |
| Linux | bubblewrap + socat | Checks for `bwrap` and `socat` |

**Error Messages** (for Linux missing deps):
```
Sandbox unavailable: Missing dependencies: bubblewrap, socat

Shell commands require sandboxing for security.
Install dependencies:

  Ubuntu/Debian: sudo apt-get install bubblewrap socat
  Fedora:        sudo dnf install bubblewrap socat
  Arch:          sudo pacman -S bubblewrap socat
  Alpine:        sudo apk add bubblewrap socat
```

**Tests**: 17 tests (14 unit + 3 integration), all passing
- Platform detection tests
- Config builder tests
- Manager creation tests
- Integration tests (sandboxed echo, write allowed, write denied)

**Workspace updated**:
- Added `arawn-sandbox` to workspace members
- Added `arawn-sandbox` to workspace dependencies