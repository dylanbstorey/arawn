//! HTTP API and WebSocket server for Arawn.
//!
//! This crate provides the network transport layer for interacting
//! with the Arawn agent via HTTP and WebSocket connections.
//!
//! # Features
//!
//! - REST API for request/response interactions
//! - WebSocket for real-time bidirectional communication
//! - SSE for streaming responses
//! - Token-based authentication
//! - Rate limiting
//! - Request logging
//!
//! # Example
//!
//! ```ignore
//! use arawn_server::{Server, ServerConfig};
//! use arawn_agent::Agent;
//!
//! let agent = Agent::new(llm_client);
//! let config = ServerConfig::new(Some("secret-token".to_string()))
//!     .with_bind_address("127.0.0.1:8080".parse()?);
//!
//! let server = Server::new(agent, config);
//! server.run().await?;
//! ```

pub mod auth;
pub mod config;
pub mod error;
pub mod ratelimit;
pub mod routes;
pub mod state;

pub use auth::{AuthError, AuthIdentity, Identity, auth_middleware};
pub use config::ServerConfig;
pub use error::{Result, ServerError};
pub use ratelimit::{RateLimitConfig, rate_limit_middleware, request_logging_middleware};
pub use routes::{ChatRequest, ChatResponse};
pub use state::AppState;

use std::net::SocketAddr;

use axum::{Router, middleware};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::info;

use arawn_agent::Agent;

/// The Arawn HTTP/WebSocket server.
pub struct Server {
    /// Application state.
    state: AppState,
}

impl Server {
    /// Create a new server with the given agent and configuration.
    pub fn new(agent: Agent, config: ServerConfig) -> Self {
        Self {
            state: AppState::new(agent, config),
        }
    }

    /// Create a server from a pre-built application state.
    pub fn from_state(state: AppState) -> Self {
        Self { state }
    }

    /// Build the router with all routes and middleware.
    pub fn router(&self) -> Router {
        use axum::routing::get;

        Router::new()
            // Health routes (no auth required for basic health)
            .merge(routes::health_routes())
            // WebSocket (auth happens via message, not HTTP header)
            .route("/ws", get(routes::ws_handler))
            // API routes will be added here
            .nest("/api/v1", self.api_routes())
            // Request logging (inner layer, runs first)
            .layer(middleware::from_fn_with_state(
                self.state.clone(),
                ratelimit::request_logging_middleware,
            ))
            // Rate limiting (outer layer, runs before request logging)
            .layer(middleware::from_fn_with_state(
                self.state.clone(),
                ratelimit::rate_limit_middleware,
            ))
            // TraceLayer for detailed HTTP tracing
            .layer(TraceLayer::new_for_http())
            .with_state(self.state.clone())
    }

    /// API routes (v1).
    ///
    /// All API routes require authentication via the auth middleware.
    fn api_routes(&self) -> Router<AppState> {
        use axum::routing::{delete, get, post};

        Router::new()
            // Chat endpoints
            .route("/chat", post(routes::chat_handler))
            .route("/chat/stream", post(routes::chat_stream_handler))
            // Session endpoints
            .route("/sessions", get(routes::list_sessions_handler))
            .route(
                "/sessions/{id}",
                get(routes::get_session_handler).delete(routes::delete_session_handler),
            )
            // Memory endpoints
            .route("/memory/search", get(routes::memory_search_handler))
            // Notes endpoints
            .route(
                "/notes",
                post(routes::create_note_handler).get(routes::list_notes_handler),
            )
            // Workstream endpoints
            .route(
                "/workstreams",
                post(routes::create_workstream_handler).get(routes::list_workstreams_handler),
            )
            .route(
                "/workstreams/{id}",
                get(routes::get_workstream_handler).delete(routes::delete_workstream_handler),
            )
            .route(
                "/workstreams/{id}/messages",
                post(routes::send_message_handler).get(routes::list_messages_handler),
            )
            .route("/workstreams/{id}/promote", post(routes::promote_handler))
            // MCP endpoints
            .route(
                "/mcp/servers",
                post(routes::add_server_handler).get(routes::list_servers_handler),
            )
            .route("/mcp/servers/{name}", delete(routes::remove_server_handler))
            .route(
                "/mcp/servers/{name}/tools",
                get(routes::list_server_tools_handler),
            )
            .route(
                "/mcp/servers/{name}/connect",
                post(routes::connect_server_handler),
            )
            .route(
                "/mcp/servers/{name}/disconnect",
                post(routes::disconnect_server_handler),
            )
            // Auth middleware for all API routes
            .layer(middleware::from_fn_with_state(
                self.state.clone(),
                auth::auth_middleware,
            ))
    }

    /// Run the server.
    pub async fn run(self) -> Result<()> {
        let addr = self.state.config.bind_address;
        let router = self.router();

        info!("Starting server on {}", addr);

        let listener = TcpListener::bind(addr)
            .await
            .map_err(|e| ServerError::Internal(format!("Failed to bind: {}", e)))?;

        axum::serve(listener, router)
            .await
            .map_err(|e| ServerError::Internal(format!("Server error: {}", e)))?;

        Ok(())
    }

    /// Run the server on a specific address (useful for testing).
    pub async fn run_on(self, addr: SocketAddr) -> Result<()> {
        let router = self.router();

        info!("Starting server on {}", addr);

        let listener = TcpListener::bind(addr)
            .await
            .map_err(|e| ServerError::Internal(format!("Failed to bind: {}", e)))?;

        axum::serve(listener, router)
            .await
            .map_err(|e| ServerError::Internal(format!("Server error: {}", e)))?;

        Ok(())
    }

    /// Get the configured bind address.
    pub fn bind_address(&self) -> SocketAddr {
        self.state.config.bind_address
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arawn_agent::{Agent, ToolRegistry};
    use arawn_llm::MockBackend;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    fn create_test_agent() -> Agent {
        let backend = MockBackend::with_text("Test response");
        Agent::builder()
            .with_backend(backend)
            .with_tools(ToolRegistry::new())
            .build()
            .expect("failed to create test agent")
    }

    #[tokio::test]
    async fn test_server_health_endpoint() {
        let agent = create_test_agent();
        let config = ServerConfig::new(Some("test-token".to_string()));
        let server = Server::new(agent, config);

        let app = server.router();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_server_config_builder() {
        let config = ServerConfig::new(Some("my-token".to_string()))
            .with_bind_address("0.0.0.0:9000".parse().unwrap())
            .with_rate_limiting(false)
            .with_request_logging(true);

        assert_eq!(config.auth_token, Some("my-token".to_string()));
        assert_eq!(config.bind_address.port(), 9000);
        assert!(!config.rate_limiting);
        assert!(config.request_logging);
    }
}
