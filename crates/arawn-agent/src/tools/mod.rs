//! Built-in tools for the agent.
//!
//! This module provides the core tools that give the agent basic capabilities:
//! - File operations (read/write)
//! - Shell command execution
//! - Note-taking/memory
//! - Web search and fetching
//! - File search (glob/grep)
//! - Memory/knowledge search
//! - Subagent delegation

mod catalog;
mod delegate;
mod file;
mod memory;
mod note;
mod search;
mod shell;
mod think;
mod web;
mod workflow;

// File tools
pub use file::{FileReadTool, FileWriteTool};

// Note tool
pub use note::{Note, NoteStorage, NoteTool, new_note_storage};

// Shell tool
pub use shell::{ShellConfig, ShellTool};

// Web tools
pub use web::{
    SearchProvider, SearchResult, WebFetchConfig, WebFetchTool, WebSearchConfig, WebSearchTool,
};

// Search tools
pub use search::{GlobTool, GrepTool};

// Memory tool
pub use memory::MemorySearchTool;

// Workflow tool
pub use workflow::WorkflowTool;

// Think tool
pub use think::ThinkTool;

// Catalog tool
pub use catalog::CatalogTool;

// Delegate tool
pub use delegate::DelegateTool;
