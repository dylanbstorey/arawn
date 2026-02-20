---
id: easy-sandbox-dependency
level: task
title: "Easy sandbox dependency installation for distribution"
short_code: "ARAWN-T-0193"
created_at: 2026-02-18T17:25:48.549342+00:00
updated_at: 2026-02-18T17:25:48.549342+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#feature"


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

## Recommended Approach

Likely a combination:
1. **Primary**: Native packages (.deb, .rpm, AUR) for major distros
2. **Fallback**: Install script for other Linux distros
3. **Container**: Official image for server deployments
4. **Nix**: Flake for Nix users

## Implementation Notes

### Dependencies
- Requires ARAWN-I-0028 (sandbox integration) to be complete first
- Requires decision on release/distribution strategy

### Considerations
- bubblewrap is LGPL - check licensing implications for bundling
- socat is GPL - same licensing review needed
- Consider using `sandbox-runtime` crate which handles some of this

## Related
- ARAWN-I-0028: Workstream and Session Path Management (sandbox integration)

## Status Updates

*To be added during implementation*