---
id: arawn-personal-research-agent-for
level: vision
title: "Arawn: Personal Research Agent for Edge Computing"
short_code: "ARAWN-V-0001"
created_at: 2026-01-27T18:50:33.850207+00:00
updated_at: 2026-01-28T01:30:15.624120+00:00
archived: false

tags:
  - "#vision"
  - "#phase/published"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Arawn Vision

A high-performance personal research agent written in Rust, designed to run on edge devices (Raspberry Pi 4/5, small single-board computers) while providing autonomous research capabilities, long-running task execution, and persistent knowledge management.

## Purpose

Build a lightweight, secure, self-hosted **personal AI agent** that can:

1. Hold natural conversations and answer questions with full context awareness
2. Execute tasks autonomously - from quick lookups to multi-hour research projects
3. Maintain persistent memory across sessions (semantic search + knowledge graph)
4. Capture and organize notes, ideas, and information as you work
5. Help plan projects, break down goals, and track progress
6. Proactively notify you when tasks complete or when something needs attention
7. Manage aspects of your digital life (calendar, email, git, file systems)
8. Prototype and iterate on software through sandboxed code execution
9. Run 24/7 on resource-constrained edge hardware you control

This is a Rust reimagining of moltbot's core value proposition - a personal AI that works for you, remembers context, and takes action on your behalf - rebuilt for performance, security, and edge deployment. We're dropping the messaging platform integrations (WhatsApp, Discord, etc.) in favor of direct interfaces (CLI, mobile apps) and focusing on what matters: an agent that thinks, acts, and persists.

## Current State

- moltbot exists as a TypeScript/Node.js personal assistant with many messaging integrations
- ~410K LOC, ~150-300MB memory baseline, requires Node 22+
- Heavy dependencies (Playwright, Baileys, Sharp) make it unsuitable for Pi Zero/edge
- Recent security exposure (clawdbot incident) highlighted risks of convenience-first auth defaults
- colliery-io ecosystem provides foundational crates: muninn (Claude OAuth), graphqlite (graph DB), cloacina (workflow orchestration)

## Future State

A single Rust binary (~15-25MB) that:

- Runs comfortably on Raspberry Pi 4/5 with 8GB RAM
- Provides HTTP API + WebSocket for CLI and mobile clients (iPad, Android)
- Persists all knowledge in a single SQLite file (sqlite-vec + graphqlite)
- Executes research workflows autonomously via cloacina
- Sandboxes code execution via bubblewrap
- Requires authentication by default with no localhost bypass
- Supports Tailscale mesh networking for secure remote access
- Pushes notifications via ntfy.sh for task completion

## Major Features

### Core Agent
- Research planning and task decomposition
- Tool execution (web search, file operations, code execution)
- Autonomous multi-step research loops
- Interruptible and resumable workflows

### Memory Layer
- **sqlite-vec**: Semantic vector search over conversation and research history
- **graphqlite**: Knowledge graph for fact association, entity relationships, source tracking
- Single SQLite database file for all persistent state

### LLM Integration
- Claude OAuth proxy (muninn pattern) for flat-rate usage with Claude Pro/Max
- Configurable embedding providers (local all-MiniLM default, remote optional)
- Model routing for cost optimization

### Task Orchestration
- cloacina integration for workflow definition and execution
- Checkpointing for long-running research tasks
- Automatic retry and failure recovery
- Background task execution with progress tracking

### Code Execution
- bubblewrap sandbox for untrusted code
- Resource limits (memory, CPU, time)
- Trust levels: sandboxed (default), approved, isolated (Docker optional)

### Integrations
- Git operations (git2-rs)
- Calendar sync (CalDAV)
- Email monitoring (IMAP/JMAP)
- File system watching (notify-rs)
- System metrics and alerts (sysinfo)

### Transport
- axum HTTP server with REST API and SSE streaming
- WebSocket for bidirectional real-time communication
- Mobile client support (iPad, Android)

### Notifications
- ntfy.sh integration for push notifications
- Task completion alerts
- System alerts (disk pressure, high memory, etc.)

## Success Criteria

1. **Performance**: Runs stable on Pi 4 (8GB) with <500MB memory under normal load
2. **Binary Size**: Single static binary under 30MB
3. **Startup**: Cold start under 2 seconds
4. **Security**: Zero authentication bypass vulnerabilities; encrypted credential storage
5. **Autonomy**: Can complete multi-hour research tasks unattended with checkpointing
6. **Usability**: CLI + mobile clients provide full functionality

## Principles

### Security by Default
- Authentication required for all endpoints (no localhost trust)
- Credentials encrypted at rest
- Refuse to run as root
- Sandboxed code execution by default
- Explicit opt-in for dangerous features

### Edge-First Design
- Minimize memory footprint
- Single-file database (portable, backupable)
- Local-first with optional cloud connectivity
- Graceful degradation on resource constraints

### Simplicity Over Features
- No messaging platform integrations (not WhatsApp, Discord, etc.)
- Focus on research and task execution
- Clear module boundaries
- Practical naming (no excessive metaphors)

### Composability
- Leverage colliery-io ecosystem (muninn, graphqlite, cloacina)
- Standard protocols (HTTP, WebSocket, SQLite)
- Tailscale for networking rather than custom tunneling

## Constraints

### Technical
- Rust (stable toolchain)
- Must compile for ARM64 (Raspberry Pi)
- SQLite as sole database (no Postgres dependency for core)
- No heavy runtimes (no Node.js, no JVM)

### Resources
- Target: 8GB RAM systems (Pi 4/5)
- Minimum: 4GB RAM (degraded performance acceptable)
- Storage: <100MB for binary + dependencies

### Scope Exclusions
- No messaging platform integrations (WhatsApp, Discord, Telegram, etc.)
- No voice/audio processing in v1
- No multi-user/multi-tenant support
- No web UI in v1 (CLI + mobile apps only)