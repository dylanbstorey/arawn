//! Context building for LLM requests.
//!
//! The [`ContextBuilder`] converts session history into LLM completion requests,
//! handling token budget management and message formatting.

use arawn_llm::{CompletionRequest, ContentBlock, Message, ToolResultBlock, ToolResultContent};

// ─────────────────────────────────────────────────────────────────────────────
// Token Estimation Utilities
// ─────────────────────────────────────────────────────────────────────────────

/// Default characters per token ratio (rough estimate for English text).
const CHARS_PER_TOKEN: usize = 4;

/// Estimate token count for a string (rough approximation).
///
/// Uses a simple heuristic of ~4 characters per token, which is
/// reasonable for English text with the Claude/GPT tokenizers.
pub fn estimate_tokens(text: &str) -> usize {
    text.len() / CHARS_PER_TOKEN
}

/// Estimate tokens for a byte count.
pub fn estimate_tokens_from_bytes(bytes: usize) -> usize {
    bytes / CHARS_PER_TOKEN
}

use crate::tool::ToolRegistry;
use crate::types::{AgentConfig, Session, Turn};

// ─────────────────────────────────────────────────────────────────────────────
// Context Tracker
// ─────────────────────────────────────────────────────────────────────────────

/// Status of context usage relative to thresholds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextStatus {
    /// Usage is below warning threshold.
    Ok {
        /// Current token count.
        current: usize,
        /// Maximum token capacity.
        max: usize,
    },
    /// Usage is between warning and critical thresholds.
    Warning {
        /// Current token count.
        current: usize,
        /// Maximum token capacity.
        max: usize,
    },
    /// Usage exceeds critical threshold.
    Critical {
        /// Current token count.
        current: usize,
        /// Maximum token capacity.
        max: usize,
    },
}

impl ContextStatus {
    /// Returns true if status is Ok.
    pub fn is_ok(&self) -> bool {
        matches!(self, Self::Ok { .. })
    }

    /// Returns true if status is Warning or Critical.
    pub fn is_warning(&self) -> bool {
        matches!(self, Self::Warning { .. } | Self::Critical { .. })
    }

    /// Returns true if status is Critical.
    pub fn is_critical(&self) -> bool {
        matches!(self, Self::Critical { .. })
    }

    /// Get current token count.
    pub fn current(&self) -> usize {
        match self {
            Self::Ok { current, .. }
            | Self::Warning { current, .. }
            | Self::Critical { current, .. } => *current,
        }
    }

    /// Get maximum token capacity.
    pub fn max(&self) -> usize {
        match self {
            Self::Ok { max, .. } | Self::Warning { max, .. } | Self::Critical { max, .. } => *max,
        }
    }

    /// Get usage as percentage (0.0 - 1.0).
    pub fn percent(&self) -> f32 {
        let max = self.max();
        if max == 0 {
            return 0.0;
        }
        (self.current() as f32) / (max as f32)
    }

    /// Get remaining tokens.
    pub fn remaining(&self) -> usize {
        self.max().saturating_sub(self.current())
    }
}

/// Tracks token usage for a session with configurable thresholds.
///
/// ContextTracker monitors context window usage and reports status based on
/// warning and critical thresholds. This enables proactive context management
/// such as triggering compaction before hitting hard limits.
#[derive(Debug, Clone)]
pub struct ContextTracker {
    /// Maximum tokens available for context.
    max_tokens: usize,
    /// Current estimated token usage.
    current_tokens: usize,
    /// Threshold for warning status (0.0 - 1.0).
    warning_threshold: f32,
    /// Threshold for critical status (0.0 - 1.0).
    critical_threshold: f32,
}

impl ContextTracker {
    /// Default warning threshold (70% of max).
    pub const DEFAULT_WARNING_THRESHOLD: f32 = 0.7;
    /// Default critical threshold (90% of max).
    pub const DEFAULT_CRITICAL_THRESHOLD: f32 = 0.9;

    /// Create a new context tracker for a model with the given max tokens.
    pub fn for_model(max_tokens: usize) -> Self {
        Self {
            max_tokens,
            current_tokens: 0,
            warning_threshold: Self::DEFAULT_WARNING_THRESHOLD,
            critical_threshold: Self::DEFAULT_CRITICAL_THRESHOLD,
        }
    }

    /// Set custom warning threshold (0.0 - 1.0).
    pub fn with_warning_threshold(mut self, threshold: f32) -> Self {
        self.warning_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Set custom critical threshold (0.0 - 1.0).
    pub fn with_critical_threshold(mut self, threshold: f32) -> Self {
        self.critical_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Update the current token count.
    pub fn update(&mut self, token_count: usize) {
        self.current_tokens = token_count;
    }

    /// Add tokens to the current count.
    pub fn add(&mut self, tokens: usize) {
        self.current_tokens = self.current_tokens.saturating_add(tokens);
    }

    /// Get the current context status based on thresholds.
    pub fn status(&self) -> ContextStatus {
        let percent = self.usage_percent();
        let current = self.current_tokens;
        let max = self.max_tokens;

        if percent >= self.critical_threshold {
            ContextStatus::Critical { current, max }
        } else if percent >= self.warning_threshold {
            ContextStatus::Warning { current, max }
        } else {
            ContextStatus::Ok { current, max }
        }
    }

    /// Get current usage as a percentage (0.0 - 1.0).
    pub fn usage_percent(&self) -> f32 {
        if self.max_tokens == 0 {
            return 0.0;
        }
        (self.current_tokens as f32) / (self.max_tokens as f32)
    }

    /// Returns true if compaction should be triggered (critical threshold exceeded).
    pub fn should_compact(&self) -> bool {
        self.status().is_critical()
    }

    /// Get current token count.
    pub fn current_tokens(&self) -> usize {
        self.current_tokens
    }

    /// Get maximum tokens.
    pub fn max_tokens(&self) -> usize {
        self.max_tokens
    }

    /// Get remaining tokens before hitting max.
    pub fn remaining_tokens(&self) -> usize {
        self.max_tokens.saturating_sub(self.current_tokens)
    }

    /// Reset the tracker to zero usage.
    pub fn reset(&mut self) {
        self.current_tokens = 0;
    }
}

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

    // ─────────────────────────────────────────────────────────────────────────
    // ContextTracker Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_context_tracker_for_model() {
        let tracker = ContextTracker::for_model(100_000);
        assert_eq!(tracker.max_tokens(), 100_000);
        assert_eq!(tracker.current_tokens(), 0);
        assert_eq!(tracker.warning_threshold, 0.7);
        assert_eq!(tracker.critical_threshold, 0.9);
    }

    #[test]
    fn test_context_tracker_custom_thresholds() {
        let tracker = ContextTracker::for_model(100_000)
            .with_warning_threshold(0.6)
            .with_critical_threshold(0.8);

        assert_eq!(tracker.warning_threshold, 0.6);
        assert_eq!(tracker.critical_threshold, 0.8);
    }

    #[test]
    fn test_context_tracker_threshold_clamping() {
        let tracker = ContextTracker::for_model(100_000)
            .with_warning_threshold(1.5)
            .with_critical_threshold(-0.5);

        assert_eq!(tracker.warning_threshold, 1.0);
        assert_eq!(tracker.critical_threshold, 0.0);
    }

    #[test]
    fn test_context_tracker_update() {
        let mut tracker = ContextTracker::for_model(100_000);
        assert_eq!(tracker.current_tokens(), 0);

        tracker.update(50_000);
        assert_eq!(tracker.current_tokens(), 50_000);

        tracker.update(75_000);
        assert_eq!(tracker.current_tokens(), 75_000);
    }

    #[test]
    fn test_context_tracker_add() {
        let mut tracker = ContextTracker::for_model(100_000);
        tracker.update(10_000);

        tracker.add(5_000);
        assert_eq!(tracker.current_tokens(), 15_000);

        tracker.add(5_000);
        assert_eq!(tracker.current_tokens(), 20_000);
    }

    #[test]
    fn test_context_tracker_usage_percent() {
        let mut tracker = ContextTracker::for_model(100_000);

        tracker.update(0);
        assert_eq!(tracker.usage_percent(), 0.0);

        tracker.update(50_000);
        assert_eq!(tracker.usage_percent(), 0.5);

        tracker.update(100_000);
        assert_eq!(tracker.usage_percent(), 1.0);
    }

    #[test]
    fn test_context_tracker_usage_percent_zero_max() {
        let tracker = ContextTracker::for_model(0);
        assert_eq!(tracker.usage_percent(), 0.0);
    }

    #[test]
    fn test_context_tracker_status_ok() {
        let mut tracker = ContextTracker::for_model(100_000);
        tracker.update(60_000); // 60% - below warning threshold

        let status = tracker.status();
        assert!(matches!(
            status,
            ContextStatus::Ok {
                current: 60_000,
                max: 100_000
            }
        ));
        assert!(status.is_ok());
        assert!(!status.is_warning());
        assert!(!status.is_critical());
        assert_eq!(status.current(), 60_000);
        assert_eq!(status.max(), 100_000);
        assert!((status.percent() - 0.6).abs() < 0.001);
    }

    #[test]
    fn test_context_tracker_status_warning() {
        let mut tracker = ContextTracker::for_model(100_000);
        tracker.update(75_000); // 75% - between warning (70%) and critical (90%)

        let status = tracker.status();
        assert!(matches!(
            status,
            ContextStatus::Warning {
                current: 75_000,
                max: 100_000
            }
        ));
        assert!(!status.is_ok());
        assert!(status.is_warning());
        assert!(!status.is_critical());
        assert_eq!(status.current(), 75_000);
        assert_eq!(status.max(), 100_000);
        assert!((status.percent() - 0.75).abs() < 0.001);
    }

    #[test]
    fn test_context_tracker_status_critical() {
        let mut tracker = ContextTracker::for_model(100_000);
        tracker.update(95_000); // 95% - above critical threshold (90%)

        let status = tracker.status();
        assert!(matches!(
            status,
            ContextStatus::Critical {
                current: 95_000,
                max: 100_000
            }
        ));
        assert!(!status.is_ok());
        assert!(status.is_warning()); // Critical is also considered a warning
        assert!(status.is_critical());
        assert_eq!(status.current(), 95_000);
        assert_eq!(status.max(), 100_000);
        assert!((status.percent() - 0.95).abs() < 0.001);
    }

    #[test]
    fn test_context_tracker_should_compact() {
        let mut tracker = ContextTracker::for_model(100_000);

        tracker.update(60_000);
        assert!(!tracker.should_compact());

        tracker.update(75_000);
        assert!(!tracker.should_compact());

        tracker.update(95_000);
        assert!(tracker.should_compact());
    }

    #[test]
    fn test_context_tracker_remaining_tokens() {
        let mut tracker = ContextTracker::for_model(100_000);

        assert_eq!(tracker.remaining_tokens(), 100_000);

        tracker.update(40_000);
        assert_eq!(tracker.remaining_tokens(), 60_000);

        tracker.update(100_000);
        assert_eq!(tracker.remaining_tokens(), 0);
    }

    #[test]
    fn test_context_tracker_reset() {
        let mut tracker = ContextTracker::for_model(100_000);
        tracker.update(50_000);
        assert_eq!(tracker.current_tokens(), 50_000);

        tracker.reset();
        assert_eq!(tracker.current_tokens(), 0);
        assert!(tracker.status().is_ok());
        assert_eq!(tracker.status().current(), 0);
    }

    #[test]
    fn test_context_status_at_exact_thresholds() {
        let mut tracker = ContextTracker::for_model(100_000);

        // Exactly at warning threshold
        tracker.update(70_000);
        assert!(matches!(tracker.status(), ContextStatus::Warning { .. }));

        // Exactly at critical threshold
        tracker.update(90_000);
        assert!(matches!(tracker.status(), ContextStatus::Critical { .. }));
    }

    #[test]
    fn test_context_status_remaining() {
        let mut tracker = ContextTracker::for_model(100_000);
        tracker.update(75_000);

        let status = tracker.status();
        assert_eq!(status.remaining(), 25_000);
    }

    #[test]
    fn test_context_status_percent_zero_max() {
        // Edge case: zero max tokens
        let status = ContextStatus::Ok { current: 0, max: 0 };
        assert_eq!(status.percent(), 0.0);
    }
}
