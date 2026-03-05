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

// Re-export key types from infrastructure crates for convenience.
// The domain facade aggregates these so transport layers (server, CLI) can depend
// on arawn-domain rather than each infrastructure crate individually.

// Agent: core agent, session, streaming types, and errors
pub use arawn_agent::{
    Agent, AgentError, CompactionResult, CompactorConfig, Session, SessionCompactor, SessionId,
    SessionIndexer, StreamChunk, ToolCall, ToolRegistry, ToolResultRecord, Turn, TurnId,
};

// Config: configuration errors
pub use arawn_config::ConfigError;

// MCP: server management and configuration
pub use arawn_mcp::{McpManager, McpServerConfig};

// Memory: storage for semantic memories, types, and IDs
pub use arawn_memory::MemoryId;
pub use arawn_memory::MemoryStore;
pub use arawn_memory::types::{ContentType, Memory, Note as MemoryNote, NoteId};

// Sandbox: OS-level sandboxing for shell commands
pub use arawn_sandbox::SandboxManager;

// Session: cache, configuration, persistence, and error types
pub use arawn_session::SessionCache as SessionCacheImpl;
pub use arawn_session::{CacheConfig, PersistenceHook};
pub use arawn_session::{Error as SessionStoreError, Result as SessionStoreResult};

// Workstream: directory management, file operations, events, and errors
pub use arawn_workstream::cleanup::{DiskPressureEvent, PressureLevel};
pub use arawn_workstream::directory::DirectoryError;
pub use arawn_workstream::store::Workstream;
pub use arawn_workstream::{
    AttachResult, Compressor, DirectoryManager, FsAction, FsChangeEvent, MessageRole,
    PathValidator, ReconstructedSession, SCRATCH_ID, SessionLoader, WatcherHandle, WorkstreamError,
    WorkstreamManager, WorkstreamMessage,
};
