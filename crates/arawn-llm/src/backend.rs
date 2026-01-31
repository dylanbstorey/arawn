//! LLM Backend trait and implementations.
//!
//! This module defines the abstraction layer for different LLM providers
//! (Anthropic, OpenAI, local models) and provides mock implementations for testing.

use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use crate::error::{LlmError, ResponseValidationError, Result, is_retryable};
use crate::types::{
    CompletionRequest, CompletionResponse, ContentBlock, StopReason, ToolDefinition, Usage,
};

// ─────────────────────────────────────────────────────────────────────────────
// Shared Retry Logic
// ─────────────────────────────────────────────────────────────────────────────

/// Execute an async operation with exponential backoff retry.
///
/// Retries only on transient errors (network failures). Non-retryable errors
/// are returned immediately.
pub async fn with_retry<F, Fut, T>(
    max_retries: u32,
    initial_backoff: Duration,
    backend_name: &str,
    mut f: F,
) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut last_error = None;
    let mut backoff = initial_backoff;

    for attempt in 0..=max_retries {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                if !is_retryable(&e) {
                    return Err(e);
                }

                last_error = Some(e);

                if attempt < max_retries {
                    tracing::warn!(
                        backend = backend_name,
                        attempt = attempt + 1,
                        max_retries = max_retries,
                        backoff_ms = backoff.as_millis() as u64,
                        "Request failed, retrying"
                    );
                    tokio::time::sleep(backoff).await;
                    backoff *= 2;
                }
            }
        }
    }

    Err(last_error.unwrap())
}

// ─────────────────────────────────────────────────────────────────────────────
// Streaming Types
// ─────────────────────────────────────────────────────────────────────────────

/// A streaming response from an LLM backend.
pub type ResponseStream = Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send + 'static>>;

/// Events emitted during streaming.
#[derive(Debug, Clone)]
pub enum StreamEvent {
    /// Message started.
    MessageStart { id: String, model: String },
    /// Content block started.
    ContentBlockStart { index: usize, content_type: String },
    /// Text delta within a content block.
    ContentBlockDelta { index: usize, delta: ContentDelta },
    /// Content block finished.
    ContentBlockStop { index: usize },
    /// Message finished with final usage stats.
    MessageDelta {
        stop_reason: StopReason,
        usage: Usage,
    },
    /// Message complete.
    MessageStop,
    /// Ping to keep connection alive.
    Ping,
    /// Error occurred.
    Error { message: String },
}

/// Delta content in a streaming response.
#[derive(Debug, Clone)]
pub enum ContentDelta {
    /// Text being streamed.
    TextDelta(String),
    /// Partial JSON for tool input.
    InputJsonDelta(String),
}

impl StreamEvent {
    /// Validate the stream event structure.
    ///
    /// Checks that the event has valid structure based on its type.
    /// Returns `Ok(())` if valid, or a validation error.
    pub fn validate(&self) -> std::result::Result<(), ResponseValidationError> {
        match self {
            StreamEvent::MessageStart { id, model } => {
                if id.trim().is_empty() {
                    return Err(ResponseValidationError::invalid_stream_event(
                        "MessageStart has empty id",
                    ));
                }
                if model.trim().is_empty() {
                    return Err(ResponseValidationError::invalid_stream_event(
                        "MessageStart has empty model",
                    ));
                }
                Ok(())
            }
            StreamEvent::ContentBlockStart { content_type, .. } => {
                // content_type should be known
                let valid_types = ["text", "tool_use"];
                if !valid_types.contains(&content_type.as_str()) {
                    tracing::warn!(
                        content_type = %content_type,
                        "Unknown content block type in stream event"
                    );
                }
                Ok(())
            }
            StreamEvent::ContentBlockDelta { .. } => {
                // Delta content is already typed, no validation needed
                Ok(())
            }
            StreamEvent::ContentBlockStop { .. } => Ok(()),
            StreamEvent::MessageDelta { .. } => {
                // stop_reason is enum, usage has u32 fields - type-safe
                Ok(())
            }
            StreamEvent::MessageStop => Ok(()),
            StreamEvent::Ping => Ok(()),
            StreamEvent::Error { message } => {
                if message.trim().is_empty() {
                    return Err(ResponseValidationError::invalid_stream_event(
                        "Error event has empty message",
                    ));
                }
                Ok(())
            }
        }
    }

    /// Returns true if this is an error event.
    pub fn is_error(&self) -> bool {
        matches!(self, StreamEvent::Error { .. })
    }

    /// Returns true if this is the final event in a message.
    pub fn is_terminal(&self) -> bool {
        matches!(self, StreamEvent::MessageStop | StreamEvent::Error { .. })
    }
}

/// A parsed tool call from model output.
#[derive(Debug, Clone)]
pub struct ParsedToolCall {
    /// Unique ID for this tool call.
    pub id: String,
    /// Name of the tool to call.
    pub name: String,
    /// Arguments as JSON.
    pub arguments: serde_json::Value,
}

impl From<ParsedToolCall> for ContentBlock {
    fn from(call: ParsedToolCall) -> Self {
        ContentBlock::ToolUse {
            id: call.id,
            name: call.name,
            input: call.arguments,
            cache_control: None,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// LLM Backend Trait
// ─────────────────────────────────────────────────────────────────────────────

/// Trait for LLM backend providers.
///
/// Implementations of this trait provide the actual connection to LLM services
/// like Anthropic's Claude API, OpenAI, or local models.
///
/// ## Tool Calling Support
///
/// Backends can support tools in two ways:
/// 1. **Native**: Tools passed via API, responses contain structured tool_use blocks
/// 2. **Prompt-based**: Tools injected into prompt, responses parsed for tool calls
///
/// Override the tool format methods to customize behavior per backend/model.
#[async_trait]
pub trait LlmBackend: Send + Sync {
    /// Execute a completion request and return the full response.
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse>;

    /// Execute a completion request and return a stream of events.
    async fn complete_stream(&self, request: CompletionRequest) -> Result<ResponseStream>;

    /// Get the name of this backend.
    fn name(&self) -> &str;

    /// Check if the backend is available and properly configured.
    async fn health_check(&self) -> Result<()>;

    // ─────────────────────────────────────────────────────────────────────────
    // Tool Format Methods (with defaults)
    // ─────────────────────────────────────────────────────────────────────────

    /// Returns true if backend handles tools natively via API.
    ///
    /// When true:
    /// - Tools are passed via `request.tools` to the API
    /// - Responses contain structured `tool_use` content blocks
    /// - No prompt injection or output parsing needed
    ///
    /// When false (default):
    /// - Tools are formatted and injected into the system prompt
    /// - Responses must be parsed for tool calls
    fn supports_native_tools(&self) -> bool {
        false
    }

    /// Instructions for HOW to call tools (model-specific format).
    ///
    /// This is appended to the system prompt to tell the model the expected
    /// output format for tool calls. Only used when `supports_native_tools()` is false.
    ///
    /// Returns None if no special instructions needed (uses default human-readable).
    fn tool_calling_instructions(&self) -> Option<&str> {
        None
    }

    /// Format tool definitions for the system prompt.
    ///
    /// Converts tool definitions into a string that's injected into the system
    /// prompt. Only used when `supports_native_tools()` is false.
    ///
    /// Default: human-readable format with parameters.
    fn format_tool_definitions(&self, tools: &[ToolDefinition]) -> String {
        default_format_tool_definitions(tools)
    }

    /// Format a tool result for the conversation.
    ///
    /// Converts tool execution results into the format expected by the model.
    /// Only used when `supports_native_tools()` is false.
    ///
    /// Default: simple text format.
    fn format_tool_result(&self, tool_use_id: &str, content: &str, is_error: bool) -> String {
        default_format_tool_result(tool_use_id, content, is_error)
    }

    /// Parse tool calls from model text output.
    ///
    /// Extracts tool calls from the model's response text and returns the
    /// remaining text along with parsed tool calls.
    /// Only used when `supports_native_tools()` is false.
    ///
    /// Default: no parsing, returns original text with empty tool calls.
    fn parse_tool_calls(&self, text: &str) -> (String, Vec<ParsedToolCall>) {
        (text.to_string(), vec![])
    }
}

/// Default human-readable format for tool definitions.
pub fn default_format_tool_definitions(tools: &[ToolDefinition]) -> String {
    if tools.is_empty() {
        return "No tools available.".to_string();
    }

    let mut output = String::from("## Available Tools\n\n");

    for tool in tools {
        output.push_str(&format!("### {}\n", tool.name));
        output.push_str(&format!("{}\n", tool.description));

        // Format input schema if it has properties
        if let Some(properties) = tool.input_schema.get("properties") {
            if let Some(props) = properties.as_object() {
                if !props.is_empty() {
                    output.push_str("\nParameters:\n");
                    for (name, schema) in props {
                        let type_str = schema.get("type").and_then(|t| t.as_str()).unwrap_or("any");
                        let desc = schema
                            .get("description")
                            .and_then(|d| d.as_str())
                            .unwrap_or("");
                        output.push_str(&format!("- `{}` ({}): {}\n", name, type_str, desc));
                    }
                }
            }
        }
        output.push('\n');
    }

    output
}

/// Default format for tool results.
pub fn default_format_tool_result(tool_use_id: &str, content: &str, is_error: bool) -> String {
    if is_error {
        format!("[Tool {} Error]: {}", tool_use_id, content)
    } else {
        format!("[Tool {} Result]: {}", tool_use_id, content)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Mock Backend
// ─────────────────────────────────────────────────────────────────────────────

/// A mock backend for testing purposes.
///
/// Returns pre-configured responses in order, useful for deterministic testing
/// of the agent loop and tool execution.
#[derive(Debug)]
pub struct MockBackend {
    name: String,
    responses: std::sync::Mutex<Vec<CompletionResponse>>,
    request_log: std::sync::Mutex<Vec<CompletionRequest>>,
}

impl MockBackend {
    /// Create a new mock backend with the given responses.
    ///
    /// Responses are returned in order. If more requests are made than
    /// responses available, an error is returned.
    pub fn new(responses: Vec<CompletionResponse>) -> Self {
        Self {
            name: "mock".to_string(),
            responses: std::sync::Mutex::new(responses),
            request_log: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// Create a mock backend with a single text response.
    pub fn with_text(text: impl Into<String>) -> Self {
        Self::new(vec![CompletionResponse::new(
            "mock_msg_1",
            "mock-model",
            vec![ContentBlock::Text {
                text: text.into(),
                cache_control: None,
            }],
            StopReason::EndTurn,
            Usage::new(10, 20),
        )])
    }

    /// Get all requests that were made to this backend.
    pub fn requests(&self) -> Vec<CompletionRequest> {
        self.request_log.lock().unwrap().clone()
    }

    /// Get the number of requests made.
    pub fn request_count(&self) -> usize {
        self.request_log.lock().unwrap().len()
    }
}

#[async_trait]
impl LlmBackend for MockBackend {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        // Log the request
        self.request_log.lock().unwrap().push(request);

        // Return the next response
        let mut responses = self.responses.lock().unwrap();
        if responses.is_empty() {
            return Err(LlmError::Backend(
                "MockBackend: no more responses available".to_string(),
            ));
        }
        Ok(responses.remove(0))
    }

    async fn complete_stream(&self, request: CompletionRequest) -> Result<ResponseStream> {
        // For mock, just convert the sync response to a stream
        let response = self.complete(request).await?;

        let events = vec![
            Ok(StreamEvent::MessageStart {
                id: response.id.clone(),
                model: response.model.clone(),
            }),
            Ok(StreamEvent::ContentBlockStart {
                index: 0,
                content_type: "text".to_string(),
            }),
            Ok(StreamEvent::ContentBlockDelta {
                index: 0,
                delta: ContentDelta::TextDelta(response.text()),
            }),
            Ok(StreamEvent::ContentBlockStop { index: 0 }),
            Ok(StreamEvent::MessageDelta {
                stop_reason: response.stop_reason.unwrap_or(StopReason::EndTurn),
                usage: response.usage,
            }),
            Ok(StreamEvent::MessageStop),
        ];

        Ok(Box::pin(futures::stream::iter(events)))
    }

    fn name(&self) -> &str {
        &self.name
    }

    async fn health_check(&self) -> Result<()> {
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Shared Backend Type
// ─────────────────────────────────────────────────────────────────────────────

/// A backend that can be shared across threads.
pub type SharedBackend = Arc<dyn LlmBackend>;

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Message;

    #[tokio::test]
    async fn test_mock_backend_single_response() {
        let backend = MockBackend::with_text("Hello!");

        let request = CompletionRequest::new("test-model", vec![Message::user("Hi")], 100);
        let response = backend.complete(request).await.unwrap();

        assert_eq!(response.text(), "Hello!");
        assert_eq!(backend.request_count(), 1);
    }

    #[tokio::test]
    async fn test_mock_backend_multiple_responses() {
        let backend = MockBackend::new(vec![
            CompletionResponse::new(
                "msg_1",
                "model",
                vec![ContentBlock::Text {
                    text: "First".to_string(),
                    cache_control: None,
                }],
                StopReason::EndTurn,
                Usage::new(10, 10),
            ),
            CompletionResponse::new(
                "msg_2",
                "model",
                vec![ContentBlock::Text {
                    text: "Second".to_string(),
                    cache_control: None,
                }],
                StopReason::EndTurn,
                Usage::new(10, 10),
            ),
        ]);

        let request = CompletionRequest::new("test-model", vec![Message::user("1")], 100);
        let r1 = backend.complete(request).await.unwrap();

        let request = CompletionRequest::new("test-model", vec![Message::user("2")], 100);
        let r2 = backend.complete(request).await.unwrap();

        assert_eq!(r1.text(), "First");
        assert_eq!(r2.text(), "Second");
        assert_eq!(backend.request_count(), 2);
    }

    #[tokio::test]
    async fn test_mock_backend_exhausted() {
        let backend = MockBackend::new(vec![]);

        let request = CompletionRequest::new("test-model", vec![Message::user("Hi")], 100);
        let result = backend.complete(request).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mock_backend_with_tool_use() {
        let backend = MockBackend::new(vec![CompletionResponse::new(
            "msg_1",
            "model",
            vec![
                ContentBlock::Text {
                    text: "Let me check.".to_string(),
                    cache_control: None,
                },
                ContentBlock::ToolUse {
                    id: "tool_1".to_string(),
                    name: "read_file".to_string(),
                    input: serde_json::json!({"path": "/foo.rs"}),
                    cache_control: None,
                },
            ],
            StopReason::ToolUse,
            Usage::new(50, 30),
        )]);

        let request = CompletionRequest::new("test-model", vec![Message::user("Read foo.rs")], 100);
        let response = backend.complete(request).await.unwrap();

        assert!(response.has_tool_use());
        assert_eq!(response.stop_reason, Some(StopReason::ToolUse));

        let tool_uses = response.tool_uses();
        assert_eq!(tool_uses.len(), 1);
        assert_eq!(tool_uses[0].name, "read_file");
    }

    #[tokio::test]
    async fn test_mock_backend_stream() {
        use futures::StreamExt;

        let backend = MockBackend::with_text("Streamed!");

        let request = CompletionRequest::new("test-model", vec![Message::user("Hi")], 100);
        let mut stream = backend.complete_stream(request).await.unwrap();

        let mut events = Vec::new();
        while let Some(event) = stream.next().await {
            events.push(event.unwrap());
        }

        // Should have: MessageStart, ContentBlockStart, ContentBlockDelta, ContentBlockStop, MessageDelta, MessageStop
        assert_eq!(events.len(), 6);
        assert!(matches!(events[0], StreamEvent::MessageStart { .. }));
        assert!(matches!(events[5], StreamEvent::MessageStop));
    }

    #[tokio::test]
    async fn test_mock_backend_health_check() {
        let backend = MockBackend::with_text("test");
        assert!(backend.health_check().await.is_ok());
    }

    #[test]
    fn test_default_format_tool_definitions() {
        let tools = vec![ToolDefinition::new(
            "read_file",
            "Read a file from disk",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the file"
                    }
                }
            }),
        )];

        let formatted = default_format_tool_definitions(&tools);
        assert!(formatted.contains("### read_file"));
        assert!(formatted.contains("Read a file from disk"));
        assert!(formatted.contains("`path` (string)"));
    }

    #[test]
    fn test_default_format_tool_result() {
        let success = default_format_tool_result("tool_1", "file contents", false);
        assert!(success.contains("[Tool tool_1 Result]"));

        let error = default_format_tool_result("tool_2", "not found", true);
        assert!(error.contains("[Tool tool_2 Error]"));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // StreamEvent Validation Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_stream_event_validate_message_start() {
        let valid = StreamEvent::MessageStart {
            id: "msg_123".to_string(),
            model: "claude-3".to_string(),
        };
        assert!(valid.validate().is_ok());

        let empty_id = StreamEvent::MessageStart {
            id: "".to_string(),
            model: "claude-3".to_string(),
        };
        assert!(empty_id.validate().is_err());

        let empty_model = StreamEvent::MessageStart {
            id: "msg_123".to_string(),
            model: "".to_string(),
        };
        assert!(empty_model.validate().is_err());
    }

    #[test]
    fn test_stream_event_validate_content_block_start() {
        let valid = StreamEvent::ContentBlockStart {
            index: 0,
            content_type: "text".to_string(),
        };
        assert!(valid.validate().is_ok());

        // Unknown types log a warning but don't fail
        let unknown = StreamEvent::ContentBlockStart {
            index: 0,
            content_type: "unknown_type".to_string(),
        };
        assert!(unknown.validate().is_ok());
    }

    #[test]
    fn test_stream_event_validate_error() {
        let valid = StreamEvent::Error {
            message: "Something went wrong".to_string(),
        };
        assert!(valid.validate().is_ok());

        let empty = StreamEvent::Error {
            message: "  ".to_string(),
        };
        assert!(empty.validate().is_err());
    }

    #[test]
    fn test_stream_event_is_error() {
        assert!(
            StreamEvent::Error {
                message: "oops".to_string()
            }
            .is_error()
        );
        assert!(!StreamEvent::Ping.is_error());
        assert!(!StreamEvent::MessageStop.is_error());
    }

    #[test]
    fn test_stream_event_is_terminal() {
        assert!(StreamEvent::MessageStop.is_terminal());
        assert!(
            StreamEvent::Error {
                message: "err".to_string()
            }
            .is_terminal()
        );
        assert!(!StreamEvent::Ping.is_terminal());
        assert!(!StreamEvent::ContentBlockStop { index: 0 }.is_terminal());
    }

    #[test]
    fn test_stream_event_validate_other_events() {
        // These should all be valid
        assert!(StreamEvent::Ping.validate().is_ok());
        assert!(StreamEvent::MessageStop.validate().is_ok());
        assert!(
            StreamEvent::ContentBlockStop { index: 0 }
                .validate()
                .is_ok()
        );
        assert!(
            StreamEvent::ContentBlockDelta {
                index: 0,
                delta: ContentDelta::TextDelta("hello".to_string()),
            }
            .validate()
            .is_ok()
        );
        assert!(
            StreamEvent::MessageDelta {
                stop_reason: StopReason::EndTurn,
                usage: Usage::new(10, 20),
            }
            .validate()
            .is_ok()
        );
    }
}
