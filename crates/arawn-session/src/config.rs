//! Configuration for the session cache.

use std::time::Duration;

/// Default maximum number of sessions to cache.
/// With ~100KB average session size, this uses ~1GB of memory.
pub const DEFAULT_MAX_SESSIONS: usize = 10_000;

/// Default TTL for sessions (none by default - sessions don't expire).
pub const DEFAULT_TTL: Option<Duration> = None;

/// Configuration for the session cache.
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum number of sessions to cache before LRU eviction.
    pub max_sessions: usize,

    /// Optional time-to-live for cached sessions.
    /// Sessions that haven't been accessed within this duration will be evicted.
    pub ttl: Option<Duration>,

    /// Whether to run periodic cleanup of expired sessions.
    /// If false, expired sessions are only cleaned up on access.
    pub enable_cleanup_task: bool,

    /// Interval for the cleanup task (if enabled).
    pub cleanup_interval: Duration,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_sessions: DEFAULT_MAX_SESSIONS,
            ttl: DEFAULT_TTL,
            enable_cleanup_task: true,
            cleanup_interval: Duration::from_secs(60),
        }
    }
}

impl CacheConfig {
    /// Create a new configuration with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the maximum number of sessions to cache.
    pub fn with_max_sessions(mut self, max: usize) -> Self {
        self.max_sessions = max;
        self
    }

    /// Set the TTL for cached sessions.
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = Some(ttl);
        self
    }

    /// Disable TTL (sessions don't expire based on time).
    pub fn without_ttl(mut self) -> Self {
        self.ttl = None;
        self
    }

    /// Enable or disable the background cleanup task.
    pub fn with_cleanup_task(mut self, enabled: bool) -> Self {
        self.enable_cleanup_task = enabled;
        self
    }

    /// Set the cleanup interval.
    pub fn with_cleanup_interval(mut self, interval: Duration) -> Self {
        self.cleanup_interval = interval;
        self
    }
}
