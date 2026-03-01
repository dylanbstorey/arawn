//! Agent core for Arawn.
//!
//! This crate provides the agent loop, tool framework, and task execution
//! capabilities that power Arawn's conversational AI features.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │  Agent                                                      │
//! │  - Orchestrates conversation loop                          │
//! │  - Manages tool execution                                   │
//! │  - Handles context building                                 │
//! └─────────────────────────────────────────────────────────────┘
//!                              │
//!              ┌───────────────┼───────────────┐
//!              ▼               ▼               ▼
//!       ┌──────────┐    ┌──────────┐    ┌──────────┐
//!       │ LlmClient│    │ToolReg   │    │ Session  │
//!       │(arawn-llm)│    │          │    │          │
//!       └──────────┘    └──────────┘    └──────────┘
//! ```
//!
//! # Core Components
//!
//! - [`Session`]: Conversation state container with turns
//! - [`Turn`]: Single exchange (user message + agent response)
//! - [`AgentConfig`]: Runtime configuration for the agent
//! - [`AgentResponse`]: Output from an agent turn

pub mod agent;
pub mod compaction;
pub mod context;
pub mod error;
pub mod indexing;
pub mod mcp;
pub mod orchestrator;
pub mod prompt;
pub mod rlm;
pub mod stream;
pub mod tool;
pub mod tools;
pub mod types;

// Re-export core types
pub use error::{AgentError, Result};
pub use types::{
    AgentConfig, AgentResponse, ResponseUsage, Session, SessionId, ToolCall, ToolResultRecord,
    Turn, TurnId,
};

// Re-export tool types
pub use tool::{Tool, ToolContext, ToolRegistry, ToolResult};

// Re-export parameter validation types
pub use tool::{
    DelegateParams, FileReadParams, FileWriteParams, MemoryRecallParams, MemoryStoreParams,
    ParamExt, ParamResult, ParameterValidationError, ShellParams, ThinkParams, WebSearchParams,
};

// Re-export output sanitization types
pub use tool::{
    DEFAULT_MAX_OUTPUT_SIZE, OutputConfig, OutputSanitizationError, sanitize_output,
    validate_json_output,
};

// Re-export agent
pub use agent::{Agent, AgentBuilder, RecallConfig};

// Re-export compaction types
pub use compaction::{
    CancellationToken, CompactionProgress, CompactionResult, CompactorConfig, ProgressCallback,
    SessionCompactor,
};

// Re-export orchestrator types
pub use orchestrator::{
    CompactionOrchestrator, OrchestrationMetadata, OrchestrationResult, OrchestratorConfig,
};

// Re-export context types
pub use context::{ContextBuilder, ContextStatus, ContextTracker};

// Re-export prompt builder
pub use prompt::{BootstrapContext, BootstrapFile, PromptMode, SystemPromptBuilder};

// Re-export streaming types
pub use stream::{AgentStream, StreamChunk, create_turn_stream};

// Re-export indexing types
#[cfg(feature = "gliner")]
pub use indexing::GlinerEngine;
pub use indexing::{
    IndexReport, IndexerConfig, NerConfig, NerEngine, NerExtraction, NerOutput, NerRelation,
    NerSpan, SessionIndexer,
};

// Re-export RLM types
pub use rlm::{
    DEFAULT_READ_ONLY_TOOLS, ExplorationMetadata, ExplorationResult, RLM_SYSTEM_PROMPT, RlmConfig,
    RlmSpawner,
};

// Re-export MCP adapter
pub use mcp::{
    MCP_PREFIX, McpToolAdapter, NAMESPACE_DELIMITER, is_mcp_tool, parse_namespaced_name,
};

// Re-export built-in tools
pub use tools::{
    // File tools
    FileReadTool,
    FileWriteTool,
    // Search tools
    GlobTool,
    GrepTool,
    // Memory tool
    MemorySearchTool,
    Note,
    NoteStorage,
    // Note tool
    NoteTool,
    SearchProvider,
    SearchResult,
    // Shell tool
    ShellConfig,
    ShellTool,
    // Think tool
    ThinkTool,
    WebFetchConfig,
    // Web tools
    WebFetchTool,
    WebSearchConfig,
    WebSearchTool,
    new_note_storage,
};
