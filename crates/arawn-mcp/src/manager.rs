//! MCP Manager for multi-server lifecycle management.
//!
//! The [`McpManager`] provides centralized management of multiple MCP server
//! connections, including initialization, tool discovery, and graceful shutdown.
//!
//! # Example
//!
//! ```rust,ignore
//! use arawn_mcp::{McpManager, McpServerConfig};
//!
//! let mut manager = McpManager::new();
//!
//! // Add servers
//! manager.add_server(McpServerConfig::new("sqlite", "mcp-server-sqlite")
//!     .with_arg("--db")
//!     .with_arg("/path/to/db.sqlite"))?;
//!
//! // Connect to all servers
//! manager.connect_all()?;
//!
//! // Get all available tools
//! let tools = manager.list_all_tools()?;
//! println!("Available tools: {}", tools.len());
//!
//! // Shutdown all servers
//! manager.shutdown_all()?;
//! ```

use std::collections::HashMap;
use std::sync::Arc;

use crate::client::{McpClient, McpServerConfig};
use crate::error::{McpError, Result};
use crate::protocol::ToolInfo;

/// Manager for multiple MCP server connections.
///
/// Provides lifecycle management for MCP servers, including:
/// - Adding and removing server configurations
/// - Connecting to and initializing all servers
/// - Discovering tools from all connected servers
/// - Graceful shutdown of all connections
#[derive(Default)]
pub struct McpManager {
    /// Server configurations (before connection).
    configs: HashMap<String, McpServerConfig>,
    /// Connected and initialized clients.
    clients: HashMap<String, Arc<McpClient>>,
}

impl McpManager {
    /// Create a new empty MCP manager.
    pub fn new() -> Self {
        Self {
            configs: HashMap::new(),
            clients: HashMap::new(),
        }
    }

    /// Create a manager with the given server configurations.
    pub fn with_configs(configs: Vec<McpServerConfig>) -> Self {
        let mut manager = Self::new();
        for config in configs {
            manager.configs.insert(config.name.clone(), config);
        }
        manager
    }

    /// Add a server configuration.
    ///
    /// The server will not be connected until [`connect_all`] is called.
    /// If a server with the same name already exists, it will be replaced.
    pub fn add_server(&mut self, config: McpServerConfig) {
        let name = config.name.clone();
        tracing::debug!(server = %name, "adding MCP server configuration");
        self.configs.insert(name, config);
    }

    /// Remove a server by name.
    ///
    /// If the server is connected, it will be disconnected first.
    /// Returns true if the server was found and removed.
    pub fn remove_server(&mut self, name: &str) -> bool {
        // Remove from clients (disconnects via Drop)
        if let Some(client) = self.clients.remove(name) {
            tracing::info!(server = %name, "disconnecting MCP server");
            // Drop the Arc, which will trigger shutdown if this is the last reference
            drop(client);
        }

        // Remove from configs
        if self.configs.remove(name).is_some() {
            tracing::debug!(server = %name, "removed MCP server configuration");
            true
        } else {
            false
        }
    }

    /// Get the names of all configured servers.
    pub fn server_names(&self) -> Vec<&str> {
        self.configs.keys().map(|s| s.as_str()).collect()
    }

    /// Get the names of all connected servers.
    pub fn connected_server_names(&self) -> Vec<&str> {
        self.clients.keys().map(|s| s.as_str()).collect()
    }

    /// Check if a server is configured.
    pub fn has_server(&self, name: &str) -> bool {
        self.configs.contains_key(name)
    }

    /// Check if a server is connected.
    pub fn is_connected(&self, name: &str) -> bool {
        self.clients.contains_key(name)
    }

    /// Get a connected client by name.
    pub fn get_client(&self, name: &str) -> Option<Arc<McpClient>> {
        self.clients.get(name).cloned()
    }

    /// Connect to all configured servers.
    ///
    /// For each configured server:
    /// 1. Spawns the server process
    /// 2. Performs the MCP initialization handshake
    /// 3. Stores the client for later use
    ///
    /// Servers that fail to connect are logged and skipped.
    /// Returns the number of successfully connected servers.
    pub fn connect_all(&mut self) -> Result<usize> {
        let mut connected = 0;

        for (name, config) in &self.configs {
            if self.clients.contains_key(name) {
                tracing::debug!(server = %name, "server already connected, skipping");
                continue;
            }

            match self.connect_server(config.clone()) {
                Ok(client) => {
                    self.clients.insert(name.clone(), Arc::new(client));
                    connected += 1;
                    tracing::info!(server = %name, "MCP server connected");
                }
                Err(e) => {
                    tracing::error!(server = %name, error = %e, "failed to connect to MCP server");
                }
            }
        }

        tracing::info!(
            connected = connected,
            total = self.configs.len(),
            "MCP server connection complete"
        );

        Ok(connected)
    }

    /// Connect to a single server.
    fn connect_server(&self, config: McpServerConfig) -> Result<McpClient> {
        let mut client = McpClient::connect_stdio(config)?;
        client.initialize()?;
        Ok(client)
    }

    /// Connect a single server by name.
    ///
    /// If the server is already connected, returns Ok without reconnecting.
    pub fn connect_server_by_name(&mut self, name: &str) -> Result<()> {
        if self.clients.contains_key(name) {
            return Ok(());
        }

        let config = self
            .configs
            .get(name)
            .ok_or_else(|| McpError::protocol(format!("server '{}' not configured", name)))?
            .clone();

        let client = self.connect_server(config)?;
        self.clients.insert(name.to_string(), Arc::new(client));
        tracing::info!(server = %name, "MCP server connected");
        Ok(())
    }

    /// List all tools from all connected servers.
    ///
    /// Returns a map of server name to list of tools.
    pub fn list_all_tools(&self) -> Result<HashMap<String, Vec<ToolInfo>>> {
        let mut all_tools = HashMap::new();

        for (name, client) in &self.clients {
            match client.list_tools() {
                Ok(tools) => {
                    tracing::debug!(server = %name, tool_count = tools.len(), "listed tools");
                    all_tools.insert(name.clone(), tools);
                }
                Err(e) => {
                    tracing::error!(server = %name, error = %e, "failed to list tools");
                }
            }
        }

        Ok(all_tools)
    }

    /// Get a flat list of all tools with their server names.
    ///
    /// Returns tuples of (server_name, tool_info).
    pub fn all_tools_flat(&self) -> Result<Vec<(String, ToolInfo)>> {
        let all = self.list_all_tools()?;
        let mut flat = Vec::new();

        for (server_name, tools) in all {
            for tool in tools {
                flat.push((server_name.clone(), tool));
            }
        }

        Ok(flat)
    }

    /// Get the total number of tools across all servers.
    pub fn tool_count(&self) -> Result<usize> {
        let all = self.list_all_tools()?;
        Ok(all.values().map(|v| v.len()).sum())
    }

    /// Get all connected clients.
    pub fn clients(&self) -> impl Iterator<Item = (&String, &Arc<McpClient>)> {
        self.clients.iter()
    }

    /// Shutdown all connected servers.
    ///
    /// This disconnects from all servers and clears the client list.
    /// Server configurations are preserved for reconnection.
    pub fn shutdown_all(&mut self) -> Result<()> {
        tracing::info!(
            server_count = self.clients.len(),
            "shutting down all MCP servers"
        );

        // Clear all clients (Drop will trigger shutdown)
        self.clients.clear();

        Ok(())
    }

    /// Shutdown a specific server by name.
    ///
    /// Returns true if the server was connected and is now disconnected.
    pub fn shutdown_server(&mut self, name: &str) -> bool {
        if let Some(client) = self.clients.remove(name) {
            tracing::info!(server = %name, "shutting down MCP server");
            drop(client);
            true
        } else {
            false
        }
    }

    /// Get the number of configured servers.
    pub fn config_count(&self) -> usize {
        self.configs.len()
    }

    /// Get the number of connected servers.
    pub fn connected_count(&self) -> usize {
        self.clients.len()
    }

    /// Check if any servers are connected.
    pub fn has_connections(&self) -> bool {
        !self.clients.is_empty()
    }
}

impl Drop for McpManager {
    fn drop(&mut self) {
        // Clients will be dropped automatically, triggering shutdown
        if !self.clients.is_empty() {
            tracing::debug!(
                count = self.clients.len(),
                "dropping McpManager, disconnecting servers"
            );
        }
    }
}

impl std::fmt::Debug for McpManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpManager")
            .field("configured", &self.configs.keys().collect::<Vec<_>>())
            .field("connected", &self.clients.keys().collect::<Vec<_>>())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_manager_empty() {
        let manager = McpManager::new();
        assert_eq!(manager.config_count(), 0);
        assert_eq!(manager.connected_count(), 0);
        assert!(!manager.has_connections());
    }

    #[test]
    fn test_with_configs() {
        let configs = vec![
            McpServerConfig::new("server1", "cmd1"),
            McpServerConfig::new("server2", "cmd2"),
        ];
        let manager = McpManager::with_configs(configs);
        assert_eq!(manager.config_count(), 2);
        assert!(manager.has_server("server1"));
        assert!(manager.has_server("server2"));
        assert!(!manager.has_server("server3"));
    }

    #[test]
    fn test_add_server() {
        let mut manager = McpManager::new();
        manager.add_server(McpServerConfig::new("test", "cmd"));
        assert_eq!(manager.config_count(), 1);
        assert!(manager.has_server("test"));
    }

    #[test]
    fn test_remove_server() {
        let mut manager = McpManager::new();
        manager.add_server(McpServerConfig::new("test", "cmd"));
        assert!(manager.has_server("test"));

        let removed = manager.remove_server("test");
        assert!(removed);
        assert!(!manager.has_server("test"));

        // Removing non-existent returns false
        let removed = manager.remove_server("nonexistent");
        assert!(!removed);
    }

    #[test]
    fn test_server_names() {
        let mut manager = McpManager::new();
        manager.add_server(McpServerConfig::new("alpha", "cmd"));
        manager.add_server(McpServerConfig::new("beta", "cmd"));

        let names = manager.server_names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"alpha"));
        assert!(names.contains(&"beta"));
    }

    #[test]
    fn test_connect_all_no_servers() {
        let mut manager = McpManager::new();
        let connected = manager.connect_all().unwrap();
        assert_eq!(connected, 0);
    }

    #[test]
    fn test_connect_all_invalid_command() {
        let mut manager = McpManager::new();
        manager.add_server(McpServerConfig::new("invalid", "nonexistent-command-12345"));

        // Should not fail, just log error and return 0 connected
        let connected = manager.connect_all().unwrap();
        assert_eq!(connected, 0);
        assert!(!manager.is_connected("invalid"));
    }

    #[test]
    fn test_debug_format() {
        let mut manager = McpManager::new();
        manager.add_server(McpServerConfig::new("test", "cmd"));
        let debug = format!("{:?}", manager);
        assert!(debug.contains("McpManager"));
        assert!(debug.contains("test"));
    }

    #[test]
    fn test_shutdown_server_not_connected() {
        let mut manager = McpManager::new();
        manager.add_server(McpServerConfig::new("test", "cmd"));

        // Not connected, so shutdown returns false
        let result = manager.shutdown_server("test");
        assert!(!result);

        // Config still exists
        assert!(manager.has_server("test"));
    }
}
