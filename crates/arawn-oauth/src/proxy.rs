//! HTTP proxy server for Claude OAuth passthrough.
//!
//! Accepts Anthropic Messages API requests on localhost and forwards them
//! upstream with OAuth Bearer token authentication and request mangling.

use axum::{
    Json, Router as AxumRouter,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
};
use futures::StreamExt;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

use crate::error::OAuthError;
use crate::passthrough::{Passthrough, PassthroughConfig, extract_api_key};
use crate::token_manager::SharedTokenManager;

/// Configuration for the proxy server.
#[derive(Debug, Clone)]
pub struct ProxyConfig {
    pub bind_addr: SocketAddr,
    pub enable_cors: bool,
    pub passthrough: PassthroughConfig,
    pub token_manager: Option<SharedTokenManager>,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            bind_addr: "127.0.0.1:0".parse().unwrap(),
            enable_cors: true,
            passthrough: PassthroughConfig::default(),
            token_manager: None,
        }
    }
}

impl ProxyConfig {
    pub fn new(bind_addr: SocketAddr) -> Self {
        Self {
            bind_addr,
            ..Default::default()
        }
    }

    pub fn with_token_manager(mut self, manager: SharedTokenManager) -> Self {
        self.token_manager = Some(manager);
        self
    }
}

/// Shared state for the proxy server.
struct ProxyState {
    passthrough: Passthrough,
}

/// The OAuth proxy server.
pub struct ProxyServer {
    config: ProxyConfig,
    state: Arc<ProxyState>,
}

impl ProxyServer {
    /// Create a passthrough-only proxy.
    pub fn new(config: ProxyConfig) -> Self {
        let mut passthrough = Passthrough::with_config(config.passthrough.clone());
        if let Some(tm) = &config.token_manager {
            passthrough = passthrough.with_token_manager(tm.clone());
        }

        Self {
            state: Arc::new(ProxyState { passthrough }),
            config,
        }
    }

    /// Build the axum router.
    pub fn router(&self) -> AxumRouter {
        let mut router = AxumRouter::new()
            .route("/v1/messages", post(handle_messages))
            .route("/health", get(handle_health))
            .with_state(self.state.clone());

        if self.config.enable_cors {
            router = router.layer(
                CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods(Any)
                    .allow_headers(Any),
            );
        }

        router
    }

    /// Run the proxy server.
    pub async fn run(self) -> std::io::Result<()> {
        let listener = TcpListener::bind(self.config.bind_addr).await?;
        let local_addr = listener.local_addr()?;
        tracing::info!(addr = %local_addr, "Starting OAuth proxy server");
        axum::serve(listener, self.router()).await
    }

    /// Run with graceful shutdown, returning the bound address.
    pub async fn run_with_shutdown(
        self,
        shutdown: impl std::future::Future<Output = ()> + Send + 'static,
    ) -> std::io::Result<SocketAddr> {
        let listener = TcpListener::bind(self.config.bind_addr).await?;
        let local_addr = listener.local_addr()?;
        tracing::info!(addr = %local_addr, "Starting OAuth proxy server");
        tokio::spawn(async move {
            axum::serve(listener, self.router())
                .with_graceful_shutdown(shutdown)
                .await
                .ok();
        });
        Ok(local_addr)
    }
}

/// Handle POST /v1/messages
async fn handle_messages(
    State(state): State<Arc<ProxyState>>,
    headers: HeaderMap,
    body: String,
) -> Result<axum::response::Response, ProxyError> {
    let raw_request: serde_json::Value = serde_json::from_str(&body)
        .map_err(|e| OAuthError::InvalidRequest(format!("Invalid JSON: {}", e)))?;

    let is_streaming = raw_request
        .get("stream")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let api_key = extract_api_key(&headers, state.passthrough.config());

    if is_streaming {
        let upstream_response = state
            .passthrough
            .forward_raw_stream(raw_request, api_key.as_deref())
            .await?;

        let content_type = upstream_response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("text/event-stream")
            .to_string();

        let stream = upstream_response
            .bytes_stream()
            .map(|result| result.map_err(std::io::Error::other));
        let body = axum::body::Body::from_stream(stream);

        let response = axum::response::Response::builder()
            .status(StatusCode::OK)
            .header("content-type", content_type)
            .header("cache-control", "no-cache")
            .body(body)
            .map_err(|e| OAuthError::Backend(format!("Failed to build response: {}", e)))?;

        Ok(response)
    } else {
        let response = state
            .passthrough
            .forward_raw(raw_request, api_key.as_deref())
            .await?;
        Ok(Json(response).into_response())
    }
}

/// Handle GET /health
async fn handle_health() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "service": "arawn-oauth-proxy"
    }))
}

/// Error type for proxy responses.
#[derive(Debug)]
pub struct ProxyError(OAuthError);

impl From<OAuthError> for ProxyError {
    fn from(err: OAuthError) -> Self {
        Self(err)
    }
}

impl IntoResponse for ProxyError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_type, message) = match &self.0 {
            OAuthError::Backend(msg) => (StatusCode::BAD_GATEWAY, "backend_error", msg.clone()),
            OAuthError::InvalidRequest(msg) => {
                (StatusCode::BAD_REQUEST, "invalid_request", msg.clone())
            }
            OAuthError::Network(msg) => (StatusCode::BAD_GATEWAY, "network_error", msg.clone()),
            OAuthError::Serialization(msg) => {
                (StatusCode::BAD_REQUEST, "serialization_error", msg.clone())
            }
            OAuthError::Config(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "config_error",
                msg.clone(),
            ),
        };

        let body = serde_json::json!({
            "type": "error",
            "error": {
                "type": error_type,
                "message": message
            }
        });

        (status, Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_health_endpoint() {
        let server = ProxyServer::new(ProxyConfig::default());
        let router = server.router();

        let response = router
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_proxy_config_default() {
        let config = ProxyConfig::default();
        assert!(config.enable_cors);
    }
}
