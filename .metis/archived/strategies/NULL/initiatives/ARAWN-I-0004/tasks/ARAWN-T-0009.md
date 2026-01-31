---
id: create-arawn-agent-crate-scaffold
level: task
title: "Create arawn-agent crate scaffold with core types"
short_code: "ARAWN-T-0009"
created_at: 2026-01-28T03:20:08.473662+00:00
updated_at: 2026-01-28T03:25:32.651082+00:00
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

# Create arawn-agent crate scaffold with core types

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[ARAWN-I-0004]]

## Objective

Set up the arawn-agent crate module structure and define foundational types that other agent components will build upon: Session, Message, AgentConfig, and error types.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `error.rs` with `AgentError` enum covering: LLM errors, tool errors, session errors, context errors, config errors
- [ ] `types.rs` with core types: `Session`, `SessionId`, `Turn`, `TurnId`, `AgentConfig`, `AgentResponse`
- [ ] `Session` supports: creating new sessions, adding turns, getting recent history, serialization
- [ ] `AgentConfig` covers: model selection, max tokens, temperature, max iterations, timeout settings
- [ ] `lib.rs` exports all public types with proper module organization
- [ ] All types implement `Debug`, `Clone`, and `Serialize`/`Deserialize` where appropriate
- [ ] Unit tests for Session operations (create, add turn, get history)
- [ ] `cargo test -p arawn-agent` passes

## Implementation Notes

### Files to Create

```
crates/arawn-agent/src/
├── lib.rs          # Module exports
├── error.rs        # AgentError enum
└── types.rs        # Session, Turn, AgentConfig, AgentResponse
```

### Key Types

```rust
// Session - conversation state
pub struct Session {
    id: SessionId,
    turns: Vec<Turn>,
    created_at: DateTime<Utc>,
    metadata: HashMap<String, Value>,
}

// Turn - single exchange (user message + assistant response)
pub struct Turn {
    id: TurnId,
    user_message: String,
    assistant_response: Option<AgentResponse>,
    tool_calls: Vec<ToolCall>,
    tool_results: Vec<ToolResult>,
    timestamp: DateTime<Utc>,
}

// AgentConfig - runtime configuration
pub struct AgentConfig {
    pub model: String,
    pub max_tokens: u32,
    pub temperature: Option<f32>,
    pub max_iterations: u32,  // tool loop limit
    pub timeout: Duration,
}
```

### Dependencies
- `arawn-llm` for LlmError conversion
- `uuid` for SessionId/TurnId
- `chrono` for timestamps
- `serde` for serialization

## Status Updates

- Created error.rs with AgentError enum
- Created types.rs with Session, Turn, AgentConfig, AgentResponse, ID types
- Updated lib.rs with module exports
- All 15 tests passing