---
id: subagent-delegation-spawn-and
level: initiative
title: "Subagent Delegation: Spawn and Coordinate Child Agents"
short_code: "ARAWN-I-0020"
created_at: 2026-02-04T15:11:29.332728+00:00
updated_at: 2026-02-07T13:05:38.741885+00:00
parent: ARAWN-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: L
strategy_id: NULL
initiative_id: subagent-delegation-spawn-and
---

# Subagent Delegation: Spawn and Coordinate Child Agents Initiative

## Context

Arawn currently has infrastructure to **create** subagents (`AgentSpawner` in `arawn-plugin`) but no way to **invoke** them at runtime. The main agent cannot delegate tasks to specialized child agents.

Multi-agent systems enable:
- **Specialization** - Research agent, coding agent, review agent with different tool sets
- **Parallelization** - Multiple subagents working concurrently
- **Isolation** - Subagents with constrained permissions for safer execution
- **Complexity management** - Break large tasks into delegatable subtasks

### Current State

| Capability | Status |
|------------|--------|
| Define subagent configs | ✅ Plugin agents in TOML/JSON |
| Create subagent instances | ✅ `AgentSpawner::spawn()` |
| Constrain subagent tools | ✅ Tool filtering works |
| Invoke subagent from tool | ❌ Missing |
| Wait for subagent completion | ❌ Missing |
| Share context/memory | ❌ Missing |
| Event notification | ❌ Needs ARAWN-T-0134 |

## Goals & Non-Goals

**Goals:**
- Main agent can delegate tasks to named subagents via `delegate` tool
- Support blocking (wait for result) and background (fire-and-forget) execution
- Subagents receive relevant context from parent session
- Results flow back to parent session/conversation
- Event notifications for subagent lifecycle (start, complete, error)
- Subagent tool constraints enforced at runtime

**Non-Goals:**
- Multi-machine distributed agents (future consideration)
- Subagent-to-subagent communication (keep hierarchy simple)
- Dynamic subagent creation at runtime (use predefined configs)
- GUI for subagent management (CLI/API only)

## Use Cases

### Use Case 1: Blocking Research Delegation
- **Actor**: Main agent during conversation
- **Scenario**: 
  1. User asks "Find recent papers on RAG architectures"
  2. Main agent invokes `delegate` tool with `agent: "researcher"`, `task: "..."`
  3. Researcher subagent executes with web_search + web_fetch tools
  4. Main agent waits for completion
  5. Result returned to main agent's context
  6. Main agent summarizes findings for user
- **Expected Outcome**: Seamless delegation with result integration

### Use Case 2: Background Code Review
- **Actor**: Main agent after code generation
- **Scenario**:
  1. Main agent generates code for user request
  2. Main agent delegates review to "reviewer" subagent in background
  3. Main agent continues conversation with user
  4. Reviewer completes, fires `SubagentCompleted` event
  5. Main agent notified, can proactively share review findings
- **Expected Outcome**: Non-blocking parallel work

### Use Case 3: Constrained Execution
- **Actor**: Main agent with untrusted task
- **Scenario**:
  1. User provides script to analyze
  2. Main agent delegates to "sandbox" subagent (no shell, no file_write)
  3. Subagent can only read files and think
  4. Analysis returned safely
- **Expected Outcome**: Security through tool constraints

## Architecture

### Overview

```
┌─────────────────────────────────────────────────────────────────┐
│ Main Agent                                                       │
│                                                                  │
│  ┌──────────────┐    ┌─────────────────┐    ┌───────────────┐  │
│  │ Tool Registry │    │ Delegate Tool   │    │ Event         │  │
│  │              │────▶│                 │────▶│ Dispatcher    │  │
│  │ delegate     │    │ - spawn subagent│    │               │  │
│  └──────────────┘    │ - manage session│    │ SubagentStart │  │
│                      │ - collect result│    │ SubagentEnd   │  │
│                      └────────┬────────┘    └───────────────┘  │
│                               │                                  │
└───────────────────────────────┼──────────────────────────────────┘
                                │
                    ┌───────────┴───────────┐
                    ▼                       ▼
          ┌─────────────────┐     ┌─────────────────┐
          │ Subagent:       │     │ Subagent:       │
          │ "researcher"    │     │ "reviewer"      │
          │                 │     │                 │
          │ Tools:          │     │ Tools:          │
          │ - web_search    │     │ - file_read     │
          │ - web_fetch     │     │ - grep          │
          │ - memory        │     │ - think         │
          │                 │     │                 │
          │ Session: child  │     │ Session: child  │
          └─────────────────┘     └─────────────────┘
```

### Sequence: Blocking Delegation

```
MainAgent          DelegateTool         AgentSpawner         Subagent
    │                   │                    │                   │
    │ delegate(task)    │                    │                   │
    │──────────────────▶│                    │                   │
    │                   │                    │                   │
    │                   │ spawn(config)      │                   │
    │                   │───────────────────▶│                   │
    │                   │                    │                   │
    │                   │◀───────────────────│                   │
    │                   │ subagent           │                   │
    │                   │                    │                   │
    │                   │ turn(session, task)                    │
    │                   │───────────────────────────────────────▶│
    │                   │                    │                   │
    │                   │                    │    tool loop      │
    │                   │                    │◀──────────────────│
    │                   │                    │──────────────────▶│
    │                   │                    │                   │
    │                   │◀───────────────────────────────────────│
    │                   │ AgentResponse      │                   │
    │                   │                    │                   │
    │◀──────────────────│                    │                   │
    │ ToolResult{text}  │                    │                   │
```

## Detailed Design

### 1. Delegate Tool

```rust
pub struct DelegateTool {
    spawner: Arc<AgentSpawner>,
    configs: HashMap<String, PluginAgentConfig>,
}

impl Tool for DelegateTool {
    fn name(&self) -> &str { "delegate" }
    
    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "agent": {
                    "type": "string",
                    "description": "Name of the subagent to delegate to"
                },
                "task": {
                    "type": "string",
                    "description": "Task description for the subagent"
                },
                "context": {
                    "type": "string",
                    "description": "Additional context to provide"
                },
                "background": {
                    "type": "boolean",
                    "default": false,
                    "description": "Run in background (non-blocking)"
                },
                "max_iterations": {
                    "type": "integer",
                    "description": "Override max iterations for this task"
                }
            },
            "required": ["agent", "task"]
        })
    }
    
    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        let agent_name = params["agent"].as_str()?;
        let task = params["task"].as_str()?;
        let background = params["background"].as_bool().unwrap_or(false);
        
        let config = self.configs.get(agent_name)
            .ok_or_else(|| AgentError::UnknownAgent(agent_name.to_string()))?;
        
        let subagent = self.spawner.spawn(config)?;
        
        if background {
            // Fire and forget - dispatch event when done
            tokio::spawn(async move {
                let mut session = Session::new();
                let result = subagent.turn(&mut session, task).await;
                // Dispatch SubagentCompleted event
            });
            
            Ok(ToolResult::text(format!(
                "Delegated to '{}' in background. You'll be notified when complete.",
                agent_name
            )))
        } else {
            // Blocking execution
            let mut session = Session::new();
            let response = subagent.turn(&mut session, task).await?;
            
            Ok(ToolResult::text(format!(
                "## Result from '{}'\n\n{}",
                agent_name,
                response.text
            )))
        }
    }
}
```

### 2. Subagent Events (Depends on ARAWN-T-0134)

```rust
pub enum AgentEvent {
    // ... existing events ...
    
    SubagentStarted {
        parent_session_id: String,
        subagent_name: String,
        task_preview: String,
        background: bool,
    },
    SubagentCompleted {
        parent_session_id: String,
        subagent_name: String,
        result_preview: String,
        duration_ms: u64,
        success: bool,
    },
}
```

### 3. Context Passing

For the subagent to have useful context:

```rust
// Build subagent session with parent context
fn create_subagent_session(parent_session: &Session, context: &str) -> Session {
    let mut session = Session::new();
    
    // Inject context as system message
    if !context.is_empty() {
        session.inject_context(format!(
            "[Context from parent agent]\n{}",
            context
        ));
    }
    
    session
}
```

### 4. Registry Integration

The `DelegateTool` needs access to agent configs at construction:

```rust
// In Agent build or PluginManager
let delegate_tool = DelegateTool::new(
    Arc::new(AgentSpawner::new(parent_tools.clone(), backend.clone())),
    plugin_manager.agent_configs(), // HashMap<String, PluginAgentConfig>
);

tools.register(delegate_tool);
```

## Dependencies

| Dependency | Status | Notes |
|------------|--------|-------|
| **ARAWN-T-0134** (Event System) | Backlog | Required for background completion notifications |
| AgentSpawner | ✅ Complete | Already implemented in `arawn-plugin` |
| Plugin agent configs | ✅ Complete | Parsed from plugin manifests |

## Alternatives Considered

### 1. Function Calls Instead of Tool
**Rejected** - Using a tool keeps the delegation visible in the conversation and allows the LLM to reason about when/how to delegate.

### 2. Automatic Routing
**Rejected** - Having the system automatically route to subagents based on task type removes agency from the main agent. Explicit delegation is more controllable and debuggable.

### 3. Shared Session
**Rejected** - Having subagents share the parent session would pollute history. Child sessions with context injection is cleaner.

## Implementation Plan

### Phase 1: Core Delegation (Blocking Only)
1. Create `DelegateTool` struct
2. Wire into tool registry with agent configs
3. Implement blocking execution path
4. Add basic tests

### Phase 2: Event Integration
*Depends on ARAWN-T-0134*
1. Define `SubagentStarted` / `SubagentCompleted` events
2. Implement background execution with event dispatch
3. Add handler for notifying main agent of completion

### Phase 3: Context & Memory
1. Context injection from parent session
2. Optional memory sharing (read-only access to parent's memory store)
3. Result summarization for long subagent outputs

### Phase 4: Polish
1. CLI commands for listing available subagents
2. Subagent execution stats/logging
3. Documentation and examples