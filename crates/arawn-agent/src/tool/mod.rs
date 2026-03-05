//! Tool framework for agent capabilities.
//!
//! This module defines the [`Tool`] trait that all agent tools must implement,
//! and the [`ToolRegistry`] for managing available tools.
//!
//! # Example
//!
//! ```rust,ignore
//! use arawn_agent::{Tool, ToolContext, ToolResult, ToolRegistry};
//!
//! struct MyTool;
//!
//! #[async_trait]
//! impl Tool for MyTool {
//!     fn name(&self) -> &str { "my_tool" }
//!     fn description(&self) -> &str { "Does something useful" }
//!     fn parameters(&self) -> Value { json!({"type": "object"}) }
//!
//!     async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
//!         Ok(ToolResult::text("Done!"))
//!     }
//! }
//!
//! let mut registry = ToolRegistry::new();
//! registry.register(MyTool);
//! ```

mod command_validator;
mod context;
mod execution;
mod gate;
mod output;
mod params;
mod registry;
mod validation;

// Re-export core types
pub use context::{OutputSender, Tool, ToolContext, ToolResult};

// Re-export parameter validation types
pub use validation::{ParamExt, ParamResult, ParameterValidationError};

// Re-export typed parameter structs
pub use params::{
    DelegateParams, FileReadParams, FileWriteParams, MemoryRecallParams, MemoryStoreParams,
    ShellParams, ThinkParams, WebSearchParams,
};

// Re-export output sanitization types
pub use output::{
    DEFAULT_MAX_OUTPUT_SIZE, OutputConfig, OutputSanitizationError, sanitize_output,
    validate_json_output,
};

// Re-export registry
pub use registry::ToolRegistry;

// Re-export command validation types
pub use command_validator::{CommandValidation, CommandValidator};

// Re-export test utilities
#[cfg(test)]
pub use registry::MockTool;
