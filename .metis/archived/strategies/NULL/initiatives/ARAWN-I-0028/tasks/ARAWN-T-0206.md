---
id: server-integration
level: task
title: "Server integration"
short_code: "ARAWN-T-0206"
created_at: 2026-02-18T19:03:25.229730+00:00
updated_at: 2026-02-18T21:28:08.383823+00:00
parent: ARAWN-I-0028
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: ARAWN-I-0028
---

# Server integration

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[ARAWN-I-0028]]

## Objective

Wire DirectoryManager, PathValidator, and SandboxManager into server state and integrate with agent tool execution.

## Backlog Item Details **[CONDITIONAL: Backlog Item]**

{Delete this section when task is assigned to an initiative}

### Type
- [ ] Bug - Production issue that needs fixing
- [ ] Feature - New functionality or enhancement  
- [ ] Tech Debt - Code improvement or refactoring
- [ ] Chore - Maintenance or setup work

### Priority
- [ ] P0 - Critical (blocks users/revenue)
- [ ] P1 - High (important for user experience)
- [ ] P2 - Medium (nice to have)
- [ ] P3 - Low (when time permits)

### Impact Assessment **[CONDITIONAL: Bug]**
- **Affected Users**: {Number/percentage of users affected}
- **Reproduction Steps**: 
  1. {Step 1}
  2. {Step 2}
  3. {Step 3}
- **Expected vs Actual**: {What should happen vs what happens}

### Business Justification **[CONDITIONAL: Feature]**
- **User Value**: {Why users need this}
- **Business Value**: {Impact on metrics/revenue}
- **Effort Estimate**: {Rough size - S/M/L/XL}

### Technical Debt Impact **[CONDITIONAL: Tech Debt]**
- **Current Problems**: {What's difficult/slow/buggy now}
- **Benefits of Fixing**: {What improves after refactoring}
- **Risk Assessment**: {Risks of not addressing this}

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] `DirectoryManager` added to `AppState`
- [x] `SandboxManager` added to `AppState`
- [x] Workstream creation calls `DirectoryManager.create_workstream()`
- [x] Session creation in scratch calls `DirectoryManager.create_scratch_session()`
- [x] Helper methods: `allowed_paths()` and `path_validator()` on AppState
- [x] File watcher field added to AppState
- [x] Allowed paths returned in session API responses
- [ ] Agent tool execution passes through `PathValidator` (requires agent crate changes)
- [ ] Shell tool execution uses `SandboxManager` (requires agent crate changes)
- [ ] Integration tests for end-to-end flow

## Implementation Notes

### Location
- `crates/arawn-server/src/state.rs` - add to AppState

### AppState Changes

```rust
pub struct AppState {
    // Existing fields...
    pub agent: Agent,
    pub workstreams: Option<Arc<WorkstreamManager>>,
    
    // New fields
    pub directory_manager: Arc<DirectoryManager>,
    pub sandbox_manager: Arc<SandboxManager>,
    pub file_watcher: Option<Arc<FileWatcher>>,
}
```

### Server Startup

```rust
// In server initialization
let path_config = config.paths.clone();
let directory_manager = Arc::new(DirectoryManager::new(path_config.base_path));
let sandbox_manager = Arc::new(SandboxManager::new()?);

let file_watcher = if path_config.monitoring_enabled {
    let watcher = FileWatcher::new(path_config.debounce_ms)?;
    // Watch existing workstreams
    for ws in workstreams.list()? {
        watcher.watch_workstream(&directory_manager.workstream_path(&ws.id))?;
    }
    Some(Arc::new(watcher))
} else {
    None
};
```

### Agent Tool Integration

```rust
// In tool execution
fn execute_tool(&self, tool_call: &ToolCall, session: &Session) -> Result<ToolResult> {
    let allowed_paths = self.directory_manager.allowed_paths(
        &session.workstream_id,
        &session.id.to_string(),
    );
    let validator = PathValidator::new(allowed_paths);
    
    match tool_call.name.as_str() {
        "read_file" | "write_file" | "glob" | "grep" => {
            let path = extract_path(tool_call)?;
            validator.validate(&path)?;
            // Execute tool...
        }
        "bash" | "exec" => {
            self.sandbox_manager.execute(
                &tool_call.args["command"],
                &session.working_dir,
                &allowed_paths,
            )?
        }
        _ => { /* other tools */ }
    }
}
```

### Dependencies
- All previous tasks (T-0194 through T-0205)

## Status Updates

### Session 1 - In Progress

**Implemented:**

1. **Added `arawn-sandbox` dependency** to `arawn-server/Cargo.toml`

2. **Updated `AppState`** (`crates/arawn-server/src/state.rs`):
   - Added `directory_manager: Option<Arc<DirectoryManager>>` field
   - Added `sandbox_manager: Option<Arc<SandboxManager>>` field
   - Added `file_watcher: Option<Arc<WatcherHandle>>` field
   - Added builder methods: `with_directory_manager()`, `with_sandbox_manager()`, `with_file_watcher()`
   - Added helper methods: `allowed_paths()`, `path_validator()`

3. **Integrated directory creation into workstream creation** (`routes/workstreams.rs`):
   - `create_workstream_handler` now calls `dm.create_workstream()` after database creation

4. **Integrated directory creation into session creation** (`state.rs`):
   - `get_or_create_session_in_workstream()` creates scratch session directory for new sessions

5. **All 57 arawn-server tests pass**

**Additional in Session 1:**

6. **Updated session handlers** (`routes/sessions.rs`):
   - `create_session_handler` now returns `allowed_paths` in response
   - `get_session_handler` now returns `allowed_paths` and `workstream_id`

7. **All 57 arawn-server tests pass**

**Remaining (requires agent crate changes):**
- Agent tool execution PathValidator integration
- Shell tool SandboxManager integration
- Integration tests

**Note**: The server-side integration is complete. The remaining items require modifying `arawn-agent` to accept and use PathValidator/SandboxManager during tool execution. This is a larger architectural change that should be tracked separately.