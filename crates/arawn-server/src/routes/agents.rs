//! Agent endpoints.
//!
//! Provides information about available agents and their capabilities.
//! Currently supports a single "main" agent, but designed for future
//! multi-agent support.

use axum::{
    Extension, Json,
    extract::{Path, State},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::auth::Identity;
use crate::error::ServerError;
use crate::state::AppState;

// ─────────────────────────────────────────────────────────────────────────────
// Types
// ─────────────────────────────────────────────────────────────────────────────

/// Information about a tool available to an agent.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AgentToolInfo {
    /// Tool name.
    pub name: String,
    /// Tool description.
    pub description: String,
}

/// Summary information about an agent.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AgentSummary {
    /// Agent ID.
    pub id: String,
    /// Agent name.
    pub name: String,
    /// Whether this is the default agent.
    pub is_default: bool,
    /// Number of tools available.
    pub tool_count: usize,
}

/// Detailed information about an agent.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AgentDetail {
    /// Agent ID.
    pub id: String,
    /// Agent name.
    pub name: String,
    /// Whether this is the default agent.
    pub is_default: bool,
    /// Tools available to this agent.
    pub tools: Vec<AgentToolInfo>,
    /// Agent capabilities/features.
    pub capabilities: AgentCapabilities,
}

/// Agent capabilities.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AgentCapabilities {
    /// Whether the agent supports streaming.
    pub streaming: bool,
    /// Whether the agent supports tool use.
    pub tool_use: bool,
    /// Maximum context length.
    pub max_context_length: Option<usize>,
}

/// Response for listing agents.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ListAgentsResponse {
    /// List of agents.
    pub agents: Vec<AgentSummary>,
    /// Total count.
    pub total: usize,
}

// ─────────────────────────────────────────────────────────────────────────────
// Handlers
// ─────────────────────────────────────────────────────────────────────────────

/// GET /api/v1/agents - List available agents.
#[utoipa::path(
    get,
    path = "/api/v1/agents",
    responses(
        (status = 200, description = "List of agents", body = ListAgentsResponse),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = [])),
    tag = "agents"
)]
pub async fn list_agents_handler(
    State(state): State<AppState>,
    Extension(_identity): Extension<Identity>,
) -> Result<Json<ListAgentsResponse>, ServerError> {
    // Currently we have a single "main" agent
    let tool_count = state.agent().tools().len();

    let agents = vec![AgentSummary {
        id: "main".to_string(),
        name: "Main Agent".to_string(),
        is_default: true,
        tool_count,
    }];

    Ok(Json(ListAgentsResponse {
        total: agents.len(),
        agents,
    }))
}

/// GET /api/v1/agents/:id - Get agent details.
#[utoipa::path(
    get,
    path = "/api/v1/agents/{id}",
    params(
        ("id" = String, Path, description = "Agent ID"),
    ),
    responses(
        (status = 200, description = "Agent details", body = AgentDetail),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Agent not found"),
    ),
    security(("bearer_auth" = [])),
    tag = "agents"
)]
pub async fn get_agent_handler(
    State(state): State<AppState>,
    Extension(_identity): Extension<Identity>,
    Path(agent_id): Path<String>,
) -> Result<Json<AgentDetail>, ServerError> {
    // Currently only "main" agent exists
    if agent_id != "main" {
        return Err(ServerError::NotFound(format!(
            "Agent '{}' not found",
            agent_id
        )));
    }

    let registry = state.agent().tools();
    let tools: Vec<AgentToolInfo> = registry
        .names()
        .into_iter()
        .filter_map(|name| {
            registry.get(name).map(|tool| AgentToolInfo {
                name: name.to_string(),
                description: tool.description().to_string(),
            })
        })
        .collect();

    Ok(Json(AgentDetail {
        id: "main".to_string(),
        name: "Main Agent".to_string(),
        is_default: true,
        tools,
        capabilities: AgentCapabilities {
            streaming: true,
            tool_use: true,
            max_context_length: None, // Provider-dependent
        },
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
            .route("/agents", get(list_agents_handler))
            .route("/agents/{id}", get(get_agent_handler))
            .layer(middleware::from_fn_with_state(
                state.clone(),
                auth_middleware,
            ))
            .with_state(state)
    }

    #[tokio::test]
    async fn test_list_agents() {
        let state = create_test_state();
        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/agents")
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
        let result: ListAgentsResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(result.total, 1);
        assert_eq!(result.agents[0].id, "main");
        assert!(result.agents[0].is_default);
    }

    #[tokio::test]
    async fn test_get_agent() {
        let state = create_test_state();
        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/agents/main")
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
        let result: AgentDetail = serde_json::from_slice(&body).unwrap();
        assert_eq!(result.id, "main");
        assert!(result.capabilities.streaming);
        assert!(result.capabilities.tool_use);
    }

    #[tokio::test]
    async fn test_get_agent_not_found() {
        let state = create_test_state();
        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/agents/nonexistent")
                    .header("Authorization", "Bearer test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_list_agents_requires_auth() {
        let state = create_test_state();
        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/agents")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
