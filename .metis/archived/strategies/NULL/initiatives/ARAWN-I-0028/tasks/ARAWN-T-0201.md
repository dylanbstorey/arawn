---
id: usage-stats-endpoint
level: task
title: "Usage stats endpoint"
short_code: "ARAWN-T-0201"
created_at: 2026-02-18T19:03:20.918528+00:00
updated_at: 2026-02-18T21:14:12.212627+00:00
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

# Usage stats endpoint

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[ARAWN-I-0028]]

## Objective

Implement disk usage tracking for workstreams with API endpoint for querying usage stats and warnings.

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

- [x] `DirectoryManager.get_usage()` calculates disk usage by area
- [x] `GET /api/v1/workstreams/:ws/usage` endpoint
- [x] Returns usage for `production/`, `work/`, and per-session breakdown
- [x] Generates warnings based on configured thresholds
- [x] Handles large directories efficiently (uses walkdir for efficient traversal)
- [x] Unit tests for usage calculation (8 tests)

## API Contract

```
GET /api/v1/workstreams/:ws/usage
Response: { 
  "production_mb": 450, 
  "work_mb": 120, 
  "sessions": [{ "id": "abc-123", "mb": 45 }],
  "total_mb": 570,
  "warnings": ["work approaching 1GB limit"]
}
```

## Implementation Notes

### Location
- `crates/arawn-workstream/src/directory.rs` - add `get_usage()` method
- `crates/arawn-server/src/routes/workstreams.rs` - add endpoint

### Core Logic

```rust
pub struct UsageStats {
    pub production_bytes: u64,
    pub work_bytes: u64,
    pub sessions: Vec<SessionUsage>,
    pub warnings: Vec<String>,
}

impl DirectoryManager {
    pub fn get_usage(&self, workstream: &str) -> Result<UsageStats, DirectoryError> {
        let ws_path = self.workstream_path(workstream);
        
        let production_bytes = dir_size(&ws_path.join("production"))?;
        let work_bytes = dir_size(&ws_path.join("work"))?;
        
        // For scratch, enumerate sessions
        let sessions = if workstream == "scratch" {
            self.get_session_usages()?
        } else {
            vec![]
        };
        
        let warnings = self.check_thresholds(production_bytes + work_bytes);
        
        Ok(UsageStats { production_bytes, work_bytes, sessions, warnings })
    }
}

fn dir_size(path: &Path) -> io::Result<u64> {
    let mut size = 0;
    for entry in walkdir::WalkDir::new(path) {
        let entry = entry?;
        if entry.file_type().is_file() {
            size += entry.metadata()?.len();
        }
    }
    Ok(size)
}
```

### Dependencies
- ARAWN-T-0194 (DirectoryManager)
- `walkdir` crate for directory traversal

## Status Updates

### Session 1 - Implementation Complete

**Completed:**

1. Added `walkdir` dependency to `arawn-workstream/Cargo.toml`

2. Implemented usage types in `directory.rs`:
   - `SessionUsage` struct with `id` and `bytes` fields
   - `UsageStats` struct with `production_bytes`, `work_bytes`, `sessions`, `total_bytes`, `warnings`
   - Helper methods `production_mb()`, `work_mb()`, `total_mb()` for MB conversion

3. Implemented `DirectoryManager::get_usage()` method:
   - Validates workstream name
   - Calculates production/ directory size recursively
   - For named workstreams: calculates work/ directory size
   - For scratch: enumerates per-session usage in sessions/*/work/
   - Generates warnings based on thresholds:
     - Work directory: 500MB warning
     - Production directory: 1GB warning
     - Session work: 100MB warning
   - Uses `walkdir` crate for efficient recursive traversal

4. Added helper methods:
   - `get_session_usages()` - enumerate session work directories and sizes
   - `dir_size()` - recursive directory size calculation with walkdir

5. Added API endpoint in `workstreams.rs`:
   - `GET /api/v1/workstreams/:ws/usage`
   - Returns `UsageResponse` with `production_mb`, `work_mb`, `sessions`, `total_mb`, `warnings`
   - Proper error handling for not found, invalid name, etc.

6. Exported new types in `lib.rs`:
   - `SessionUsage`, `UsageStats` from directory module
   - `get_usage_handler`, `SessionUsageResponse`, `UsageResponse` from routes

7. Added 8 unit tests:
   - `test_get_usage_basic` - named workstream with files
   - `test_get_usage_scratch_with_sessions` - scratch with per-session breakdown
   - `test_get_usage_empty_workstream` - empty workstream returns zeros
   - `test_get_usage_nonexistent_workstream` - proper error
   - `test_get_usage_invalid_name` - proper error
   - `test_get_usage_nested_directories` - recursive calculation
   - `test_usage_stats_mb_conversions` - MB helper methods
   - `test_dir_size_nonexistent` - handles missing paths

**Tests:** 124 workstream tests pass, all server tests pass