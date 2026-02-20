//! Scheduled cleanup tasks for workstream management.
//!
//! Provides functions for cleaning up inactive scratch sessions and monitoring
//! disk pressure. These are designed to be used with cloacina's scheduler.

use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::directory::{DirectoryManager, SCRATCH_WORKSTREAM};
use crate::WorkstreamManager;

/// Configuration for cleanup tasks.
#[derive(Debug, Clone)]
pub struct CleanupConfig {
    /// Number of days after which inactive scratch sessions are cleaned up.
    pub scratch_cleanup_days: i64,
    /// Total disk usage warning threshold in bytes.
    pub total_usage_warning_bytes: u64,
    /// Per-workstream disk usage warning threshold in bytes.
    pub workstream_usage_warning_bytes: u64,
    /// Dry run mode - log actions but don't delete anything.
    pub dry_run: bool,
}

impl Default for CleanupConfig {
    fn default() -> Self {
        Self {
            scratch_cleanup_days: 7,
            total_usage_warning_bytes: 10 * 1024 * 1024 * 1024, // 10 GB
            workstream_usage_warning_bytes: 2 * 1024 * 1024 * 1024, // 2 GB
            dry_run: false,
        }
    }
}

/// Result of a scratch cleanup operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupResult {
    /// Number of sessions checked.
    pub sessions_checked: usize,
    /// Number of sessions cleaned up.
    pub sessions_cleaned: usize,
    /// Total bytes reclaimed.
    pub bytes_reclaimed: u64,
    /// IDs of cleaned sessions.
    pub cleaned_session_ids: Vec<String>,
    /// Whether this was a dry run.
    pub dry_run: bool,
}

/// Disk pressure alert levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PressureLevel {
    /// Usage is within acceptable limits.
    Ok,
    /// Usage is approaching the limit.
    Warning,
    /// Usage has exceeded the limit.
    Critical,
}

impl std::fmt::Display for PressureLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PressureLevel::Ok => write!(f, "ok"),
            PressureLevel::Warning => write!(f, "warning"),
            PressureLevel::Critical => write!(f, "critical"),
        }
    }
}

/// Disk pressure event for notification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskPressureEvent {
    /// Alert level.
    pub level: PressureLevel,
    /// Scope of the alert (e.g., "total" or workstream ID).
    pub scope: String,
    /// Current usage in megabytes.
    pub usage_mb: f64,
    /// Limit in megabytes.
    pub limit_mb: f64,
    /// Timestamp of the check.
    pub timestamp: DateTime<Utc>,
}

impl DiskPressureEvent {
    /// Create a new disk pressure event.
    pub fn new(level: PressureLevel, scope: impl Into<String>, usage_mb: f64, limit_mb: f64) -> Self {
        Self {
            level,
            scope: scope.into(),
            usage_mb,
            limit_mb,
            timestamp: Utc::now(),
        }
    }
}

/// Result of a disk pressure check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskPressureResult {
    /// Total usage across all workstreams in bytes.
    pub total_usage_bytes: u64,
    /// Per-workstream usage details.
    pub workstream_usage: Vec<WorkstreamUsage>,
    /// Any pressure events detected.
    pub events: Vec<DiskPressureEvent>,
    /// Timestamp of the check.
    pub timestamp: DateTime<Utc>,
}

/// Usage for a single workstream.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkstreamUsage {
    /// Workstream ID.
    pub id: String,
    /// Total usage in bytes.
    pub bytes: u64,
}

/// Clean up inactive scratch sessions.
///
/// Sessions are considered inactive if their last activity was more than
/// `config.scratch_cleanup_days` days ago.
///
/// # Arguments
///
/// * `dir_manager` - Directory manager for file operations
/// * `workstream_manager` - Workstream manager for session queries
/// * `config` - Cleanup configuration
///
/// # Returns
///
/// Result of the cleanup operation including number of sessions cleaned.
pub fn cleanup_scratch_sessions(
    dir_manager: &DirectoryManager,
    workstream_manager: &WorkstreamManager,
    config: &CleanupConfig,
) -> CleanupResult {
    let cutoff = Utc::now() - Duration::days(config.scratch_cleanup_days);
    let mut result = CleanupResult {
        sessions_checked: 0,
        sessions_cleaned: 0,
        bytes_reclaimed: 0,
        cleaned_session_ids: Vec::new(),
        dry_run: config.dry_run,
    };

    // List scratch sessions
    let sessions = match workstream_manager.list_sessions(SCRATCH_WORKSTREAM) {
        Ok(s) => s,
        Err(e) => {
            warn!("Failed to list scratch sessions: {}", e);
            return result;
        }
    };

    for session in sessions {
        result.sessions_checked += 1;

        // Check if session is inactive
        let last_activity = session.ended_at.unwrap_or(session.started_at);
        if last_activity >= cutoff {
            debug!(
                session_id = %session.id,
                last_activity = %last_activity,
                "Session still active, skipping"
            );
            continue;
        }

        // Get usage before cleanup
        let usage = dir_manager
            .get_usage(SCRATCH_WORKSTREAM)
            .ok()
            .and_then(|u| {
                u.sessions
                    .iter()
                    .find(|s| s.id == session.id)
                    .map(|s| s.bytes)
            })
            .unwrap_or(0);

        if config.dry_run {
            info!(
                session_id = %session.id,
                last_activity = %last_activity,
                bytes = usage,
                "Would clean up inactive scratch session (dry run)"
            );
        } else {
            // Delete the session's work directory
            if let Err(e) = delete_scratch_session_work(dir_manager, &session.id) {
                warn!(
                    session_id = %session.id,
                    error = %e,
                    "Failed to clean up scratch session"
                );
                continue;
            }

            info!(
                session_id = %session.id,
                last_activity = %last_activity,
                bytes = usage,
                "Cleaned up inactive scratch session"
            );
        }

        result.sessions_cleaned += 1;
        result.bytes_reclaimed += usage;
        result.cleaned_session_ids.push(session.id.clone());
    }

    info!(
        sessions_checked = result.sessions_checked,
        sessions_cleaned = result.sessions_cleaned,
        bytes_reclaimed = result.bytes_reclaimed,
        dry_run = config.dry_run,
        "Scratch cleanup completed"
    );

    result
}

/// Delete a scratch session's work directory.
fn delete_scratch_session_work(
    dir_manager: &DirectoryManager,
    session_id: &str,
) -> std::io::Result<()> {
    let session_path = dir_manager
        .workstream_path(SCRATCH_WORKSTREAM)
        .join("sessions")
        .join(session_id);

    if session_path.exists() {
        std::fs::remove_dir_all(&session_path)?;
    }

    Ok(())
}

/// Check disk pressure across workstreams.
///
/// # Arguments
///
/// * `dir_manager` - Directory manager for usage queries
/// * `workstream_manager` - Workstream manager for listing workstreams
/// * `config` - Cleanup configuration with thresholds
///
/// # Returns
///
/// Result including any pressure events detected.
pub fn check_disk_pressure(
    dir_manager: &DirectoryManager,
    workstream_manager: &WorkstreamManager,
    config: &CleanupConfig,
) -> DiskPressureResult {
    let mut result = DiskPressureResult {
        total_usage_bytes: 0,
        workstream_usage: Vec::new(),
        events: Vec::new(),
        timestamp: Utc::now(),
    };

    // Get list of workstreams
    let workstreams = match workstream_manager.list_workstreams() {
        Ok(ws) => ws,
        Err(e) => {
            warn!("Failed to list workstreams for disk pressure check: {}", e);
            return result;
        }
    };

    // Check each workstream
    for ws in workstreams {
        let usage = match dir_manager.get_usage(&ws.id) {
            Ok(u) => u,
            Err(e) => {
                debug!(
                    workstream_id = %ws.id,
                    error = %e,
                    "Failed to get usage for workstream"
                );
                continue;
            }
        };

        let ws_bytes = usage.total_bytes;
        result.total_usage_bytes += ws_bytes;
        result.workstream_usage.push(WorkstreamUsage {
            id: ws.id.clone(),
            bytes: ws_bytes,
        });

        // Check per-workstream threshold
        if ws_bytes > config.workstream_usage_warning_bytes {
            let level = if ws_bytes > config.workstream_usage_warning_bytes * 2 {
                PressureLevel::Critical
            } else {
                PressureLevel::Warning
            };

            result.events.push(DiskPressureEvent::new(
                level,
                &ws.id,
                ws_bytes as f64 / 1_048_576.0,
                config.workstream_usage_warning_bytes as f64 / 1_048_576.0,
            ));

            warn!(
                workstream_id = %ws.id,
                level = %level,
                usage_mb = ws_bytes as f64 / 1_048_576.0,
                limit_mb = config.workstream_usage_warning_bytes as f64 / 1_048_576.0,
                "Disk pressure detected for workstream"
            );
        }
    }

    // Also check scratch workstream
    if let Ok(scratch_usage) = dir_manager.get_usage(SCRATCH_WORKSTREAM) {
        result.total_usage_bytes += scratch_usage.total_bytes;
        result.workstream_usage.push(WorkstreamUsage {
            id: SCRATCH_WORKSTREAM.to_string(),
            bytes: scratch_usage.total_bytes,
        });
    }

    // Check total threshold
    if result.total_usage_bytes > config.total_usage_warning_bytes {
        let level = if result.total_usage_bytes > config.total_usage_warning_bytes * 2 {
            PressureLevel::Critical
        } else {
            PressureLevel::Warning
        };

        result.events.push(DiskPressureEvent::new(
            level,
            "total",
            result.total_usage_bytes as f64 / 1_048_576.0,
            config.total_usage_warning_bytes as f64 / 1_048_576.0,
        ));

        warn!(
            level = %level,
            total_usage_mb = result.total_usage_bytes as f64 / 1_048_576.0,
            limit_mb = config.total_usage_warning_bytes as f64 / 1_048_576.0,
            "Total disk pressure detected"
        );
    }

    info!(
        total_usage_mb = result.total_usage_bytes as f64 / 1_048_576.0,
        workstreams_checked = result.workstream_usage.len(),
        events = result.events.len(),
        "Disk pressure check completed"
    );

    result
}

/// Cleanup task context for cloacina integration.
///
/// This struct holds references needed by cleanup tasks and can be stored
/// in cloacina's context for scheduled execution.
#[derive(Clone)]
pub struct CleanupContext {
    /// Directory manager for file operations.
    pub dir_manager: Arc<DirectoryManager>,
    /// Workstream manager for session/workstream queries.
    pub workstream_manager: Arc<WorkstreamManager>,
    /// Cleanup configuration.
    pub config: CleanupConfig,
}

impl CleanupContext {
    /// Create a new cleanup context.
    pub fn new(
        dir_manager: Arc<DirectoryManager>,
        workstream_manager: Arc<WorkstreamManager>,
        config: CleanupConfig,
    ) -> Self {
        Self {
            dir_manager,
            workstream_manager,
            config,
        }
    }

    /// Run scratch cleanup.
    pub fn run_scratch_cleanup(&self) -> CleanupResult {
        cleanup_scratch_sessions(&self.dir_manager, &self.workstream_manager, &self.config)
    }

    /// Run disk pressure check.
    pub fn run_disk_pressure_check(&self) -> DiskPressureResult {
        check_disk_pressure(&self.dir_manager, &self.workstream_manager, &self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn setup() -> (tempfile::TempDir, DirectoryManager) {
        let dir = tempdir().unwrap();
        let manager = DirectoryManager::new(dir.path());
        (dir, manager)
    }

    #[test]
    fn test_cleanup_config_default() {
        let config = CleanupConfig::default();
        assert_eq!(config.scratch_cleanup_days, 7);
        assert_eq!(config.total_usage_warning_bytes, 10 * 1024 * 1024 * 1024);
        assert_eq!(config.workstream_usage_warning_bytes, 2 * 1024 * 1024 * 1024);
        assert!(!config.dry_run);
    }

    #[test]
    fn test_pressure_level_display() {
        assert_eq!(format!("{}", PressureLevel::Ok), "ok");
        assert_eq!(format!("{}", PressureLevel::Warning), "warning");
        assert_eq!(format!("{}", PressureLevel::Critical), "critical");
    }

    #[test]
    fn test_disk_pressure_event_new() {
        let event = DiskPressureEvent::new(PressureLevel::Warning, "my-ws", 1500.0, 2000.0);
        assert_eq!(event.level, PressureLevel::Warning);
        assert_eq!(event.scope, "my-ws");
        assert!((event.usage_mb - 1500.0).abs() < 0.01);
        assert!((event.limit_mb - 2000.0).abs() < 0.01);
    }

    #[test]
    fn test_disk_pressure_event_serialization() {
        let event = DiskPressureEvent::new(PressureLevel::Critical, "total", 12000.0, 10000.0);
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"level\":\"critical\""));
        assert!(json.contains("\"scope\":\"total\""));

        let deserialized: DiskPressureEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.level, event.level);
        assert_eq!(deserialized.scope, event.scope);
    }

    #[test]
    fn test_cleanup_result_serialization() {
        let result = CleanupResult {
            sessions_checked: 10,
            sessions_cleaned: 3,
            bytes_reclaimed: 1024 * 1024,
            cleaned_session_ids: vec!["s1".to_string(), "s2".to_string()],
            dry_run: false,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"sessions_checked\":10"));
        assert!(json.contains("\"sessions_cleaned\":3"));
        assert!(json.contains("\"bytes_reclaimed\":1048576"));

        let deserialized: CleanupResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.sessions_checked, result.sessions_checked);
        assert_eq!(deserialized.cleaned_session_ids.len(), 2);
    }

    #[test]
    fn test_delete_scratch_session_work_nonexistent() {
        let (_dir, manager) = setup();
        // Should not error for non-existent session
        let result = delete_scratch_session_work(&manager, "nonexistent-session");
        assert!(result.is_ok());
    }

    #[test]
    fn test_delete_scratch_session_work() {
        let (_dir, manager) = setup();

        // Create a scratch session with files
        let work_path = manager.create_scratch_session("test-session").unwrap();
        std::fs::write(work_path.join("file.txt"), "content").unwrap();
        assert!(work_path.exists());

        // Delete it
        delete_scratch_session_work(&manager, "test-session").unwrap();

        // Session directory should be gone
        let session_path = manager
            .workstream_path(SCRATCH_WORKSTREAM)
            .join("sessions")
            .join("test-session");
        assert!(!session_path.exists());
    }
}
