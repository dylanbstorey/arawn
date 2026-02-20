---
id: manual-cleanup-endpoint
level: task
title: "Manual cleanup endpoint"
short_code: "ARAWN-T-0204"
created_at: 2026-02-18T19:03:23.410551+00:00
updated_at: 2026-02-18T21:17:38.148425+00:00
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

# Manual cleanup endpoint

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[ARAWN-I-0028]]

## Objective

Implement manual cleanup endpoint for user-triggered cleanup of workstream work directories.

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

- [x] `POST /api/v1/workstreams/:ws/cleanup` endpoint
- [x] Optional `older_than_days` parameter for age-based cleanup
- [x] Returns count of deleted files and freed space
- [x] Does NOT delete `production/` content (safety) - only cleans work/
- [x] Confirmation required for large deletions (>100 files)
- [x] Unit and integration tests (10 tests)

## API Contract

```
POST /api/v1/workstreams/:ws/cleanup
Request:  { "target": "work", "older_than_days": 7 }
Response: { "deleted_files": 12, "freed_mb": 85 }
```

## Implementation Notes

### Location
- `crates/arawn-workstream/src/directory.rs` - add `cleanup()` method
- `crates/arawn-server/src/routes/workstreams.rs` - add endpoint

### Core Logic

```rust
pub struct CleanupRequest {
    pub target: CleanupTarget,      // "work" only for safety
    pub older_than_days: Option<u32>,
    pub confirm_large: bool,        // Required if >100 files
}

pub enum CleanupTarget {
    Work,
}

impl DirectoryManager {
    pub fn cleanup(
        &self,
        workstream: &str,
        request: &CleanupRequest,
    ) -> Result<CleanupResult, DirectoryError> {
        let work_path = self.workstream_path(workstream).join("work");
        
        let cutoff = request.older_than_days.map(|days| {
            SystemTime::now() - Duration::from_secs(days as u64 * 86400)
        });
        
        let mut deleted = 0;
        let mut freed = 0;
        
        for entry in walkdir::WalkDir::new(&work_path) {
            let entry = entry?;
            if !entry.file_type().is_file() { continue; }
            
            // Check age if cutoff specified
            if let Some(cutoff) = cutoff {
                if entry.metadata()?.modified()? > cutoff {
                    continue;
                }
            }
            
            freed += entry.metadata()?.len();
            fs::remove_file(entry.path())?;
            deleted += 1;
        }
        
        Ok(CleanupResult { deleted_files: deleted, freed_bytes: freed })
    }
}
```

### Safety
- Never deletes from `production/` - only `work/`
- Requires `confirm_large: true` if >100 files would be deleted

### Dependencies
- ARAWN-T-0194 (DirectoryManager)
- ARAWN-T-0201 (Usage stats)

## Status Updates

### Session 1 - 2026-02-18

**Completed:**
- Added `ManualCleanupResult` struct with deleted_files, freed_bytes, pending_files, requires_confirmation
- Implemented `DirectoryManager::cleanup_work()` method with:
  - Workstream validation
  - Optional `older_than_days` age filter (only deletes files older than cutoff)
  - Confirmation requirement for >100 files (returns `requires_confirmation: true` without deleting)
  - Work directory cleanup for named workstreams
  - Session work directory cleanup for scratch workstream
  - Empty directory cleanup after file deletion
  - NEVER deletes from production/ (safety feature)
- Added `POST /api/v1/workstreams/:ws/cleanup` endpoint
- Added request/response types `CleanupRequest` and `CleanupResponse`
- Added 10 unit tests covering all scenarios
- All 17 cleanup-related tests pass

**Files modified:**
- `crates/arawn-workstream/src/directory.rs` (ManualCleanupResult, cleanup_work(), remove_empty_dirs())
- `crates/arawn-workstream/src/lib.rs` (export ManualCleanupResult)
- `crates/arawn-server/src/routes/workstreams.rs` (cleanup_handler, CleanupRequest, CleanupResponse)
- `crates/arawn-server/src/routes/mod.rs` (exports)
- `crates/arawn-server/src/lib.rs` (route registration)

**API Contract:**
```
POST /api/v1/workstreams/:ws/cleanup
Request:  { "older_than_days": 7, "confirm": false }
Response: { "deleted_files": 12, "freed_mb": 85.5 }
          OR { "pending_files": 105, "requires_confirmation": true }
```