---
id: file-promote-operation
level: task
title: "File promote operation"
short_code: "ARAWN-T-0197"
created_at: 2026-02-18T19:03:13.997738+00:00
updated_at: 2026-02-18T19:51:10.743404+00:00
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

# File promote operation

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[ARAWN-I-0028]]

## Objective

Implement file promotion from `work/` to `production/` with API endpoint and conflict handling.

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

## Acceptance Criteria

- [ ] `DirectoryManager.promote()` moves file from `work/` to `production/`
- [ ] `POST /api/v1/workstreams/:ws/files/promote` endpoint
- [ ] Path validation on both source and destination
- [ ] Conflict handling: append `(1)` to filename if exists
- [ ] WebSocket alert sent on conflict rename
- [ ] Returns promoted path and file size
- [ ] Unit and integration tests

## API Contract

```
POST /api/v1/workstreams/:ws/files/promote
Request:  { "source": "draft.md", "destination": "blog/final.md" }
Response: { "path": "production/blog/final.md", "bytes": 1234 }
Conflict: { "path": "production/blog/final(1).md", "bytes": 1234, "renamed": true }
```

## Implementation Notes

### Location
- `crates/arawn-workstream/src/directory.rs` - add `promote()` method
- `crates/arawn-server/src/routes/workstreams.rs` - add endpoint

### Core Logic

```rust
impl DirectoryManager {
    pub fn promote(
        &self,
        workstream: &str,
        source: &Path,      // Relative to work/
        destination: &Path, // Relative to production/
    ) -> Result<PromoteResult, DirectoryError> {
        let work_path = self.workstream_path(workstream).join("work");
        let prod_path = self.workstream_path(workstream).join("production");
        
        let src_full = work_path.join(source);
        let mut dest_full = prod_path.join(destination);
        
        // Create destination directory if needed
        if let Some(parent) = dest_full.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Handle conflict
        let renamed = if dest_full.exists() {
            dest_full = self.resolve_conflict(&dest_full);
            true
        } else {
            false
        };
        
        // Move file
        fs::rename(&src_full, &dest_full)?;
        
        Ok(PromoteResult { path: dest_full, renamed })
    }
    
    fn resolve_conflict(&self, path: &Path) -> PathBuf {
        // Append (1), (2), etc. until unique
    }
}
```

### Dependencies
- ARAWN-T-0194 (DirectoryManager)
- ARAWN-T-0195 (PathValidator)

## Status Updates

### Session 1 (2026-02-18)
- Implemented `PromoteResult` struct with path, bytes, renamed, and original_destination fields
- Added error variants: `SourceNotFound`, `NotAFile`, `WorkstreamNotFound`
- Implemented `DirectoryManager::promote()` method with:
  - Workstream validation
  - Source file validation
  - Automatic destination directory creation
  - Conflict resolution via `resolve_conflict()` helper
  - File move via `fs::rename()`
- Implemented `resolve_conflict()` to append `(1)`, `(2)`, etc. for conflicts
- Added API endpoint `POST /api/v1/workstreams/:ws/files/promote`
- Created `PromoteFileRequest` and `PromoteFileResponse` types
- Added 12 unit tests covering all acceptance criteria
- All 92 tests pass (31 in directory module)

### Acceptance Criteria Status
- [x] `DirectoryManager.promote()` moves file from `work/` to `production/`
- [x] `POST /api/v1/workstreams/:ws/files/promote` endpoint
- [x] Path validation on both source and destination
- [x] Conflict handling: append `(1)` to filename if exists
- [ ] WebSocket alert sent on conflict rename → **Split to ARAWN-T-0208**
- [x] Returns promoted path and file size
- [x] Unit and integration tests

**Note**: Task completed for core functionality. WebSocket notification split to separate task ARAWN-T-0208 as it requires protocol changes and broadcast system integration.