//! Domain facade for Arawn.
//!
//! This crate provides a unified interface for orchestrating the core Arawn
//! services: agent execution, session management, and MCP server interactions.
//!
//! The domain layer acts as a facade between transport layers (HTTP server, CLI)
//! and the infrastructure crates, providing:
//!
//! - **Chat orchestration**: Coordinates agent, session, and workstream for conversations
//! - **MCP management**: Tool discovery and invocation across MCP servers
//!
//! # Example
//!
//! ```ignore
//! use arawn_domain::DomainServices;
//!
//! let services = DomainServices::new(agent, None, None, None, None, None);
//!
//! // Execute a chat turn
//! let response = services.chat().turn(&mut session, "Hello!").await?;
//! ```

mod error;
pub mod services;

pub use error::{DomainError, Result};
pub use services::DomainServices;
pub use services::chat::{ChatResponse, ChatService, ToolCallSummary, TurnOptions};
pub use services::mcp::{McpServerInfo, McpService, McpToolInfo, SharedMcpManager};
pub use services::memory::MemoryService;

// Re-export key types from infrastructure crates for convenience
pub use arawn_agent::{Agent, Session, SessionId};
pub use arawn_mcp::McpServerConfig;
