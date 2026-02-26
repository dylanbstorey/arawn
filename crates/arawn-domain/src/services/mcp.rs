//! MCP service for tool discovery and management.
//!
//! The MCP service provides access to Model Context Protocol servers
//! and their tools.

use std::sync::Arc;

use arawn_mcp::{McpManager, McpServerConfig};
use tokio::sync::RwLock;
use tracing::debug;

use crate::error::{DomainError, Result};

/// Shared MCP manager type.
pub type SharedMcpManager = Arc<RwLock<McpManager>>;

/// Information about an MCP server.
#[derive(Debug, Clone)]
pub struct McpServerInfo {
    /// Server name.
    pub name: String,
    /// Server command.
    pub command: String,
    /// Whether the server is connected.
    pub connected: bool,
    /// Number of tools provided.
    pub tool_count: usize,
}

/// Information about an MCP tool.
#[derive(Debug, Clone)]
pub struct McpToolInfo {
    /// Tool name.
    pub name: String,
    /// Tool description.
    pub description: Option<String>,
    /// Server that provides this tool.
    pub server: String,
}

/// MCP service for tool discovery and management.
#[derive(Clone)]
pub struct McpService {
    manager: Option<SharedMcpManager>,
}

impl McpService {
    /// Create a new MCP service.
    pub fn new(manager: Option<SharedMcpManager>) -> Self {
        Self { manager }
    }

    /// Check if MCP is enabled.
    pub fn is_enabled(&self) -> bool {
        self.manager.is_some()
    }

    /// Get the MCP manager.
    pub fn manager(&self) -> Option<&SharedMcpManager> {
        self.manager.as_ref()
    }

    /// List all configured MCP server names.
    pub async fn list_server_names(&self) -> Result<Vec<String>> {
        let manager = self
            .manager
            .as_ref()
            .ok_or_else(|| DomainError::Mcp("MCP not enabled".to_string()))?;

        let guard = manager.read().await;
        let names: Vec<String> = guard.server_names().iter().map(|s| s.to_string()).collect();

        debug!(server_count = names.len(), "Listed MCP servers");
        Ok(names)
    }

    /// Check if a server is connected.
    pub async fn is_server_connected(&self, name: &str) -> Result<bool> {
        let manager = self
            .manager
            .as_ref()
            .ok_or_else(|| DomainError::Mcp("MCP not enabled".to_string()))?;

        let guard = manager.read().await;
        Ok(guard.is_connected(name))
    }

    /// Add a new MCP server configuration.
    pub async fn add_server(&self, config: McpServerConfig) -> Result<()> {
        let manager = self
            .manager
            .as_ref()
            .ok_or_else(|| DomainError::Mcp("MCP not enabled".to_string()))?;

        let name = config.name.clone();
        let mut guard = manager.write().await;
        guard.add_server(config);

        debug!(server = %name, "Added MCP server");
        Ok(())
    }

    /// Remove an MCP server.
    pub async fn remove_server(&self, name: &str) -> Result<bool> {
        let manager = self
            .manager
            .as_ref()
            .ok_or_else(|| DomainError::Mcp("MCP not enabled".to_string()))?;

        let mut guard = manager.write().await;
        let removed = guard.remove_server(name);

        if removed {
            debug!(server = name, "Removed MCP server");
        }
        Ok(removed)
    }

    /// Connect to all configured MCP servers.
    pub async fn connect_all(&self) -> Result<()> {
        let manager = self
            .manager
            .as_ref()
            .ok_or_else(|| DomainError::Mcp("MCP not enabled".to_string()))?;

        let mut guard = manager.write().await;
        guard
            .connect_all()
            .map_err(|e| DomainError::Mcp(e.to_string()))?;

        debug!("Connected to all MCP servers");
        Ok(())
    }

    /// Shutdown all MCP server connections.
    pub async fn shutdown_all(&self) -> Result<()> {
        let manager = self
            .manager
            .as_ref()
            .ok_or_else(|| DomainError::Mcp("MCP not enabled".to_string()))?;

        let mut guard = manager.write().await;
        guard
            .shutdown_all()
            .map_err(|e| DomainError::Mcp(e.to_string()))?;

        debug!("Shutdown all MCP servers");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_service_disabled() {
        let service = McpService::new(None);
        assert!(!service.is_enabled());
    }
}
