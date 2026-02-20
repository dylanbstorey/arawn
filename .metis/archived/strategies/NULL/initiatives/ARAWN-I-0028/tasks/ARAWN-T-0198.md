---
id: file-export-operation
level: task
title: "File export operation"
short_code: "ARAWN-T-0198"
created_at: 2026-02-18T19:03:14.828701+00:00
updated_at: 2026-02-18T21:14:11.111862+00:00
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

# File export operation

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[ARAWN-I-0028]]

## Objective

Implement file export from `production/` to external paths with API endpoint.

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

- [ ] `DirectoryManager.export()` copies file from `production/` to external path
- [ ] `POST /api/v1/workstreams/:ws/files/export` endpoint
- [ ] Source path validated within workstream production/
- [ ] Destination can be any local path (user responsibility)
- [ ] Copies file (not moves) - production content preserved
- [ ] Returns exported path and file size
- [ ] Unit and integration tests

## API Contract

```
POST /api/v1/workstreams/:ws/files/export
Request:  { "source": "report.pdf", "destination": "/mnt/dropbox/reports/" }
Response: { "exported_to": "/mnt/dropbox/reports/report.pdf", "bytes": 45678 }
```

## Implementation Notes

### Location
- `crates/arawn-workstream/src/directory.rs` - add `export()` method
- `crates/arawn-server/src/routes/workstreams.rs` - add endpoint

### Core Logic

```rust
impl DirectoryManager {
    pub fn export(
        &self,
        workstream: &str,
        source: &Path,       // Relative to production/
        destination: &Path,  // Absolute external path
    ) -> Result<ExportResult, DirectoryError> {
        let prod_path = self.workstream_path(workstream).join("production");
        let src_full = prod_path.join(source);
        
        // Validate source is in production
        if !src_full.starts_with(&prod_path) {
            return Err(DirectoryError::PathNotAllowed);
        }
        
        // Ensure source exists
        if !src_full.exists() {
            return Err(DirectoryError::NotFound(src_full));
        }
        
        // Determine destination file path
        let dest_full = if destination.is_dir() {
            destination.join(source.file_name().unwrap())
        } else {
            destination.to_path_buf()
        };
        
        // Create destination directory if needed
        if let Some(parent) = dest_full.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Copy file
        let bytes = fs::copy(&src_full, &dest_full)?;
        
        Ok(ExportResult { path: dest_full, bytes })
    }
}
```

### Security Note
Export destination is outside sandbox - user is responsible for choosing safe destinations. Consider warning in API response if destination looks sensitive.

### Dependencies
- ARAWN-T-0194 (DirectoryManager)
- ARAWN-T-0195 (PathValidator)

## Status Updates

### Session 1 (2026-02-18)
- Implemented `ExportResult` struct with path and bytes fields
- Implemented `DirectoryManager::export()` method with:
  - Workstream validation
  - Source file validation (exists, is file)
  - Smart destination handling (directory → append filename, file → use as-is)
  - Automatic destination directory creation
  - File copy (preserves source)
- Added API endpoint `POST /api/v1/workstreams/:ws/files/export`
- Created `ExportFileRequest` and `ExportFileResponse` types
- Added 9 unit tests covering all acceptance criteria
- All 101 tests pass (40 in directory module, 4 doctests)

### Acceptance Criteria Status
- [x] `DirectoryManager.export()` copies file from `production/` to external path
- [x] `POST /api/v1/workstreams/:ws/files/export` endpoint
- [x] Source path validated within workstream production/
- [x] Destination can be any local path (user responsibility)
- [x] Copies file (not moves) - production content preserved
- [x] Returns exported path and file size
- [x] Unit and integration tests