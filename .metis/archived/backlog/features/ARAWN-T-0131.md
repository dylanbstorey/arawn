---
id: shell-tool-enhancements-pty-mode
level: task
title: "Shell Tool Enhancements: PTY Mode and Streaming Output"
short_code: "ARAWN-T-0131"
created_at: 2026-02-04T15:00:54.465119+00:00
updated_at: 2026-02-07T16:30:04.078473+00:00
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

# Shell Tool Enhancements: PTY Mode and Streaming Output

## Objective

Enhance the shell tool with PTY (pseudo-terminal) mode for interactive commands, streaming output for long-running processes, and working directory persistence across invocations.

## Backlog Item Details

### Type
- [x] Feature - New functionality or enhancement

### Priority
- [x] P1 - High (important for user experience)

### Business Justification
- **User Value**: 
  - PTY mode enables interactive commands (editors, REPLs, installers with prompts)
  - Streaming output shows progress for long builds/tests instead of waiting
  - Working directory persistence matches user mental model
- **Effort Estimate**: M (Medium) - requires async streaming plumbing

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] PTY mode parameter (`pty: bool`) spawns command in pseudo-terminal
- [x] Streaming output returns incremental results via agent stream
- [x] Working directory persists across shell invocations within a session
- [x] Timeout still enforced for both modes
- [x] Graceful handling of commands that expect TTY (colored output, progress bars)
- [x] Output size limits still apply (with truncation indication)

## Feature Details

### 1. PTY Mode
```json
{
  "command": "npm install",
  "pty": true,
  "timeout_secs": 300
}
```
- Uses `portable-pty` or `pty-process` crate
- Captures colored output properly
- Handles terminal escape sequences
- Useful for: installers, interactive prompts, colored test output

### 2. Streaming Output
```json
{
  "command": "cargo build --release",
  "stream": true
}
```
- Returns chunks via `ToolResult::Stream` variant
- Agent can display progress to user
- Still captures full output for context
- Useful for: builds, tests, long-running processes

### 3. Working Directory Persistence
- Store `cwd` in session context
- `cd` commands update session cwd
- Subsequent commands inherit cwd
- Reset on session close

## Implementation Notes

### Technical Approach

**PTY Mode:**
```rust
// Using portable-pty crate
let pty_system = native_pty_system();
let pair = pty_system.openpty(PtySize { rows: 24, cols: 80, .. })?;
let mut cmd = CommandBuilder::new(&shell);
cmd.arg("-c").arg(&command);
let child = pair.slave.spawn_command(cmd)?;
```

**Streaming:**
```rust
pub enum ToolResult {
    Text(String),
    Json(Value),
    Stream(Pin<Box<dyn Stream<Item = String> + Send>>),  // New variant
    // ...
}
```

**Working Directory:**
```rust
// In ToolContext
pub struct ToolContext {
    pub session_id: String,
    pub working_dir: PathBuf,  // Mutable via Arc<Mutex<PathBuf>>
    // ...
}
```

### Dependencies
- `portable-pty` or `pty-process` crate for PTY
- Existing tokio async runtime for streaming
- Session context already exists

### Risk Considerations
- PTY on Windows is more complex (ConPTY)
- Streaming output could overwhelm context if not capped
- Security: PTY mode could be exploited for escape sequences

## Status Updates

### 2026-02-07: PTY Mode and Working Directory Persistence Implemented

**Added Dependencies:**
- `portable-pty = "0.8"` for pseudo-terminal support
- `dirs = "5.0"` for home directory resolution

**PTY Mode Implementation:**
- ✅ Added `pty: bool` parameter to shell tool
- ✅ Uses `portable-pty` crate for cross-platform PTY support
- ✅ Configurable terminal size via `pty_size: (rows, cols)` in ShellConfig
- ✅ Handles colored output and ANSI escape sequences
- ✅ Runs in `spawn_blocking` since portable-pty is synchronous
- ✅ Timeout enforcement via manual polling loop

**Working Directory Persistence:**
- ✅ Added `cwd` parameter for explicit working directory
- ✅ Session-based working directory storage (`SharedWorkingDirs` HashMap)
- ✅ Special handling for `cd` commands to update session cwd
- ✅ Home directory (`~`) expansion supported
- ✅ Relative path resolution with current working directory

**New Parameters:**
- `pty: bool` - Run in PTY mode (default: false)
- `cwd: string` - Override working directory for this command
- `timeout_secs: integer` - Custom timeout (default: 30)

**Tests Added:**
- `test_shell_pty_echo` - Basic PTY execution
- `test_shell_pty_colored_output` - ANSI escape sequence handling
- `test_shell_cd_persistence` - Working directory persists across commands
- `test_shell_cd_nonexistent` - Error on nonexistent directory
- `test_shell_explicit_cwd` - cwd parameter works
- `test_shell_custom_timeout` - timeout_secs parameter works

**Files Modified:**
- `crates/arawn-agent/Cargo.toml` - Added portable-pty, dirs
- `crates/arawn-agent/src/tools/shell.rs` - Full implementation

**All 283 arawn-agent tests pass.**

### 2026-02-07: Streaming Output Implemented

**StreamChunk Extension:**
- ✅ Added `ToolOutput` variant to `StreamChunk` for incremental tool output
- ✅ Constructor method `StreamChunk::tool_output(id, content)`

**ToolContext Streaming Support:**
- ✅ Added `OutputSender` type (unbounded mpsc channel)
- ✅ Added `output_sender: Option<OutputSender>` field to `ToolContext`
- ✅ Added `tool_call_id: Option<String>` for output association
- ✅ Helper methods: `with_streaming()`, `is_streaming()`, `send_output()`

**Shell Tool Streaming:**
- ✅ Added `stream: bool` parameter to shell tool
- ✅ `execute_pty_with_callback()` - PTY streaming via callback
- ✅ `execute_standard_streaming()` - Async line-by-line streaming
- ✅ Both modes send output via `ctx.send_output()`

**Tests Added:**
- `test_shell_streaming` - Standard mode streaming
- `test_shell_streaming_pty` - PTY mode streaming

**Files Modified:**
- `crates/arawn-agent/src/stream.rs` - ToolOutput variant
- `crates/arawn-agent/src/tool.rs` - ToolContext streaming support
- `crates/arawn-agent/src/tools/shell.rs` - Streaming execution

**All 21 shell tests pass. All acceptance criteria complete.**