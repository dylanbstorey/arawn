//! System prompt generation module.
//!
//! This module provides a modular system for building agent system prompts.
//! Prompts are assembled from configurable sections including identity, tools,
//! workspace context, and bootstrap files.
//!
//! # Example
//!
//! ```rust,ignore
//! use arawn_agent::prompt::{SystemPromptBuilder, PromptMode};
//!
//! let prompt = SystemPromptBuilder::new()
//!     .with_mode(PromptMode::Full)
//!     .with_identity("Arawn", "A helpful AI assistant")
//!     .with_tools(&tool_registry)
//!     .with_workspace("/path/to/project")
//!     .with_datetime(None)
//!     .build();
//! ```

mod bootstrap;
mod builder;
mod mode;

pub use bootstrap::{BootstrapContext, BootstrapFile};
pub use builder::SystemPromptBuilder;
pub use mode::PromptMode;
