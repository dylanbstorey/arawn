//! Configuration traits for decoupled config passing between crates.
//!
//! These traits allow components to depend on configuration capabilities without
//! requiring direct knowledge of the full configuration structure. Each trait
//! represents a specific configuration capability.

use std::time::Duration;

/// Base trait for all configuration types.
///
/// Provides common functionality expected of all config types. Implementations
/// should be cheaply cloneable and thread-safe.
pub trait ConfigProvider: Clone + Send + Sync + 'static {}

/// Session management configuration.
///
/// Provides settings for session cache behavior including LRU eviction
/// and cleanup intervals.
pub trait HasSessionConfig: ConfigProvider {
    /// Maximum number of sessions to keep in cache before LRU eviction.
    fn max_sessions(&self) -> usize;

    /// Interval between cleanup runs for expired sessions.
    fn cleanup_interval(&self) -> Duration;

    /// Optional TTL for sessions (None = no expiry).
    fn session_ttl(&self) -> Option<Duration> {
        None
    }
}

/// Tool execution configuration.
///
/// Provides settings for tool execution limits and timeouts.
pub trait HasToolConfig: ConfigProvider {
    /// Timeout for shell command execution.
    fn shell_timeout(&self) -> Duration;

    /// Timeout for web/HTTP requests.
    fn web_timeout(&self) -> Duration;

    /// Maximum size of tool output in bytes before truncation.
    fn max_output_bytes(&self) -> usize;
}

/// Agent execution configuration.
///
/// Provides settings for agent behavior and limits.
pub trait HasAgentConfig: ConfigProvider {
    /// Maximum iterations for agent tool loops.
    fn max_iterations(&self) -> u32;

    /// Default timeout for agent operations.
    fn default_timeout(&self) -> Duration {
        Duration::from_secs(300) // 5 minutes
    }
}

/// Rate limiting configuration.
///
/// Provides settings for request rate limiting.
pub trait HasRateLimitConfig: ConfigProvider {
    /// Whether rate limiting is enabled.
    fn rate_limiting_enabled(&self) -> bool;

    /// Requests per minute per client.
    fn requests_per_minute(&self) -> u32;

    /// Burst allowance above steady rate.
    fn burst_size(&self) -> u32 {
        10
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Default implementations for common types
// ─────────────────────────────────────────────────────────────────────────────

/// Default session configuration values.
pub mod defaults {
    use std::time::Duration;

    pub const MAX_SESSIONS: usize = 10_000;
    pub const CLEANUP_INTERVAL_SECS: u64 = 60;
    pub const SHELL_TIMEOUT_SECS: u64 = 30;
    pub const WEB_TIMEOUT_SECS: u64 = 30;
    pub const MAX_OUTPUT_BYTES: usize = 102_400; // 100KB
    pub const MAX_ITERATIONS: u32 = 25;
    pub const REQUESTS_PER_MINUTE: u32 = 120;
    pub const BURST_SIZE: u32 = 10;
    pub const DEFAULT_PORT: u16 = 8080;
    pub const DEFAULT_BIND: &str = "127.0.0.1";
    /// Context usage warning threshold (percentage).
    pub const CONTEXT_WARNING_PERCENT: u8 = 70;
    /// Context usage critical threshold (percentage).
    pub const CONTEXT_CRITICAL_PERCENT: u8 = 90;

    pub fn cleanup_interval() -> Duration {
        Duration::from_secs(CLEANUP_INTERVAL_SECS)
    }

    pub fn shell_timeout() -> Duration {
        Duration::from_secs(SHELL_TIMEOUT_SECS)
    }

    pub fn web_timeout() -> Duration {
        Duration::from_secs(WEB_TIMEOUT_SECS)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Simple wrapper types for standalone config passing
// ─────────────────────────────────────────────────────────────────────────────

/// Standalone session configuration.
#[derive(Debug, Clone)]
pub struct SessionConfigProvider {
    pub max_sessions: usize,
    pub cleanup_interval: Duration,
    pub session_ttl: Option<Duration>,
}

impl Default for SessionConfigProvider {
    fn default() -> Self {
        Self {
            max_sessions: defaults::MAX_SESSIONS,
            cleanup_interval: defaults::cleanup_interval(),
            session_ttl: None,
        }
    }
}

impl ConfigProvider for SessionConfigProvider {}

impl HasSessionConfig for SessionConfigProvider {
    fn max_sessions(&self) -> usize {
        self.max_sessions
    }

    fn cleanup_interval(&self) -> Duration {
        self.cleanup_interval
    }

    fn session_ttl(&self) -> Option<Duration> {
        self.session_ttl
    }
}

/// Standalone tool configuration.
#[derive(Debug, Clone)]
pub struct ToolConfigProvider {
    pub shell_timeout: Duration,
    pub web_timeout: Duration,
    pub max_output_bytes: usize,
}

impl Default for ToolConfigProvider {
    fn default() -> Self {
        Self {
            shell_timeout: defaults::shell_timeout(),
            web_timeout: defaults::web_timeout(),
            max_output_bytes: defaults::MAX_OUTPUT_BYTES,
        }
    }
}

impl ConfigProvider for ToolConfigProvider {}

impl HasToolConfig for ToolConfigProvider {
    fn shell_timeout(&self) -> Duration {
        self.shell_timeout
    }

    fn web_timeout(&self) -> Duration {
        self.web_timeout
    }

    fn max_output_bytes(&self) -> usize {
        self.max_output_bytes
    }
}

/// Standalone agent configuration.
#[derive(Debug, Clone)]
pub struct AgentConfigProvider {
    pub max_iterations: u32,
    pub default_timeout: Duration,
}

impl Default for AgentConfigProvider {
    fn default() -> Self {
        Self {
            max_iterations: defaults::MAX_ITERATIONS,
            default_timeout: Duration::from_secs(300),
        }
    }
}

impl ConfigProvider for AgentConfigProvider {}

impl HasAgentConfig for AgentConfigProvider {
    fn max_iterations(&self) -> u32 {
        self.max_iterations
    }

    fn default_timeout(&self) -> Duration {
        self.default_timeout
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_config_defaults() {
        let config = SessionConfigProvider::default();
        assert_eq!(config.max_sessions(), defaults::MAX_SESSIONS);
        assert_eq!(config.cleanup_interval(), defaults::cleanup_interval());
        assert!(config.session_ttl().is_none());
    }

    #[test]
    fn test_tool_config_defaults() {
        let config = ToolConfigProvider::default();
        assert_eq!(config.shell_timeout(), defaults::shell_timeout());
        assert_eq!(config.web_timeout(), defaults::web_timeout());
        assert_eq!(config.max_output_bytes(), defaults::MAX_OUTPUT_BYTES);
    }

    #[test]
    fn test_agent_config_defaults() {
        let config = AgentConfigProvider::default();
        assert_eq!(config.max_iterations(), defaults::MAX_ITERATIONS);
    }

    #[test]
    fn test_custom_session_config() {
        let config = SessionConfigProvider {
            max_sessions: 5000,
            cleanup_interval: Duration::from_secs(120),
            session_ttl: Some(Duration::from_secs(3600)),
        };
        assert_eq!(config.max_sessions(), 5000);
        assert_eq!(config.cleanup_interval(), Duration::from_secs(120));
        assert_eq!(config.session_ttl(), Some(Duration::from_secs(3600)));
    }
}
