//! Streaming response support for the agent.
//!
//! This module provides streaming capabilities allowing real-time
//! token-by-token output during agent responses.

use futures::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

use arawn_llm::{
    CompletionRequest, ContentDelta, Message, SharedBackend, StreamEvent, ToolResultBlock,
};

use crate::tool::{ToolContext, ToolRegistry, ToolResult};
use crate::types::{AgentConfig, SessionId, ToolCall, ToolResultRecord, TurnId};

// ─────────────────────────────────────────────────────────────────────────────
// Stream Chunk
// ─────────────────────────────────────────────────────────────────────────────

/// A chunk emitted during streaming response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamChunk {
    /// Text content being streamed.
    Text {
        /// The text delta.
        content: String,
    },
    /// A tool is starting execution.
    ToolStart {
        /// Tool call ID.
        id: String,
        /// Name of the tool being called.
        name: String,
    },
    /// Partial output from a tool during execution.
    ToolOutput {
        /// Tool call ID.
        id: String,
        /// The output delta.
        content: String,
    },
    /// A tool has finished execution.
    ToolEnd {
        /// Tool call ID.
        id: String,
        /// Whether the tool succeeded.
        success: bool,
        /// Result content.
        content: String,
    },
    /// Response is complete.
    Done {
        /// Total iterations used.
        iterations: u32,
    },
    /// An error occurred.
    Error {
        /// Error message.
        message: String,
    },
}

impl StreamChunk {
    /// Create a text chunk.
    pub fn text(content: impl Into<String>) -> Self {
        Self::Text {
            content: content.into(),
        }
    }

    /// Create a tool start chunk.
    pub fn tool_start(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self::ToolStart {
            id: id.into(),
            name: name.into(),
        }
    }

    /// Create a tool output chunk (partial output during execution).
    pub fn tool_output(id: impl Into<String>, content: impl Into<String>) -> Self {
        Self::ToolOutput {
            id: id.into(),
            content: content.into(),
        }
    }

    /// Create a tool end chunk.
    pub fn tool_end(id: impl Into<String>, success: bool, content: impl Into<String>) -> Self {
        Self::ToolEnd {
            id: id.into(),
            success,
            content: content.into(),
        }
    }

    /// Create a done chunk.
    pub fn done(iterations: u32) -> Self {
        Self::Done { iterations }
    }

    /// Create an error chunk.
    pub fn error(message: impl Into<String>) -> Self {
        Self::Error {
            message: message.into(),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Agent Response Stream
// ─────────────────────────────────────────────────────────────────────────────

/// A boxed stream of chunks.
pub type AgentStream = Pin<Box<dyn Stream<Item = StreamChunk> + Send + 'static>>;

/// State for streaming agent responses.
struct StreamState {
    backend: SharedBackend,
    tools: Arc<ToolRegistry>,
    config: AgentConfig,
    messages: Vec<Message>,
    session_id: SessionId,
    turn_id: TurnId,
    cancellation: CancellationToken,
    iterations: u32,
    tool_calls: Vec<ToolCall>,
    tool_results: Vec<ToolResultRecord>,
}

/// Create a streaming response for an agent turn.
///
/// This returns a stream that yields chunks as the agent processes the request,
/// including text deltas, tool executions, and completion events.
pub fn create_turn_stream(
    backend: SharedBackend,
    tools: Arc<ToolRegistry>,
    config: AgentConfig,
    messages: Vec<Message>,
    session_id: SessionId,
    turn_id: TurnId,
    cancellation: CancellationToken,
) -> AgentStream {
    let state = StreamState {
        backend,
        tools,
        config,
        messages,
        session_id,
        turn_id,
        cancellation,
        iterations: 0,
        tool_calls: Vec::new(),
        tool_results: Vec::new(),
    };

    Box::pin(async_stream::stream! {
        let mut state = state;
        loop {
            // Check cancellation
            if state.cancellation.is_cancelled() {
                yield StreamChunk::error("Cancelled");
                return;
            }

            state.iterations += 1;

            if state.iterations > state.config.max_iterations {
                yield StreamChunk::error("Max iterations exceeded");
                yield StreamChunk::done(state.iterations);
                return;
            }

            // Build request
            let request = build_stream_request(&state);

            // Start streaming from LLM
            let stream_result = state.backend.complete_stream(request).await;

            let mut llm_stream = match stream_result {
                Ok(s) => s,
                Err(e) => {
                    yield StreamChunk::error(e.to_string());
                    return;
                }
            };

            // Track tool JSON accumulation during streaming
            let mut tool_json_buffer: std::collections::HashMap<usize, String> = std::collections::HashMap::new();

            while let Some(event_result) = llm_stream.next().await {
                // Check cancellation
                if state.cancellation.is_cancelled() {
                    yield StreamChunk::error("Cancelled");
                    return;
                }

                let event = match event_result {
                    Ok(e) => e,
                    Err(e) => {
                        yield StreamChunk::error(e.to_string());
                        return;
                    }
                };

                match event {
                    StreamEvent::ContentBlockDelta { index, delta } => {
                        match delta {
                            ContentDelta::TextDelta(text) => {
                                yield StreamChunk::text(&text);
                            }
                            ContentDelta::InputJsonDelta(json) => {
                                tool_json_buffer
                                    .entry(index)
                                    .or_default()
                                    .push_str(&json);
                            }
                        }
                    }
                    StreamEvent::ContentBlockStart { index: _, content_type } => {
                        if content_type == "tool_use" {
                            // Will be populated by deltas
                        }
                    }
                    StreamEvent::ContentBlockStop { index } => {
                        // If this was a tool_use block, parse the accumulated JSON
                        if let Some(_json_str) = tool_json_buffer.remove(&index) {
                            // We need to extract tool info from the accumulated data
                            // This is simplified - real impl would track tool_use starts
                        }
                    }
                    StreamEvent::MessageDelta { stop_reason: _, .. } => {
                        // Message is finishing
                    }
                    StreamEvent::MessageStop => {
                        break;
                    }
                    StreamEvent::Error { message } => {
                        yield StreamChunk::error(message);
                        return;
                    }
                    _ => {}
                }
            }

            // Get the full response to check for tool calls
            let request = build_sync_request(&state);
            let response = match state.backend.complete(request).await {
                Ok(r) => r,
                Err(e) => {
                    yield StreamChunk::error(e.to_string());
                    return;
                }
            };

            // Check for tool use
            if response.has_tool_use() {
                let ctx = ToolContext::with_cancellation(
                    state.session_id,
                    state.turn_id,
                    state.cancellation.clone(),
                );

                // Execute tools
                for tool_use in response.tool_uses() {
                    let tool_call = ToolCall {
                        id: tool_use.id.clone(),
                        name: tool_use.name.clone(),
                        arguments: tool_use.input.clone(),
                    };
                    state.tool_calls.push(tool_call);

                    yield StreamChunk::tool_start(&tool_use.id, &tool_use.name);

                    let result = match state.tools.execute(&tool_use.name, tool_use.input.clone(), &ctx).await {
                        Ok(r) => r,
                        Err(e) => {
                            tracing::warn!(tool = %tool_use.name, error = %e, "Tool execution failed");
                            ToolResult::error(e.to_string())
                        }
                    };

                    let success = result.is_success();
                    let content = result.to_llm_content();

                    state.tool_results.push(ToolResultRecord {
                        tool_call_id: tool_use.id.clone(),
                        success,
                        content: content.clone(),
                    });

                    yield StreamChunk::tool_end(&tool_use.id, success, &content);
                }

                // Add assistant message with tool calls to history
                state.messages.push(Message::assistant_blocks(response.content.clone()));

                // Add tool results to history
                let tool_result_blocks: Vec<ToolResultBlock> = state.tool_results
                    .iter()
                    .filter(|r| state.tool_calls.iter().any(|tc| tc.id == r.tool_call_id))
                    .map(|r| {
                        if r.success {
                            ToolResultBlock::success(&r.tool_call_id, &r.content)
                        } else {
                            ToolResultBlock::error(&r.tool_call_id, &r.content)
                        }
                    })
                    .collect();

                state.messages.push(Message::tool_results(tool_result_blocks));

                // Continue loop for next LLM call
                continue;
            }

            // No tool use - we're done
            yield StreamChunk::done(state.iterations);
            return;
        }
    })
}

fn build_stream_request(state: &StreamState) -> CompletionRequest {
    let mut request = CompletionRequest::new(
        &state.config.model,
        state.messages.clone(),
        state.config.max_tokens,
    )
    .with_streaming();

    if let Some(ref prompt) = state.config.system_prompt {
        request = request.with_system(prompt);
    }

    if let Some(temp) = state.config.temperature {
        request = request.with_temperature(temp);
    }

    let tool_defs = state.tools.to_llm_definitions();
    if !tool_defs.is_empty() {
        request = request.with_tools(tool_defs);
    }

    request
}

fn build_sync_request(state: &StreamState) -> CompletionRequest {
    let mut request = CompletionRequest::new(
        &state.config.model,
        state.messages.clone(),
        state.config.max_tokens,
    );

    if let Some(ref prompt) = state.config.system_prompt {
        request = request.with_system(prompt);
    }

    if let Some(temp) = state.config.temperature {
        request = request.with_temperature(temp);
    }

    let tool_defs = state.tools.to_llm_definitions();
    if !tool_defs.is_empty() {
        request = request.with_tools(tool_defs);
    }

    request
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_chunk_text() {
        let chunk = StreamChunk::text("hello");
        assert!(matches!(chunk, StreamChunk::Text { content } if content == "hello"));
    }

    #[test]
    fn test_stream_chunk_tool_start() {
        let chunk = StreamChunk::tool_start("call_1", "read_file");
        assert!(matches!(
            chunk,
            StreamChunk::ToolStart { id, name } if id == "call_1" && name == "read_file"
        ));
    }

    #[test]
    fn test_stream_chunk_tool_end() {
        let chunk = StreamChunk::tool_end("call_1", true, "result");
        assert!(matches!(
            chunk,
            StreamChunk::ToolEnd { id, success, content }
            if id == "call_1" && success && content == "result"
        ));
    }

    #[test]
    fn test_stream_chunk_done() {
        let chunk = StreamChunk::done(3);
        assert!(matches!(chunk, StreamChunk::Done { iterations: 3 }));
    }

    #[test]
    fn test_stream_chunk_error() {
        let chunk = StreamChunk::error("something failed");
        assert!(matches!(
            chunk,
            StreamChunk::Error { message } if message == "something failed"
        ));
    }

    #[test]
    fn test_stream_chunk_serialization() {
        let chunk = StreamChunk::text("test");
        let json = serde_json::to_string(&chunk).unwrap();
        assert!(json.contains("text"));
        assert!(json.contains("test"));

        let restored: StreamChunk = serde_json::from_str(&json).unwrap();
        assert!(matches!(restored, StreamChunk::Text { content } if content == "test"));
    }
}
