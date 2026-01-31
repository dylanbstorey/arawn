//! Health check endpoints.

use axum::{Json, Router, extract::State, routing::get};
use serde::{Deserialize, Serialize};

use crate::state::AppState;

/// Health check response.
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Service status.
    pub status: String,
    /// Service version.
    pub version: String,
}

/// Detailed health check response.
#[derive(Debug, Serialize)]
pub struct DetailedHealthResponse {
    /// Service status.
    pub status: String,
    /// Service version.
    pub version: String,
    /// Agent status.
    pub agent: AgentHealth,
}

/// Agent health status.
#[derive(Debug, Serialize)]
pub struct AgentHealth {
    /// Whether the agent is ready.
    pub ready: bool,
}

/// Simple health check (no auth required).
async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// Detailed health check (requires auth).
async fn health_detailed(State(_state): State<AppState>) -> Json<DetailedHealthResponse> {
    // TODO: Actually check agent health
    Json(DetailedHealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        agent: AgentHealth { ready: true },
    })
}

/// Create health check routes.
pub fn health_routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/health/detailed", get(health_detailed))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_health_endpoint() {
        let app = Router::new().route("/health", get(health));

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

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let health: HealthResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(health.status, "ok");
        assert!(!health.version.is_empty());
    }
}
