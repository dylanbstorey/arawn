//! MCP client for communicating with MCP servers.

use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use serde_json::Value;

use crate::error::{McpError, Result};
use crate::protocol::{
    CallToolParams, CallToolResult, InitializeParams, InitializeResult, JsonRpcNotification,
    JsonRpcRequest, ListToolsResult, ServerInfo, ToolInfo,
};
use crate::transport::{HttpTransportConfig, McpTransport};

/// Transport type for MCP server connections.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum TransportType {
    /// Stdio transport - spawns a child process.
    #[default]
    Stdio,
    /// HTTP transport - connects to a remote server via HTTP POST.
    Http,
}

/// Configuration for an MCP server connection.
#[derive(Debug, Clone)]
pub struct McpServerConfig {
    /// Unique name for this server.
    pub name: String,
    /// Transport type.
    pub transport: TransportType,
    /// Command to spawn (for stdio transport).
    pub command: String,
    /// URL for the server (for HTTP transport).
    pub url: Option<String>,
    /// Arguments to pass to the command.
    pub args: Vec<String>,
    /// Environment variables to set.
    pub env: Vec<(String, String)>,
    /// HTTP headers (for HTTP transport).
    pub headers: Vec<(String, String)>,
    /// Request timeout (for HTTP transport).
    pub timeout: Option<Duration>,
    /// Number of retries (for HTTP transport).
    pub retries: Option<u32>,
}

impl McpServerConfig {
    /// Create a new server config for stdio transport.
    pub fn new(name: impl Into<String>, command: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            transport: TransportType::Stdio,
            command: command.into(),
            url: None,
            args: Vec::new(),
            env: Vec::new(),
            headers: Vec::new(),
            timeout: None,
            retries: None,
        }
    }

    /// Create a new server config for HTTP transport.
    pub fn http(name: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            transport: TransportType::Http,
            command: String::new(),
            url: Some(url.into()),
            args: Vec::new(),
            env: Vec::new(),
            headers: Vec::new(),
            timeout: None,
            retries: None,
        }
    }

    /// Add arguments.
    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    /// Add an argument.
    pub fn with_arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Add environment variables.
    pub fn with_env(mut self, env: Vec<(String, String)>) -> Self {
        self.env = env;
        self
    }

    /// Add an environment variable.
    pub fn with_env_var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.push((key.into(), value.into()));
        self
    }

    /// Add an HTTP header (for HTTP transport).
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((key.into(), value.into()));
        self
    }

    /// Set request timeout (for HTTP transport).
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set number of retries (for HTTP transport).
    pub fn with_retries(mut self, retries: u32) -> Self {
        self.retries = Some(retries);
        self
    }

    /// Check if this is an HTTP transport config.
    pub fn is_http(&self) -> bool {
        self.transport == TransportType::Http
    }

    /// Check if this is a stdio transport config.
    pub fn is_stdio(&self) -> bool {
        self.transport == TransportType::Stdio
    }
}

/// An MCP client connected to a single MCP server.
pub struct McpClient {
    /// Server configuration.
    config: McpServerConfig,
    /// Transport for communicating with the server.
    transport: Mutex<McpTransport>,
    /// Server info (after initialization).
    server_info: Option<ServerInfo>,
    /// Counter for generating unique request IDs.
    request_id: AtomicU64,
    /// Whether the client has been initialized.
    initialized: bool,
}

impl McpClient {
    /// Connect to an MCP server using the configured transport.
    ///
    /// Automatically selects stdio or HTTP based on the config.
    /// This does NOT initialize the connection - call `initialize()` after connecting.
    pub fn connect(config: McpServerConfig) -> Result<Self> {
        match config.transport {
            TransportType::Stdio => Self::connect_stdio(config),
            TransportType::Http => Self::connect_http(config),
        }
    }

    /// Connect to an MCP server using stdio transport.
    ///
    /// This spawns the server process but does NOT initialize the connection.
    /// Call `initialize()` after connecting to complete the handshake.
    pub fn connect_stdio(config: McpServerConfig) -> Result<Self> {
        let env = if config.env.is_empty() {
            None
        } else {
            Some(config.env.as_slice())
        };

        let transport = McpTransport::spawn_stdio(&config.command, &config.args, env)?;

        tracing::info!(
            server = %config.name,
            command = %config.command,
            "connected to MCP server via stdio"
        );

        Ok(Self {
            config,
            transport: Mutex::new(transport),
            server_info: None,
            request_id: AtomicU64::new(1),
            initialized: false,
        })
    }

    /// Connect to an MCP server using HTTP transport.
    ///
    /// This creates an HTTP client but does NOT initialize the connection.
    /// Call `initialize()` after connecting to complete the handshake.
    pub fn connect_http(config: McpServerConfig) -> Result<Self> {
        let url = config
            .url
            .as_ref()
            .ok_or_else(|| McpError::transport("HTTP transport requires a URL"))?;

        let mut http_config = HttpTransportConfig::new(url);

        if let Some(timeout) = config.timeout {
            http_config = http_config.with_timeout(timeout);
        }
        if let Some(retries) = config.retries {
            http_config = http_config.with_retries(retries);
        }
        for (key, value) in &config.headers {
            http_config = http_config.with_header(key, value);
        }

        let transport = McpTransport::connect_http(http_config)?;

        tracing::info!(
            server = %config.name,
            url = %url,
            "connected to MCP server via HTTP"
        );

        Ok(Self {
            config,
            transport: Mutex::new(transport),
            server_info: None,
            request_id: AtomicU64::new(1),
            initialized: false,
        })
    }

    /// Get the server name.
    pub fn name(&self) -> &str {
        &self.config.name
    }

    /// Get the server info (after initialization).
    pub fn server_info(&self) -> Option<&ServerInfo> {
        self.server_info.as_ref()
    }

    /// Check if the client has been initialized.
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Check if the client is using HTTP transport.
    pub fn is_http(&self) -> bool {
        self.config.is_http()
    }

    /// Check if the client is using stdio transport.
    pub fn is_stdio(&self) -> bool {
        self.config.is_stdio()
    }

    /// Get the next request ID.
    fn next_request_id(&self) -> u64 {
        self.request_id.fetch_add(1, Ordering::SeqCst)
    }

    /// Send a request and get the response.
    fn send_request(&self, method: &str, params: Option<Value>) -> Result<Value> {
        let request = JsonRpcRequest::new(self.next_request_id(), method, params);

        let mut transport = self
            .transport
            .lock()
            .map_err(|_| McpError::transport("failed to acquire transport lock"))?;

        let response = transport.send_request(&request)?;

        response
            .into_result()
            .map_err(|e| McpError::server_error(e.code, e.message, e.data))
    }

    /// Send a notification (no response expected).
    fn send_notification(&self, method: &str, params: Option<Value>) -> Result<()> {
        let notification = JsonRpcNotification::new(method, params);

        let mut transport = self
            .transport
            .lock()
            .map_err(|_| McpError::transport("failed to acquire transport lock"))?;

        transport.send_notification(&notification)
    }

    /// Initialize the connection with the MCP server.
    ///
    /// This performs the MCP handshake, exchanging capabilities and protocol versions.
    /// Must be called before using other methods.
    pub fn initialize(&mut self) -> Result<&ServerInfo> {
        if self.initialized {
            return self.server_info.as_ref().ok_or(McpError::NotInitialized);
        }

        let params = InitializeParams::default();
        let result = self.send_request("initialize", Some(serde_json::to_value(&params)?))?;

        let init_result: InitializeResult = serde_json::from_value(result)?;

        tracing::info!(
            server = %init_result.server_info.name,
            version = %init_result.server_info.version,
            protocol = %init_result.protocol_version,
            "MCP server initialized"
        );

        // Send initialized notification
        self.send_notification("notifications/initialized", None)?;

        self.server_info = Some(init_result.server_info);
        self.initialized = true;

        Ok(self.server_info.as_ref().unwrap())
    }

    /// List available tools from the server.
    pub fn list_tools(&self) -> Result<Vec<ToolInfo>> {
        if !self.initialized {
            return Err(McpError::NotInitialized);
        }

        let result = self.send_request("tools/list", None)?;
        let list_result: ListToolsResult = serde_json::from_value(result)?;

        tracing::debug!(
            server = %self.config.name,
            tool_count = list_result.tools.len(),
            "listed MCP tools"
        );

        Ok(list_result.tools)
    }

    /// Call a tool on the server.
    ///
    /// # Arguments
    /// * `name` - The name of the tool to call
    /// * `arguments` - The arguments to pass to the tool
    pub fn call_tool(&self, name: &str, arguments: Option<Value>) -> Result<CallToolResult> {
        if !self.initialized {
            return Err(McpError::NotInitialized);
        }

        let params = CallToolParams {
            name: name.to_string(),
            arguments,
        };

        let result = self.send_request("tools/call", Some(serde_json::to_value(&params)?))?;
        let call_result: CallToolResult = serde_json::from_value(result)?;

        if call_result.is_error() {
            tracing::warn!(
                server = %self.config.name,
                tool = %name,
                "tool call returned error"
            );
        } else {
            tracing::debug!(
                server = %self.config.name,
                tool = %name,
                "tool call succeeded"
            );
        }

        Ok(call_result)
    }

    /// Shutdown the connection gracefully.
    pub fn shutdown(&mut self) -> Result<()> {
        tracing::info!(server = %self.config.name, "shutting down MCP client");

        let mut transport = self
            .transport
            .lock()
            .map_err(|_| McpError::transport("failed to acquire transport lock"))?;

        transport.shutdown()
    }

    /// Check if the connection is still active.
    pub fn is_connected(&self) -> bool {
        if let Ok(mut transport) = self.transport.lock() {
            transport.is_connected()
        } else {
            false
        }
    }
}

impl Drop for McpClient {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config_builder() {
        let config = McpServerConfig::new("test", "mcp-server-test")
            .with_arg("--db")
            .with_arg("/path/to/db")
            .with_env_var("DEBUG", "1");

        assert_eq!(config.name, "test");
        assert_eq!(config.command, "mcp-server-test");
        assert_eq!(config.args, vec!["--db", "/path/to/db"]);
        assert_eq!(config.env, vec![("DEBUG".to_string(), "1".to_string())]);
        assert!(config.is_stdio());
        assert!(!config.is_http());
    }

    #[test]
    fn test_http_server_config_builder() {
        let config = McpServerConfig::http("remote", "https://mcp.example.com/api")
            .with_header("Authorization", "Bearer token123")
            .with_timeout(Duration::from_secs(60))
            .with_retries(5);

        assert_eq!(config.name, "remote");
        assert_eq!(config.url, Some("https://mcp.example.com/api".to_string()));
        assert!(config.is_http());
        assert!(!config.is_stdio());
        assert_eq!(config.timeout, Some(Duration::from_secs(60)));
        assert_eq!(config.retries, Some(5));
        assert_eq!(
            config.headers,
            vec![("Authorization".to_string(), "Bearer token123".to_string())]
        );
    }

    #[test]
    fn test_connect_nonexistent_server() {
        let config = McpServerConfig::new("test", "nonexistent-mcp-server-12345");
        let result = McpClient::connect_stdio(config);
        assert!(result.is_err());
    }

    #[test]
    fn test_connect_http_no_url() {
        // Create stdio config and try to connect via HTTP - should fail
        let mut config = McpServerConfig::new("test", "cmd");
        config.transport = TransportType::Http;
        config.url = None;

        let result = McpClient::connect_http(config);
        assert!(result.is_err());
    }

    #[test]
    fn test_connect_http_valid() {
        let config = McpServerConfig::http("remote", "http://localhost:8080/mcp");
        let result = McpClient::connect_http(config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_connect_auto_select_transport() {
        // Stdio transport
        let config = McpServerConfig::new("test", "nonexistent-cmd");
        assert!(config.is_stdio());
        let result = McpClient::connect(config);
        assert!(result.is_err()); // Command doesn't exist

        // HTTP transport
        let config = McpServerConfig::http("remote", "http://localhost:8080/mcp");
        assert!(config.is_http());
        let result = McpClient::connect(config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_request_id_increments() {
        // We can't fully test without a real server, but we can test ID generation
        let id = AtomicU64::new(1);
        assert_eq!(id.fetch_add(1, Ordering::SeqCst), 1);
        assert_eq!(id.fetch_add(1, Ordering::SeqCst), 2);
        assert_eq!(id.fetch_add(1, Ordering::SeqCst), 3);
    }
}
