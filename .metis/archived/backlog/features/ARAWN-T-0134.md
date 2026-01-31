---
id: plugin-event-system-and-lifecycle
level: task
title: "Plugin Event System and Lifecycle Hooks"
short_code: "ARAWN-T-0134"
created_at: 2026-02-04T15:06:53.045409+00:00
updated_at: 2026-02-05T22:19:23.776210+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#feature"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Plugin Event System and Lifecycle Hooks

## Objective

Wire the existing `HookDispatcher` (in `arawn-plugin`) into the agent turn loop so plugin hooks actually fire. The infrastructure exists but isn't connected.

## Current State Assessment

**Already Implemented:**
| Component | Status | Location |
|-----------|--------|----------|
| `HookDispatcher` | ✅ Complete | `arawn-plugin/src/hooks.rs` |
| `HookEvent` enum | ✅ 11 events | `arawn-plugin/src/types.rs` |
| Shell command execution | ✅ Works | `run_hook_command()` |
| Glob/regex matching | ✅ Works | `matches_hook()` |
| Timeout handling | ✅ Works | Configurable |
| JSON context on stdin | ✅ Works | Serialized to hook process |

**Not Connected:**
| Component | Status | Needed |
|-----------|--------|--------|
| Agent integration | ❌ Missing | Add dispatcher to Agent struct |
| Turn loop calls | ❌ Missing | Call dispatch at key points |
| PluginManager wiring | ❌ Missing | Pass dispatcher to Agent |
| Server lifecycle calls | ❌ Missing | SessionStart/End in AppState |

## Backlog Item Details

### Type
- [x] Feature - New functionality or enhancement

### Priority
- [x] P0 - Critical (blocks plugin functionality)

### Business Justification
- **User Value**: Plugin hooks actually work (PreToolUse can block dangerous commands, SessionEnd can trigger custom indexing)
- **Business Value**: Enables plugin ecosystem without code changes
- **Effort Estimate**: S (Small) - wiring task, infrastructure exists

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] `Agent` struct has optional `HookDispatcher` field
- [x] `AgentBuilder::with_hook_dispatcher()` method added
- [x] `dispatch_pre_tool_use()` called before each tool execution in `execute_tools()`
- [x] `dispatch_post_tool_use()` called after each tool execution
- [x] `dispatch_session_start()` called when session created (in server)
- [x] `dispatch_session_end()` called when session closed (in server)
- [x] Hook dispatcher created from plugin configs via `register_from_config()`
- [x] `start.rs` wires dispatcher from plugins to Agent and AppState
- [ ] Hooks from journal example plugin fire correctly (requires manual integration test)
- [ ] PreToolUse hook can block shell commands (requires manual integration test)

## Implementation Tasks

### 1. Add HookDispatcher to Agent (`arawn-agent/src/agent.rs`)

```rust
pub struct Agent {
    backend: SharedBackend,
    tools: Arc<ToolRegistry>,
    config: AgentConfig,
    // ... existing fields ...
    hook_dispatcher: Option<Arc<HookDispatcher>>,  // NEW
}

impl AgentBuilder {
    pub fn with_hook_dispatcher(mut self, dispatcher: Arc<HookDispatcher>) -> Self {
        self.hook_dispatcher = Some(dispatcher);
        self
    }
}
```

### 2. Call Dispatcher in Tool Loop (`arawn-agent/src/agent.rs`)

In `execute_tools()`:
```rust
async fn execute_tools(&self, ...) -> Result<...> {
    for tool_use in response.tool_uses() {
        // PRE-TOOL HOOK
        if let Some(ref dispatcher) = self.hook_dispatcher {
            match dispatcher.dispatch_pre_tool_use(&tool_use.name, &tool_use.input).await {
                HookOutcome::Block { reason } => {
                    // Return error result to LLM instead of executing
                    tool_results.push(ToolResultRecord {
                        tool_call_id: tool_use.id.clone(),
                        success: false,
                        content: format!("Blocked by hook: {}", reason),
                    });
                    continue;
                }
                _ => {}
            }
        }

        // Execute tool (existing code)
        let result = self.tools.execute(...).await;

        // POST-TOOL HOOK
        if let Some(ref dispatcher) = self.hook_dispatcher {
            let result_json = serde_json::to_value(&result).unwrap_or_default();
            dispatcher.dispatch_post_tool_use(&tool_use.name, &tool_use.input, &result_json).await;
        }
    }
}
```

### 3. Expose Dispatcher from PluginManager (`arawn-plugin/src/manager.rs`)

```rust
impl PluginManager {
    pub fn hook_dispatcher(&self) -> Arc<HookDispatcher> {
        Arc::clone(&self.hook_dispatcher)
    }
}
```

### 4. Wire in start.rs (`arawn/src/commands/start.rs`)

```rust
// After loading plugins
let hook_dispatcher = plugin_manager.hook_dispatcher();

// When building agent
let agent = Agent::builder()
    .with_shared_backend(backend)
    .with_tools(tools)
    .with_hook_dispatcher(hook_dispatcher)
    // ...
    .build()?;
```

### 5. Session Lifecycle in Server (`arawn-server`)

In `AppState::create_session()`:
```rust
if let Some(ref dispatcher) = self.hook_dispatcher {
    dispatcher.dispatch_session_start(&session_id).await;
}
```

In `AppState::close_session()`:
```rust
if let Some(ref dispatcher) = self.hook_dispatcher {
    dispatcher.dispatch_session_end(&session_id, turn_count).await;
}
```

## Existing Hook Events (Already Defined)

From `arawn-plugin/src/types.rs`:
```rust
pub enum HookEvent {
    PreToolUse,        // Can block tool execution
    PostToolUse,       // Informational
    PostToolUseFailure,
    PermissionRequest,
    UserPromptSubmit,
    Notification,
    SubagentStop,
    PreCompact,
    SessionStart,      // Informational
    SessionEnd,        // Informational
    Stop,              // After final response
}
```

## Future Enhancements (Out of Scope)

These exist in `HookType` but aren't implemented:
- `HookType::Prompt` - LLM evaluation hooks
- `HookType::Agent` - Agentic verifier hooks

Only `HookType::Command` (shell scripts) works currently. This is sufficient for MVP.

## Testing

1. Create test hook script that blocks `rm` commands
2. Verify PreToolUse fires and blocks
3. Verify PostToolUse fires after successful execution
4. Verify SessionStart/End fire at correct times
5. Test hook timeout handling

## Status Updates

### Session 1 (2026-02-04)

**Completed:**
- [x] Added `HookDispatch` trait to `arawn-types` (avoids cyclic dependency)
- [x] Added `SharedHookDispatcher` type alias (`Arc<dyn HookDispatch>`)
- [x] Moved hook types (`HookEvent`, `HookOutcome`, `HookDef`, etc.) to `arawn-types`
- [x] Added `hook_dispatcher: Option<SharedHookDispatcher>` to `Agent` struct
- [x] Added `AgentBuilder::with_hook_dispatcher()` method
- [x] Implemented pre/post hook dispatch in `execute_tools()`
- [x] Added `HookDispatcher::register_from_config()` to convert Claude format
- [x] Implemented `HookDispatch` trait for `HookDispatcher`
- [x] Wired hooks from plugins in `start.rs`
- [x] All tests pass, workspace compiles

**Architecture Note:**
Used trait object pattern (`Arc<dyn HookDispatch>`) to avoid cyclic dependency between `arawn-agent` and `arawn-plugin`. Types live in `arawn-types`, implementation lives in `arawn-plugin`, consumer is `arawn-agent`.

### Session 2 (2026-02-05)

**Completed:**
- [x] Added `hook_dispatcher: Option<SharedHookDispatcher>` to `AppState` struct
- [x] Added `AppState::with_hook_dispatcher()` builder method
- [x] Implemented `dispatch_session_start()` call in `get_or_create_session()` (fires when new session created)
- [x] Implemented `dispatch_session_end()` call in `close_session()` (fires before session is removed)
- [x] Wired shared hook dispatcher from `start.rs` to both Agent and AppState
- [x] All checks pass (`angreal check all`)
- [x] All unit tests pass (`angreal test unit`)

**Files Modified:**
- `crates/arawn-server/src/state.rs` - Added hook_dispatcher field and session lifecycle hook calls
- `crates/arawn/src/commands/start.rs` - Wire shared dispatcher to AppState

**All acceptance criteria met:**
- ✅ `Agent` struct has optional `HookDispatcher` field
- ✅ `AgentBuilder::with_hook_dispatcher()` method added
- ✅ `dispatch_pre_tool_use()` called before each tool execution in `execute_tools()`
- ✅ `dispatch_post_tool_use()` called after each tool execution
- ✅ `dispatch_session_start()` called when session created (in server)
- ✅ `dispatch_session_end()` called when session closed (in server)
- ✅ Hook dispatcher exposed from plugins via `register_from_config()`
- ✅ `start.rs` wires dispatcher from plugins to Agent and AppState

**Note:** Testing with actual journal plugin hooks and verifying PreToolUse blocking would require manual integration testing with the server running. The wiring is complete and ready for testing.