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

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

use crate::error::{AgentError, Result};
use crate::types::{SessionId, TurnId};

// ─────────────────────────────────────────────────────────────────────────────
// Parameter Validation
// ─────────────────────────────────────────────────────────────────────────────

/// Error type for tool parameter validation failures.
///
/// Provides detailed error messages that help the LLM understand what went wrong
/// and how to fix it.
#[derive(Debug, Clone, thiserror::Error)]
pub enum ParameterValidationError {
    /// A required parameter is missing.
    #[error("missing required parameter '{name}': {hint}")]
    MissingRequired {
        /// The parameter name.
        name: &'static str,
        /// Hint for the LLM on how to fix.
        hint: &'static str,
    },

    /// A parameter has an invalid type.
    #[error("invalid type for '{name}': expected {expected}, got {actual}")]
    InvalidType {
        /// The parameter name.
        name: &'static str,
        /// The expected type.
        expected: &'static str,
        /// The actual type found.
        actual: String,
    },

    /// A parameter value is out of range.
    #[error("'{name}' value {value} is out of range: {constraint}")]
    OutOfRange {
        /// The parameter name.
        name: &'static str,
        /// The actual value as string.
        value: String,
        /// Description of the valid range.
        constraint: String,
    },

    /// A parameter value doesn't match expected pattern/enum.
    #[error("'{name}' has invalid value '{value}': {message}")]
    InvalidValue {
        /// The parameter name.
        name: &'static str,
        /// The invalid value.
        value: String,
        /// Why it's invalid.
        message: String,
    },

    /// Multiple validation errors.
    #[error("parameter validation failed: {}", .0.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("; "))]
    Multiple(Vec<ParameterValidationError>),
}

impl ParameterValidationError {
    /// Create a missing required parameter error.
    pub fn missing(name: &'static str, hint: &'static str) -> Self {
        Self::MissingRequired { name, hint }
    }

    /// Create an invalid type error.
    pub fn invalid_type(
        name: &'static str,
        expected: &'static str,
        actual: impl Into<String>,
    ) -> Self {
        Self::InvalidType {
            name,
            expected,
            actual: actual.into(),
        }
    }

    /// Create an out of range error.
    pub fn out_of_range(
        name: &'static str,
        value: impl ToString,
        constraint: impl Into<String>,
    ) -> Self {
        Self::OutOfRange {
            name,
            value: value.to_string(),
            constraint: constraint.into(),
        }
    }

    /// Create an invalid value error.
    pub fn invalid_value(
        name: &'static str,
        value: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::InvalidValue {
            name,
            value: value.into(),
            message: message.into(),
        }
    }

    /// Create from multiple errors.
    pub fn multiple(errors: Vec<ParameterValidationError>) -> Self {
        Self::Multiple(errors)
    }

    /// Get the parameter name associated with this error (if single error).
    pub fn parameter_name(&self) -> Option<&str> {
        match self {
            Self::MissingRequired { name, .. } => Some(name),
            Self::InvalidType { name, .. } => Some(name),
            Self::OutOfRange { name, .. } => Some(name),
            Self::InvalidValue { name, .. } => Some(name),
            Self::Multiple(_) => None,
        }
    }
}

impl From<ParameterValidationError> for AgentError {
    fn from(err: ParameterValidationError) -> Self {
        AgentError::Tool(err.to_string())
    }
}

/// Result type for parameter validation.
pub type ParamResult<T> = std::result::Result<T, ParameterValidationError>;

/// Helper trait for extracting and validating parameters from JSON.
pub trait ParamExt {
    /// Get a required string parameter.
    fn required_str(&self, name: &'static str, hint: &'static str) -> ParamResult<&str>;

    /// Get an optional string parameter.
    fn optional_str(&self, name: &str) -> Option<&str>;

    /// Get a required integer parameter.
    fn required_i64(&self, name: &'static str, hint: &'static str) -> ParamResult<i64>;

    /// Get an optional integer parameter with default.
    fn optional_i64(&self, name: &str, default: i64) -> i64;

    /// Get an optional u64 parameter with default.
    fn optional_u64(&self, name: &str, default: u64) -> u64;

    /// Get a required boolean parameter.
    fn required_bool(&self, name: &'static str, hint: &'static str) -> ParamResult<bool>;

    /// Get an optional boolean parameter with default.
    fn optional_bool(&self, name: &str, default: bool) -> bool;

    /// Get an optional array parameter.
    fn optional_array(&self, name: &str) -> Option<&Vec<serde_json::Value>>;
}

impl ParamExt for serde_json::Value {
    fn required_str(&self, name: &'static str, hint: &'static str) -> ParamResult<&str> {
        self.get(name)
            .and_then(|v| v.as_str())
            .ok_or_else(|| ParameterValidationError::missing(name, hint))
    }

    fn optional_str(&self, name: &str) -> Option<&str> {
        self.get(name).and_then(|v| v.as_str())
    }

    fn required_i64(&self, name: &'static str, hint: &'static str) -> ParamResult<i64> {
        self.get(name)
            .and_then(|v| v.as_i64())
            .ok_or_else(|| ParameterValidationError::missing(name, hint))
    }

    fn optional_i64(&self, name: &str, default: i64) -> i64 {
        self.get(name).and_then(|v| v.as_i64()).unwrap_or(default)
    }

    fn optional_u64(&self, name: &str, default: u64) -> u64 {
        self.get(name).and_then(|v| v.as_u64()).unwrap_or(default)
    }

    fn required_bool(&self, name: &'static str, hint: &'static str) -> ParamResult<bool> {
        self.get(name)
            .and_then(|v| v.as_bool())
            .ok_or_else(|| ParameterValidationError::missing(name, hint))
    }

    fn optional_bool(&self, name: &str, default: bool) -> bool {
        self.get(name).and_then(|v| v.as_bool()).unwrap_or(default)
    }

    fn optional_array(&self, name: &str) -> Option<&Vec<serde_json::Value>> {
        self.get(name).and_then(|v| v.as_array())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Typed Parameter Structs
// ─────────────────────────────────────────────────────────────────────────────

/// Validated parameters for the shell tool.
#[derive(Debug, Clone)]
pub struct ShellParams {
    /// The command to execute.
    pub command: String,
    /// Whether to run in PTY mode.
    pub pty: bool,
    /// Whether to stream output.
    pub stream: bool,
    /// Working directory override.
    pub cwd: Option<String>,
    /// Timeout in seconds.
    pub timeout_secs: Option<u64>,
}

impl TryFrom<serde_json::Value> for ShellParams {
    type Error = ParameterValidationError;

    fn try_from(params: serde_json::Value) -> std::result::Result<Self, Self::Error> {
        let command = params.required_str("command", "provide the shell command to execute")?;

        // Validate command is not empty
        if command.trim().is_empty() {
            return Err(ParameterValidationError::invalid_value(
                "command",
                command,
                "command cannot be empty",
            ));
        }

        let timeout_secs = params.get("timeout_secs").and_then(|v| v.as_u64());

        // Validate timeout if provided
        if let Some(timeout) = timeout_secs {
            if timeout == 0 {
                return Err(ParameterValidationError::out_of_range(
                    "timeout_secs",
                    timeout,
                    "must be greater than 0",
                ));
            }
            if timeout > 3600 {
                return Err(ParameterValidationError::out_of_range(
                    "timeout_secs",
                    timeout,
                    "must be at most 3600 (1 hour)",
                ));
            }
        }

        Ok(Self {
            command: command.to_string(),
            pty: params.optional_bool("pty", false),
            stream: params.optional_bool("stream", false),
            cwd: params.optional_str("cwd").map(String::from),
            timeout_secs,
        })
    }
}

/// Validated parameters for file read tool.
#[derive(Debug, Clone)]
pub struct FileReadParams {
    /// Path to the file to read.
    pub path: String,
}

impl TryFrom<serde_json::Value> for FileReadParams {
    type Error = ParameterValidationError;

    fn try_from(params: serde_json::Value) -> std::result::Result<Self, Self::Error> {
        let path = params.required_str("path", "provide the file path to read")?;

        // Validate path is not empty
        if path.trim().is_empty() {
            return Err(ParameterValidationError::invalid_value(
                "path",
                path,
                "path cannot be empty",
            ));
        }

        Ok(Self {
            path: path.to_string(),
        })
    }
}

/// Validated parameters for file write tool.
#[derive(Debug, Clone)]
pub struct FileWriteParams {
    /// Path to the file to write.
    pub path: String,
    /// Content to write.
    pub content: String,
    /// Whether to append instead of overwrite.
    pub append: bool,
}

impl TryFrom<serde_json::Value> for FileWriteParams {
    type Error = ParameterValidationError;

    fn try_from(params: serde_json::Value) -> std::result::Result<Self, Self::Error> {
        let path = params.required_str("path", "provide the file path to write")?;
        let content = params.required_str("content", "provide the content to write")?;

        // Validate path is not empty
        if path.trim().is_empty() {
            return Err(ParameterValidationError::invalid_value(
                "path",
                path,
                "path cannot be empty",
            ));
        }

        Ok(Self {
            path: path.to_string(),
            content: content.to_string(),
            append: params.optional_bool("append", false),
        })
    }
}

/// Validated parameters for web search tool.
#[derive(Debug, Clone)]
pub struct WebSearchParams {
    /// The search query.
    pub query: String,
    /// Maximum number of results.
    pub max_results: u64,
}

impl TryFrom<serde_json::Value> for WebSearchParams {
    type Error = ParameterValidationError;

    fn try_from(params: serde_json::Value) -> std::result::Result<Self, Self::Error> {
        let query = params.required_str("query", "provide a search query")?;

        // Validate query is not empty
        if query.trim().is_empty() {
            return Err(ParameterValidationError::invalid_value(
                "query",
                query,
                "query cannot be empty",
            ));
        }

        let max_results = params.optional_u64("max_results", 10);

        // Validate max_results range
        if max_results == 0 {
            return Err(ParameterValidationError::out_of_range(
                "max_results",
                max_results,
                "must be at least 1",
            ));
        }
        if max_results > 100 {
            return Err(ParameterValidationError::out_of_range(
                "max_results",
                max_results,
                "must be at most 100",
            ));
        }

        Ok(Self {
            query: query.to_string(),
            max_results,
        })
    }
}

/// Validated parameters for think tool.
#[derive(Debug, Clone)]
pub struct ThinkParams {
    /// The thought content.
    pub thought: String,
}

impl TryFrom<serde_json::Value> for ThinkParams {
    type Error = ParameterValidationError;

    fn try_from(params: serde_json::Value) -> std::result::Result<Self, Self::Error> {
        let thought = params.required_str("thought", "provide the thought content")?;

        // Validate thought is not empty
        if thought.trim().is_empty() {
            return Err(ParameterValidationError::invalid_value(
                "thought",
                thought,
                "thought cannot be empty",
            ));
        }

        Ok(Self {
            thought: thought.to_string(),
        })
    }
}

/// Validated parameters for memory store tool.
#[derive(Debug, Clone)]
pub struct MemoryStoreParams {
    /// Content to store.
    pub content: String,
    /// Optional memory type tag.
    pub memory_type: Option<String>,
    /// Optional importance score (0.0-1.0).
    pub importance: Option<f64>,
}

impl TryFrom<serde_json::Value> for MemoryStoreParams {
    type Error = ParameterValidationError;

    fn try_from(params: serde_json::Value) -> std::result::Result<Self, Self::Error> {
        let content = params.required_str("content", "provide the memory content to store")?;

        // Validate content is not empty
        if content.trim().is_empty() {
            return Err(ParameterValidationError::invalid_value(
                "content",
                content,
                "content cannot be empty",
            ));
        }

        // Validate importance if provided
        let importance = params.get("importance").and_then(|v| v.as_f64());
        if let Some(imp) = importance
            && !(0.0..=1.0).contains(&imp)
        {
            return Err(ParameterValidationError::out_of_range(
                "importance",
                imp,
                "must be between 0.0 and 1.0",
            ));
        }

        Ok(Self {
            content: content.to_string(),
            memory_type: params.optional_str("memory_type").map(String::from),
            importance,
        })
    }
}

/// Validated parameters for memory recall tool.
#[derive(Debug, Clone)]
pub struct MemoryRecallParams {
    /// Query to search memories.
    pub query: String,
    /// Maximum number of results.
    pub limit: u64,
    /// Optional memory type filter.
    pub memory_type: Option<String>,
}

impl TryFrom<serde_json::Value> for MemoryRecallParams {
    type Error = ParameterValidationError;

    fn try_from(params: serde_json::Value) -> std::result::Result<Self, Self::Error> {
        let query = params.required_str("query", "provide a query to search memories")?;

        // Validate query is not empty
        if query.trim().is_empty() {
            return Err(ParameterValidationError::invalid_value(
                "query",
                query,
                "query cannot be empty",
            ));
        }

        let limit = params.optional_u64("limit", 10);

        // Validate limit range
        if limit == 0 {
            return Err(ParameterValidationError::out_of_range(
                "limit",
                limit,
                "must be at least 1",
            ));
        }
        if limit > 100 {
            return Err(ParameterValidationError::out_of_range(
                "limit",
                limit,
                "must be at most 100",
            ));
        }

        Ok(Self {
            query: query.to_string(),
            limit,
            memory_type: params.optional_str("memory_type").map(String::from),
        })
    }
}

/// Validated parameters for delegate tool.
#[derive(Debug, Clone)]
pub struct DelegateParams {
    /// The task to delegate.
    pub task: String,
    /// Optional agent type to delegate to.
    pub agent_type: Option<String>,
}

impl TryFrom<serde_json::Value> for DelegateParams {
    type Error = ParameterValidationError;

    fn try_from(params: serde_json::Value) -> std::result::Result<Self, Self::Error> {
        let task = params.required_str("task", "provide the task to delegate")?;

        // Validate task is not empty
        if task.trim().is_empty() {
            return Err(ParameterValidationError::invalid_value(
                "task",
                task,
                "task cannot be empty",
            ));
        }

        Ok(Self {
            task: task.to_string(),
            agent_type: params.optional_str("agent_type").map(String::from),
        })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Output Sanitization
// ─────────────────────────────────────────────────────────────────────────────

/// Default maximum output size in bytes (100KB).
pub const DEFAULT_MAX_OUTPUT_SIZE: usize = 100 * 1024;

/// Configuration for sanitizing tool output.
///
/// Controls size limits, truncation behavior, and content sanitization
/// to prevent context overflow and malformed responses.
#[derive(Debug, Clone)]
pub struct OutputConfig {
    /// Maximum size in bytes before truncation.
    pub max_size_bytes: usize,
    /// Message to append when output is truncated.
    pub truncation_message: String,
    /// Whether to strip control characters (except newlines, tabs).
    pub strip_control_chars: bool,
    /// Whether to strip null bytes.
    pub strip_null_bytes: bool,
    /// Whether to validate JSON structure for JSON outputs.
    pub validate_json: bool,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            max_size_bytes: DEFAULT_MAX_OUTPUT_SIZE,
            truncation_message: "\n\n[Output truncated - exceeded size limit]".to_string(),
            strip_control_chars: true,
            strip_null_bytes: true,
            validate_json: true,
        }
    }
}

impl OutputConfig {
    /// Create a new output config with the given size limit.
    pub fn with_max_size(max_size_bytes: usize) -> Self {
        Self {
            max_size_bytes,
            ..Default::default()
        }
    }

    /// Configuration for shell output (100KB default).
    pub fn for_shell() -> Self {
        Self::with_max_size(100 * 1024)
    }

    /// Configuration for file read output (500KB default).
    pub fn for_file_read() -> Self {
        Self::with_max_size(500 * 1024)
    }

    /// Configuration for web fetch output (200KB default).
    pub fn for_web_fetch() -> Self {
        Self::with_max_size(200 * 1024)
    }

    /// Configuration for search output (50KB default).
    pub fn for_search() -> Self {
        Self::with_max_size(50 * 1024)
    }

    /// Set a custom truncation message.
    pub fn with_truncation_message(mut self, message: impl Into<String>) -> Self {
        self.truncation_message = message.into();
        self
    }

    /// Disable control character stripping.
    pub fn without_control_char_stripping(mut self) -> Self {
        self.strip_control_chars = false;
        self
    }
}

/// Error type for output sanitization failures.
#[derive(Debug, Clone, thiserror::Error)]
pub enum OutputSanitizationError {
    /// Output appears to be binary data.
    #[error(
        "output appears to be binary data (detected {null_bytes} null bytes in first {checked_bytes} bytes)"
    )]
    BinaryContent {
        /// Number of null bytes detected.
        null_bytes: usize,
        /// Number of bytes checked.
        checked_bytes: usize,
    },

    /// JSON output is malformed.
    #[error("JSON output is malformed: {reason}")]
    MalformedJson {
        /// Why the JSON is invalid.
        reason: String,
    },
}

/// Sanitize a string according to the output configuration.
///
/// This function:
/// 1. Detects and rejects binary content
/// 2. Strips null bytes if configured
/// 3. Strips control characters (except newlines, tabs) if configured
/// 4. Truncates to max size if needed
///
/// Returns the sanitized string and whether it was truncated.
pub fn sanitize_output(
    input: &str,
    config: &OutputConfig,
) -> std::result::Result<(String, bool), OutputSanitizationError> {
    // Check for binary content by looking for null bytes in the first 8KB
    let check_len = std::cmp::min(input.len(), 8 * 1024);
    let check_bytes = &input.as_bytes()[..check_len];
    let null_count = check_bytes.iter().filter(|&&b| b == 0).count();

    // If more than 1% null bytes, treat as binary
    if null_count > check_len / 100 && null_count > 10 {
        return Err(OutputSanitizationError::BinaryContent {
            null_bytes: null_count,
            checked_bytes: check_len,
        });
    }

    let mut output = input.to_string();

    // Strip null bytes if configured
    if config.strip_null_bytes {
        output = output.replace('\0', "");
    }

    // Strip control characters if configured (keep newlines, tabs, carriage returns)
    if config.strip_control_chars {
        output = output
            .chars()
            .filter(|c| !c.is_control() || *c == '\n' || *c == '\t' || *c == '\r')
            .collect();
    }

    // Truncate if needed
    let truncated = if output.len() > config.max_size_bytes {
        // Find a safe truncation point (don't break UTF-8)
        let mut truncate_at = config.max_size_bytes;
        while truncate_at > 0 && !output.is_char_boundary(truncate_at) {
            truncate_at -= 1;
        }

        // Reserve space for truncation message
        let msg_len = config.truncation_message.len();
        if truncate_at > msg_len {
            truncate_at -= msg_len;
            while truncate_at > 0 && !output.is_char_boundary(truncate_at) {
                truncate_at -= 1;
            }
        }

        output.truncate(truncate_at);
        output.push_str(&config.truncation_message);
        true
    } else {
        false
    };

    Ok((output, truncated))
}

/// Validate that a JSON value has the expected structure.
///
/// Returns an error if the JSON is malformed or has unexpected structure.
pub fn validate_json_output(
    value: &serde_json::Value,
) -> std::result::Result<(), OutputSanitizationError> {
    // Basic validation - ensure it's a valid JSON value
    // The value is already parsed, so it's syntactically valid
    // We check for some edge cases

    // Check for excessively nested structures (could cause stack overflow during processing)
    fn check_depth(value: &serde_json::Value, depth: usize, max_depth: usize) -> bool {
        if depth > max_depth {
            return false;
        }
        match value {
            serde_json::Value::Array(arr) => {
                arr.iter().all(|v| check_depth(v, depth + 1, max_depth))
            }
            serde_json::Value::Object(obj) => {
                obj.values().all(|v| check_depth(v, depth + 1, max_depth))
            }
            _ => true,
        }
    }

    const MAX_JSON_DEPTH: usize = 50;
    if !check_depth(value, 0, MAX_JSON_DEPTH) {
        return Err(OutputSanitizationError::MalformedJson {
            reason: format!("JSON nesting exceeds maximum depth of {}", MAX_JSON_DEPTH),
        });
    }

    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Tool Trait
// ─────────────────────────────────────────────────────────────────────────────

/// Trait for agent tools.
///
/// Tools provide capabilities to the agent - file operations, web search,
/// shell commands, etc. Each tool defines its parameters as a JSON Schema
/// and implements async execution.
#[async_trait]
pub trait Tool: Send + Sync {
    /// Get the unique name of this tool.
    fn name(&self) -> &str;

    /// Get a human-readable description of what this tool does.
    fn description(&self) -> &str;

    /// Get the JSON Schema for this tool's parameters.
    ///
    /// Should return a JSON Schema object describing the expected input.
    fn parameters(&self) -> serde_json::Value;

    /// Execute the tool with the given parameters.
    ///
    /// # Arguments
    /// * `params` - The parameters as a JSON value matching the schema
    /// * `ctx` - Execution context with session info and cancellation
    ///
    /// # Returns
    /// A `ToolResult` indicating success or failure
    async fn execute(&self, params: serde_json::Value, ctx: &ToolContext) -> Result<ToolResult>;
}

// ─────────────────────────────────────────────────────────────────────────────
// Tool Context
// ─────────────────────────────────────────────────────────────────────────────

/// Sender for streaming tool output chunks.
pub type OutputSender = tokio::sync::mpsc::UnboundedSender<String>;

/// Context provided to tools during execution.
#[derive(Debug, Clone)]
pub struct ToolContext {
    /// ID of the session this tool is running in.
    pub session_id: SessionId,
    /// ID of the turn this tool is running in.
    pub turn_id: TurnId,
    /// Token to check for cancellation.
    pub cancellation: CancellationToken,
    /// Optional sender for streaming output during execution.
    pub output_sender: Option<OutputSender>,
    /// Tool call ID (for streaming output association).
    pub tool_call_id: Option<String>,
}

impl ToolContext {
    /// Create a new tool context.
    pub fn new(session_id: SessionId, turn_id: TurnId) -> Self {
        Self {
            session_id,
            turn_id,
            cancellation: CancellationToken::new(),
            output_sender: None,
            tool_call_id: None,
        }
    }

    /// Create a context with a cancellation token.
    pub fn with_cancellation(
        session_id: SessionId,
        turn_id: TurnId,
        cancellation: CancellationToken,
    ) -> Self {
        Self {
            session_id,
            turn_id,
            cancellation,
            output_sender: None,
            tool_call_id: None,
        }
    }

    /// Add streaming output support to this context.
    pub fn with_streaming(mut self, sender: OutputSender, tool_call_id: impl Into<String>) -> Self {
        self.output_sender = Some(sender);
        self.tool_call_id = Some(tool_call_id.into());
        self
    }

    /// Check if execution has been cancelled.
    pub fn is_cancelled(&self) -> bool {
        self.cancellation.is_cancelled()
    }

    /// Check if streaming output is enabled.
    pub fn is_streaming(&self) -> bool {
        self.output_sender.is_some()
    }

    /// Send streaming output chunk.
    /// Returns true if sent successfully, false if streaming is disabled or channel closed.
    pub fn send_output(&self, content: impl Into<String>) -> bool {
        if let Some(ref sender) = self.output_sender {
            sender.send(content.into()).is_ok()
        } else {
            false
        }
    }
}

impl Default for ToolContext {
    fn default() -> Self {
        Self {
            session_id: SessionId::new(),
            turn_id: TurnId::new(),
            cancellation: CancellationToken::new(),
            output_sender: None,
            tool_call_id: None,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tool Result
// ─────────────────────────────────────────────────────────────────────────────

/// Result of a tool execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ToolResult {
    /// Successful text output.
    Text {
        /// The text content.
        content: String,
    },
    /// Successful JSON output.
    Json {
        /// The JSON content.
        content: serde_json::Value,
    },
    /// Tool execution failed.
    Error {
        /// Error message.
        message: String,
        /// Whether the error is recoverable (agent can try again).
        recoverable: bool,
    },
}

impl ToolResult {
    /// Create a text result.
    pub fn text(content: impl Into<String>) -> Self {
        Self::Text {
            content: content.into(),
        }
    }

    /// Create a JSON result.
    pub fn json(content: serde_json::Value) -> Self {
        Self::Json { content }
    }

    /// Create a recoverable error result.
    pub fn error(message: impl Into<String>) -> Self {
        Self::Error {
            message: message.into(),
            recoverable: true,
        }
    }

    /// Create a non-recoverable error result.
    pub fn fatal_error(message: impl Into<String>) -> Self {
        Self::Error {
            message: message.into(),
            recoverable: false,
        }
    }

    /// Check if this result is an error.
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error { .. })
    }

    /// Check if this result is successful.
    pub fn is_success(&self) -> bool {
        !self.is_error()
    }

    /// Get the content as a string for LLM consumption.
    pub fn to_llm_content(&self) -> String {
        match self {
            Self::Text { content } => content.clone(),
            Self::Json { content } => {
                serde_json::to_string_pretty(content).unwrap_or_else(|_| content.to_string())
            }
            Self::Error { message, .. } => format!("Error: {}", message),
        }
    }

    /// Sanitize this result according to the given configuration.
    ///
    /// This method:
    /// - Enforces size limits with truncation
    /// - Strips null bytes and control characters (except newlines, tabs)
    /// - Detects and rejects binary content
    /// - Validates JSON structure for JSON results
    ///
    /// Returns a sanitized result, or converts to an error if sanitization fails.
    pub fn sanitize(self, config: &OutputConfig) -> Self {
        match self {
            Self::Text { content } => match sanitize_output(&content, config) {
                Ok((sanitized, _truncated)) => Self::Text { content: sanitized },
                Err(e) => Self::error(format!("Output sanitization failed: {}", e)),
            },
            Self::Json { content } => {
                // Validate JSON structure if configured
                if config.validate_json
                    && let Err(e) = validate_json_output(&content)
                {
                    return Self::error(format!("JSON validation failed: {}", e));
                }

                // For JSON, we serialize to string to check size and content
                let json_str = match serde_json::to_string_pretty(&content) {
                    Ok(s) => s,
                    Err(e) => {
                        return Self::error(format!("Failed to serialize JSON: {}", e));
                    }
                };

                match sanitize_output(&json_str, config) {
                    Ok((sanitized, truncated)) => {
                        if truncated {
                            // If truncated, return as text since JSON is no longer valid
                            Self::Text { content: sanitized }
                        } else {
                            // Re-parse to ensure valid JSON after sanitization
                            match serde_json::from_str(&sanitized) {
                                Ok(v) => Self::Json { content: v },
                                Err(_) => Self::Text { content: sanitized },
                            }
                        }
                    }
                    Err(e) => Self::error(format!("Output sanitization failed: {}", e)),
                }
            }
            Self::Error {
                message,
                recoverable,
            } => {
                // Sanitize error messages too, but with a smaller limit
                let error_config = OutputConfig {
                    max_size_bytes: 10 * 1024, // 10KB for errors
                    ..config.clone()
                };
                match sanitize_output(&message, &error_config) {
                    Ok((sanitized, _)) => Self::Error {
                        message: sanitized,
                        recoverable,
                    },
                    Err(_) => Self::Error {
                        message: "[Error message contained invalid content]".to_string(),
                        recoverable,
                    },
                }
            }
        }
    }

    /// Sanitize this result with default configuration.
    pub fn sanitize_default(self) -> Self {
        self.sanitize(&OutputConfig::default())
    }

    /// Check if this result was truncated (looks for truncation indicator).
    pub fn was_truncated(&self) -> bool {
        match self {
            Self::Text { content } => content.contains("[Output truncated"),
            Self::Json { .. } => false,
            Self::Error { message, .. } => message.contains("[Output truncated"),
        }
    }

    /// Get the size of the content in bytes.
    pub fn content_size(&self) -> usize {
        match self {
            Self::Text { content } => content.len(),
            Self::Json { content } => serde_json::to_string(content).map(|s| s.len()).unwrap_or(0),
            Self::Error { message, .. } => message.len(),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tool Registry
// ─────────────────────────────────────────────────────────────────────────────

/// Registry for managing available tools.
///
/// The registry maintains a collection of tools that can be used by the agent.
/// It provides lookup by name and conversion to LLM tool definitions.
#[derive(Default, Clone)]
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
    /// Per-tool output config overrides from user configuration.
    output_overrides: HashMap<String, OutputConfig>,
}

impl ToolRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            output_overrides: HashMap::new(),
        }
    }

    /// Set a per-tool output config override.
    ///
    /// This override takes precedence over the hardcoded defaults in
    /// `output_config_for()`. Multiple tool names can map to the same
    /// config (e.g., "shell" and "bash" share a limit).
    pub fn set_output_config(&mut self, name: impl Into<String>, config: OutputConfig) {
        self.output_overrides.insert(name.into(), config);
    }

    /// Register a tool.
    ///
    /// If a tool with the same name already exists, it will be replaced.
    pub fn register<T: Tool + 'static>(&mut self, tool: T) {
        let name = tool.name().to_string();
        self.tools.insert(name, Arc::new(tool));
    }

    /// Register a tool from an Arc.
    pub fn register_arc(&mut self, tool: Arc<dyn Tool>) {
        let name = tool.name().to_string();
        self.tools.insert(name, tool);
    }

    /// Get a tool by name.
    pub fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(name).cloned()
    }

    /// Check if a tool exists.
    pub fn contains(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    /// Get all tool names.
    pub fn names(&self) -> Vec<&str> {
        self.tools.keys().map(|s| s.as_str()).collect()
    }

    /// Get the number of registered tools.
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }

    /// Convert all tools to LLM tool definitions.
    pub fn to_llm_definitions(&self) -> Vec<arawn_llm::ToolDefinition> {
        self.tools
            .values()
            .map(|tool| {
                arawn_llm::ToolDefinition::new(tool.name(), tool.description(), tool.parameters())
            })
            .collect()
    }

    /// Execute a tool by name.
    ///
    /// The result is automatically sanitized with default configuration.
    pub async fn execute(
        &self,
        name: &str,
        params: serde_json::Value,
        ctx: &ToolContext,
    ) -> Result<ToolResult> {
        self.execute_with_config(name, params, ctx, &OutputConfig::default())
            .await
    }

    /// Execute a tool by name with custom output configuration.
    ///
    /// The result is sanitized according to the provided configuration.
    pub async fn execute_with_config(
        &self,
        name: &str,
        params: serde_json::Value,
        ctx: &ToolContext,
        output_config: &OutputConfig,
    ) -> Result<ToolResult> {
        let tool = self
            .get(name)
            .ok_or_else(|| AgentError::ToolNotFound(name.to_string()))?;

        let result = tool.execute(params, ctx).await?;
        Ok(result.sanitize(output_config))
    }

    /// Execute a tool by name without sanitization.
    ///
    /// Use this only when you need the raw, unsanitized output.
    pub async fn execute_raw(
        &self,
        name: &str,
        params: serde_json::Value,
        ctx: &ToolContext,
    ) -> Result<ToolResult> {
        let tool = self
            .get(name)
            .ok_or_else(|| AgentError::ToolNotFound(name.to_string()))?;
        tool.execute(params, ctx).await
    }

    /// Create a new registry containing only tools whose names are in the allowlist.
    ///
    /// Returns a new `ToolRegistry` with cloned `Arc` refs for matching tools.
    /// Names not matching any registered tool are silently ignored.
    /// Output config overrides for matching tools are also carried over.
    pub fn filtered_by_names(&self, names: &[&str]) -> ToolRegistry {
        let tools: HashMap<String, Arc<dyn Tool>> = names
            .iter()
            .filter_map(|&name| {
                self.tools
                    .get(name)
                    .map(|tool| (name.to_string(), Arc::clone(tool)))
            })
            .collect();

        let output_overrides: HashMap<String, OutputConfig> = names
            .iter()
            .filter_map(|&name| {
                self.output_overrides
                    .get(name)
                    .map(|config| (name.to_string(), config.clone()))
            })
            .collect();

        ToolRegistry {
            tools,
            output_overrides,
        }
    }

    /// Get the output config for a tool by name.
    ///
    /// Checks user-configured overrides first, then falls back to
    /// hardcoded per-tool defaults.
    pub fn output_config_for(&self, name: &str) -> OutputConfig {
        // Check overrides first
        if let Some(config) = self.output_overrides.get(name) {
            return config.clone();
        }

        // Fall back to hardcoded per-tool defaults
        match name {
            "shell" | "bash" => OutputConfig::for_shell(),
            "file_read" | "read_file" => OutputConfig::for_file_read(),
            "web_fetch" | "fetch" => OutputConfig::for_web_fetch(),
            "grep" | "glob" | "search" | "memory_search" => OutputConfig::for_search(),
            _ => OutputConfig::default(),
        }
    }
}

impl std::fmt::Debug for ToolRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolRegistry")
            .field("tools", &self.names())
            .finish()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Mock Tool (for testing)
// ─────────────────────────────────────────────────────────────────────────────

/// A mock tool for testing.
///
/// Returns configurable responses and tracks calls for verification.
#[cfg(test)]
#[derive(Debug)]
pub struct MockTool {
    name: String,
    description: String,
    parameters: serde_json::Value,
    response: std::sync::Mutex<Option<ToolResult>>,
    calls: std::sync::Mutex<Vec<serde_json::Value>>,
}

#[cfg(test)]
impl MockTool {
    /// Create a new mock tool.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: "A mock tool for testing".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
            response: std::sync::Mutex::new(None),
            calls: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// Set the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Set the parameters schema.
    pub fn with_parameters(mut self, parameters: serde_json::Value) -> Self {
        self.parameters = parameters;
        self
    }

    /// Set the response to return.
    pub fn with_response(self, response: ToolResult) -> Self {
        *self.response.lock().unwrap() = Some(response);
        self
    }

    /// Get the calls that were made to this tool.
    pub fn calls(&self) -> Vec<serde_json::Value> {
        self.calls.lock().unwrap().clone()
    }

    /// Get the number of calls made.
    pub fn call_count(&self) -> usize {
        self.calls.lock().unwrap().len()
    }

    /// Clear recorded calls.
    pub fn clear_calls(&self) {
        self.calls.lock().unwrap().clear();
    }
}

#[cfg(test)]
#[async_trait]
impl Tool for MockTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn parameters(&self) -> serde_json::Value {
        self.parameters.clone()
    }

    async fn execute(&self, params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult> {
        // Record the call
        self.calls.lock().unwrap().push(params);

        // Return configured response or default
        Ok(self
            .response
            .lock()
            .unwrap()
            .clone()
            .unwrap_or_else(|| ToolResult::text("mock response")))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_result_text() {
        let result = ToolResult::text("hello");
        assert!(result.is_success());
        assert!(!result.is_error());
        assert_eq!(result.to_llm_content(), "hello");
    }

    #[test]
    fn test_tool_result_json() {
        let result = ToolResult::json(serde_json::json!({"key": "value"}));
        assert!(result.is_success());
        let content = result.to_llm_content();
        assert!(content.contains("key"));
        assert!(content.contains("value"));
    }

    #[test]
    fn test_tool_result_error() {
        let result = ToolResult::error("something failed");
        assert!(result.is_error());
        assert!(!result.is_success());
        assert!(result.to_llm_content().contains("Error:"));
    }

    #[test]
    fn test_tool_result_serialization() {
        let result = ToolResult::text("test");
        let json = serde_json::to_string(&result).unwrap();
        let restored: ToolResult = serde_json::from_str(&json).unwrap();
        assert!(matches!(restored, ToolResult::Text { content } if content == "test"));
    }

    #[test]
    fn test_tool_context() {
        let ctx = ToolContext::default();
        assert!(!ctx.is_cancelled());

        let token = CancellationToken::new();
        let ctx2 = ToolContext::with_cancellation(SessionId::new(), TurnId::new(), token.clone());
        assert!(!ctx2.is_cancelled());
        token.cancel();
        assert!(ctx2.is_cancelled());
    }

    #[test]
    fn test_registry_empty() {
        let registry = ToolRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
        assert!(registry.names().is_empty());
    }

    #[test]
    fn test_registry_register_and_get() {
        let mut registry = ToolRegistry::new();

        let tool = MockTool::new("test_tool");
        registry.register(tool);

        assert!(!registry.is_empty());
        assert_eq!(registry.len(), 1);
        assert!(registry.contains("test_tool"));
        assert!(!registry.contains("other"));

        let retrieved = registry.get("test_tool");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name(), "test_tool");
    }

    #[test]
    fn test_registry_names() {
        let mut registry = ToolRegistry::new();
        registry.register(MockTool::new("tool_a"));
        registry.register(MockTool::new("tool_b"));

        let names = registry.names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"tool_a"));
        assert!(names.contains(&"tool_b"));
    }

    #[test]
    fn test_registry_to_llm_definitions() {
        let mut registry = ToolRegistry::new();
        registry.register(
            MockTool::new("read_file")
                .with_description("Read a file from disk")
                .with_parameters(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string", "description": "File path"}
                    },
                    "required": ["path"]
                })),
        );

        let definitions = registry.to_llm_definitions();
        assert_eq!(definitions.len(), 1);
        assert_eq!(definitions[0].name, "read_file");
        assert_eq!(definitions[0].description, "Read a file from disk");
    }

    #[tokio::test]
    async fn test_mock_tool_execution() {
        let tool = MockTool::new("test").with_response(ToolResult::text("custom response"));

        let ctx = ToolContext::default();
        let params = serde_json::json!({"arg": "value"});

        let result = tool.execute(params.clone(), &ctx).await.unwrap();
        assert!(matches!(result, ToolResult::Text { content } if content == "custom response"));

        assert_eq!(tool.call_count(), 1);
        assert_eq!(tool.calls()[0], params);
    }

    #[tokio::test]
    async fn test_registry_execute() {
        let mut registry = ToolRegistry::new();
        registry.register(MockTool::new("my_tool").with_response(ToolResult::text("result")));

        let ctx = ToolContext::default();
        let params = serde_json::json!({});

        // Execute existing tool
        let result = registry.execute("my_tool", params, &ctx).await;
        assert!(result.is_ok());

        // Execute non-existent tool
        let result = registry
            .execute("unknown", serde_json::json!({}), &ctx)
            .await;
        assert!(matches!(result, Err(AgentError::ToolNotFound(_))));
    }

    #[test]
    fn test_mock_tool_clear_calls() {
        let tool = MockTool::new("test");
        tool.calls.lock().unwrap().push(serde_json::json!({}));
        assert_eq!(tool.call_count(), 1);

        tool.clear_calls();
        assert_eq!(tool.call_count(), 0);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Parameter Validation Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_param_validation_error_missing() {
        let err = ParameterValidationError::missing("command", "provide a shell command");
        assert!(err.to_string().contains("command"));
        assert!(err.to_string().contains("missing"));
        assert_eq!(err.parameter_name(), Some("command"));
    }

    #[test]
    fn test_param_validation_error_invalid_type() {
        let err = ParameterValidationError::invalid_type("count", "integer", "string");
        assert!(err.to_string().contains("count"));
        assert!(err.to_string().contains("integer"));
        assert!(err.to_string().contains("string"));
    }

    #[test]
    fn test_param_validation_error_out_of_range() {
        let err = ParameterValidationError::out_of_range("timeout", 0, "must be > 0 and < 3600");
        assert!(err.to_string().contains("timeout"));
        assert!(err.to_string().contains("0"));
        assert!(err.to_string().contains("3600"));
    }

    #[test]
    fn test_param_validation_error_invalid_value() {
        let err = ParameterValidationError::invalid_value(
            "action",
            "delete",
            "must be 'read' or 'write'",
        );
        assert!(err.to_string().contains("action"));
        assert!(err.to_string().contains("delete"));
    }

    #[test]
    fn test_param_validation_error_multiple() {
        let errors = vec![
            ParameterValidationError::missing("a", "hint"),
            ParameterValidationError::missing("b", "hint"),
        ];
        let err = ParameterValidationError::multiple(errors);
        let msg = err.to_string();
        assert!(msg.contains("a"));
        assert!(msg.contains("b"));
        assert_eq!(err.parameter_name(), None);
    }

    #[test]
    fn test_param_ext_required_str() {
        let params = serde_json::json!({"command": "ls -la"});
        assert_eq!(params.required_str("command", "hint").unwrap(), "ls -la");

        let err = params.required_str("missing", "provide it").unwrap_err();
        assert!(matches!(
            err,
            ParameterValidationError::MissingRequired {
                name: "missing",
                ..
            }
        ));
    }

    #[test]
    fn test_param_ext_optional_str() {
        let params = serde_json::json!({"path": "/tmp"});
        assert_eq!(params.optional_str("path"), Some("/tmp"));
        assert_eq!(params.optional_str("missing"), None);
    }

    #[test]
    fn test_param_ext_required_i64() {
        let params = serde_json::json!({"count": 42});
        assert_eq!(params.required_i64("count", "hint").unwrap(), 42);

        let err = params.required_i64("missing", "hint").unwrap_err();
        assert!(matches!(
            err,
            ParameterValidationError::MissingRequired {
                name: "missing",
                ..
            }
        ));
    }

    #[test]
    fn test_param_ext_optional_i64() {
        let params = serde_json::json!({"limit": 100});
        assert_eq!(params.optional_i64("limit", 10), 100);
        assert_eq!(params.optional_i64("missing", 10), 10);
    }

    #[test]
    fn test_param_ext_optional_u64() {
        let params = serde_json::json!({"timeout": 30});
        assert_eq!(params.optional_u64("timeout", 60), 30);
        assert_eq!(params.optional_u64("missing", 60), 60);
    }

    #[test]
    fn test_param_ext_required_bool() {
        let params = serde_json::json!({"enabled": true});
        assert!(params.required_bool("enabled", "hint").unwrap());

        let err = params.required_bool("missing", "hint").unwrap_err();
        assert!(matches!(
            err,
            ParameterValidationError::MissingRequired {
                name: "missing",
                ..
            }
        ));
    }

    #[test]
    fn test_param_ext_optional_bool() {
        let params = serde_json::json!({"verbose": true});
        assert!(params.optional_bool("verbose", false));
        assert!(!params.optional_bool("missing", false));
    }

    #[test]
    fn test_param_ext_optional_array() {
        let params = serde_json::json!({"items": [1, 2, 3]});
        assert!(params.optional_array("items").is_some());
        assert!(params.optional_array("missing").is_none());
    }

    #[test]
    fn test_param_validation_error_into_agent_error() {
        let param_err = ParameterValidationError::missing("test", "hint");
        let agent_err: AgentError = param_err.into();
        assert!(matches!(agent_err, AgentError::Tool(_)));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Typed Parameter Struct Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_shell_params_valid() {
        let params = serde_json::json!({
            "command": "ls -la",
            "pty": true,
            "stream": false,
            "cwd": "/tmp",
            "timeout_secs": 60
        });
        let shell = ShellParams::try_from(params).unwrap();
        assert_eq!(shell.command, "ls -la");
        assert!(shell.pty);
        assert!(!shell.stream);
        assert_eq!(shell.cwd, Some("/tmp".to_string()));
        assert_eq!(shell.timeout_secs, Some(60));
    }

    #[test]
    fn test_shell_params_minimal() {
        let params = serde_json::json!({"command": "echo hello"});
        let shell = ShellParams::try_from(params).unwrap();
        assert_eq!(shell.command, "echo hello");
        assert!(!shell.pty);
        assert!(!shell.stream);
        assert!(shell.cwd.is_none());
        assert!(shell.timeout_secs.is_none());
    }

    #[test]
    fn test_shell_params_missing_command() {
        let params = serde_json::json!({});
        let err = ShellParams::try_from(params).unwrap_err();
        assert!(matches!(
            err,
            ParameterValidationError::MissingRequired {
                name: "command",
                ..
            }
        ));
    }

    #[test]
    fn test_shell_params_empty_command() {
        let params = serde_json::json!({"command": "   "});
        let err = ShellParams::try_from(params).unwrap_err();
        assert!(matches!(
            err,
            ParameterValidationError::InvalidValue {
                name: "command",
                ..
            }
        ));
    }

    #[test]
    fn test_shell_params_timeout_zero() {
        let params = serde_json::json!({"command": "ls", "timeout_secs": 0});
        let err = ShellParams::try_from(params).unwrap_err();
        assert!(matches!(
            err,
            ParameterValidationError::OutOfRange {
                name: "timeout_secs",
                ..
            }
        ));
    }

    #[test]
    fn test_shell_params_timeout_too_large() {
        let params = serde_json::json!({"command": "ls", "timeout_secs": 7200});
        let err = ShellParams::try_from(params).unwrap_err();
        assert!(matches!(
            err,
            ParameterValidationError::OutOfRange {
                name: "timeout_secs",
                ..
            }
        ));
    }

    #[test]
    fn test_file_read_params_valid() {
        let params = serde_json::json!({"path": "/tmp/file.txt"});
        let file = FileReadParams::try_from(params).unwrap();
        assert_eq!(file.path, "/tmp/file.txt");
    }

    #[test]
    fn test_file_read_params_missing_path() {
        let params = serde_json::json!({});
        let err = FileReadParams::try_from(params).unwrap_err();
        assert!(matches!(
            err,
            ParameterValidationError::MissingRequired { name: "path", .. }
        ));
    }

    #[test]
    fn test_file_read_params_empty_path() {
        let params = serde_json::json!({"path": ""});
        let err = FileReadParams::try_from(params).unwrap_err();
        assert!(matches!(
            err,
            ParameterValidationError::InvalidValue { name: "path", .. }
        ));
    }

    #[test]
    fn test_file_write_params_valid() {
        let params = serde_json::json!({
            "path": "/tmp/out.txt",
            "content": "hello world",
            "append": true
        });
        let file = FileWriteParams::try_from(params).unwrap();
        assert_eq!(file.path, "/tmp/out.txt");
        assert_eq!(file.content, "hello world");
        assert!(file.append);
    }

    #[test]
    fn test_file_write_params_missing_content() {
        let params = serde_json::json!({"path": "/tmp/file.txt"});
        let err = FileWriteParams::try_from(params).unwrap_err();
        assert!(matches!(
            err,
            ParameterValidationError::MissingRequired {
                name: "content",
                ..
            }
        ));
    }

    #[test]
    fn test_web_search_params_valid() {
        let params = serde_json::json!({"query": "rust programming", "max_results": 20});
        let search = WebSearchParams::try_from(params).unwrap();
        assert_eq!(search.query, "rust programming");
        assert_eq!(search.max_results, 20);
    }

    #[test]
    fn test_web_search_params_default_max() {
        let params = serde_json::json!({"query": "test"});
        let search = WebSearchParams::try_from(params).unwrap();
        assert_eq!(search.max_results, 10);
    }

    #[test]
    fn test_web_search_params_max_zero() {
        let params = serde_json::json!({"query": "test", "max_results": 0});
        let err = WebSearchParams::try_from(params).unwrap_err();
        assert!(matches!(
            err,
            ParameterValidationError::OutOfRange {
                name: "max_results",
                ..
            }
        ));
    }

    #[test]
    fn test_web_search_params_max_too_large() {
        let params = serde_json::json!({"query": "test", "max_results": 200});
        let err = WebSearchParams::try_from(params).unwrap_err();
        assert!(matches!(
            err,
            ParameterValidationError::OutOfRange {
                name: "max_results",
                ..
            }
        ));
    }

    #[test]
    fn test_think_params_valid() {
        let params = serde_json::json!({"thought": "I should analyze this"});
        let think = ThinkParams::try_from(params).unwrap();
        assert_eq!(think.thought, "I should analyze this");
    }

    #[test]
    fn test_think_params_empty() {
        let params = serde_json::json!({"thought": "  "});
        let err = ThinkParams::try_from(params).unwrap_err();
        assert!(matches!(
            err,
            ParameterValidationError::InvalidValue {
                name: "thought",
                ..
            }
        ));
    }

    #[test]
    fn test_memory_store_params_valid() {
        let params = serde_json::json!({
            "content": "user prefers dark mode",
            "memory_type": "preference",
            "importance": 0.8
        });
        let mem = MemoryStoreParams::try_from(params).unwrap();
        assert_eq!(mem.content, "user prefers dark mode");
        assert_eq!(mem.memory_type, Some("preference".to_string()));
        assert_eq!(mem.importance, Some(0.8));
    }

    #[test]
    fn test_memory_store_params_importance_invalid() {
        let params = serde_json::json!({"content": "test", "importance": 1.5});
        let err = MemoryStoreParams::try_from(params).unwrap_err();
        assert!(matches!(
            err,
            ParameterValidationError::OutOfRange {
                name: "importance",
                ..
            }
        ));
    }

    #[test]
    fn test_memory_store_params_importance_negative() {
        let params = serde_json::json!({"content": "test", "importance": -0.1});
        let err = MemoryStoreParams::try_from(params).unwrap_err();
        assert!(matches!(
            err,
            ParameterValidationError::OutOfRange {
                name: "importance",
                ..
            }
        ));
    }

    #[test]
    fn test_memory_recall_params_valid() {
        let params = serde_json::json!({
            "query": "user preferences",
            "limit": 5,
            "memory_type": "preference"
        });
        let recall = MemoryRecallParams::try_from(params).unwrap();
        assert_eq!(recall.query, "user preferences");
        assert_eq!(recall.limit, 5);
        assert_eq!(recall.memory_type, Some("preference".to_string()));
    }

    #[test]
    fn test_memory_recall_params_limit_zero() {
        let params = serde_json::json!({"query": "test", "limit": 0});
        let err = MemoryRecallParams::try_from(params).unwrap_err();
        assert!(matches!(
            err,
            ParameterValidationError::OutOfRange { name: "limit", .. }
        ));
    }

    #[test]
    fn test_delegate_params_valid() {
        let params = serde_json::json!({
            "task": "search for documentation",
            "agent_type": "researcher"
        });
        let delegate = DelegateParams::try_from(params).unwrap();
        assert_eq!(delegate.task, "search for documentation");
        assert_eq!(delegate.agent_type, Some("researcher".to_string()));
    }

    #[test]
    fn test_delegate_params_empty_task() {
        let params = serde_json::json!({"task": ""});
        let err = DelegateParams::try_from(params).unwrap_err();
        assert!(matches!(
            err,
            ParameterValidationError::InvalidValue { name: "task", .. }
        ));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Output Sanitization Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_output_config_defaults() {
        let config = OutputConfig::default();
        assert_eq!(config.max_size_bytes, DEFAULT_MAX_OUTPUT_SIZE);
        assert!(config.strip_control_chars);
        assert!(config.strip_null_bytes);
        assert!(config.validate_json);
    }

    #[test]
    fn test_output_config_per_tool() {
        let shell = OutputConfig::for_shell();
        assert_eq!(shell.max_size_bytes, 100 * 1024);

        let file = OutputConfig::for_file_read();
        assert_eq!(file.max_size_bytes, 500 * 1024);

        let web = OutputConfig::for_web_fetch();
        assert_eq!(web.max_size_bytes, 200 * 1024);

        let search = OutputConfig::for_search();
        assert_eq!(search.max_size_bytes, 50 * 1024);
    }

    #[test]
    fn test_sanitize_output_normal() {
        let config = OutputConfig::default();
        let (result, truncated) = sanitize_output("Hello, world!", &config).unwrap();
        assert_eq!(result, "Hello, world!");
        assert!(!truncated);
    }

    #[test]
    fn test_sanitize_output_strips_null_bytes() {
        let config = OutputConfig::default();
        let input = "Hello\0World\0!";
        let (result, _) = sanitize_output(input, &config).unwrap();
        assert_eq!(result, "HelloWorld!");
    }

    #[test]
    fn test_sanitize_output_strips_control_chars() {
        let config = OutputConfig::default();
        // ASCII control chars (except newline, tab, CR)
        let input = "Hello\x07World\x1B!";
        let (result, _) = sanitize_output(input, &config).unwrap();
        assert_eq!(result, "HelloWorld!");
    }

    #[test]
    fn test_sanitize_output_preserves_newlines_tabs() {
        let config = OutputConfig::default();
        let input = "Hello\nWorld\tTest\r\nEnd";
        let (result, _) = sanitize_output(input, &config).unwrap();
        assert_eq!(result, "Hello\nWorld\tTest\r\nEnd");
    }

    #[test]
    fn test_sanitize_output_truncates() {
        let config = OutputConfig::with_max_size(50);
        let input = "A".repeat(200);
        let (result, truncated) = sanitize_output(&input, &config).unwrap();
        assert!(truncated);
        assert!(result.len() <= 50);
        assert!(result.contains("[Output truncated"));
    }

    #[test]
    fn test_sanitize_output_truncates_utf8_safe() {
        let config = OutputConfig::with_max_size(50);
        // Multi-byte UTF-8 characters
        let input = "日本語".repeat(20); // Each char is 3 bytes
        let (result, truncated) = sanitize_output(&input, &config).unwrap();
        assert!(truncated);
        // Should not panic or produce invalid UTF-8
        assert!(result.is_ascii() || !result.is_empty());
    }

    #[test]
    fn test_sanitize_output_detects_binary() {
        let config = OutputConfig::default();
        // Lots of null bytes = binary content
        let input = "\0".repeat(1000);
        let result = sanitize_output(&input, &config);
        assert!(matches!(
            result,
            Err(OutputSanitizationError::BinaryContent { .. })
        ));
    }

    #[test]
    fn test_sanitize_output_few_nulls_ok() {
        let config = OutputConfig::default();
        // Just a few null bytes is fine
        let input = format!("Hello{}World", "\0".repeat(5));
        let result = sanitize_output(&input, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_json_output_valid() {
        let value = serde_json::json!({"key": "value", "nested": {"a": 1}});
        let result = validate_json_output(&value);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_json_output_deep_nesting() {
        // Create deeply nested JSON
        let mut value = serde_json::json!("leaf");
        for _ in 0..60 {
            value = serde_json::json!({"nested": value});
        }
        let result = validate_json_output(&value);
        assert!(matches!(
            result,
            Err(OutputSanitizationError::MalformedJson { .. })
        ));
    }

    #[test]
    fn test_tool_result_sanitize_text() {
        let result = ToolResult::text("Hello\0World");
        let config = OutputConfig::default();
        let sanitized = result.sanitize(&config);
        match sanitized {
            ToolResult::Text { content } => {
                assert_eq!(content, "HelloWorld");
            }
            _ => panic!("Expected Text result"),
        }
    }

    #[test]
    fn test_tool_result_sanitize_text_truncated() {
        let result = ToolResult::text("A".repeat(200));
        let config = OutputConfig::with_max_size(50);
        let sanitized = result.sanitize(&config);
        assert!(sanitized.was_truncated());
    }

    #[test]
    fn test_tool_result_sanitize_json() {
        let result = ToolResult::json(serde_json::json!({"key": "value"}));
        let config = OutputConfig::default();
        let sanitized = result.sanitize(&config);
        match sanitized {
            ToolResult::Json { content } => {
                assert_eq!(content["key"], "value");
            }
            _ => panic!("Expected Json result"),
        }
    }

    #[test]
    fn test_tool_result_sanitize_json_truncated_becomes_text() {
        let large_json = serde_json::json!({
            "data": "A".repeat(1000)
        });
        let result = ToolResult::json(large_json);
        let config = OutputConfig::with_max_size(50);
        let sanitized = result.sanitize(&config);
        // Truncated JSON becomes Text since it's no longer valid JSON
        assert!(matches!(sanitized, ToolResult::Text { .. }));
    }

    #[test]
    fn test_tool_result_sanitize_error() {
        let result = ToolResult::error("Something\0went wrong");
        let config = OutputConfig::default();
        let sanitized = result.sanitize(&config);
        match sanitized {
            ToolResult::Error { message, .. } => {
                assert_eq!(message, "Somethingwent wrong");
            }
            _ => panic!("Expected Error result"),
        }
    }

    #[test]
    fn test_tool_result_sanitize_binary_becomes_error() {
        let binary_content = "\0".repeat(1000);
        let result = ToolResult::text(binary_content);
        let config = OutputConfig::default();
        let sanitized = result.sanitize(&config);
        assert!(sanitized.is_error());
        assert!(sanitized.to_llm_content().contains("binary"));
    }

    #[test]
    fn test_tool_result_content_size() {
        let text = ToolResult::text("hello");
        assert_eq!(text.content_size(), 5);

        let json = ToolResult::json(serde_json::json!({"a": 1}));
        assert!(json.content_size() > 0);

        let error = ToolResult::error("oops");
        assert_eq!(error.content_size(), 4);
    }

    #[test]
    fn test_tool_result_sanitize_default() {
        let result = ToolResult::text("Hello\x00World");
        let sanitized = result.sanitize_default();
        match sanitized {
            ToolResult::Text { content } => {
                assert_eq!(content, "HelloWorld");
            }
            _ => panic!("Expected Text result"),
        }
    }

    #[test]
    fn test_registry_output_config_for() {
        let registry = ToolRegistry::new();

        let shell_config = registry.output_config_for("shell");
        assert_eq!(shell_config.max_size_bytes, 100 * 1024);

        let file_config = registry.output_config_for("file_read");
        assert_eq!(file_config.max_size_bytes, 500 * 1024);

        let unknown_config = registry.output_config_for("unknown_tool");
        assert_eq!(unknown_config.max_size_bytes, DEFAULT_MAX_OUTPUT_SIZE);
    }

    #[test]
    fn test_registry_output_config_override() {
        let mut registry = ToolRegistry::new();

        // Set a custom override for shell
        registry.set_output_config("shell", OutputConfig::with_max_size(256 * 1024));

        // Override should take precedence
        let shell_config = registry.output_config_for("shell");
        assert_eq!(shell_config.max_size_bytes, 256 * 1024);

        // "bash" alias still uses hardcoded default (no override set for it)
        let bash_config = registry.output_config_for("bash");
        assert_eq!(bash_config.max_size_bytes, 100 * 1024);

        // Unoverridden tools still use defaults
        let file_config = registry.output_config_for("file_read");
        assert_eq!(file_config.max_size_bytes, 500 * 1024);
    }

    #[test]
    fn test_registry_output_config_override_all_aliases() {
        let mut registry = ToolRegistry::new();

        // Override both shell aliases
        let config = OutputConfig::with_max_size(200 * 1024);
        registry.set_output_config("shell", config.clone());
        registry.set_output_config("bash", config);

        assert_eq!(
            registry.output_config_for("shell").max_size_bytes,
            200 * 1024
        );
        assert_eq!(
            registry.output_config_for("bash").max_size_bytes,
            200 * 1024
        );
    }

    #[tokio::test]
    async fn test_registry_execute_sanitizes() {
        let mut registry = ToolRegistry::new();
        // Create a tool that returns content with null bytes
        registry
            .register(MockTool::new("test_tool").with_response(ToolResult::text("Hello\0World")));

        let ctx = ToolContext::default();
        let result = registry
            .execute("test_tool", serde_json::json!({}), &ctx)
            .await
            .unwrap();

        // Should be sanitized (null bytes removed)
        match result {
            ToolResult::Text { content } => {
                assert_eq!(content, "HelloWorld");
            }
            _ => panic!("Expected Text result"),
        }
    }

    #[tokio::test]
    async fn test_registry_execute_raw_no_sanitize() {
        let mut registry = ToolRegistry::new();
        registry
            .register(MockTool::new("test_tool").with_response(ToolResult::text("Hello\0World")));

        let ctx = ToolContext::default();
        let result = registry
            .execute_raw("test_tool", serde_json::json!({}), &ctx)
            .await
            .unwrap();

        // Should NOT be sanitized
        match result {
            ToolResult::Text { content } => {
                assert!(content.contains('\0'));
            }
            _ => panic!("Expected Text result"),
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // ToolRegistry Filtering Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_filtered_by_names_includes_matching() {
        let mut registry = ToolRegistry::new();
        registry.register(MockTool::new("glob"));
        registry.register(MockTool::new("grep"));
        registry.register(MockTool::new("file_read"));
        registry.register(MockTool::new("shell"));

        let filtered = registry.filtered_by_names(&["glob", "grep", "file_read"]);

        assert_eq!(filtered.len(), 3);
        assert!(filtered.contains("glob"));
        assert!(filtered.contains("grep"));
        assert!(filtered.contains("file_read"));
        assert!(!filtered.contains("shell"));
    }

    #[test]
    fn test_filtered_by_names_excludes_non_matching() {
        let mut registry = ToolRegistry::new();
        registry.register(MockTool::new("shell"));
        registry.register(MockTool::new("file_write"));

        let filtered = registry.filtered_by_names(&["glob", "grep"]);

        assert!(filtered.is_empty());
    }

    #[test]
    fn test_filtered_by_names_ignores_unknown() {
        let mut registry = ToolRegistry::new();
        registry.register(MockTool::new("glob"));

        let filtered = registry.filtered_by_names(&["glob", "nonexistent", "also_missing"]);

        assert_eq!(filtered.len(), 1);
        assert!(filtered.contains("glob"));
    }

    #[test]
    fn test_filtered_by_names_preserves_original() {
        let mut registry = ToolRegistry::new();
        registry.register(MockTool::new("glob"));
        registry.register(MockTool::new("shell"));

        let _filtered = registry.filtered_by_names(&["glob"]);

        // Original unchanged
        assert_eq!(registry.len(), 2);
        assert!(registry.contains("glob"));
        assert!(registry.contains("shell"));
    }

    #[test]
    fn test_filtered_by_names_carries_output_overrides() {
        let mut registry = ToolRegistry::new();
        registry.register(MockTool::new("glob"));
        registry.register(MockTool::new("shell"));
        registry.set_output_config("glob", OutputConfig::with_max_size(999));

        let filtered = registry.filtered_by_names(&["glob"]);

        let config = filtered.output_config_for("glob");
        assert_eq!(config.max_size_bytes, 999);
    }

    #[test]
    fn test_filtered_by_names_llm_definitions() {
        let mut registry = ToolRegistry::new();
        registry.register(MockTool::new("glob").with_description("Search files"));
        registry.register(MockTool::new("shell").with_description("Run commands"));

        let filtered = registry.filtered_by_names(&["glob"]);
        let defs = filtered.to_llm_definitions();

        assert_eq!(defs.len(), 1);
        assert_eq!(defs[0].name, "glob");
    }

    #[test]
    fn test_filtered_by_names_empty_allowlist() {
        let mut registry = ToolRegistry::new();
        registry.register(MockTool::new("glob"));

        let filtered = registry.filtered_by_names(&[]);

        assert!(filtered.is_empty());
    }
}
