//! LLM client abstraction for Arawn.
//!
//! This crate provides a unified interface for interacting with various LLM providers
//! (Anthropic, OpenAI, Ollama, etc.) with support for streaming responses and tool calling.
//!
//! # Architecture
//!
//! The core abstraction is the [`LlmBackend`] trait which all providers implement.
//! This allows the agent to use any provider interchangeably.
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │  LlmBackend trait                       │
//! │  - complete() -> Response               │
//! │  - complete_stream() -> Stream<Event>   │
//! └─────────────────────────────────────────┘
//!                    │
//!     ┌──────────────┼──────────────┐
//!     ▼              ▼              ▼
//! ┌────────┐   ┌──────────┐   ┌────────┐
//! │Anthropic│   │  OpenAI  │   │ Ollama │
//! └────────┘   └──────────┘   └────────┘
//! ```

pub mod backend;
pub mod client;
pub mod embeddings;
pub mod error;
pub mod interaction_log;
pub mod types;

// Provider implementations
pub mod anthropic;
pub mod openai;

pub use backend::{
    ContentDelta, LlmBackend, MockBackend, MockResponse, ResponseStream, SharedBackend,
    StreamEvent,
};
pub use error::{LlmError, ResponseValidationError, Result};
pub use types::{
    CacheControl, CompletionRequest, CompletionResponse, Content, ContentBlock, Message, Role,
    StopReason, SystemPrompt, ToolChoice, ToolDefinition, ToolResultBlock, ToolResultContent,
    ToolUseBlock, Usage,
};

// Re-export embeddings
pub use embeddings::{
    Embedder, EmbedderSpec, MockEmbedder, OpenAiEmbedder, OpenAiEmbedderConfig, SharedEmbedder,
    build_embedder, cosine_similarity, euclidean_distance,
};

// Re-export provider configs
pub use anthropic::{AnthropicBackend, AnthropicConfig};
pub use openai::{OpenAiBackend, OpenAiConfig};

// Re-export client
pub use client::{LlmClient, LlmClientConfig, Provider};

// Re-export local embeddings when feature is enabled
#[cfg(feature = "local-embeddings")]
pub use embeddings::local::LocalEmbedder;
