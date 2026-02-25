---
id: easy-sandbox-dependency
level: task
title: "Easy sandbox dependency installation for distribution"
short_code: "ARAWN-T-0193"
created_at: 2026-02-18T17:25:48.549342+00:00
updated_at: 2026-02-25T01:24:10.858042+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#feature"
  - "#phase/active"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Easy sandbox dependency installation for distribution

## Objective

Make sandbox dependencies (bubblewrap, socat on Linux) seamlessly available when users install Arawn, eliminating manual dependency installation.

## Context

Arawn requires sandbox for shell execution (Option C - no shell without sandbox). On Linux, this depends on `bubblewrap` and `socat` which are not installed by default on most distributions. Users shouldn't need to manually install system packages before using Arawn.

## Backlog Item Details

### Type
- [x] Feature - New functionality or enhancement

### Priority
- [x] P2 - Medium (nice to have for initial release, required for broad adoption)

### Business Justification
- **User Value**: Zero-friction installation - `cargo install arawn` or downloading a binary just works
- **Business Value**: Reduces support burden, increases adoption
- **Effort Estimate**: M (research + implementation across multiple packaging formats)

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Linux users can use shell commands without manually installing bubblewrap/socat
- [ ] macOS works out of the box (already true - uses built-in sandbox-exec)
- [ ] Clear error messages if sandbox unavailable with remediation steps
- [ ] Works across major distribution methods (see options below)

## Distribution Options to Evaluate

### Option A: Package Manager Packages
Create native packages that declare dependencies:
- `.deb` package with `Depends: bubblewrap, socat`
- `.rpm` package with `Requires: bubblewrap socat`
- AUR package with `depends=('bubblewrap' 'socat')`
- Homebrew formula (macOS - no deps needed)

**Pros**: Native, handles updates, familiar to users
**Cons**: Maintenance burden, multiple packages to maintain

### Option B: Static/Bundled Binary
Bundle bubblewrap binary with Arawn distribution:
- Download pre-built bwrap for target platform
- Ship alongside arawn binary
- Use bundled version if system version unavailable

**Pros**: Single download, works everywhere
**Cons**: Binary size, security updates require new release, licensing considerations (LGPL for bwrap)

### Option C: Container Image
Provide official Docker/Podman image with all dependencies:
```dockerfile
FROM ubuntu:24.04
RUN apt-get install -y bubblewrap socat
COPY arawn /usr/local/bin/
```

**Pros**: Guaranteed environment, easy deployment
**Cons**: Requires container runtime, overhead

### Option D: Nix Flake
Provide Nix flake for reproducible builds:
```nix
{
  packages.arawn = pkgs.callPackage ./arawn.nix {
    inherit (pkgs) bubblewrap socat;
  };
}
```

**Pros**: Reproducible, handles all deps
**Cons**: Requires Nix

### Option E: Install Script
Provide `curl | sh` style installer that:
1. Detects platform and package manager
2. Installs dependencies via appropriate method
3. Installs Arawn binary

**Pros**: Single command installation
**Cons**: Security concerns with curl|sh, requires sudo

## Chosen Approach: `curl | sh` Install Script (Option E)

### Deliverables

#### 1. `scripts/install.sh` — Main install script

Bash script invokable as `curl -fsSL <url> | sh`:

**Flags:**
- `--help` — usage info
- `--skip-deps` — skip sandbox dependency installation
- `--install-dir DIR` — target directory (default: `~/.local/bin`)
- `--dry-run` — show what would happen
- `--version VER` — specific version (default: latest)
- `--from-source` — force cargo build, skip binary download

**Flow:**
1. Detect OS (`linux`/`darwin`) and arch (`x86_64`/`aarch64`)
2. Detect package manager (`apt-get`/`dnf`/`pacman`/`apk`/`zypper`)
3. Install missing sandbox deps on Linux (bubblewrap, socat) — skip already-installed
4. Try downloading pre-built binary from GitHub Releases (`arawn-{os}-{arch}.tar.gz`)
5. Fall back to `cargo install --git <repo> arawn --root <dir>` if no binary available
6. Verify installation (`arawn --version`, `bwrap --version`, `socat -V`)
7. Print post-install instructions (PATH setup if needed)

**Design choices:**
- `~/.local/bin` default (no sudo for binary install)
- Colored output with TTY detection
- `set -euo pipefail`, trap-based temp dir cleanup
- Bash 3.2 compatible (macOS ships old bash — no associative arrays)
- Idempotent: safe to re-run

#### 2. Update `crates/arawn-sandbox/src/platform.rs` — Better error messages

Update `check_linux()` install hint (lines 154–163) to:
- Add openSUSE/zypper instructions
- Reference the install script as an alternative

#### 3. `.angreal/task_install.py` — Developer convenience

Angreal task `install deps` for developers who have the repo cloned. Follows the pattern in `task_build.py`. Detects package manager, installs missing bubblewrap/socat. No Flox wrapping (system packages are outside Flox).

### Files

| File | Action |
|------|--------|
| `scripts/install.sh` | Create |
| `crates/arawn-sandbox/src/platform.rs` | Edit lines 154–163 |
| `.angreal/task_install.py` | Create |

### Verification

1. `shellcheck scripts/install.sh` — no warnings
2. `./scripts/install.sh --dry-run` on macOS — shows skip deps, attempts binary download
3. `./scripts/install.sh --help` — prints usage
4. `cargo test -p arawn-sandbox` — existing tests pass with updated hint text
5. `angreal install deps --dry-run` — shows what would be installed

## Status Updates

### 2026-02-24 — Implementation complete

**2 deliverables implemented (angreal task dropped per user decision):**

1. **`scripts/install.sh`** — Created. Bash 3.2 compatible install script with:
   - `--help`, `--skip-deps`, `--install-dir`, `--dry-run`, `--version` flags
   - Platform detection (linux/darwin, x86_64/aarch64)
   - Package manager detection (apt-get/dnf/pacman/apk/zypper)
   - Downloads from GitHub Releases only (latest or tagged version)
   - TTY-aware colored output, trap-based cleanup
   - Repo namespace: `colliery-io/arawn`

2. **`crates/arawn-sandbox/src/platform.rs`** — Updated `check_linux()` install hint:
   - Added openSUSE/zypper instructions
   - Added install script URL reference
   - Updated repo namespace to `colliery-io/arawn`

**Verification results:**
- `./scripts/install.sh --help` — prints usage correctly
- `./scripts/install.sh --dry-run --version v0.1.0` — shows correct download URL
- `./scripts/install.sh --dry-run` (latest) — errors clearly when no releases exist yet
- `angreal test unit` — all tests pass (incl. 14 arawn-sandbox tests)