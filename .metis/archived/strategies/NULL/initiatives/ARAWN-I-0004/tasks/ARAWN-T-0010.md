---
id: implement-tool-trait-and
level: task
title: "Implement Tool trait and ToolRegistry"
short_code: "ARAWN-T-0010"
created_at: 2026-01-28T03:20:08.570718+00:00
updated_at: 2026-01-28T03:28:45.980014+00:00
parent: ARAWN-I-0004
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0004
---

# Implement Tool trait and ToolRegistry

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[ARAWN-I-0004]]

## Objective

Define the `Tool` trait that all agent tools must implement, and create a `ToolRegistry` for managing and looking up available tools. This provides the extensible foundation for agent capabilities.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `Tool` trait with: `name()`, `description()`, `parameters() -> JsonSchema`, `execute(params, ctx) -> ToolResult`
- [ ] `ToolContext` struct providing execution context (session info, cancellation token, etc.)
- [ ] `ToolResult` enum with success/error variants, supporting text, JSON, and binary content
- [ ] `ToolRegistry` with: `register()`, `get()`, `list()`, `to_llm_definitions()`
- [ ] `to_llm_definitions()` converts tools to `arawn_llm::ToolDefinition` format
- [ ] `MockTool` implementation for testing
- [ ] Thread-safe registry (tools are `Send + Sync`)
- [ ] Unit tests for registry operations and mock tool execution
- [ ] `cargo test -p arawn-agent` passes

## Implementation Notes

### Files to Create

```
crates/arawn-agent/src/
├── tool.rs         # Tool trait, ToolContext, ToolResult, ToolRegistry
```

### Key Types

```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> serde_json::Value;  // JSON Schema
    
    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult>;
}

pub struct ToolContext {
    pub session_id: SessionId,
    pub turn_id: TurnId,
    pub cancellation: CancellationToken,
}

pub enum ToolResult {
    Text(String),
    Json(Value),
    Error { message: String, recoverable: bool },
}

pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}
```

### Integration with arawn-llm

The registry must convert tools to `ToolDefinition` format for LLM requests:

```rust
impl ToolRegistry {
    pub fn to_llm_definitions(&self) -> Vec<arawn_llm::ToolDefinition> {
        self.tools.values()
            .map(|t| ToolDefinition::new(t.name(), t.description(), t.parameters()))
            .collect()
    }
}
```

### Dependencies
- ARAWN-T-0009 (core types for SessionId, TurnId)
- `arawn-llm` for ToolDefinition

## Status Updates

- Created tool.rs with Tool trait, ToolContext, ToolResult, ToolRegistry
- Implemented MockTool for testing
- Added tokio-util dependency for CancellationToken
- Integration with arawn_llm::ToolDefinition working
- All 27 tests passing