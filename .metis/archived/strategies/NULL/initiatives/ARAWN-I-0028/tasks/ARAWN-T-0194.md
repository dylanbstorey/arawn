---
id: directorymanager-core
level: task
title: "DirectoryManager core implementation"
short_code: "ARAWN-T-0194"
created_at: 2026-02-18T19:03:11.415661+00:00
updated_at: 2026-02-18T19:29:42.606457+00:00
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

# DirectoryManager core implementation

## Parent Initiative

[[ARAWN-I-0028]] - Workstream and Session Path Management

## Objective

Implement the core `DirectoryManager` struct in `arawn-workstream` that manages the convention-based directory structure for workstreams and sessions.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] `DirectoryManager` struct with configurable `base_path` (default `~/.arawn`)
- [x] `allowed_paths(workstream, session_id)` returns correct paths per access matrix
- [x] `create_workstream(name)` creates `production/` and `work/` directories
- [x] `create_scratch_session(session_id)` creates `scratch/sessions/<id>/work/`
- [x] Scratch sessions get isolated paths, named workstreams get shared paths
- [x] Unit tests for all methods
- [x] Integration with existing workstream creation flow

## Implementation Notes

### Location
`crates/arawn-workstream/src/directory.rs` (new file)

### Core API

```rust
pub struct DirectoryManager {
    base_path: PathBuf,
}

impl DirectoryManager {
    pub fn new(base_path: impl Into<PathBuf>) -> Self;
    pub fn default() -> Self; // Uses ~/.arawn
    
    /// Get allowed paths for a session based on workstream
    pub fn allowed_paths(&self, workstream: &str, session_id: &str) -> Vec<PathBuf>;
    
    /// Create workstream directory structure
    pub fn create_workstream(&self, name: &str) -> Result<PathBuf, DirectoryError>;
    
    /// Create scratch session work directory
    pub fn create_scratch_session(&self, session_id: &str) -> Result<PathBuf, DirectoryError>;
    
    /// Get workstream path
    pub fn workstream_path(&self, name: &str) -> PathBuf;
    
    /// Check if workstream exists
    pub fn workstream_exists(&self, name: &str) -> bool;
}
```

### Directory Structure Created

```
~/.arawn/workstreams/
├── scratch/sessions/<session-id>/work/   # Isolated per-session
├── <workstream>/production/              # Shared deliverables
└── <workstream>/work/                    # Shared working area
```

### Access Rules

| Workstream | Session | Allowed Paths |
|------------|---------|---------------|
| scratch | S1 | `scratch/sessions/S1/work/` only |
| my-blog | any | `my-blog/production/`, `my-blog/work/` |

### Dependencies
- None (foundational task)

### Risk Considerations
- Ensure thread-safety (struct should be `Send + Sync`)
- Handle race conditions in directory creation
- Consider atomic directory creation

## Status Updates

### 2026-02-18 - Implementation Complete

**Created**: `crates/arawn-workstream/src/directory.rs`

**Core API implemented**:
- `DirectoryManager::new(base_path)` - Custom base path
- `DirectoryManager::default()` - Uses `~/.arawn`
- `allowed_paths(workstream, session_id)` - Returns allowed paths per access matrix
- `create_workstream(name)` - Creates `production/` and `work/` directories
- `create_scratch_session(session_id)` - Creates `scratch/sessions/<id>/work/`
- `workstream_path()`, `production_path()`, `work_path()`, `scratch_session_path()`
- `workstream_exists()`, `is_valid_name()`, `is_valid_session_id()`
- `remove_scratch_session()` - Cleanup helper
- `list_scratch_sessions()`, `list_workstreams()` - Discovery helpers

**Access rules enforced**:
- Scratch sessions: Isolated to `scratch/sessions/<session-id>/work/` only
- Named workstreams: Shared `<workstream>/production/` and `<workstream>/work/`

**Validation**:
- Name/ID validation prevents path traversal (`../`, empty, invalid chars)

**Integration**:
- Added `DirectoryManager` field to `WorkstreamManager` (optional)
- Added `with_directory_manager()` builder method
- `create_workstream()` now creates production/work dirs when DM is configured

**Tests**: 19 unit tests + 1 doc test, all passing
- Thread-safety verified (`Send + Sync`)
- Idempotent operations verified
- Invalid input handling verified

**Dependencies added**:
- `dirs = "6.0"` (workspace + arawn-workstream)