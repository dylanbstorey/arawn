//! MCP (Model Context Protocol) client for Arawn.
//!
//! This crate provides a client implementation for the Model Context Protocol,
//! enabling Arawn to connect to MCP servers and discover/invoke their tools.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │  McpClient                                                  │
//! │  - Connects to MCP server via stdio                         │
//! │  - Implements initialize, tools/list, tools/call            │
//! └─────────────────────────────────────────────────────────────┘
//!                           │
//!                           ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │  McpTransport                                               │
//! │  - JSON-RPC 2.0 with Content-Length framing                 │
//! │  - Stdio transport (spawn child process)                    │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Usage
//!
//! ```rust,ignore
//! use arawn_mcp::{McpClient, McpServerConfig};
//!
//! // Configure the server
//! let config = McpServerConfig::new("sqlite", "mcp-server-sqlite")
//!     .with_arg("--db")
//!     .with_arg("/path/to/database.db");
//!
//! // Connect and initialize
//! let mut client = McpClient::connect_stdio(config)?;
//! let server_info = client.initialize()?;
//! println!("Connected to: {} v{}", server_info.name, server_info.version);
//!
//! // List available tools
//! let tools = client.list_tools()?;
//! for tool in &tools {
//!     println!("Tool: {} - {:?}", tool.name, tool.description);
//! }
//!
//! // Call a tool
//! let result = client.call_tool("query", Some(json!({"sql": "SELECT * FROM users"})))?;
//! println!("Result: {:?}", result.text());
//! ```
//!
//! # MCP Protocol
//!
//! MCP uses JSON-RPC 2.0 over stdio with Content-Length framing:
//!
//! ```text
//! Content-Length: <length>\r\n
//! \r\n
//! {"jsonrpc": "2.0", "id": 1, "method": "...", "params": {...}}
//! ```
//!
//! The protocol flow is:
//! 1. Client sends `initialize` with capabilities
//! 2. Server responds with its capabilities
//! 3. Client sends `notifications/initialized`
//! 4. Client can now call `tools/list` and `tools/call`

pub mod client;
pub mod error;
pub mod manager;
pub mod protocol;
pub mod transport;

// Re-export main types
pub use client::{McpClient, McpServerConfig, TransportType};
pub use error::{McpError, Result};
pub use manager::McpManager;
pub use protocol::{
    CallToolParams, CallToolResult, InitializeParams, InitializeResult, JsonRpcError,
    JsonRpcNotification, JsonRpcRequest, JsonRpcResponse, ListToolsResult, ServerCapabilities,
    ServerInfo, ToolContent, ToolInfo, ToolsCapability,
};
pub use transport::{HttpTransportConfig, McpTransport};
