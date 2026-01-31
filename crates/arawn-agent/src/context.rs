//! Context building for LLM requests.
//!
//! The [`ContextBuilder`] converts session history into LLM completion requests,
//! handling token budget management and message formatting.

use arawn_llm::{CompletionRequest, ContentBlock, Message, ToolResultBlock, ToolResultContent};

use crate::tool::ToolRegistry;
use crate::types::{AgentConfig, Session, Turn};

// ─────────────────────────────────────────────────────────────────────────────
// Context Builder
// ─────────────────────────────────────────────────────────────────────────────

/// Builds LLM completion requests from session context.
///
/// The ContextBuilder handles:
/// - Converting session turns to LLM message format
/// - Managing context window size by truncating old messages
/// - Including system prompts and tool definitions
#[derive(Debug, Clone)]
pub struct ContextBuilder {
    /// Maximum estimated tokens for context (approximate).
    max_context_tokens: usize,
    /// Average characters per token (rough estimate).
    chars_per_token: usize,
    /// System prompt to include.
    system_prompt: Option<String>,
}

impl ContextBuilder {
    /// Create a new context builder with default settings.
    pub fn new() -> Self {
        Self {
            max_context_tokens: 100_000, // Default for Claude
            chars_per_token: 4,          // Rough estimate
            system_prompt: None,
        }
    }

    /// Set the maximum context tokens.
    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_context_tokens = max_tokens;
        self
    }

    /// Set the system prompt.
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    /// Estimate token count for a string (rough approximation).
    fn estimate_tokens(&self, text: &str) -> usize {
        text.len() / self.chars_per_token
    }

    /// Estimate token count for a message.
    fn estimate_message_tokens(&self, message: &Message) -> usize {
        // Base overhead for message structure
        let mut tokens = 10;

        // Add content tokens
        for block in message.content.blocks() {
            tokens += match block {
                ContentBlock::Text { text, .. } => self.estimate_tokens(&text),
                ContentBlock::ToolUse { name, input, .. } => {
                    self.estimate_tokens(&name) + self.estimate_tokens(&input.to_string())
                }
                ContentBlock::ToolResult { content, .. } => {
                    if let Some(c) = content {
                        match c {
                            ToolResultContent::Text(text) => self.estimate_tokens(&text),
                            ToolResultContent::Blocks(blocks) => self.estimate_tokens(
                                &serde_json::to_string(&blocks).unwrap_or_default(),
                            ),
                        }
                    } else {
                        0
                    }
                }
            };
        }

        tokens
    }

    /// Build a completion request from session and user message.
    pub fn build(
        &self,
        session: &Session,
        user_message: &str,
        config: &AgentConfig,
        tools: &ToolRegistry,
    ) -> CompletionRequest {
        let messages = self.build_messages(session, user_message);
        self.build_request(messages, config, tools, session.context_preamble())
    }

    /// Build messages from session history.
    ///
    /// Converts session turns to LLM message format, respecting token budget.
    pub fn build_messages(&self, session: &Session, user_message: &str) -> Vec<Message> {
        let mut messages = Vec::new();
        let mut total_tokens = 0usize;

        // Reserve space for the new user message and response
        let user_msg_tokens = self.estimate_tokens(user_message);
        let reserved_tokens = user_msg_tokens + 4096; // Reserve for response

        // Calculate available tokens for history
        let available_tokens = self.max_context_tokens.saturating_sub(reserved_tokens);

        // Build messages from turns in reverse order (most recent first)
        // Then reverse to get chronological order
        let mut turn_messages: Vec<Vec<Message>> = Vec::new();

        for turn in session.all_turns().iter().rev() {
            let turn_msgs = self.turn_to_messages(turn);
            let turn_tokens: usize = turn_msgs
                .iter()
                .map(|m| self.estimate_message_tokens(m))
                .sum();

            if total_tokens + turn_tokens > available_tokens {
                // Would exceed budget - stop adding older turns
                break;
            }

            total_tokens += turn_tokens;
            turn_messages.push(turn_msgs);
        }

        // Reverse to get chronological order
        turn_messages.reverse();

        // Flatten into messages
        for turn_msgs in turn_messages {
            messages.extend(turn_msgs);
        }

        // Add the new user message
        messages.push(Message::user(user_message));

        messages
    }

    /// Convert a single turn to LLM messages.
    fn turn_to_messages(&self, turn: &Turn) -> Vec<Message> {
        let mut messages = Vec::new();

        // If turn has no response yet (current turn), just return user message
        if turn.assistant_response.is_none() && turn.tool_calls.is_empty() {
            messages.push(Message::user(&turn.user_message));
            return messages;
        }

        // Add user message
        messages.push(Message::user(&turn.user_message));

        // Build assistant content blocks
        let mut assistant_blocks: Vec<ContentBlock> = Vec::new();

        // Add tool calls as ToolUse blocks
        for tc in &turn.tool_calls {
            assistant_blocks.push(ContentBlock::ToolUse {
                id: tc.id.clone(),
                name: tc.name.clone(),
                input: tc.arguments.clone(),
                cache_control: None,
            });
        }

        // Add final text response if present
        if let Some(ref response) = turn.assistant_response {
            if !response.is_empty() {
                assistant_blocks.push(ContentBlock::Text {
                    text: response.clone(),
                    cache_control: None,
                });
            }
        }

        if !assistant_blocks.is_empty() {
            messages.push(Message::assistant_blocks(assistant_blocks));
        }

        // Add tool results
        if !turn.tool_results.is_empty() {
            let result_blocks: Vec<ToolResultBlock> = turn
                .tool_results
                .iter()
                .map(|r| {
                    if r.success {
                        ToolResultBlock::success(&r.tool_call_id, &r.content)
                    } else {
                        ToolResultBlock::error(&r.tool_call_id, &r.content)
                    }
                })
                .collect();
            messages.push(Message::tool_results(result_blocks));
        }

        messages
    }

    /// Build a completion request from messages.
    fn build_request(
        &self,
        messages: Vec<Message>,
        config: &AgentConfig,
        tools: &ToolRegistry,
        context_preamble: Option<&str>,
    ) -> CompletionRequest {
        let mut request = CompletionRequest::new(&config.model, messages, config.max_tokens);

        // Build system prompt with optional context preamble
        let base_prompt = config
            .system_prompt
            .as_ref()
            .or(self.system_prompt.as_ref());
        let system_prompt = match (base_prompt, context_preamble) {
            (Some(prompt), Some(preamble)) => Some(format!(
                "[Session Context]\n{}\n\n---\n\n{}",
                preamble, prompt
            )),
            (Some(prompt), None) => Some(prompt.clone()),
            (None, Some(preamble)) => Some(format!("[Session Context]\n{}", preamble)),
            (None, None) => None,
        };

        if let Some(ref prompt) = system_prompt {
            request = request.with_system(prompt);
        }

        // Add temperature
        if let Some(temp) = config.temperature {
            request = request.with_temperature(temp);
        }

        // Add tools
        let tool_defs = tools.to_llm_definitions();
        if !tool_defs.is_empty() {
            request = request.with_tools(tool_defs);
        }

        request
    }

    /// Get message count for a session (for diagnostics).
    pub fn count_messages(&self, session: &Session) -> usize {
        session
            .all_turns()
            .iter()
            .map(|t| self.turn_to_messages(t).len())
            .sum()
    }

    /// Estimate total tokens for a session (for diagnostics).
    pub fn estimate_session_tokens(&self, session: &Session) -> usize {
        session
            .all_turns()
            .iter()
            .flat_map(|t| self.turn_to_messages(t))
            .map(|m| self.estimate_message_tokens(&m))
            .sum()
    }
}

impl Default for ContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ToolCall, ToolResultRecord};

    #[test]
    fn test_context_builder_default() {
        let builder = ContextBuilder::new();
        assert_eq!(builder.max_context_tokens, 100_000);
        assert!(builder.system_prompt.is_none());
    }

    #[test]
    fn test_context_builder_config() {
        let builder = ContextBuilder::new()
            .with_max_tokens(50_000)
            .with_system_prompt("You are helpful.");

        assert_eq!(builder.max_context_tokens, 50_000);
        assert_eq!(builder.system_prompt, Some("You are helpful.".to_string()));
    }

    #[test]
    fn test_build_messages_empty_session() {
        let builder = ContextBuilder::new();
        let session = Session::new();

        let messages = builder.build_messages(&session, "Hello");

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content.as_text(), Some("Hello"));
    }

    #[test]
    fn test_build_messages_with_history() {
        let builder = ContextBuilder::new();
        let mut session = Session::new();

        // Add a completed turn
        let turn = session.start_turn("First message");
        turn.complete("First response");

        let messages = builder.build_messages(&session, "Second message");

        // Should have: user1, assistant1, user2
        assert_eq!(messages.len(), 3);
    }

    #[test]
    fn test_build_messages_with_tool_calls() {
        let builder = ContextBuilder::new();
        let mut session = Session::new();

        // Add a turn with tool calls
        let turn = session.start_turn("Use a tool");
        turn.add_tool_call(ToolCall {
            id: "call_1".to_string(),
            name: "test_tool".to_string(),
            arguments: serde_json::json!({"arg": "value"}),
        });
        turn.add_tool_result(ToolResultRecord {
            tool_call_id: "call_1".to_string(),
            success: true,
            content: "tool output".to_string(),
        });
        turn.complete("Done using tool");

        let messages = builder.build_messages(&session, "Next");

        // Should have: user, assistant (with tool use), tool results, user (new)
        assert_eq!(messages.len(), 4);
    }

    #[test]
    fn test_build_messages_truncation() {
        // Create builder with very small token budget
        let builder = ContextBuilder::new().with_max_tokens(100);

        let mut session = Session::new();

        // Add many turns
        for i in 0..10 {
            let turn = session.start_turn(format!("Message {}", i));
            turn.complete(format!(
                "Response {} with some extra text to take up tokens",
                i
            ));
        }

        let messages = builder.build_messages(&session, "New message");

        // Should have fewer messages than total due to truncation
        // At minimum: some history + new user message
        assert!(messages.len() < 21); // Less than 10*2 + 1
        assert!(messages.len() >= 1); // At least the new message

        // Last message should be the new user message
        assert_eq!(
            messages.last().unwrap().content.as_text(),
            Some("New message")
        );
    }

    #[test]
    fn test_build_request_with_tools() {
        let builder = ContextBuilder::new();
        let session = Session::new();
        let config = AgentConfig::new("test-model")
            .with_system_prompt("Be helpful")
            .with_temperature(0.7);

        let mut tools = ToolRegistry::new();
        tools.register(crate::tool::MockTool::new("my_tool"));

        let request = builder.build(&session, "Hello", &config, &tools);

        assert_eq!(request.model, "test-model");
        assert_eq!(request.temperature, Some(0.7));
        assert!(request.system.is_some());
        assert_eq!(request.tools.len(), 1);
    }

    #[test]
    fn test_estimate_tokens() {
        let builder = ContextBuilder::new();

        // With default 4 chars per token
        assert_eq!(builder.estimate_tokens("hello"), 1); // 5 chars = 1 token
        assert_eq!(builder.estimate_tokens("hello world test"), 4); // 16 chars = 4 tokens
    }

    #[test]
    fn test_count_messages() {
        let builder = ContextBuilder::new();
        let mut session = Session::new();

        assert_eq!(builder.count_messages(&session), 0);

        let turn = session.start_turn("Hello");
        turn.complete("Hi!");

        // One turn = user + assistant = 2 messages
        assert_eq!(builder.count_messages(&session), 2);
    }

    #[test]
    fn test_estimate_session_tokens() {
        let builder = ContextBuilder::new();
        let mut session = Session::new();

        let turn = session.start_turn("Hello");
        turn.complete("Hi there!");

        let tokens = builder.estimate_session_tokens(&session);
        assert!(tokens > 0);
    }
}
