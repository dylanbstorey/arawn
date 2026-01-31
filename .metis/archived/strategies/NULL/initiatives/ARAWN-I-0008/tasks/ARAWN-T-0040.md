---
id: integrate-systempromptbuilder-with
level: task
title: "Integrate SystemPromptBuilder with AgentBuilder"
short_code: "ARAWN-T-0040"
created_at: 2026-01-28T15:50:13.372222+00:00
updated_at: 2026-01-28T15:58:40.885071+00:00
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

# Integrate SystemPromptBuilder with AgentBuilder

## Parent Initiative
[[ARAWN-I-0008]]

## Objective

Wire `SystemPromptBuilder` into `AgentBuilder` so agents can use dynamic prompt generation instead of static system prompts.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Add `with_prompt_builder(SystemPromptBuilder)` to AgentBuilder
- [ ] Add `workspace_path: Option<PathBuf>` to AgentConfig
- [ ] Agent uses builder to generate system prompt if provided
- [ ] Falls back to existing `system_prompt` field if no builder
- [ ] Update agent run loop to regenerate prompt when needed (tool changes, etc.)

## Implementation Notes

### Changes to AgentBuilder
```rust
impl AgentBuilder {
    // Existing
    pub fn with_system_prompt(self, prompt: impl Into<String>) -> Self;
    
    // New
    pub fn with_prompt_builder(self, builder: SystemPromptBuilder) -> Self;
    pub fn with_workspace(self, path: impl AsRef<Path>) -> Self;
}
```

### Changes to AgentConfig
```rust
pub struct AgentConfig {
    // Existing fields...
    pub workspace_path: Option<PathBuf>,
}
```

### Prompt Generation Logic
In `Agent::new()` or `build()`:
1. If prompt_builder is set, use it with tools from registry
2. Else use static system_prompt
3. Store builder for potential regeneration

### Backward Compatibility
- Existing code using `with_system_prompt()` continues to work
- Builder is optional enhancement

### Dependencies
- Depends on ARAWN-T-0038 (core module)
- Depends on ARAWN-T-0039 (bootstrap loader)

## Status Updates

### 2026-01-28
- Added `workspace_path: Option<PathBuf>` to `AgentConfig` in types.rs
- Added `with_workspace()` method to `AgentConfig`
- Added `prompt_builder: Option<SystemPromptBuilder>` to `AgentBuilder`
- Added `with_prompt_builder()` and `with_workspace()` methods to `AgentBuilder`
- Updated `build()` to use prompt builder if present (with tools and workspace auto-configured)
- Added 3 new tests for prompt builder integration:
  - `test_agent_with_prompt_builder`
  - `test_agent_prompt_builder_with_static_fallback`
  - `test_agent_prompt_builder_overrides_static`
- All 137 arawn-agent tests passing
- All 327 workspace tests passing

**Files modified:**
- `crates/arawn-agent/src/types.rs` - Added workspace_path to AgentConfig
- `crates/arawn-agent/src/agent.rs` - Integrated SystemPromptBuilder into AgentBuilder