---
id: project-scaffold-and-workspace
level: initiative
title: "Project Scaffold and Workspace Setup"
short_code: "ARAWN-I-0001"
created_at: 2026-01-28T01:37:21.963241+00:00
updated_at: 2026-01-28T01:37:21.963241+00:00
parent: ARAWN-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: S
strategy_id: NULL
initiative_id: project-scaffold-and-workspace
---

# Project Scaffold and Workspace Setup

## Context

Foundation for all other initiatives. Sets up the Cargo workspace structure, CI, and development environment before any feature work begins.

## Goals & Non-Goals

**Goals:**
- Cargo workspace with multi-crate structure
- CI pipeline (GitHub Actions) for build, test, lint
- Cross-compilation setup for ARM64 (Raspberry Pi)
- Development tooling (justfile/Makefile, formatting, linting)
- Basic project documentation (README, CONTRIBUTING)

**Non-Goals:**
- Any feature implementation
- Release/publishing automation (not publishing to crates.io)

## Detailed Design

### Workspace Structure

```
arawn/
├── Cargo.toml              # Workspace root
├── Cargo.lock
├── .github/
│   └── workflows/
│       └── ci.yml          # Build, test, lint, cross-compile check
├── .angreal/               # Angreal task automation
│   ├── init.py
│   └── task_*.py           # Build, test, release tasks
├── rust-toolchain.toml     # Pin Rust version
├── .cargo/
│   └── config.toml         # Cross-compilation targets
│
├── crates/
│   ├── arawn/              # Main binary + agent core
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs
│   │
│   ├── arawn-memory/       # Knowledge store (sqlite-vec + graphqlite)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs
│   │
│   ├── arawn-llm/          # LLM proxy + embeddings
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs
│   │
│   └── arawn-types/        # Shared types, config
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs
│
├── README.md
└── .metis/                 # Project management (already exists)
```

### CI Pipeline

- **Build**: `cargo build --workspace`
- **Test**: `cargo test --workspace`
- **Lint**: `cargo clippy --workspace -- -D warnings`
- **Format**: `cargo fmt --check`
- **Cross-compile check**: `cargo check --target aarch64-unknown-linux-gnu`

### Tooling

- **angreal**: Task automation (build, test, release, cross-compile)
- **cargo-deny**: Dependency auditing
- **rustfmt**: Formatting (with config)
- **clippy**: Linting

### Angreal Tasks

- `angreal build` - Build workspace (debug/release)
- `angreal test` - Run tests with optional coverage
- `angreal lint` - Clippy + format check
- `angreal cross` - Cross-compile for ARM64
- `angreal ci` - Full CI pipeline locally

## Alternatives Considered

- **Single crate**: Rejected - want compile isolation as code stabilizes
- **just/Makefile**: Angreal preferred for consistency with colliery-io tooling

## Implementation Plan

1. Initialize Cargo workspace with empty crates
2. Set up angreal with core tasks
3. Add CI workflow (calls angreal tasks)
4. Configure cross-compilation
5. README with setup instructions