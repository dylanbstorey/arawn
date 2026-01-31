//! Rate limiting middleware using governor.
//!
//! Provides per-IP rate limiting for API endpoints to prevent abuse.

use std::num::NonZeroU32;
use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use governor::{
    Quota, RateLimiter,
    state::{InMemoryState, NotKeyed},
};
use serde::Serialize;

use crate::state::AppState;

// ─────────────────────────────────────────────────────────────────────────────
// Types
// ─────────────────────────────────────────────────────────────────────────────

/// Rate limiter type alias (uses default clock).
pub type SharedRateLimiter =
    Arc<RateLimiter<NotKeyed, InMemoryState, governor::clock::DefaultClock>>;

/// Rate limit configuration.
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Requests per minute for chat endpoints.
    pub chat_rpm: u32,
    /// Requests per minute for general API endpoints.
    pub api_rpm: u32,
    /// Enable rate limiting.
    pub enabled: bool,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            chat_rpm: 60,
            api_rpm: 120,
            enabled: true,
        }
    }
}

/// Rate limit error response.
#[derive(Debug, Serialize)]
struct RateLimitError {
    error: String,
    code: u16,
    retry_after_seconds: Option<u64>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Rate Limiter Factory
// ─────────────────────────────────────────────────────────────────────────────

/// Create a rate limiter with the specified requests per minute.
pub fn create_rate_limiter(requests_per_minute: u32) -> SharedRateLimiter {
    let quota = Quota::per_minute(
        NonZeroU32::new(requests_per_minute).unwrap_or(NonZeroU32::new(60).unwrap()),
    );
    Arc::new(RateLimiter::direct(quota))
}

// ─────────────────────────────────────────────────────────────────────────────
// Middleware
// ─────────────────────────────────────────────────────────────────────────────

/// Rate limiting middleware for API endpoints.
///
/// Uses a global rate limiter (not per-IP) for simplicity.
/// Per-IP rate limiting would require extracting client IP from headers
/// which depends on reverse proxy configuration.
pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    // Skip if rate limiting is disabled
    if !state.config.rate_limiting {
        return next.run(request).await;
    }

    // Get or create the rate limiter
    let limiter = get_global_limiter();

    // Check rate limit
    match limiter.check() {
        Ok(_) => next.run(request).await,
        Err(_not_until) => {
            // Use a fixed retry-after of 1 second for simplicity
            // (the actual wait time depends on the quota period)
            let retry_after = 1u64;

            tracing::warn!(
                path = %request.uri().path(),
                retry_after_seconds = retry_after,
                "Rate limit exceeded"
            );

            let error = RateLimitError {
                error: "Rate limit exceeded".to_string(),
                code: 429,
                retry_after_seconds: Some(retry_after),
            };

            (
                StatusCode::TOO_MANY_REQUESTS,
                [("Retry-After", retry_after.to_string())],
                axum::Json(error),
            )
                .into_response()
        }
    }
}

/// Get the global rate limiter (120 requests per minute default).
fn get_global_limiter() -> SharedRateLimiter {
    use std::sync::OnceLock;
    static LIMITER: OnceLock<SharedRateLimiter> = OnceLock::new();
    LIMITER.get_or_init(|| create_rate_limiter(120)).clone()
}

// ─────────────────────────────────────────────────────────────────────────────
// Request Logging
// ─────────────────────────────────────────────────────────────────────────────

/// Structured request logging middleware.
///
/// Logs request details including method, path, status, and duration.
/// Uses tracing for structured logging that can be captured by log aggregators.
pub async fn request_logging_middleware(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    // Skip if request logging is disabled
    if !state.config.request_logging {
        return next.run(request).await;
    }

    let method = request.method().clone();
    let uri = request.uri().clone();
    let path = uri.path().to_string();

    let start = std::time::Instant::now();

    // Run the request
    let response = next.run(request).await;

    let duration = start.elapsed();
    let status = response.status();

    // Log based on status code
    if status.is_server_error() {
        tracing::error!(
            method = %method,
            path = %path,
            status = %status.as_u16(),
            duration_ms = %duration.as_millis(),
            "Request completed with server error"
        );
    } else if status.is_client_error() {
        tracing::warn!(
            method = %method,
            path = %path,
            status = %status.as_u16(),
            duration_ms = %duration.as_millis(),
            "Request completed with client error"
        );
    } else {
        tracing::info!(
            method = %method,
            path = %path,
            status = %status.as_u16(),
            duration_ms = %duration.as_millis(),
            "Request completed"
        );
    }

    response
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ServerConfig;
    use arawn_agent::{Agent, ToolRegistry};
    use arawn_llm::MockBackend;
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode},
        middleware,
        routing::get,
    };
    use tower::ServiceExt;

    fn create_test_state(rate_limiting: bool) -> AppState {
        let backend = MockBackend::with_text("Test");
        let agent = Agent::builder()
            .with_backend(backend)
            .with_tools(ToolRegistry::new())
            .build()
            .unwrap();

        let config =
            ServerConfig::new(Some("test-token".to_string())).with_rate_limiting(rate_limiting);
        AppState::new(agent, config)
    }

    async fn test_handler() -> &'static str {
        "ok"
    }

    fn create_test_router(state: AppState) -> Router {
        Router::new()
            .route("/test", get(test_handler))
            .layer(middleware::from_fn_with_state(
                state.clone(),
                rate_limit_middleware,
            ))
            .with_state(state)
    }

    #[tokio::test]
    async fn test_rate_limit_disabled() {
        let state = create_test_state(false);
        let app = create_test_router(state);

        // Should always succeed when disabled
        for _ in 0..10 {
            let response = app
                .clone()
                .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
                .await
                .unwrap();

            assert_eq!(response.status(), StatusCode::OK);
        }
    }

    #[tokio::test]
    async fn test_rate_limit_allows_requests() {
        let state = create_test_state(true);
        let app = create_test_router(state);

        // First request should succeed
        let response = app
            .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_create_rate_limiter() {
        let limiter = create_rate_limiter(60);
        // Should allow at least one request
        assert!(limiter.check().is_ok());
    }

    #[test]
    fn test_rate_limit_config_default() {
        let config = RateLimitConfig::default();
        assert_eq!(config.chat_rpm, 60);
        assert_eq!(config.api_rpm, 120);
        assert!(config.enabled);
    }
}
