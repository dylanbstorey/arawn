//! Path management configuration for workstreams.
//!
//! Controls directory paths, usage thresholds, cleanup intervals, and filesystem
//! monitoring behavior.
//!
//! # Configuration
//!
//! ```toml
//! [paths]
//! base_path = "~/.arawn"
//!
//! [paths.usage]
//! total_warning_gb = 10
//! workstream_warning_gb = 1
//! session_warning_mb = 200
//!
//! [paths.cleanup]
//! scratch_cleanup_days = 7
//! dry_run = false
//!
//! [paths.monitoring]
//! enabled = true
//! debounce_ms = 500
//! polling_interval_secs = 30
//! ```
//!
//! # Environment Variables
//!
//! - `ARAWN_BASE_PATH` - Override the base path for all workstream data
//! - `ARAWN_MONITORING_ENABLED` - Enable/disable filesystem monitoring ("true"/"false")

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Path management configuration.
///
/// Controls the base directory for workstreams, usage thresholds for warnings,
/// cleanup policies, and filesystem monitoring behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
#[derive(Default)]
pub struct PathConfig {
    /// Base path for all workstream data.
    /// Default: `~/.arawn`
    ///
    /// Can be overridden by the `ARAWN_BASE_PATH` environment variable.
    pub base_path: Option<PathBuf>,

    /// Usage threshold configuration.
    pub usage: UsageThresholds,

    /// Cleanup configuration.
    pub cleanup: CleanupConfig,

    /// Filesystem monitoring configuration.
    pub monitoring: MonitoringConfig,
}


impl PathConfig {
    /// Get the effective base path, checking environment variable first.
    ///
    /// Resolution order:
    /// 1. `ARAWN_BASE_PATH` environment variable
    /// 2. Configured `base_path` value
    /// 3. Default: `~/.arawn`
    pub fn effective_base_path(&self) -> PathBuf {
        // Check environment variable first
        if let Ok(env_path) = std::env::var("ARAWN_BASE_PATH") {
            return PathBuf::from(env_path);
        }

        // Use configured value or default
        self.base_path.clone().unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".arawn")
        })
    }

    /// Get total usage warning threshold in bytes.
    pub fn total_warning_bytes(&self) -> u64 {
        self.usage.total_warning_gb * 1024 * 1024 * 1024
    }

    /// Get per-workstream usage warning threshold in bytes.
    pub fn workstream_warning_bytes(&self) -> u64 {
        self.usage.workstream_warning_gb * 1024 * 1024 * 1024
    }

    /// Get per-session usage warning threshold in bytes.
    pub fn session_warning_bytes(&self) -> u64 {
        self.usage.session_warning_mb * 1024 * 1024
    }

    /// Check if filesystem monitoring is enabled (respects env var).
    pub fn monitoring_enabled(&self) -> bool {
        // Check environment variable first
        if let Ok(env_val) = std::env::var("ARAWN_MONITORING_ENABLED") {
            return env_val.eq_ignore_ascii_case("true") || env_val == "1";
        }
        self.monitoring.enabled
    }
}

/// Disk usage warning thresholds.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UsageThresholds {
    /// Total usage warning threshold in gigabytes.
    /// Default: 10 GB
    pub total_warning_gb: u64,

    /// Per-workstream usage warning threshold in gigabytes.
    /// Default: 1 GB
    pub workstream_warning_gb: u64,

    /// Per-session usage warning threshold in megabytes.
    /// Default: 200 MB
    pub session_warning_mb: u64,
}

impl Default for UsageThresholds {
    fn default() -> Self {
        Self {
            total_warning_gb: 10,
            workstream_warning_gb: 1,
            session_warning_mb: 200,
        }
    }
}

/// Cleanup configuration for scratch sessions and disk pressure.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CleanupConfig {
    /// Number of days after which inactive scratch sessions are cleaned up.
    /// Default: 7 days
    pub scratch_cleanup_days: u32,

    /// Dry run mode - log cleanup actions but don't actually delete.
    /// Default: false
    pub dry_run: bool,
}

impl Default for CleanupConfig {
    fn default() -> Self {
        Self {
            scratch_cleanup_days: 7,
            dry_run: false,
        }
    }
}

/// Filesystem monitoring configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MonitoringConfig {
    /// Enable filesystem monitoring.
    /// Default: true
    ///
    /// Can be overridden by `ARAWN_MONITORING_ENABLED` environment variable.
    pub enabled: bool,

    /// Event debounce interval in milliseconds.
    /// Rapid file changes within this window are coalesced.
    /// Default: 500ms
    pub debounce_ms: u64,

    /// Polling fallback interval in seconds.
    /// Used on platforms where native FS events aren't available.
    /// Default: 30 seconds
    pub polling_interval_secs: u64,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            debounce_ms: 500,
            polling_interval_secs: 30,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_config_defaults() {
        let config = PathConfig::default();
        assert!(config.base_path.is_none());
        assert_eq!(config.usage.total_warning_gb, 10);
        assert_eq!(config.usage.workstream_warning_gb, 1);
        assert_eq!(config.usage.session_warning_mb, 200);
        assert_eq!(config.cleanup.scratch_cleanup_days, 7);
        assert!(!config.cleanup.dry_run);
        assert!(config.monitoring.enabled);
        assert_eq!(config.monitoring.debounce_ms, 500);
        assert_eq!(config.monitoring.polling_interval_secs, 30);
    }

    #[test]
    fn test_usage_thresholds_defaults() {
        let thresholds = UsageThresholds::default();
        assert_eq!(thresholds.total_warning_gb, 10);
        assert_eq!(thresholds.workstream_warning_gb, 1);
        assert_eq!(thresholds.session_warning_mb, 200);
    }

    #[test]
    fn test_cleanup_config_defaults() {
        let config = CleanupConfig::default();
        assert_eq!(config.scratch_cleanup_days, 7);
        assert!(!config.dry_run);
    }

    #[test]
    fn test_monitoring_config_defaults() {
        let config = MonitoringConfig::default();
        assert!(config.enabled);
        assert_eq!(config.debounce_ms, 500);
        assert_eq!(config.polling_interval_secs, 30);
    }

    #[test]
    fn test_total_warning_bytes() {
        let config = PathConfig::default();
        // 10 GB in bytes
        assert_eq!(config.total_warning_bytes(), 10 * 1024 * 1024 * 1024);
    }

    #[test]
    fn test_workstream_warning_bytes() {
        let config = PathConfig::default();
        // 1 GB in bytes
        assert_eq!(config.workstream_warning_bytes(), 1024 * 1024 * 1024);
    }

    #[test]
    fn test_session_warning_bytes() {
        let config = PathConfig::default();
        // 200 MB in bytes
        assert_eq!(config.session_warning_bytes(), 200 * 1024 * 1024);
    }

    #[test]
    fn test_effective_base_path_default() {
        // Clear env var if set
        // SAFETY: Tests run single-threaded with --test-threads=1 or serially
        unsafe { std::env::remove_var("ARAWN_BASE_PATH") };

        let config = PathConfig::default();
        let path = config.effective_base_path();

        // Should end with .arawn
        assert!(path.ends_with(".arawn"));
    }

    #[test]
    fn test_effective_base_path_configured() {
        // SAFETY: Tests run single-threaded with --test-threads=1 or serially
        unsafe { std::env::remove_var("ARAWN_BASE_PATH") };

        let mut config = PathConfig::default();
        config.base_path = Some(PathBuf::from("/custom/path"));

        let path = config.effective_base_path();
        assert_eq!(path, PathBuf::from("/custom/path"));
    }

    #[test]
    fn test_effective_base_path_env_override() {
        // Set env var - should override configured value
        // SAFETY: Tests run single-threaded with --test-threads=1 or serially
        unsafe { std::env::set_var("ARAWN_BASE_PATH", "/from/env") };

        let mut config = PathConfig::default();
        config.base_path = Some(PathBuf::from("/configured/path"));

        let path = config.effective_base_path();
        assert_eq!(path, PathBuf::from("/from/env"));

        // Clean up
        unsafe { std::env::remove_var("ARAWN_BASE_PATH") };
    }

    #[test]
    fn test_monitoring_enabled_default() {
        // SAFETY: Tests run single-threaded with --test-threads=1 or serially
        unsafe { std::env::remove_var("ARAWN_MONITORING_ENABLED") };

        let config = PathConfig::default();
        assert!(config.monitoring_enabled());
    }

    #[test]
    fn test_monitoring_enabled_configured_false() {
        // SAFETY: Tests run single-threaded with --test-threads=1 or serially
        unsafe { std::env::remove_var("ARAWN_MONITORING_ENABLED") };

        let mut config = PathConfig::default();
        config.monitoring.enabled = false;

        assert!(!config.monitoring_enabled());
    }

    #[test]
    fn test_monitoring_enabled_env_true() {
        // SAFETY: Tests run single-threaded with --test-threads=1 or serially
        unsafe { std::env::set_var("ARAWN_MONITORING_ENABLED", "true") };

        let mut config = PathConfig::default();
        config.monitoring.enabled = false;

        // Env var should override
        assert!(config.monitoring_enabled());

        unsafe { std::env::remove_var("ARAWN_MONITORING_ENABLED") };
    }

    #[test]
    fn test_monitoring_enabled_env_false() {
        // SAFETY: Tests run single-threaded with --test-threads=1 or serially
        unsafe { std::env::set_var("ARAWN_MONITORING_ENABLED", "false") };

        let config = PathConfig::default();

        // Env var should override default true
        assert!(!config.monitoring_enabled());

        unsafe { std::env::remove_var("ARAWN_MONITORING_ENABLED") };
    }

    #[test]
    fn test_monitoring_enabled_env_numeric() {
        // SAFETY: Tests run single-threaded with --test-threads=1 or serially
        unsafe { std::env::set_var("ARAWN_MONITORING_ENABLED", "1") };

        let config = PathConfig::default();
        assert!(config.monitoring_enabled());

        unsafe { std::env::remove_var("ARAWN_MONITORING_ENABLED") };
    }

    #[test]
    fn test_custom_usage_thresholds() {
        let mut config = PathConfig::default();
        config.usage.total_warning_gb = 20;
        config.usage.workstream_warning_gb = 2;
        config.usage.session_warning_mb = 500;

        assert_eq!(config.total_warning_bytes(), 20 * 1024 * 1024 * 1024);
        assert_eq!(config.workstream_warning_bytes(), 2 * 1024 * 1024 * 1024);
        assert_eq!(config.session_warning_bytes(), 500 * 1024 * 1024);
    }
}
