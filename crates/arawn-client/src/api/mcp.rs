//! MCP (Model Context Protocol) API.

use crate::client::ArawnClient;
use crate::error::Result;
use crate::types::{AddServerRequest, AddServerResponse, ListServersResponse, ListToolsResponse};

/// MCP API client.
pub struct McpApi {
    client: ArawnClient,
}

impl McpApi {
    pub(crate) fn new(client: ArawnClient) -> Self {
        Self { client }
    }

    /// List all MCP servers.
    pub async fn list_servers(&self) -> Result<ListServersResponse> {
        self.client.get("mcp/servers").await
    }

    /// Add an MCP server.
    pub async fn add_server(&self, request: AddServerRequest) -> Result<AddServerResponse> {
        self.client.post("mcp/servers", &request).await
    }

    /// Add a stdio MCP server.
    pub async fn add_stdio_server(
        &self,
        name: &str,
        command: &str,
        args: Vec<String>,
        auto_connect: bool,
    ) -> Result<AddServerResponse> {
        self.add_server(AddServerRequest {
            name: name.to_string(),
            command: Some(command.to_string()),
            args,
            env: Default::default(),
            url: None,
            auto_connect,
        })
        .await
    }

    /// Add an HTTP MCP server.
    pub async fn add_http_server(
        &self,
        name: &str,
        url: &str,
        auto_connect: bool,
    ) -> Result<AddServerResponse> {
        self.add_server(AddServerRequest {
            name: name.to_string(),
            command: None,
            args: Vec::new(),
            env: Default::default(),
            url: Some(url.to_string()),
            auto_connect,
        })
        .await
    }

    /// Remove an MCP server.
    pub async fn remove_server(&self, name: &str) -> Result<()> {
        self.client.delete(&format!("mcp/servers/{}", name)).await
    }

    /// List tools for a server.
    pub async fn list_tools(&self, server_name: &str) -> Result<ListToolsResponse> {
        self.client
            .get(&format!("mcp/servers/{}/tools", server_name))
            .await
    }

    /// Connect to a server.
    pub async fn connect(&self, server_name: &str) -> Result<()> {
        self.client
            .post::<serde_json::Value, _>(
                &format!("mcp/servers/{}/connect", server_name),
                &serde_json::json!({}),
            )
            .await?;
        Ok(())
    }

    /// Disconnect from a server.
    pub async fn disconnect(&self, server_name: &str) -> Result<()> {
        self.client
            .post::<serde_json::Value, _>(
                &format!("mcp/servers/{}/disconnect", server_name),
                &serde_json::json!({}),
            )
            .await?;
        Ok(())
    }
}
