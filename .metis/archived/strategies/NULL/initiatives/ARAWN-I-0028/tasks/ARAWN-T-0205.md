---
id: path-management-configuration
level: task
title: "Path management configuration"
short_code: "ARAWN-T-0205"
created_at: 2026-02-18T19:03:24.432222+00:00
updated_at: 2026-02-18T21:22:17.361258+00:00
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

# Path management configuration

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[ARAWN-I-0028]]

## Objective

Add configuration settings for path management including base path, usage thresholds, cleanup intervals, and monitoring options.

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

- [x] Configuration struct for path management settings
- [x] Base path configurable via env var and config file (default `~/.arawn`)
- [x] Usage warning thresholds: total (10GB), workstream (1GB), session (200MB)
- [x] Scratch cleanup interval (default 7 days)
- [x] FS monitoring toggle (default on) and polling interval (30s)
- [x] Event debounce interval (500ms)
- [x] Settings loaded at server startup (available via ArawnConfig)
- [x] Documentation of all settings (module docs + TOML example in config)

## Implementation Notes

### Location
- `crates/arawn-config/src/paths.rs` (new file)

### Configuration Struct

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct PathConfig {
    /// Base path for workstreams (default: ~/.arawn)
    #[serde(default = "default_base_path")]
    pub base_path: PathBuf,
    
    /// Total usage warning threshold in bytes (default: 10GB)
    #[serde(default = "default_total_warning")]
    pub total_usage_warning_bytes: u64,
    
    /// Per-workstream warning threshold (default: 1GB)
    #[serde(default = "default_workstream_warning")]
    pub workstream_usage_warning_bytes: u64,
    
    /// Per-session warning threshold (default: 200MB)
    #[serde(default = "default_session_warning")]
    pub session_usage_warning_bytes: u64,
    
    /// Scratch cleanup: days of inactivity (default: 7)
    #[serde(default = "default_cleanup_days")]
    pub scratch_cleanup_days: u32,
    
    /// Enable filesystem monitoring (default: true)
    #[serde(default = "default_monitoring_enabled")]
    pub monitoring_enabled: bool,
    
    /// Polling fallback interval in seconds (default: 30)
    #[serde(default = "default_polling_interval")]
    pub polling_interval_secs: u64,
    
    /// Event debounce in milliseconds (default: 500)
    #[serde(default = "default_debounce")]
    pub debounce_ms: u64,
}
```

### Environment Variables
- `ARAWN_BASE_PATH` - Override base path
- `ARAWN_TOTAL_USAGE_WARNING_GB` - Override total warning
- `ARAWN_MONITORING_ENABLED` - Enable/disable monitoring

### Dependencies
- None (foundational)

## Status Updates

### Session 2 - Completed
**Implementation completed:**

1. **Created `crates/arawn-config/src/paths.rs`** with:
   - `PathConfig` struct with base_path, usage thresholds, cleanup config, and monitoring config
   - `UsageThresholds` struct (total_warning_gb, workstream_warning_gb, session_warning_mb)
   - `CleanupConfig` struct (scratch_cleanup_days, dry_run)
   - `MonitoringConfig` struct (enabled, debounce_ms, polling_interval_secs)
   - Environment variable support: `ARAWN_BASE_PATH`, `ARAWN_MONITORING_ENABLED`
   - Helper methods: `effective_base_path()`, `total_warning_bytes()`, `workstream_warning_bytes()`, `session_warning_bytes()`, `monitoring_enabled()`
   - 15 unit tests

2. **Integrated into config system** (`crates/arawn-config/src/lib.rs`, `types.rs`):
   - Added `pub mod paths;` and type exports
   - Added `paths: Option<PathConfig>` to `ArawnConfig`
   - Added paths field to `RawConfig` for TOML parsing
   - Added merge logic for paths in `ArawnConfig::merge()`
   - Added 4 integration tests for TOML parsing/roundtrip

3. **Tests**: All 129 arawn-config tests pass

**Configuration example:**
```toml
[paths]
base_path = "~/.arawn"

[paths.usage]
total_warning_gb = 10
workstream_warning_gb = 1
session_warning_mb = 200

[paths.cleanup]
scratch_cleanup_days = 7
dry_run = false

[paths.monitoring]
enabled = true
debounce_ms = 500
polling_interval_secs = 30
```