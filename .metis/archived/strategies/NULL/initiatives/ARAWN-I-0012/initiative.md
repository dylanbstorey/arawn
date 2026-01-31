---
id: cloacina-integration-resilient
level: initiative
title: "Cloacina Integration: Resilient Execution and Orchestration Layer"
short_code: "ARAWN-I-0012"
created_at: 2026-01-29T01:21:48.294269+00:00
updated_at: 2026-01-30T03:40:45.799229+00:00
parent: ARAWN-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: L
strategy_id: NULL
initiative_id: cloacina-integration-resilient
---

# Cloacina Integration: Resilient Execution and Orchestration Layer

## Context

Arawn needs a general-purpose execution backbone for resilient, async, and scheduled work. Rather than building custom retry logic, background job queues, and cron scheduling into Arawn itself, we embed **Cloacina** — a Rust-native workflow orchestration engine that provides:

- **Database-backed state persistence** (SQLite or PostgreSQL)
- **Automatic retries with configurable delay** (`retry_attempts`, `retry_delay_ms`)
- **Task dependency DAGs** — tasks declare dependencies, engine resolves execution order
- **Cron scheduling** — guaranteed scheduled execution with recovery
- **Event triggers** — user-defined polling functions that fire workflows on conditions
- **Context passing** — JSON-serializable data flows between tasks via `Context<Value>`
- **Async-first** — all tasks are async, runs on tokio

Cloacina embeds directly as a crate dependency — no external service, no RPC. This makes it the natural execution layer for:

- **Post-session indexing** (I-0017): entity extraction + summarization as a Cloacina workflow triggered on session close
- **Memory reindexing**: batch re-embedding as a resilient workflow with per-batch retry
- **Scheduled agent tasks**: cron-scheduled research, digests, reminders
- **Background processing**: any work that shouldn't block the request path

The `web_fetch` HTTP extension work originally in this initiative has been moved to ARAWN-T-0071 (backlog feature).

**Related ADRs:** ARAWN-A-0002 (Sandboxed Script Execution via Wasmtime + Rust-to-WASM)

## Goals & Non-Goals

**Goals:**
- Embed Cloacina as a first-class execution layer (`arawn-pipeline` crate)
- Use Cloacina's core API (not macros) for **dynamic, runtime-defined workflows** — the agent creates workflows without compilation of workflow definitions
- Initialize `DefaultRunner` on startup with SQLite backend
- Define Arawn's first built-in workflow stubs: session indexing, manual reindex (logic wired in later initiatives)
- Build a **declarative workflow definition format** (files on disk, hot-reloadable)
- Expose Cloacina to the agent via a `workflow` tool (create, schedule, run, list, cancel, status)
- Support **sandboxed script execution** as a workflow action type via Wasmtime + Rust-to-WASM (per ARAWN-A-0002)
- Per-workflow configuration (not monolithic `[pipeline]` section)
- Wire workflow execution into the server lifecycle (start runner on boot, graceful shutdown)
- Support both push triggers (direct invocation) and cron scheduling

**Non-Goals:**
- Multi-tenancy — single-user agent for now
- PostgreSQL backend — SQLite is sufficient for edge deployment
- Python bindings — Rust-only
- Plugin-defined workflows — plugins will be able to register workflows in I-0013
- Browser automation, image tools — out of scope

## Detailed Design

### 1. `arawn-pipeline` Crate

New crate wrapping Cloacina for Arawn-specific workflows:

```rust
// crates/arawn-pipeline/src/lib.rs
pub struct PipelineEngine {
    runner: DefaultRunner,
}

impl PipelineEngine {
    /// Initialize with SQLite at the Arawn data dir
    pub async fn new(db_path: &Path, config: PipelineConfig) -> Result<Self>;
    
    /// Execute a named workflow with context
    pub async fn execute(&self, workflow: &str, context: Context<Value>) -> Result<ExecutionResult>;
    
    /// Register a cron schedule for a workflow
    pub async fn schedule_cron(&self, workflow: &str, cron_expr: &str) -> Result<String>;
    
    /// List scheduled/active workflows
    pub async fn list_schedules(&self) -> Result<Vec<ScheduleInfo>>;
    
    /// Cancel a scheduled workflow
    pub async fn cancel_schedule(&self, schedule_id: &str) -> Result<()>;
    
    /// Graceful shutdown
    pub async fn shutdown(self) -> Result<()>;
}
```

### 2. Declarative Workflow Definitions

Workflows are defined as files on disk (TOML or JSON), hot-reloadable without restart. The engine watches the workflow directory and picks up changes.

```toml
# ~/.arawn/workflows/session_indexing.toml
[workflow]
name = "session_indexing"
description = "Post-session entity extraction and summarization"

[[workflow.tasks]]
id = "extract_entities"
action = { type = "tool", name = "llm_generate", params = { prompt = "Extract entities from: {{input.session_history}}" } }
retry_attempts = 2

[[workflow.tasks]]
id = "generate_summary"
action = { type = "tool", name = "llm_generate", params = { prompt = "Summarize: {{input.session_history}}" } }
retry_attempts = 2

[[workflow.tasks]]
id = "store_results"
action = { type = "tool", name = "memory_store", params = { entities = "{{extract_entities.output}}", summary = "{{generate_summary.output}}" } }
dependencies = ["extract_entities", "generate_summary"]
```

**Action types:**
- `tool` — invoke an existing Arawn agent tool (web_fetch, memory_search, llm_generate, etc.)
- `script` — compile and execute a Rust script in the Wasmtime sandbox (per ARAWN-A-0002)
- `llm` — direct LLM call with prompt template and context

**Context flow:** Context is always passed between tasks. Template expressions (`{{task_id.output.field}}`) resolve upstream task outputs. Each task receives the full context and extracts what it needs on startup.

**Runtime construction:** The declarative definitions are parsed and translated to Cloacina core API calls at runtime — no macro compilation needed. This is the same approach Cloacina's Python bindings (Cloaca) use.

### 3. Sandboxed Script Execution (ARAWN-A-0002)

For `script` action type tasks:

```toml
[[workflow.tasks]]
id = "process_data"
action = { type = "script", language = "rust", source_file = "scripts/process.rs" }
capabilities = { filesystem = ["/tmp/arawn-sandbox"], network = false }
retry_attempts = 1
```

Execution flow:
1. Agent writes Rust source implementing a pre-defined SDK trait
2. `arawn-pipeline` compiles to `wasm32-wasip1` (cached — identical source skips recompile)
3. Wasmtime executes with WASI capability grants scoped to the task definition
4. Context in via stdin, output captured via stdout
5. Compile errors fed back to agent for self-correction

A pre-compiled `arawn-script-sdk` crate provides JSON, HTTP, regex, and common utilities so scripts don't pull external dependencies.

### 4. Built-in Workflow Stubs

Two built-in workflows ship with Arawn (task bodies implemented in later initiatives):

- **`session_indexing`** — Triggered via push on session close. Entity extraction + summarization in parallel, then store. (Logic wired in I-0017)
- **`memory_reindex`** — Manual or scheduled. Batch re-embedding with per-chunk retry. (Logic wired in I-0014)

These are defined as workflow files in the default config directory, using the same declarative format as agent-created workflows.

### 5. Agent-Facing `workflow` Tool

```rust
pub struct WorkflowTool {
    engine: Arc<PipelineEngine>,
}

// Actions:
// - "create"   — Write a new workflow definition file
// - "run"      — Execute a workflow immediately with context
// - "schedule" — Register a cron schedule for a workflow
// - "list"     — List workflows, schedules, and recent executions
// - "cancel"   — Cancel a schedule by ID
// - "status"   — Check execution status/history for a workflow run
```

This lets the agent autonomously create, schedule, and monitor workflows:
- "Create a workflow that fetches news and summarizes it"
- "Schedule the digest workflow to run daily at 9am"
- "Check the status of the last reindex run"

### 6. Per-Workflow Configuration

Configuration lives alongside each workflow definition rather than a monolithic section:

```toml
# Per-workflow scheduling and runtime config
[workflow.schedule]
cron = "0 9 * * *"
timezone = "America/New_York"

[workflow.runtime]
timeout_secs = 300
max_retries = 3

[workflow.triggers]
on_event = "session_close"  # push trigger — other Arawn components call engine.trigger()
```

Global engine settings (database path, cron polling) remain minimal:

```toml
# ~/.arawn/config.toml
[pipeline]
database = "pipeline.db"    # default: alongside memory.db
workflow_dir = "workflows"  # hot-reloaded workflow definitions
```

### 7. Server Lifecycle Integration

In `start.rs`:
1. Initialize `PipelineEngine` with SQLite path from config
2. Load workflow definitions from `workflow_dir` (hot-reload watcher)
3. Register push trigger endpoints for built-in events (session_close, etc.)
4. Start cron scheduler for workflows with `[schedule]` config
5. Pass `Arc<PipelineEngine>` to agent builder (for workflow tool)
6. On shutdown: `engine.shutdown()` for graceful drain

## Alternatives Considered

- **Custom background job system**: Building our own retry/scheduling/persistence. Cloacina already solves this with a clean embedded API. No reason to reinvent.
- **Cloacina as external service**: Running a separate Cloacina server. Adds operational complexity for no benefit — the embedded library is the point.
- **Tokio tasks for background work**: Simple `tokio::spawn` for fire-and-forget. No persistence, no retry, no scheduling. Fine for trivial work, not for reliable pipelines.
- **Separate crate per workflow**: Each workflow in its own crate. Overkill — `arawn-pipeline` hosts all built-in workflows, plugins add their own later.

## Implementation Plan

1. Create `arawn-pipeline` crate, add `cloacina` dependency, explore core API for dynamic workflow construction
2. Implement `PipelineEngine` (init, execute, schedule, trigger, shutdown)
3. Build declarative workflow definition parser (TOML files → Cloacina core API)
4. Implement hot-reload file watcher for workflow directory
5. Implement context template resolution (`{{task_id.output.field}}` expressions)
6. Integrate Wasmtime for `script` action type (compile Rust to wasm32-wasip1, execute with WASI capabilities)
7. Build `arawn-script-sdk` crate (JSON, HTTP, regex utilities for sandboxed scripts)
8. Define built-in workflow stubs (session_indexing, memory_reindex)
9. Implement `WorkflowTool` for agent-facing workflow CRUD + scheduling + monitoring
10. Add minimal `[pipeline]` config to `arawn-config` (database path, workflow_dir)
11. Wire `PipelineEngine` into `start.rs` lifecycle (init, cron, triggers, shutdown)
12. Tests: engine lifecycle, dynamic workflow construction, file loading, hot-reload, context templating, script compilation + sandbox execution, workflow tool actions