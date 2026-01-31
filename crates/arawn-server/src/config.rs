//! Server configuration.

use std::net::SocketAddr;

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

    /// Enable request logging.
    pub request_logging: bool,

    /// CORS allowed origins (empty = no CORS).
    pub cors_origins: Vec<String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1:8080".parse().unwrap(),
            auth_token: None,
            tailscale_users: None,
            rate_limiting: true,
            request_logging: true,
            cors_origins: Vec::new(),
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
}
