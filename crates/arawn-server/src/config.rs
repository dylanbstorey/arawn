//! Server configuration.

use std::net::SocketAddr;
use std::time::Duration;

/// Default grace period for session reconnect tokens (30 seconds).
pub const DEFAULT_RECONNECT_GRACE_PERIOD: Duration = Duration::from_secs(30);

/// Default max message size for WebSocket (1 MB).
pub const DEFAULT_MAX_WS_MESSAGE_SIZE: usize = 1024 * 1024;

/// Default max body size for REST requests (10 MB).
pub const DEFAULT_MAX_BODY_SIZE: usize = 10 * 1024 * 1024;

/// Default WebSocket connections per minute per IP.
pub const DEFAULT_WS_CONNECTIONS_PER_MINUTE: u32 = 30;

/// Server configuration.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Address to bind the server to.
    pub bind_address: SocketAddr,

    /// Authentication token. `None` means auth is disabled (localhost mode).
    pub auth_token: Option<String>,

    /// Optional list of allowed Tailscale users.
    pub tailscale_users: Option<Vec<String>>,

    /// Enable rate limiting.
    pub rate_limiting: bool,

    /// Rate limit: requests per minute for API endpoints.
    pub api_rpm: u32,

    /// Enable request logging.
    pub request_logging: bool,

    /// CORS allowed origins (empty = no CORS).
    pub cors_origins: Vec<String>,

    /// Grace period for session ownership reconnect tokens.
    /// After disconnect, ownership is held for this duration to allow reconnection.
    pub reconnect_grace_period: Duration,

    // ─────────────────────────────────────────────────────────────────────────
    // Security settings
    // ─────────────────────────────────────────────────────────────────────────
    /// Maximum WebSocket message size in bytes.
    /// Messages exceeding this limit are rejected. Default: 1 MB.
    pub max_ws_message_size: usize,

    /// Maximum REST request body size in bytes.
    /// Requests exceeding this limit are rejected. Default: 10 MB.
    pub max_body_size: usize,

    /// Allowed origins for WebSocket connections.
    /// If empty, all origins are allowed (development mode only).
    /// In production, should contain allowed origin URLs.
    pub ws_allowed_origins: Vec<String>,

    /// Maximum WebSocket connections per minute per IP address.
    /// Prevents connection flood attacks. Default: 30.
    pub ws_connections_per_minute: u32,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1:8080".parse().unwrap(),
            auth_token: None,
            tailscale_users: None,
            rate_limiting: true,
            api_rpm: 120,
            request_logging: true,
            cors_origins: Vec::new(),
            reconnect_grace_period: DEFAULT_RECONNECT_GRACE_PERIOD,
            max_ws_message_size: DEFAULT_MAX_WS_MESSAGE_SIZE,
            max_body_size: DEFAULT_MAX_BODY_SIZE,
            ws_allowed_origins: Vec::new(),
            ws_connections_per_minute: DEFAULT_WS_CONNECTIONS_PER_MINUTE,
        }
    }
}

impl ServerConfig {
    /// Create a new server config with an optional auth token.
    /// Pass `None` to disable authentication (localhost mode).
    pub fn new(auth_token: Option<String>) -> Self {
        Self {
            auth_token,
            ..Default::default()
        }
    }

    /// Set the bind address.
    pub fn with_bind_address(mut self, addr: SocketAddr) -> Self {
        self.bind_address = addr;
        self
    }

    /// Set allowed Tailscale users.
    pub fn with_tailscale_users(mut self, users: Vec<String>) -> Self {
        self.tailscale_users = Some(users);
        self
    }

    /// Enable or disable rate limiting.
    pub fn with_rate_limiting(mut self, enabled: bool) -> Self {
        self.rate_limiting = enabled;
        self
    }

    /// Enable or disable request logging.
    pub fn with_request_logging(mut self, enabled: bool) -> Self {
        self.request_logging = enabled;
        self
    }

    /// Set CORS allowed origins.
    pub fn with_cors_origins(mut self, origins: Vec<String>) -> Self {
        self.cors_origins = origins;
        self
    }

    /// Set the API rate limit (requests per minute).
    pub fn with_api_rpm(mut self, rpm: u32) -> Self {
        self.api_rpm = rpm;
        self
    }

    /// Set the reconnect grace period for session ownership.
    pub fn with_reconnect_grace_period(mut self, duration: Duration) -> Self {
        self.reconnect_grace_period = duration;
        self
    }

    /// Set the maximum WebSocket message size.
    pub fn with_max_ws_message_size(mut self, size: usize) -> Self {
        self.max_ws_message_size = size;
        self
    }

    /// Set the maximum REST request body size.
    pub fn with_max_body_size(mut self, size: usize) -> Self {
        self.max_body_size = size;
        self
    }

    /// Set allowed origins for WebSocket connections.
    pub fn with_ws_allowed_origins(mut self, origins: Vec<String>) -> Self {
        self.ws_allowed_origins = origins;
        self
    }

    /// Set the maximum WebSocket connections per minute per IP.
    pub fn with_ws_connections_per_minute(mut self, rate: u32) -> Self {
        self.ws_connections_per_minute = rate;
        self
    }
}
