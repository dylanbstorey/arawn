//! MCP (Model Context Protocol) server management endpoints.
//!
//! Provides REST API for runtime MCP server registration and management:
//! - `POST /api/v1/mcp/servers` - Add a new MCP server
//! - `DELETE /api/v1/mcp/servers/:name` - Remove an MCP server
//! - `GET /api/v1/mcp/servers` - List all connected servers and their tools
//! - `GET /api/v1/mcp/servers/:name/tools` - List tools for a specific server

use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use arawn_mcp::McpServerConfig;

use crate::auth::Identity;
use crate::error::ServerError;
use crate::state::AppState;

// ─────────────────────────────────────────────────────────────────────────────
// Request/Response Types
// ─────────────────────────────────────────────────────────────────────────────

/// Request to add a new MCP server.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AddServerRequest {
    /// Unique name for this server.
    pub name: String,

    /// Transport type: "stdio" or "http".
    #[serde(default)]
    pub transport: String,

    /// Command to execute (for stdio transport).
    #[serde(default)]
    pub command: String,

    /// URL for the server (for http transport).
    #[serde(default)]
    pub url: Option<String>,

    /// Arguments to pass to the command (for stdio transport).
    #[serde(default)]
    pub args: Vec<String>,

    /// Environment variables as key-value pairs (for stdio transport).
    #[serde(default)]
    #[schema(value_type = Vec<Vec<String>>)]
    pub env: Vec<(String, String)>,

    /// HTTP headers as key-value pairs (for http transport).
    #[serde(default)]
    #[schema(value_type = Vec<Vec<String>>)]
    pub headers: Vec<(String, String)>,

    /// Request timeout in seconds (for http transport).
    #[serde(default)]
    pub timeout_secs: Option<u64>,

    /// Number of retries (for http transport).
    #[serde(default)]
    pub retries: Option<u32>,

    /// Whether to connect immediately after adding.
    #[serde(default = "default_connect")]
    pub connect: bool,
}

fn default_connect() -> bool {
    true
}

/// Response after adding a server.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AddServerResponse {
    /// Server name.
    pub name: String,
    /// Whether the server was connected.
    pub connected: bool,
    /// Number of tools discovered (if connected).
    pub tool_count: Option<usize>,
    /// Error message if connection failed.
    pub error: Option<String>,
}

/// Information about a connected MCP server.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ServerInfo {
    /// Server name.
    pub name: String,
    /// Whether the server is connected.
    pub connected: bool,
    /// Number of tools available.
    pub tool_count: usize,
    /// Tool names.
    pub tools: Vec<String>,
}

/// Response for listing servers.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ListServersResponse {
    /// List of servers.
    pub servers: Vec<ServerInfo>,
    /// Total number of configured servers.
    pub total: usize,
    /// Total number of connected servers.
    pub connected: usize,
}

/// Information about a tool.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ToolInfo {
    /// Tool name.
    pub name: String,
    /// Tool description.
    pub description: Option<String>,
    /// Input schema (JSON Schema).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Object)]
    pub input_schema: Option<serde_json::Value>,
}

/// Response for listing tools from a server.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ListToolsResponse {
    /// Server name.
    pub server: String,
    /// List of tools.
    pub tools: Vec<ToolInfo>,
}

/// Response after removing a server.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RemoveServerResponse {
    /// Server name.
    pub name: String,
    /// Whether the server was removed.
    pub removed: bool,
}

// ─────────────────────────────────────────────────────────────────────────────
// Handlers
// ─────────────────────────────────────────────────────────────────────────────

/// POST /api/v1/mcp/servers - Add a new MCP server.
#[utoipa::path(
    post,
    path = "/api/v1/mcp/servers",
    request_body = AddServerRequest,
    responses(
        (status = 200, description = "Server added", body = AddServerResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "MCP not enabled"),
    ),
    security(("bearer_auth" = [])),
    tag = "mcp"
)]
pub async fn add_server_handler(
    State(state): State<AppState>,
    Extension(_identity): Extension<Identity>,
    Json(request): Json<AddServerRequest>,
) -> Result<Json<AddServerResponse>, ServerError> {
    let mcp_manager = state.mcp_manager.as_ref().ok_or_else(|| {
        ServerError::Internal("MCP not enabled on this server".to_string())
    })?;

    // Validate request
    if request.name.is_empty() {
        return Err(ServerError::BadRequest("Server name is required".to_string()));
    }

    // Check if server already exists
    {
        let manager = mcp_manager.read().await;
        if manager.has_server(&request.name) {
            return Err(ServerError::BadRequest(format!(
                "Server '{}' already exists",
                request.name
            )));
        }
    }

    // Build server config based on transport type
    let transport_type = request.transport.to_lowercase();
    let config = if transport_type == "http" {
        let url = request.url.as_ref().ok_or_else(|| {
            ServerError::BadRequest("URL is required for HTTP transport".to_string())
        })?;

        let mut config = McpServerConfig::http(&request.name, url);

        for (key, value) in &request.headers {
            config = config.with_header(key.clone(), value.clone());
        }

        if let Some(timeout) = request.timeout_secs {
            config = config.with_timeout(std::time::Duration::from_secs(timeout));
        }

        if let Some(retries) = request.retries {
            config = config.with_retries(retries);
        }

        config
    } else {
        // Default to stdio
        if request.command.is_empty() {
            return Err(ServerError::BadRequest(
                "Command is required for stdio transport".to_string(),
            ));
        }

        McpServerConfig::new(&request.name, &request.command)
            .with_args(request.args.clone())
            .with_env(request.env.clone())
    };

    // Add server to manager
    {
        let mut manager = mcp_manager.write().await;
        manager.add_server(config);
    }

    // Optionally connect
    let (connected, tool_count, error) = if request.connect {
        let mut manager = mcp_manager.write().await;
        match manager.connect_server_by_name(&request.name) {
            Ok(()) => {
                // Count tools
                let tools = manager
                    .list_all_tools()
                    .unwrap_or_default()
                    .get(&request.name)
                    .map(|t| t.len())
                    .unwrap_or(0);
                (true, Some(tools), None)
            }
            Err(e) => (false, None, Some(e.to_string())),
        }
    } else {
        (false, None, None)
    };

    Ok(Json(AddServerResponse {
        name: request.name,
        connected,
        tool_count,
        error,
    }))
}

/// DELETE /api/v1/mcp/servers/:name - Remove an MCP server.
#[utoipa::path(
    delete,
    path = "/api/v1/mcp/servers/{name}",
    params(
        ("name" = String, Path, description = "Server name"),
    ),
    responses(
        (status = 200, description = "Server removed", body = RemoveServerResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Server not found"),
        (status = 500, description = "MCP not enabled"),
    ),
    security(("bearer_auth" = [])),
    tag = "mcp"
)]
pub async fn remove_server_handler(
    State(state): State<AppState>,
    Extension(_identity): Extension<Identity>,
    Path(server_name): Path<String>,
) -> Result<Json<RemoveServerResponse>, ServerError> {
    let mcp_manager = state.mcp_manager.as_ref().ok_or_else(|| {
        ServerError::Internal("MCP not enabled on this server".to_string())
    })?;

    let removed = {
        let mut manager = mcp_manager.write().await;
        manager.remove_server(&server_name)
    };

    if removed {
        Ok(Json(RemoveServerResponse {
            name: server_name,
            removed: true,
        }))
    } else {
        Err(ServerError::NotFound(format!(
            "Server '{}' not found",
            server_name
        )))
    }
}

/// GET /api/v1/mcp/servers - List all MCP servers.
#[utoipa::path(
    get,
    path = "/api/v1/mcp/servers",
    responses(
        (status = 200, description = "List of servers", body = ListServersResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "MCP not enabled"),
    ),
    security(("bearer_auth" = [])),
    tag = "mcp"
)]
pub async fn list_servers_handler(
    State(state): State<AppState>,
    Extension(_identity): Extension<Identity>,
) -> Result<Json<ListServersResponse>, ServerError> {
    let mcp_manager = state.mcp_manager.as_ref().ok_or_else(|| {
        ServerError::Internal("MCP not enabled on this server".to_string())
    })?;

    let manager = mcp_manager.read().await;

    let all_tools = manager.list_all_tools().unwrap_or_default();
    let server_names = manager.server_names();
    let total = server_names.len();
    let connected = manager.connected_count();

    let servers: Vec<ServerInfo> = server_names
        .into_iter()
        .map(|name| {
            let is_connected = manager.is_connected(name);
            let tools = all_tools
                .get(name)
                .map(|t| t.iter().map(|ti| ti.name.clone()).collect())
                .unwrap_or_default();
            let tool_count = if is_connected {
                all_tools.get(name).map(|t| t.len()).unwrap_or(0)
            } else {
                0
            };

            ServerInfo {
                name: name.to_string(),
                connected: is_connected,
                tool_count,
                tools,
            }
        })
        .collect();

    Ok(Json(ListServersResponse {
        servers,
        total,
        connected,
    }))
}

/// GET /api/v1/mcp/servers/:name/tools - List tools for a specific server.
#[utoipa::path(
    get,
    path = "/api/v1/mcp/servers/{name}/tools",
    params(
        ("name" = String, Path, description = "Server name"),
    ),
    responses(
        (status = 200, description = "List of tools", body = ListToolsResponse),
        (status = 400, description = "Server not connected"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Server not found"),
        (status = 500, description = "MCP not enabled"),
    ),
    security(("bearer_auth" = [])),
    tag = "mcp"
)]
pub async fn list_server_tools_handler(
    State(state): State<AppState>,
    Extension(_identity): Extension<Identity>,
    Path(server_name): Path<String>,
) -> Result<Json<ListToolsResponse>, ServerError> {
    let mcp_manager = state.mcp_manager.as_ref().ok_or_else(|| {
        ServerError::Internal("MCP not enabled on this server".to_string())
    })?;

    let manager = mcp_manager.read().await;

    // Check if server exists
    if !manager.has_server(&server_name) {
        return Err(ServerError::NotFound(format!(
            "Server '{}' not found",
            server_name
        )));
    }

    // Check if server is connected
    if !manager.is_connected(&server_name) {
        return Err(ServerError::BadRequest(format!(
            "Server '{}' is not connected",
            server_name
        )));
    }

    // Get client and list tools
    let client = manager.get_client(&server_name).ok_or_else(|| {
        ServerError::Internal(format!("Failed to get client for '{}'", server_name))
    })?;

    let tools = client
        .list_tools()
        .map_err(|e| ServerError::Internal(format!("Failed to list tools: {}", e)))?;

    let tool_infos: Vec<ToolInfo> = tools
        .into_iter()
        .map(|t| ToolInfo {
            name: t.name,
            description: t.description,
            input_schema: t.input_schema,
        })
        .collect();

    Ok(Json(ListToolsResponse {
        server: server_name,
        tools: tool_infos,
    }))
}

/// POST /api/v1/mcp/servers/:name/connect - Connect to a specific server.
#[utoipa::path(
    post,
    path = "/api/v1/mcp/servers/{name}/connect",
    params(
        ("name" = String, Path, description = "Server name"),
    ),
    responses(
        (status = 200, description = "Server connected"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Server not found"),
        (status = 500, description = "MCP not enabled or connection failed"),
    ),
    security(("bearer_auth" = [])),
    tag = "mcp"
)]
pub async fn connect_server_handler(
    State(state): State<AppState>,
    Extension(_identity): Extension<Identity>,
    Path(server_name): Path<String>,
) -> Result<StatusCode, ServerError> {
    let mcp_manager = state.mcp_manager.as_ref().ok_or_else(|| {
        ServerError::Internal("MCP not enabled on this server".to_string())
    })?;

    let mut manager = mcp_manager.write().await;

    if !manager.has_server(&server_name) {
        return Err(ServerError::NotFound(format!(
            "Server '{}' not found",
            server_name
        )));
    }

    if manager.is_connected(&server_name) {
        return Ok(StatusCode::OK); // Already connected
    }

    manager
        .connect_server_by_name(&server_name)
        .map_err(|e| ServerError::Internal(format!("Failed to connect: {}", e)))?;

    Ok(StatusCode::OK)
}

/// POST /api/v1/mcp/servers/:name/disconnect - Disconnect from a specific server.
#[utoipa::path(
    post,
    path = "/api/v1/mcp/servers/{name}/disconnect",
    params(
        ("name" = String, Path, description = "Server name"),
    ),
    responses(
        (status = 200, description = "Server disconnected"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Server not found"),
        (status = 500, description = "MCP not enabled"),
    ),
    security(("bearer_auth" = [])),
    tag = "mcp"
)]
pub async fn disconnect_server_handler(
    State(state): State<AppState>,
    Extension(_identity): Extension<Identity>,
    Path(server_name): Path<String>,
) -> Result<StatusCode, ServerError> {
    let mcp_manager = state.mcp_manager.as_ref().ok_or_else(|| {
        ServerError::Internal("MCP not enabled on this server".to_string())
    })?;

    let mut manager = mcp_manager.write().await;

    if !manager.has_server(&server_name) {
        return Err(ServerError::NotFound(format!(
            "Server '{}' not found",
            server_name
        )));
    }

    manager.shutdown_server(&server_name);
    Ok(StatusCode::OK)
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
    use arawn_mcp::McpManager;
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode},
        middleware,
        routing::{delete, get, post},
    };
    use tower::ServiceExt;

    fn create_test_state_with_mcp() -> AppState {
        let backend = MockBackend::with_text("Test");
        let agent = Agent::builder()
            .with_backend(backend)
            .with_tools(ToolRegistry::new())
            .build()
            .unwrap();

        AppState::new(agent, ServerConfig::new(Some("test-token".to_string())))
            .with_mcp_manager(McpManager::new())
    }

    fn create_test_state_without_mcp() -> AppState {
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
            .route("/mcp/servers", post(add_server_handler).get(list_servers_handler))
            .route(
                "/mcp/servers/{name}",
                delete(remove_server_handler),
            )
            .route("/mcp/servers/{name}/tools", get(list_server_tools_handler))
            .route("/mcp/servers/{name}/connect", post(connect_server_handler))
            .route("/mcp/servers/{name}/disconnect", post(disconnect_server_handler))
            .layer(middleware::from_fn_with_state(
                state.clone(),
                auth_middleware,
            ))
            .with_state(state)
    }

    #[tokio::test]
    async fn test_list_servers_empty() {
        let state = create_test_state_with_mcp();
        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/mcp/servers")
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
        let result: ListServersResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(result.total, 0);
        assert_eq!(result.connected, 0);
        assert!(result.servers.is_empty());
    }

    #[tokio::test]
    async fn test_list_servers_mcp_disabled() {
        let state = create_test_state_without_mcp();
        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/mcp/servers")
                    .header("Authorization", "Bearer test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_add_server_missing_name() {
        let state = create_test_state_with_mcp();
        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/mcp/servers")
                    .header("Authorization", "Bearer test-token")
                    .header("Content-Type", "application/json")
                    .body(Body::from(r#"{"name": "", "command": "some-cmd"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_add_server_missing_command() {
        let state = create_test_state_with_mcp();
        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/mcp/servers")
                    .header("Authorization", "Bearer test-token")
                    .header("Content-Type", "application/json")
                    .body(Body::from(r#"{"name": "test", "command": "", "connect": false}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_add_server_success_no_connect() {
        let state = create_test_state_with_mcp();
        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/mcp/servers")
                    .header("Authorization", "Bearer test-token")
                    .header("Content-Type", "application/json")
                    .body(Body::from(
                        r#"{"name": "test-server", "command": "some-cmd", "connect": false}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let result: AddServerResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(result.name, "test-server");
        assert!(!result.connected);
        assert!(result.error.is_none());
    }

    #[tokio::test]
    async fn test_add_server_duplicate() {
        let state = create_test_state_with_mcp();

        // Add a server first
        {
            let mut manager = state.mcp_manager.as_ref().unwrap().write().await;
            manager.add_server(McpServerConfig::new("existing", "cmd"));
        }

        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/mcp/servers")
                    .header("Authorization", "Bearer test-token")
                    .header("Content-Type", "application/json")
                    .body(Body::from(
                        r#"{"name": "existing", "command": "another-cmd", "connect": false}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_remove_server_not_found() {
        let state = create_test_state_with_mcp();
        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri("/mcp/servers/nonexistent")
                    .header("Authorization", "Bearer test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_remove_server_success() {
        let state = create_test_state_with_mcp();

        // Add a server first
        {
            let mut manager = state.mcp_manager.as_ref().unwrap().write().await;
            manager.add_server(McpServerConfig::new("to-remove", "cmd"));
        }

        let app = create_test_router(state.clone());

        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri("/mcp/servers/to-remove")
                    .header("Authorization", "Bearer test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Verify removed
        let manager = state.mcp_manager.as_ref().unwrap().read().await;
        assert!(!manager.has_server("to-remove"));
    }

    #[tokio::test]
    async fn test_list_server_tools_not_found() {
        let state = create_test_state_with_mcp();
        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/mcp/servers/nonexistent/tools")
                    .header("Authorization", "Bearer test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_list_server_tools_not_connected() {
        let state = create_test_state_with_mcp();

        // Add a server but don't connect
        {
            let mut manager = state.mcp_manager.as_ref().unwrap().write().await;
            manager.add_server(McpServerConfig::new("not-connected", "cmd"));
        }

        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/mcp/servers/not-connected/tools")
                    .header("Authorization", "Bearer test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_connect_server_not_found() {
        let state = create_test_state_with_mcp();
        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/mcp/servers/nonexistent/connect")
                    .header("Authorization", "Bearer test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_disconnect_server_not_found() {
        let state = create_test_state_with_mcp();
        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/mcp/servers/nonexistent/disconnect")
                    .header("Authorization", "Bearer test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_disconnect_server_success() {
        let state = create_test_state_with_mcp();

        // Add a server (not connected but it should still succeed)
        {
            let mut manager = state.mcp_manager.as_ref().unwrap().write().await;
            manager.add_server(McpServerConfig::new("to-disconnect", "cmd"));
        }

        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/mcp/servers/to-disconnect/disconnect")
                    .header("Authorization", "Bearer test-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_add_http_server() {
        let state = create_test_state_with_mcp();
        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/mcp/servers")
                    .header("Authorization", "Bearer test-token")
                    .header("Content-Type", "application/json")
                    .body(Body::from(
                        r#"{
                            "name": "http-server",
                            "transport": "http",
                            "url": "http://localhost:8080/mcp",
                            "connect": false
                        }"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let result: AddServerResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(result.name, "http-server");
        assert!(!result.connected);
    }

    #[tokio::test]
    async fn test_add_http_server_missing_url() {
        let state = create_test_state_with_mcp();
        let app = create_test_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/mcp/servers")
                    .header("Authorization", "Bearer test-token")
                    .header("Content-Type", "application/json")
                    .body(Body::from(
                        r#"{
                            "name": "http-server",
                            "transport": "http",
                            "connect": false
                        }"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
