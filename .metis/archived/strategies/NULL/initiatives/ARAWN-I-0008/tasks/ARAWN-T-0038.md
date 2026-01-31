---
id: create-systempromptbuilder-core
level: task
title: "Create SystemPromptBuilder Core Module"
short_code: "ARAWN-T-0038"
created_at: 2026-01-28T15:50:13.221231+00:00
updated_at: 2026-01-28T15:55:17.666891+00:00
parent: ARAWN-I-0008
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0008
---

# Create SystemPromptBuilder Core Module

## Parent Initiative
[[ARAWN-I-0008]]

## Objective

Create the core `prompt` module in arawn-agent with `SystemPromptBuilder` struct and `PromptMode` enum.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Create `crates/arawn-agent/src/prompt/mod.rs` module
- [ ] Implement `PromptMode` enum with `Full`, `Minimal`, `Identity` variants
- [ ] Implement `SystemPromptBuilder` with builder pattern
- [ ] Section builders: identity, tools, workspace, datetime, memory hints
- [ ] `build()` method assembles sections with double-newline separators
- [ ] Export from arawn-agent lib.rs

## Implementation Notes

### Files to Create
- `crates/arawn-agent/src/prompt/mod.rs` - Main module
- `crates/arawn-agent/src/prompt/builder.rs` - SystemPromptBuilder
- `crates/arawn-agent/src/prompt/mode.rs` - PromptMode enum

### API Design
```rust
pub enum PromptMode {
    Full,      // All sections (main agent)
    Minimal,   // Reduced sections (subagents)
    Identity,  // Just identity line
}

pub struct SystemPromptBuilder {
    mode: PromptMode,
    identity: Option<(String, String)>,  // (name, description)
    tools: Option<Vec<ToolSummary>>,
    workspace_path: Option<PathBuf>,
    timezone: Option<String>,
    memory_enabled: bool,
    bootstrap_context: Option<BootstrapContext>,
}

impl SystemPromptBuilder {
    pub fn new() -> Self;
    pub fn with_mode(self, mode: PromptMode) -> Self;
    pub fn with_identity(self, name: &str, description: &str) -> Self;
    pub fn with_tools(self, registry: &ToolRegistry) -> Self;
    pub fn with_workspace(self, path: impl AsRef<Path>) -> Self;
    pub fn with_datetime(self, timezone: Option<&str>) -> Self;
    pub fn with_memory_hints(self) -> Self;
    pub fn with_bootstrap(self, context: BootstrapContext) -> Self;
    pub fn build(self) -> String;
}
```

## Status Updates

### 2026-01-28
- Created `prompt` module with three submodules:
  - `prompt/mod.rs` - Main module with exports
  - `prompt/mode.rs` - PromptMode enum (Full, Minimal, Identity)
  - `prompt/builder.rs` - SystemPromptBuilder with fluent API
  - `prompt/bootstrap.rs` - BootstrapContext loader (implemented as part of this task)
- Added exports to arawn-agent lib.rs
- All 28 prompt module tests passing
- All 134 arawn-agent tests passing

**Files created:**
- `crates/arawn-agent/src/prompt/mod.rs`
- `crates/arawn-agent/src/prompt/mode.rs`
- `crates/arawn-agent/src/prompt/builder.rs`
- `crates/arawn-agent/src/prompt/bootstrap.rs`