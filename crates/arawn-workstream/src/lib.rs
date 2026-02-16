//! Workstream management for Arawn.
//!
//! Provides persistent conversational contexts (workstreams) with JSONL message
//! history as the source of truth and SQLite as an operational cache layer.

pub mod compression;
pub mod context;
pub mod error;
pub mod manager;
pub mod message_store;
pub mod scratch;
pub mod session;
pub mod session_loader;
pub mod store;
pub mod types;

pub use compression::{Compressor, CompressorConfig};
pub use context::{AssembledContext, ContextAssembler, ContextMessage, ContextRole};
pub use error::{Result, WorkstreamError};
pub use manager::{WorkstreamConfig, WorkstreamManager};
pub use message_store::MessageStore;
pub use scratch::{SCRATCH_ID, ScratchManager};
pub use session::SessionManager;
pub use session_loader::{
    ReconstructedSession, ReconstructedToolCall, ReconstructedToolResult, ReconstructedTurn,
    SessionLoader, ToolResultMetadata, ToolUseMetadata,
};
pub use store::WorkstreamStore;
pub use types::{MessageRole, WorkstreamMessage};
