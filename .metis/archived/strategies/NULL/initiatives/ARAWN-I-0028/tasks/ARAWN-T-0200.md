---
id: attach-session-file-migration
level: task
title: "Attach session file migration"
short_code: "ARAWN-T-0200"
created_at: 2026-02-18T19:03:16.499249+00:00
updated_at: 2026-02-18T20:09:25.837718+00:00
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

# Attach session file migration

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[ARAWN-I-0028]]

## Objective

Enhance the existing session reassignment endpoint to migrate files when moving a session from scratch to a named workstream.

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

- [x] `DirectoryManager.attach_session()` migrates files from scratch to workstream
- [x] Existing `PATCH /api/v1/sessions/:id` handles file migration when `workstream_id` changes
- [x] Files moved to session-named subfolder in workstream `work/` to avoid conflicts
- [x] Empty scratch session directory cleaned up after migration
- [x] Response includes count of migrated files and new allowed paths
- [x] Integration test for scratch → workstream migration (7 unit tests added)

## API Enhancement

```
PATCH /api/v1/sessions/:id
Request:  { "workstream_id": "my-blog" }
Response: { 
  "id": "abc-123",
  "workstream_id": "my-blog",
  "files_migrated": 3,
  "allowed_paths": ["production/", "work/"]
}
```

## Implementation Notes

### Location
- `crates/arawn-workstream/src/directory.rs` - add `attach_session()` method
- `crates/arawn-server/src/routes/sessions.rs` - enhance existing `update_session_handler`

### Core Logic

```rust
impl DirectoryManager {
    pub fn attach_session(
        &self,
        session_id: &str,
        target_workstream: &str,
    ) -> Result<AttachResult, DirectoryError> {
        let scratch_work = self.base_path
            .join("workstreams/scratch/sessions")
            .join(session_id)
            .join("work");
        
        // Destination: work/<session_id>/ to avoid conflicts
        let dest_work = self.workstream_path(target_workstream)
            .join("work")
            .join(session_id);
        
        fs::create_dir_all(&dest_work)?;
        
        // Move all files
        let mut count = 0;
        for entry in fs::read_dir(&scratch_work)? {
            let entry = entry?;
            fs::rename(entry.path(), dest_work.join(entry.file_name()))?;
            count += 1;
        }
        
        // Clean up empty scratch session directory
        let scratch_session = scratch_work.parent().unwrap();
        fs::remove_dir_all(scratch_session)?;
        
        Ok(AttachResult {
            files_migrated: count,
            new_work_path: dest_work,
        })
    }
}
```

### Integration with Session Update

In `update_session_handler`:
```rust
if let Some(ref new_workstream_id) = request.workstream_id {
    // Check if moving FROM scratch
    let current_workstream = get_session_workstream(&session_id)?;
    if current_workstream == "scratch" && new_workstream_id != "scratch" {
        // Migrate files
        let result = directory_manager.attach_session(&session_id, new_workstream_id)?;
        // Include in response
    }
    // Existing reassignment logic...
}
```

### Dependencies
- ARAWN-T-0194 (DirectoryManager)

## Status Updates

### Session 1 - Implementation Complete

**Completed:**
1. Implemented `DirectoryManager::attach_session()` method in `directory.rs`:
   - Takes `session_id` and `target_workstream` parameters
   - Returns `AttachResult` with `files_migrated`, `new_work_path`, and `allowed_paths`
   - Validates session ID and workstream name
   - Handles cross-filesystem moves with `copy_dir_recursive()` fallback
   - Cleans up empty scratch session directories after migration
   - Added `SessionWorkNotFound` error variant for sessions without work directories

2. Updated `update_session_handler` in `sessions.rs`:
   - Gets current workstream ID before reassignment
   - Detects scratch → named workstream migration
   - Calls `DirectoryManager::attach_session()` when appropriate
   - Non-fatal error handling for missing work directories
   - Returns migration info in response via `session_to_detail_with_migration()`

3. Updated `SessionDetail` struct:
   - Added `workstream_id: Option<String>`
   - Added `files_migrated: Option<usize>`
   - Added `allowed_paths: Option<Vec<String>>`
   - Created `session_to_detail_with_migration()` helper

4. Added 7 unit tests for `attach_session()`:
   - Basic migration
   - Non-existent workstream error
   - Non-existent session work directory
   - Invalid session ID
   - Invalid workstream name
   - Cross-filesystem move fallback
   - Empty directory handling

**Tests:** All 116 workstream tests pass, all 114 server tests pass