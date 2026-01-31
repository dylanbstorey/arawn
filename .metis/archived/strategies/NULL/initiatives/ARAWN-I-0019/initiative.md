---
id: pipeline-runtime-uniform-wasm-task
level: initiative
title: "Pipeline Runtime: Uniform WASM Task Execution"
short_code: "ARAWN-I-0019"
created_at: 2026-01-30T03:27:46.766694+00:00
updated_at: 2026-01-31T02:29:55.234411+00:00
parent: ARAWN-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
strategy_id: NULL
initiative_id: pipeline-runtime-uniform-wasm-task
---

# Pipeline Runtime: Uniform WASM Task Execution

## Context

ARAWN-I-0012 (Cloacina Integration) built the pipeline engine, workflow definitions, Wasmtime sandbox, and script SDK. However, the `ActionExecutorFactory` — the bridge between declarative workflow tasks and actual execution — is currently a no-op. Each workflow task declares an action type (tool, script, llm) but nothing dispatches those actions to real executors.

The current design requires special-casing per action type in Rust code. Instead, inspired by Airflow's operator model, we adopt a **uniform runtime model**: every workflow task is a WASM module that receives typed configuration via stdin and produces structured output via stdout. The `ScriptExecutor` (T-0076) becomes the single execution layer for all task types.

## Goals & Non-Goals

**Goals:**
- Define a standard runtime protocol (JSON envelope over stdin/stdout) for all workflow tasks
- Build a set of built-in WASM runtimes: `http`, `file_write`, `file_read`, `shell`, `transform`
- Implement a real `ActionExecutorFactory` that loads the named WASM runtime, passes config + pipeline context, and runs it in the sandbox
- Update the workflow TOML schema to use `runtime` + `config` instead of `action.type` + `action.name`
- Agent-authored scripts (arawn-script-sdk) and built-in runtimes use the same execution path

**Non-Goals:**
- GUI workflow editor
- Distributed execution (tasks run locally in the Wasmtime sandbox)
- Remote runtime registries (catalog is local-first)

## Detailed Design

### Runtime Protocol

Every runtime receives on stdin:
```json
{
  "config": { /* task-specific configuration from TOML */ },
  "context": { /* pipeline context from upstream tasks */ }
}
```

Every runtime writes to stdout:
```json
{
  "status": "ok",
  "output": { /* arbitrary JSON merged into pipeline context */ }
}
```

Or on failure:
```json
{
  "status": "error",
  "error": "Human-readable error message"
}
```

### Workflow TOML Schema (new)

```toml
[workflow]
name = "fetch_and_save"

[[workflow.tasks]]
id = "fetch"
runtime = "http"
config = { url = "https://api.example.com/data", method = "GET" }

[[workflow.tasks]]
id = "save"
runtime = "file_write"
config = { path = "/tmp/output.json", content_key = "fetch.body" }
depends_on = ["fetch"]
```

### Built-in Runtimes

| Runtime | Config Keys | Description |
|---------|------------|-------------|
| `http` | `url`, `method`, `headers`, `body` | HTTP request, returns status + body |
| `file_read` | `path` | Read file contents |
| `file_write` | `path`, `content` or `content_key` | Write content to file |
| `shell` | `command`, `args`, `timeout_secs` | Execute shell command |
| `transform` | `jq` or `template` | Transform context data (jq-style or template) |
| `passthrough` | _(none)_ | No-op, passes context through (for testing) |

Each is a Rust crate compiled to `wasm32-wasip1` using `arawn-script-sdk`.

### Context Propagation

Each task's output is registered in the pipeline context under its task ID. Downstream tasks receive the full accumulated context, enabling data flow through the DAG.

```
Pipeline context after each step:

1. Initial:    { "input": { "query": "rust async" } }
2. After fetch: { "input": {...}, "fetch": { "status": 200, "body": "..." } }
3. After parse: { "input": {...}, "fetch": {...}, "parse": { "records": [...] } }
4. After save:  { "input": {...}, "fetch": {...}, "parse": {...}, "save": { "path": "/tmp/out.json" } }
```

Each runtime receives the **full accumulated context** so it can reference any upstream task's output. The `config` can use dot-path references like `fetch.body` to pull specific values from prior outputs.

**Context merge rules:**
- Each task's stdout `output` object is stored under `context[task_id]`
- If a task returns `"output": null`, its entry is still registered (as null) so downstream tasks know it ran
- The `input` key is reserved for the initial workflow invocation context
- Task IDs are unique within a workflow, so no collisions

### ActionExecutorFactory

```rust
let factory: ActionExecutorFactory = Arc::new(move |task_id, action| {
    let executor = Arc::clone(&script_executor);
    let runtime_name = action.runtime.clone();
    let config = action.config.clone();
    let task_id = task_id.to_string();

    Arc::new(move |mut ctx| {
        let executor = Arc::clone(&executor);
        let task_id = task_id.clone();
        Box::pin(async move {
            // Build input envelope: runtime gets its config + full upstream context
            let input = json!({
                "config": config,
                "context": ctx.snapshot(),  // all prior task outputs
            });

            let result = executor.execute_runtime(&runtime_name, &input).await?;

            // Register output under task_id so downstream tasks can reference it
            match result.status.as_str() {
                "ok" => {
                    ctx.insert(task_id, result.output.unwrap_or(Value::Null))?;
                    Ok(ctx)
                }
                _ => Err(format!("Task '{}' failed: {}", task_id, 
                    result.error.unwrap_or_default()).into())
            }
        })
    })
});
```

### Runtime Catalog

A local catalog of reusable WASM runtimes — both built-in and agent-authored. This is what makes the agent self-extending: it can write a new runtime, compile it, register it in the catalog, and then reference it by name in any future workflow.

```
~/.config/arawn/runtimes/
├── catalog.toml              # Registry: name → metadata
├── builtin/
│   ├── http.wasm
│   ├── file_read.wasm
│   ├── file_write.wasm
│   ├── shell.wasm
│   └── transform.wasm
└── custom/
    ├── csv_parser.wasm        # Agent-authored
    ├── json_validator.wasm    # Agent-authored
    └── slack_notify.wasm      # Agent-authored
```

**catalog.toml:**
```toml
[[runtime]]
name = "http"
source = "builtin"
description = "HTTP request runtime"
config_schema = { url = "string", method = "string", headers = "object?", body = "string?" }

[[runtime]]
name = "csv_parser"
source = "custom"
description = "Parse CSV data into JSON records"
config_schema = { delimiter = "string?", has_header = "bool?" }
created_by = "agent"
created_at = "2026-01-30T03:30:00Z"
source_hash = "sha256:abc123..."
```

**Agent self-extension flow:**
1. Agent encounters a task it can't handle with existing runtimes
2. Writes a Rust script using `arawn-script-sdk`
3. Compiles to WASM via `ScriptExecutor.compile()`
4. Registers in the catalog with name, description, and config schema
5. Uses the new runtime in workflows by name — persists across sessions

**Catalog operations** (exposed via WorkflowTool or a new CatalogTool):
- `catalog_list` — list all available runtimes with descriptions
- `catalog_register` — register a compiled WASM module with metadata
- `catalog_inspect` — show config schema and description for a runtime
- `catalog_remove` — unregister a custom runtime

`ScriptExecutor` gains `execute_runtime(name, input)` which looks up the catalog, loads the `.wasm` module (with disk + memory caching), and runs it in the sandbox.

## Alternatives Considered

1. **Rust-native executors per action type** — Each action type (tool, script, llm) has a Rust implementation dispatched by the factory. Rejected: requires new Rust code for every new capability, no sandboxing for built-in actions, two execution paths (Rust vs WASM).

2. **Plugin-based executors** — Load shared libraries (.so/.dylib) at runtime. Rejected: platform-specific, no sandboxing, complex ABI.

3. **Container-based execution** — Run each task in a Docker container. Rejected: heavy for edge computing, slow startup, requires Docker runtime.

## Implementation Plan

1. Define runtime protocol types and update `WorkflowDefinition` schema (`runtime` + `config` fields)
2. Build runtime catalog: `catalog.toml` format, loader, CRUD operations
3. Build the `passthrough` and `http` runtimes as WASM modules using arawn-script-sdk
4. Add `execute_runtime(name, input)` to `ScriptExecutor` — catalog lookup + cached WASM execution
5. Implement the real `ActionExecutorFactory` using `ScriptExecutor` + catalog
6. Wire factory into `WorkflowTool` and server startup
7. Add catalog actions to WorkflowTool (or new CatalogTool): list, register, inspect, remove
8. Build remaining built-in runtimes (`file_read`, `file_write`, `shell`, `transform`)
9. Agent self-extension test: agent writes a custom runtime, registers it, uses it in a workflow
10. End-to-end test: create + run a multi-step workflow through the agent