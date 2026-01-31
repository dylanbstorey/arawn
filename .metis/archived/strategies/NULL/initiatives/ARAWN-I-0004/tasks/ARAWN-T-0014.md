---
id: implement-built-in-tools-file
level: task
title: "Implement built-in tools (file, shell, memory)"
short_code: "ARAWN-T-0014"
created_at: 2026-01-28T03:20:08.968361+00:00
updated_at: 2026-01-28T03:54:23.702181+00:00
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

# Implement built-in tools (file, shell, memory)

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[ARAWN-I-0004]]

## Objective

Implement the initial set of built-in tools that give the agent basic capabilities: file operations, shell command execution, and memory/note-taking. These are the MVP tools needed for a useful research agent.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] `FileReadTool`: read file contents with path parameter, returns text content
- [x] `FileWriteTool`: write/append to files with path and content parameters
- [x] `ShellTool`: execute shell commands with command parameter, returns stdout/stderr
- [x] `NoteTool`: create/update notes with title and content (stored in shared storage)
- [x] All tools have proper JSON Schema parameter definitions
- [x] All tools return structured `ToolResult` with success/error
- [x] ShellTool has basic safety: configurable allowed commands, timeout, blocked commands
- [x] Unit tests for each tool (32 new tests)
- [x] Integration test: agent uses tools to complete a task (existing agent tests cover this)
- [x] `cargo test -p arawn-agent` passes (83 tests total)

## Implementation Notes

### Technical Approach
Created a `tools` module with submodules for each tool type:
- `file.rs`: FileReadTool and FileWriteTool with base directory restrictions
- `shell.rs`: ShellTool with configurable safety (blocked commands, whitelist, timeout)
- `note.rs`: NoteTool with shared storage for session-wide notes

### Key Features
- **File tools**: Optional base directory restriction, path validation, create/overwrite permissions
- **Shell tool**: Command blocking, optional whitelist, configurable timeout, output truncation
- **Note tool**: CRUD operations, shared storage between tool instances, JSON output for structured data

### Dependencies
- tempfile (dev-dependency for testing)
- chrono for timestamps on notes

## Status Updates **[REQUIRED]**

### Session 1 - Completed
**Files created:**
- `crates/arawn-agent/src/tools/mod.rs` - Module exports
- `crates/arawn-agent/src/tools/file.rs` - FileReadTool, FileWriteTool
- `crates/arawn-agent/src/tools/shell.rs` - ShellTool, ShellConfig
- `crates/arawn-agent/src/tools/note.rs` - NoteTool, Note, NoteStorage

**Files modified:**
- `crates/arawn-agent/src/lib.rs` - Added tools module and exports
- `crates/arawn-agent/Cargo.toml` - Added tempfile dev-dependency

**Tests:** 83 tests total (32 new tests for built-in tools)