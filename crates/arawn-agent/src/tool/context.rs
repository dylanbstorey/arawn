//! Core tool types: the Tool trait, ToolContext, and ToolResult.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;

use arawn_types::{SharedFsGate, SharedSecretResolver};

use crate::error::Result;
use crate::types::{SessionId, TurnId};

use super::output::{OutputConfig, sanitize_output, validate_json_output};

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
///
/// # Examples
///
/// ```rust,ignore
/// use arawn_agent::tool::ToolContext;
/// use arawn_agent::types::{SessionId, TurnId};
///
/// let ctx = ToolContext::new(SessionId::new(), TurnId::new());
/// assert!(!ctx.is_cancelled());
/// assert!(!ctx.is_streaming());
/// ```
#[derive(Clone)]
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
    /// Optional filesystem gate for workstream sandbox enforcement.
    pub fs_gate: Option<SharedFsGate>,
    /// Optional secret resolver for `${{secrets.*}}` handle resolution.
    pub secret_resolver: Option<SharedSecretResolver>,
}

impl std::fmt::Debug for ToolContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolContext")
            .field("session_id", &self.session_id)
            .field("turn_id", &self.turn_id)
            .field("tool_call_id", &self.tool_call_id)
            .field("fs_gate", &self.fs_gate.as_ref().map(|_| "Some(<gate>)"))
            .field(
                "secret_resolver",
                &self.secret_resolver.as_ref().map(|_| "Some(<resolver>)"),
            )
            .finish()
    }
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
            fs_gate: None,
            secret_resolver: None,
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
            fs_gate: None,
            secret_resolver: None,
        }
    }

    /// Set the filesystem gate for workstream sandbox enforcement.
    pub fn with_fs_gate(mut self, gate: SharedFsGate) -> Self {
        self.fs_gate = Some(gate);
        self
    }

    /// Set the secret resolver for `${{secrets.*}}` handle resolution.
    pub fn with_secret_resolver(mut self, resolver: SharedSecretResolver) -> Self {
        self.secret_resolver = Some(resolver);
        self
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
            fs_gate: None,
            secret_resolver: None,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tool Result
// ─────────────────────────────────────────────────────────────────────────────

/// Result of a tool execution.
///
/// # Examples
///
/// ```rust,ignore
/// use arawn_agent::tool::ToolResult;
///
/// let ok = ToolResult::text("Operation succeeded");
/// assert!(ok.is_success());
///
/// let err = ToolResult::error("File not found");
/// assert!(err.is_error());
/// ```
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
}
