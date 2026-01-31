//! Core types for LLM requests and responses.
//!
//! These types are designed to be compatible with the Anthropic Messages API
//! while being provider-agnostic for use with other backends.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;

use crate::error::ResponseValidationError;

// ─────────────────────────────────────────────────────────────────────────────
// System Prompt
// ─────────────────────────────────────────────────────────────────────────────

/// System prompt - can be a string or array of text blocks.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SystemPrompt {
    /// Simple string prompt.
    Text(String),
    /// Array of text blocks (for cache control).
    Blocks(Vec<SystemBlock>),
}

/// A text block in a system prompt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemBlock {
    /// The text content.
    pub text: String,
    /// Block type (always "text").
    #[serde(rename = "type")]
    pub block_type: String,
    /// Optional cache control.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

impl SystemPrompt {
    /// Create a simple text system prompt.
    pub fn text(content: impl Into<String>) -> Self {
        SystemPrompt::Text(content.into())
    }

    /// Get the text content of the system prompt.
    pub fn to_text(&self) -> String {
        match self {
            SystemPrompt::Text(s) => s.clone(),
            SystemPrompt::Blocks(blocks) => blocks
                .iter()
                .map(|b| b.text.as_str())
                .collect::<Vec<_>>()
                .join("\n"),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Completion Request
// ─────────────────────────────────────────────────────────────────────────────

/// A completion request to an LLM provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    /// The model to use for completion.
    pub model: String,

    /// The messages in the conversation.
    pub messages: Vec<Message>,

    /// Maximum tokens to generate.
    pub max_tokens: u32,

    /// System prompt (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<SystemPrompt>,

    /// Tools available for the model to use.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<ToolDefinition>,

    /// How the model should use tools.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,

    /// Whether to stream the response.
    #[serde(default)]
    pub stream: bool,

    /// Temperature for sampling (0.0 to 1.0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Top-p sampling parameter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// Top-k sampling parameter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,

    /// Stop sequences.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stop_sequences: Vec<String>,

    /// Additional metadata.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl CompletionRequest {
    /// Create a new completion request with the given model and messages.
    pub fn new(model: impl Into<String>, messages: Vec<Message>, max_tokens: u32) -> Self {
        Self {
            model: model.into(),
            messages,
            max_tokens,
            system: None,
            tools: Vec::new(),
            tool_choice: None,
            stream: false,
            temperature: None,
            top_p: None,
            top_k: None,
            stop_sequences: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Set the system prompt.
    pub fn with_system(mut self, system: impl Into<String>) -> Self {
        self.system = Some(SystemPrompt::Text(system.into()));
        self
    }

    /// Add tools to the request.
    pub fn with_tools(mut self, tools: Vec<ToolDefinition>) -> Self {
        self.tools = tools;
        self
    }

    /// Set tool choice.
    pub fn with_tool_choice(mut self, choice: ToolChoice) -> Self {
        self.tool_choice = Some(choice);
        self
    }

    /// Enable streaming.
    pub fn with_streaming(mut self) -> Self {
        self.stream = true;
        self
    }

    /// Set temperature.
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Messages
// ─────────────────────────────────────────────────────────────────────────────

/// A message in the conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// The role of the message author.
    pub role: Role,

    /// The content of the message.
    pub content: Content,
}

impl Message {
    /// Create a user message with text content.
    pub fn user(text: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: Content::Text(text.into()),
        }
    }

    /// Create an assistant message with text content.
    pub fn assistant(text: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: Content::Text(text.into()),
        }
    }

    /// Create an assistant message with content blocks.
    pub fn assistant_blocks(blocks: Vec<ContentBlock>) -> Self {
        Self {
            role: Role::Assistant,
            content: Content::Blocks(blocks),
        }
    }

    /// Create a user message with tool results.
    pub fn tool_results(results: Vec<ToolResultBlock>) -> Self {
        Self {
            role: Role::User,
            content: Content::Blocks(results.into_iter().map(|r| r.into()).collect()),
        }
    }
}

/// The role of a message author.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
}

/// Message content - either a simple string or structured blocks.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Content {
    /// Simple text content.
    Text(String),
    /// Structured content blocks.
    Blocks(Vec<ContentBlock>),
}

impl Content {
    /// Get the text content if this is simple text.
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Content::Text(s) => Some(s),
            Content::Blocks(_) => None,
        }
    }

    /// Get the content blocks.
    pub fn blocks(&self) -> Vec<ContentBlock> {
        match self {
            Content::Text(s) => vec![ContentBlock::Text {
                text: s.clone(),
                cache_control: None,
            }],
            Content::Blocks(blocks) => blocks.clone(),
        }
    }

    /// Extract all text from the content.
    pub fn to_text(&self) -> String {
        match self {
            Content::Text(s) => s.clone(),
            Content::Blocks(blocks) => blocks
                .iter()
                .filter_map(|b| match b {
                    ContentBlock::Text { text, .. } => Some(text.as_str()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join(""),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Content Blocks
// ─────────────────────────────────────────────────────────────────────────────

/// Cache control for prompt caching.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CacheControl {
    /// Ephemeral cache control.
    Ephemeral,
}

/// A content block in a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    /// Text content.
    Text {
        /// The text content.
        text: String,
        /// Optional cache control.
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
    /// Tool use request from the assistant.
    ToolUse {
        /// Unique ID for this tool use.
        id: String,
        /// Name of the tool to use.
        name: String,
        /// Input arguments for the tool.
        input: serde_json::Value,
        /// Optional cache control.
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
    /// Tool result from the user.
    ToolResult {
        /// ID of the tool use this is a result for.
        tool_use_id: String,
        /// The result content (optional).
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<ToolResultContent>,
        /// Whether the tool execution resulted in an error.
        #[serde(default)]
        is_error: bool,
        /// Optional cache control.
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
}

impl ContentBlock {
    /// Create a text content block.
    pub fn text(content: impl Into<String>) -> Self {
        ContentBlock::Text {
            text: content.into(),
            cache_control: None,
        }
    }

    /// Create a tool use content block.
    pub fn tool_use(
        id: impl Into<String>,
        name: impl Into<String>,
        input: serde_json::Value,
    ) -> Self {
        ContentBlock::ToolUse {
            id: id.into(),
            name: name.into(),
            input,
            cache_control: None,
        }
    }

    /// Create a successful tool result block.
    pub fn tool_result_success(tool_use_id: impl Into<String>, content: impl Into<String>) -> Self {
        ContentBlock::ToolResult {
            tool_use_id: tool_use_id.into(),
            content: Some(ToolResultContent::Text(content.into())),
            is_error: false,
            cache_control: None,
        }
    }

    /// Create an error tool result block.
    pub fn tool_result_error(tool_use_id: impl Into<String>, error: impl Into<String>) -> Self {
        ContentBlock::ToolResult {
            tool_use_id: tool_use_id.into(),
            content: Some(ToolResultContent::Text(error.into())),
            is_error: true,
            cache_control: None,
        }
    }
}

/// Tool result content - can be a string or array of content blocks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolResultContent {
    Text(String),
    Blocks(Vec<serde_json::Value>),
}

// ─────────────────────────────────────────────────────────────────────────────
// Tool Use/Result Blocks (convenience types)
// ─────────────────────────────────────────────────────────────────────────────

/// Convenience struct for creating tool use blocks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUseBlock {
    /// Unique ID for this tool use.
    pub id: String,
    /// Name of the tool to use.
    pub name: String,
    /// Input arguments for the tool.
    pub input: serde_json::Value,
}

impl From<ToolUseBlock> for ContentBlock {
    fn from(block: ToolUseBlock) -> Self {
        ContentBlock::ToolUse {
            id: block.id,
            name: block.name,
            input: block.input,
            cache_control: None,
        }
    }
}

/// Convenience struct for creating tool result blocks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResultBlock {
    /// ID of the tool use this is a result for.
    pub tool_use_id: String,
    /// The result content (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<ToolResultContent>,
    /// Whether the tool execution resulted in an error.
    #[serde(default)]
    pub is_error: bool,
}

impl ToolResultBlock {
    /// Create a successful tool result.
    pub fn success(tool_use_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            tool_use_id: tool_use_id.into(),
            content: Some(ToolResultContent::Text(content.into())),
            is_error: false,
        }
    }

    /// Create an error tool result.
    pub fn error(tool_use_id: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            tool_use_id: tool_use_id.into(),
            content: Some(ToolResultContent::Text(error.into())),
            is_error: true,
        }
    }
}

impl From<ToolResultBlock> for ContentBlock {
    fn from(block: ToolResultBlock) -> Self {
        ContentBlock::ToolResult {
            tool_use_id: block.tool_use_id,
            content: block.content,
            is_error: block.is_error,
            cache_control: None,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tools
// ─────────────────────────────────────────────────────────────────────────────

/// Definition of a tool available to the model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Name of the tool.
    pub name: String,

    /// Description of what the tool does.
    pub description: String,

    /// JSON Schema for the tool's input parameters.
    pub input_schema: serde_json::Value,
}

impl ToolDefinition {
    /// Create a new tool definition.
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        input_schema: serde_json::Value,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            input_schema,
        }
    }
}

/// How the model should choose which tool to use.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ToolChoice {
    /// Model decides whether to use tools.
    Auto,
    /// Model must use a tool.
    Any,
    /// Model must use a specific tool.
    Tool { name: String },
    /// Model should not use tools.
    None,
}

// ─────────────────────────────────────────────────────────────────────────────
// Completion Response
// ─────────────────────────────────────────────────────────────────────────────

/// A completion response from the model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    /// Unique ID for this response.
    pub id: String,

    /// The type of response (always "message").
    #[serde(rename = "type", default = "default_message_type")]
    pub response_type: String,

    /// The role (always "assistant").
    pub role: Role,

    /// The content blocks in the response.
    pub content: Vec<ContentBlock>,

    /// The model that generated the response.
    pub model: String,

    /// Why the model stopped generating.
    pub stop_reason: Option<StopReason>,

    /// Token usage statistics.
    pub usage: Usage,
}

fn default_message_type() -> String {
    "message".to_string()
}

impl CompletionResponse {
    /// Create a new completion response.
    pub fn new(
        id: impl Into<String>,
        model: impl Into<String>,
        content: Vec<ContentBlock>,
        stop_reason: StopReason,
        usage: Usage,
    ) -> Self {
        Self {
            id: id.into(),
            response_type: "message".to_string(),
            role: Role::Assistant,
            content,
            model: model.into(),
            stop_reason: Some(stop_reason),
            usage,
        }
    }

    /// Get all tool use blocks from the response.
    pub fn tool_uses(&self) -> Vec<ToolUseBlock> {
        self.content
            .iter()
            .filter_map(|block| match block {
                ContentBlock::ToolUse {
                    id, name, input, ..
                } => Some(ToolUseBlock {
                    id: id.clone(),
                    name: name.clone(),
                    input: input.clone(),
                }),
                _ => None,
            })
            .collect()
    }

    /// Get the text content from the response.
    pub fn text(&self) -> String {
        self.content
            .iter()
            .filter_map(|block| match block {
                ContentBlock::Text { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("")
    }

    /// Check if the response contains tool use requests.
    pub fn has_tool_use(&self) -> bool {
        self.content
            .iter()
            .any(|block| matches!(block, ContentBlock::ToolUse { .. }))
    }

    /// Validate the response structure.
    ///
    /// Checks that all required fields are present, tool_use blocks are valid,
    /// and token counts are sensible. Returns a list of all validation errors found.
    ///
    /// # Example
    ///
    /// ```
    /// use arawn_llm::{CompletionResponse, ContentBlock, StopReason, Usage};
    ///
    /// let response = CompletionResponse::new(
    ///     "msg_123",
    ///     "claude-3",
    ///     vec![ContentBlock::text("Hello")],
    ///     StopReason::EndTurn,
    ///     Usage::new(10, 20),
    /// );
    ///
    /// // Valid response returns Ok
    /// assert!(response.validate().is_ok());
    /// ```
    pub fn validate(&self) -> Result<(), ResponseValidationError> {
        let mut errors = Vec::new();

        // Validate required fields
        if self.id.trim().is_empty() {
            errors.push(ResponseValidationError::missing_field("id"));
        }

        if self.model.trim().is_empty() {
            errors.push(ResponseValidationError::missing_field("model"));
        }

        // Validate content blocks
        let mut seen_tool_ids: HashSet<String> = HashSet::new();

        for (index, block) in self.content.iter().enumerate() {
            if let Some(err) = self.validate_content_block(block, index, &mut seen_tool_ids) {
                errors.push(err);
            }
        }

        // Validate stop_reason consistency
        if self.stop_reason == Some(StopReason::ToolUse) && !self.has_tool_use() {
            errors.push(ResponseValidationError::invalid_stop_reason(
                "stop_reason is 'tool_use' but no tool_use blocks found",
            ));
        }

        // Note: Usage validation is done separately since Usage fields are u32 (non-negative by type)
        // But we can add sanity checks for very large values if needed

        if errors.is_empty() {
            Ok(())
        } else if errors.len() == 1 {
            Err(errors.remove(0))
        } else {
            Err(ResponseValidationError::multiple(errors))
        }
    }

    /// Validate a single content block.
    fn validate_content_block(
        &self,
        block: &ContentBlock,
        index: usize,
        seen_tool_ids: &mut HashSet<String>,
    ) -> Option<ResponseValidationError> {
        match block {
            ContentBlock::Text { text, .. } => {
                // Text blocks can be empty, no validation needed
                let _ = text;
                None
            }
            ContentBlock::ToolUse {
                id, name, input, ..
            } => {
                // Validate tool_use block
                if id.trim().is_empty() {
                    return Some(ResponseValidationError::invalid_tool_use(
                        id,
                        "id cannot be empty",
                    ));
                }

                if name.trim().is_empty() {
                    return Some(ResponseValidationError::invalid_tool_use(
                        id,
                        "name cannot be empty",
                    ));
                }

                // Check for valid identifier format (alphanumeric + underscores)
                if !name
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
                {
                    return Some(ResponseValidationError::invalid_tool_use(
                        id,
                        format!("name '{}' contains invalid characters", name),
                    ));
                }

                // Check for duplicate IDs
                if seen_tool_ids.contains(id) {
                    return Some(ResponseValidationError::invalid_tool_use(
                        id,
                        "duplicate tool_use id",
                    ));
                }
                seen_tool_ids.insert(id.clone());

                // Validate input is an object
                if !input.is_object() {
                    return Some(ResponseValidationError::invalid_tool_use(
                        id,
                        format!("input must be an object, got {}", json_type_name(input)),
                    ));
                }

                None
            }
            ContentBlock::ToolResult { tool_use_id, .. } => {
                // ToolResult in a response is unusual but not invalid
                if tool_use_id.trim().is_empty() {
                    return Some(ResponseValidationError::malformed_content(
                        index,
                        "tool_result has empty tool_use_id",
                    ));
                }
                None
            }
        }
    }

    /// Validate and return the response, or return an error.
    ///
    /// This is a convenience method that validates and returns self on success.
    pub fn validated(self) -> Result<Self, ResponseValidationError> {
        self.validate()?;
        Ok(self)
    }
}

/// Get a human-readable name for a JSON value type.
fn json_type_name(value: &serde_json::Value) -> &'static str {
    match value {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "boolean",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

/// Why the model stopped generating.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    /// Natural end of response.
    EndTurn,
    /// Model wants to use a tool.
    ToolUse,
    /// Hit max_tokens limit.
    MaxTokens,
    /// Hit a stop sequence.
    StopSequence,
}

/// Token usage statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Usage {
    /// Tokens in the input.
    pub input_tokens: u32,
    /// Tokens in the output.
    pub output_tokens: u32,
    /// Tokens used for cache creation (if applicable).
    #[serde(default)]
    pub cache_creation_input_tokens: u32,
    /// Tokens read from cache (if applicable).
    #[serde(default)]
    pub cache_read_input_tokens: u32,
}

impl Usage {
    /// Create new usage statistics.
    pub fn new(input_tokens: u32, output_tokens: u32) -> Self {
        Self {
            input_tokens,
            output_tokens,
            cache_creation_input_tokens: 0,
            cache_read_input_tokens: 0,
        }
    }

    /// Total tokens used.
    pub fn total(&self) -> u32 {
        self.input_tokens + self.output_tokens
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_user() {
        let msg = Message::user("Hello");
        assert_eq!(msg.role, Role::User);
        assert_eq!(msg.content.as_text(), Some("Hello"));
    }

    #[test]
    fn test_message_assistant() {
        let msg = Message::assistant("Hi there");
        assert_eq!(msg.role, Role::Assistant);
        assert_eq!(msg.content.as_text(), Some("Hi there"));
    }

    #[test]
    fn test_completion_request_builder() {
        let request = CompletionRequest::new(
            "claude-sonnet-4-20250514",
            vec![Message::user("Hello")],
            1024,
        )
        .with_system("You are helpful.")
        .with_streaming()
        .with_temperature(0.7);

        assert_eq!(request.model, "claude-sonnet-4-20250514");
        assert_eq!(request.max_tokens, 1024);
        assert!(request.system.is_some());
        assert!(request.stream);
        assert_eq!(request.temperature, Some(0.7));
    }

    #[test]
    fn test_completion_response_tool_uses() {
        let response = CompletionResponse {
            id: "msg_123".to_string(),
            response_type: "message".to_string(),
            role: Role::Assistant,
            content: vec![
                ContentBlock::Text {
                    text: "Let me help.".to_string(),
                    cache_control: None,
                },
                ContentBlock::ToolUse {
                    id: "tool_1".to_string(),
                    name: "read_file".to_string(),
                    input: serde_json::json!({"path": "/foo.rs"}),
                    cache_control: None,
                },
            ],
            model: "claude-sonnet-4-20250514".to_string(),
            stop_reason: Some(StopReason::ToolUse),
            usage: Usage::new(100, 50),
        };

        assert!(response.has_tool_use());
        let tool_uses = response.tool_uses();
        assert_eq!(tool_uses.len(), 1);
        assert_eq!(tool_uses[0].name, "read_file");
    }

    #[test]
    fn test_tool_result_block() {
        let success = ToolResultBlock::success("tool_1", "file contents here");
        assert!(!success.is_error);
        assert_eq!(
            success.content,
            Some(ToolResultContent::Text("file contents here".to_string()))
        );

        let error = ToolResultBlock::error("tool_2", "file not found");
        assert!(error.is_error);
    }

    #[test]
    fn test_serialize_deserialize_request() {
        let request = CompletionRequest::new(
            "claude-sonnet-4-20250514",
            vec![Message::user("Hello")],
            1024,
        );

        let json = serde_json::to_string(&request).unwrap();
        let parsed: CompletionRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.model, request.model);
        assert_eq!(parsed.max_tokens, request.max_tokens);
    }

    #[test]
    fn test_content_blocks() {
        let text = Content::Text("hello".to_string());
        assert_eq!(text.blocks().len(), 1);

        let blocks = Content::Blocks(vec![
            ContentBlock::Text {
                text: "one".to_string(),
                cache_control: None,
            },
            ContentBlock::Text {
                text: "two".to_string(),
                cache_control: None,
            },
        ]);
        assert_eq!(blocks.to_text(), "onetwo");
    }

    #[test]
    fn test_usage() {
        let usage = Usage::new(100, 50);
        assert_eq!(usage.total(), 150);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Response Validation Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_validate_valid_response() {
        let response = CompletionResponse::new(
            "msg_123",
            "claude-3",
            vec![ContentBlock::text("Hello, world!")],
            StopReason::EndTurn,
            Usage::new(10, 20),
        );

        assert!(response.validate().is_ok());
    }

    #[test]
    fn test_validate_response_with_tool_use() {
        let response = CompletionResponse::new(
            "msg_123",
            "claude-3",
            vec![
                ContentBlock::text("Let me check."),
                ContentBlock::tool_use("tool_1", "read_file", serde_json::json!({"path": "/tmp"})),
            ],
            StopReason::ToolUse,
            Usage::new(50, 30),
        );

        assert!(response.validate().is_ok());
    }

    #[test]
    fn test_validate_empty_id() {
        let response = CompletionResponse {
            id: "".to_string(),
            response_type: "message".to_string(),
            role: Role::Assistant,
            content: vec![ContentBlock::text("Hi")],
            model: "claude-3".to_string(),
            stop_reason: Some(StopReason::EndTurn),
            usage: Usage::new(10, 10),
        };

        let err = response.validate().unwrap_err();
        assert!(err.to_string().contains("id"));
    }

    #[test]
    fn test_validate_empty_model() {
        let response = CompletionResponse {
            id: "msg_123".to_string(),
            response_type: "message".to_string(),
            role: Role::Assistant,
            content: vec![ContentBlock::text("Hi")],
            model: "  ".to_string(), // whitespace only
            stop_reason: Some(StopReason::EndTurn),
            usage: Usage::new(10, 10),
        };

        let err = response.validate().unwrap_err();
        assert!(err.to_string().contains("model"));
    }

    #[test]
    fn test_validate_tool_use_empty_id() {
        let response = CompletionResponse::new(
            "msg_123",
            "claude-3",
            vec![ContentBlock::ToolUse {
                id: "".to_string(),
                name: "read_file".to_string(),
                input: serde_json::json!({}),
                cache_control: None,
            }],
            StopReason::ToolUse,
            Usage::new(10, 10),
        );

        let err = response.validate().unwrap_err();
        assert!(err.to_string().contains("id cannot be empty"));
    }

    #[test]
    fn test_validate_tool_use_empty_name() {
        let response = CompletionResponse::new(
            "msg_123",
            "claude-3",
            vec![ContentBlock::ToolUse {
                id: "tool_1".to_string(),
                name: "".to_string(),
                input: serde_json::json!({}),
                cache_control: None,
            }],
            StopReason::ToolUse,
            Usage::new(10, 10),
        );

        let err = response.validate().unwrap_err();
        assert!(err.to_string().contains("name cannot be empty"));
    }

    #[test]
    fn test_validate_tool_use_invalid_name_chars() {
        let response = CompletionResponse::new(
            "msg_123",
            "claude-3",
            vec![ContentBlock::ToolUse {
                id: "tool_1".to_string(),
                name: "read file".to_string(), // space not allowed
                input: serde_json::json!({}),
                cache_control: None,
            }],
            StopReason::ToolUse,
            Usage::new(10, 10),
        );

        let err = response.validate().unwrap_err();
        assert!(err.to_string().contains("invalid characters"));
    }

    #[test]
    fn test_validate_tool_use_duplicate_ids() {
        let response = CompletionResponse::new(
            "msg_123",
            "claude-3",
            vec![
                ContentBlock::tool_use("tool_1", "read_file", serde_json::json!({"path": "/a"})),
                ContentBlock::tool_use("tool_1", "write_file", serde_json::json!({"path": "/b"})),
            ],
            StopReason::ToolUse,
            Usage::new(10, 10),
        );

        let err = response.validate().unwrap_err();
        assert!(err.to_string().contains("duplicate"));
    }

    #[test]
    fn test_validate_tool_use_input_not_object() {
        let response = CompletionResponse::new(
            "msg_123",
            "claude-3",
            vec![ContentBlock::ToolUse {
                id: "tool_1".to_string(),
                name: "read_file".to_string(),
                input: serde_json::json!("not an object"),
                cache_control: None,
            }],
            StopReason::ToolUse,
            Usage::new(10, 10),
        );

        let err = response.validate().unwrap_err();
        assert!(err.to_string().contains("must be an object"));
    }

    #[test]
    fn test_validate_stop_reason_mismatch() {
        // stop_reason is tool_use but no tool_use blocks
        let response = CompletionResponse::new(
            "msg_123",
            "claude-3",
            vec![ContentBlock::text("No tools here")],
            StopReason::ToolUse,
            Usage::new(10, 10),
        );

        let err = response.validate().unwrap_err();
        assert!(err.to_string().contains("tool_use"));
        assert!(err.to_string().contains("no tool_use blocks"));
    }

    #[test]
    fn test_validate_multiple_errors() {
        let response = CompletionResponse {
            id: "".to_string(), // error 1
            response_type: "message".to_string(),
            role: Role::Assistant,
            content: vec![ContentBlock::ToolUse {
                id: "".to_string(), // error 2
                name: "read_file".to_string(),
                input: serde_json::json!({}),
                cache_control: None,
            }],
            model: "".to_string(), // error 3
            stop_reason: Some(StopReason::ToolUse),
            usage: Usage::new(10, 10),
        };

        let err = response.validate().unwrap_err();
        // Should be a Multiple error containing all issues
        let msg = err.to_string();
        assert!(msg.contains("id") || msg.contains("multiple"));
    }

    #[test]
    fn test_validated_convenience() {
        let response = CompletionResponse::new(
            "msg_123",
            "claude-3",
            vec![ContentBlock::text("Hello")],
            StopReason::EndTurn,
            Usage::new(10, 10),
        );

        let validated = response.validated().unwrap();
        assert_eq!(validated.id, "msg_123");
    }

    #[test]
    fn test_json_type_name() {
        assert_eq!(json_type_name(&serde_json::json!(null)), "null");
        assert_eq!(json_type_name(&serde_json::json!(true)), "boolean");
        assert_eq!(json_type_name(&serde_json::json!(42)), "number");
        assert_eq!(json_type_name(&serde_json::json!("hello")), "string");
        assert_eq!(json_type_name(&serde_json::json!([1, 2])), "array");
        assert_eq!(json_type_name(&serde_json::json!({"a": 1})), "object");
    }
}
