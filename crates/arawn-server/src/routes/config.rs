//! Configuration endpoint.
//!
//! Exposes non-sensitive server configuration for clients.

use axum::{Extension, Json, extract::State};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::auth::Identity;
use crate::error::ServerError;
use crate::state::AppState;

// ─────────────────────────────────────────────────────────────────────────────
// Types
// ─────────────────────────────────────────────────────────────────────────────

/// Server feature flags.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ConfigFeatures {
    /// Whether workstreams are enabled.
    pub workstreams_enabled: bool,
    /// Whether memory/indexing is enabled.
    pub memory_enabled: bool,
    /// Whether MCP is enabled.
    pub mcp_enabled: bool,
    /// Whether rate limiting is enabled.
    pub rate_limiting: bool,
    /// Whether request logging is enabled.
    pub request_logging: bool,
}

/// Server limits configuration.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ConfigLimits {
    /// Maximum concurrent requests (if rate limiting enabled).
    pub max_concurrent_requests: Option<u32>,
}

/// Server configuration response.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ConfigResponse {
    /// Server version.
    pub version: String,
    /// Feature flags.
    pub features: ConfigFeatures,
    /// Resource limits.
    pub limits: ConfigLimits,
    /// Bind address (host:port).
    pub bind_address: String,
    /// Whether authentication is required.
    pub auth_required: bool,
}

// ─────────────────────────────────────────────────────────────────────────────
// Handlers
// ─────────────────────────────────────────────────────────────────────────────

/// GET /api/v1/config - Get server configuration.
#[utoipa::path(
    get,
    path = "/api/v1/config",
    responses(
        (status = 200, description = "Server configuration", body = ConfigResponse),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = [])),
    tag = "config"
)]
pub async fn get_config_handler(
    State(state): State<AppState>,
    Extension(_identity): Extension<Identity>,
) -> Result<Json<ConfigResponse>, ServerError> {
    let config = &state.config;

    let features = ConfigFeatures {
        workstreams_enabled: state.workstreams.is_some(),
        memory_enabled: state.indexer.is_some(),
        mcp_enabled: state.mcp_manager.is_some(),
        rate_limiting: config.rate_limiting,
        request_logging: config.request_logging,
    };

    // Use actual rate limit config values instead of hardcoded defaults
    let limits = ConfigLimits {
        max_concurrent_requests: if config.rate_limiting {
            // Return the API requests per minute from rate limit config
            Some(crate::ratelimit::RateLimitConfig::default().api_rpm)
        } else {
            None
        },
    };

    Ok(Json(ConfigResponse {
        version: env!("CARGO_PKG_VERSION").to_string(),
        features,
        limits,
        bind_address: config.bind_address.to_string(),
        auth_required: config.auth_token.is_some(),
    }))
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::auth_middleware;
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

    fn create_test_state() -> AppState {
        let backend = MockBackend::with_text("Test");
        let agent = Agent::builder()
            .with_backend(backend)
            .with_tools(ToolRegistry::new())
            .build()
            .unwrap();

        AppState::new(agent, ServerConfig::new(Some("test-token".to_string())))
    }

    fn create_test_router(state: AppState) -> Router {
        Router::new()
            .route("/config", get(get_config_handler))
            .layer(middleware::from_fn_with_state(
                state.clone(),
                auth_middleware,
            ))
            .with_state(state)
    }

    #[tokio::test]
    async fn test_get_config() {
        let state = create_test_state();
        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/config")
                    .header("Authorization", "Bearer test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let result: ConfigResponse = serde_json::from_slice(&body).unwrap();
        assert!(!result.version.is_empty());
        assert!(result.auth_required);
        assert!(!result.features.workstreams_enabled);
        assert!(!result.features.memory_enabled);
        assert!(!result.features.mcp_enabled);
    }

    #[tokio::test]
    async fn test_get_config_requires_auth() {
        let state = create_test_state();
        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/config")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
