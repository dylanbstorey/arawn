//! Chat service for conversation orchestration.
//!
//! The chat service coordinates agent execution with session management
//! and workstream persistence.

use std::sync::Arc;

use arawn_agent::{Agent, AgentResponse, Session, SessionId, SessionIndexer};
use arawn_workstream::{DirectoryManager, WorkstreamManager};
use tracing::{debug, info, warn};

use crate::error::{DomainError, Result};

/// Response from a chat turn.
#[derive(Debug, Clone)]
pub struct ChatResponse {
    /// The session ID.
    pub session_id: SessionId,
    /// The agent's response text.
    pub response: String,
    /// Whether the response was truncated (hit max turns).
    pub truncated: bool,
    /// Input tokens used.
    pub input_tokens: u32,
    /// Output tokens generated.
    pub output_tokens: u32,
    /// Tool calls made during the turn.
    pub tool_calls: Vec<ToolCallSummary>,
}

/// Summary of a tool call.
#[derive(Debug, Clone)]
pub struct ToolCallSummary {
    /// Tool call ID.
    pub id: String,
    /// Tool name.
    pub name: String,
    /// Whether the call succeeded (based on tool_results).
    pub success: bool,
}

/// Options for executing a turn.
#[derive(Debug, Clone, Default)]
pub struct TurnOptions {
    /// Maximum message size in bytes.
    pub max_message_bytes: Option<usize>,
}

/// Chat service for conversation orchestration.
#[derive(Clone)]
pub struct ChatService {
    agent: Arc<Agent>,
    workstreams: Option<Arc<WorkstreamManager>>,
    directory_manager: Option<Arc<DirectoryManager>>,
    indexer: Option<Arc<SessionIndexer>>,
}

impl ChatService {
    /// Create a new chat service.
    pub fn new(
        agent: Arc<Agent>,
        workstreams: Option<Arc<WorkstreamManager>>,
        directory_manager: Option<Arc<DirectoryManager>>,
        indexer: Option<Arc<SessionIndexer>>,
    ) -> Self {
        Self {
            agent,
            workstreams,
            directory_manager,
            indexer,
        }
    }

    /// Get the underlying agent.
    pub fn agent(&self) -> &Arc<Agent> {
        &self.agent
    }

    /// Get the workstream manager.
    pub fn workstreams(&self) -> Option<&Arc<WorkstreamManager>> {
        self.workstreams.as_ref()
    }

    /// Get the directory manager.
    pub fn directory_manager(&self) -> Option<&Arc<DirectoryManager>> {
        self.directory_manager.as_ref()
    }

    /// Get the session indexer.
    pub fn indexer(&self) -> Option<&Arc<SessionIndexer>> {
        self.indexer.as_ref()
    }

    /// Execute a chat turn with an existing session.
    ///
    /// This is the core chat operation that:
    /// 1. Executes the agent turn
    /// 2. Returns the response
    ///
    /// Note: Session persistence is handled separately via the workstream manager.
    pub async fn turn(&self, session: &mut Session, message: &str) -> Result<ChatResponse> {
        let session_id = session.id;

        debug!(session_id = %session_id, message_len = message.len(), "Executing chat turn");

        // Execute the agent turn
        let response = self.agent.turn(session, message).await?;

        // Build response
        let chat_response = self.build_response(session_id, &response);

        debug!(
            session_id = %session_id,
            response_len = chat_response.response.len(),
            tool_calls = chat_response.tool_calls.len(),
            "Chat turn completed"
        );

        Ok(chat_response)
    }

    /// Create a scratch session directory.
    pub fn create_scratch_session(&self, session_id: &str) -> Result<()> {
        if let Some(ref dm) = self.directory_manager {
            dm.create_scratch_session(session_id)
                .map_err(|e| DomainError::Internal(e.to_string()))?;
        }
        Ok(())
    }

    /// Get allowed paths for a session.
    pub fn allowed_paths(
        &self,
        workstream_id: &str,
        session_id: &str,
    ) -> Option<Vec<std::path::PathBuf>> {
        self.directory_manager
            .as_ref()
            .map(|dm| dm.allowed_paths(workstream_id, session_id))
    }

    /// Index a closed session for memory search.
    pub async fn index_session(&self, session_id: &str, session: &Session) {
        if let Some(ref indexer) = self.indexer
            && !session.is_empty()
        {
            let messages = session_to_messages(session);
            let refs = messages_as_refs(&messages);

            let report = indexer.index_session(session_id, &refs).await;
            info!(
                session_id = session_id,
                report = %report,
                "Session indexed"
            );
            if report.has_errors() {
                warn!(
                    session_id = session_id,
                    errors = ?report.errors,
                    "Session indexing completed with errors"
                );
            }
        }
    }

    /// Build a ChatResponse from an AgentResponse.
    fn build_response(&self, session_id: SessionId, response: &AgentResponse) -> ChatResponse {
        // Build tool call summaries from tool_calls and tool_results
        let tool_results_success: std::collections::HashMap<String, bool> = response
            .tool_results
            .iter()
            .map(|tr| (tr.tool_call_id.clone(), tr.success))
            .collect();

        let tool_calls: Vec<ToolCallSummary> = response
            .tool_calls
            .iter()
            .map(|tc| ToolCallSummary {
                id: tc.id.clone(),
                name: tc.name.clone(),
                success: tool_results_success.get(&tc.id).copied().unwrap_or(false),
            })
            .collect();

        ChatResponse {
            session_id,
            response: response.text.clone(),
            truncated: response.truncated,
            input_tokens: response.usage.input_tokens,
            output_tokens: response.usage.output_tokens,
            tool_calls,
        }
    }
}

/// Convert a session's turns into owned `(role, content)` pairs.
fn session_to_messages(session: &Session) -> Vec<(String, String)> {
    let mut messages = Vec::new();
    for turn in session.all_turns() {
        messages.push(("user".to_string(), turn.user_message.clone()));
        if let Some(ref response) = turn.assistant_response {
            messages.push(("assistant".to_string(), response.clone()));
        }
    }
    messages
}

/// Convert owned message pairs to borrowed slices for the indexer API.
fn messages_as_refs(messages: &[(String, String)]) -> Vec<(&str, &str)> {
    messages
        .iter()
        .map(|(r, c)| (r.as_str(), c.as_str()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use arawn_agent::ToolRegistry;
    use arawn_llm::MockBackend;

    fn create_test_agent() -> Arc<Agent> {
        let backend = MockBackend::with_text("Hello!");
        Arc::new(
            Agent::builder()
                .with_backend(backend)
                .with_tools(ToolRegistry::new())
                .build()
                .expect("failed to create test agent"),
        )
    }

    #[tokio::test]
    async fn test_chat_turn() {
        let agent = create_test_agent();
        let chat = ChatService::new(agent, None, None, None);

        let mut session = Session::new();
        let response = chat.turn(&mut session, "Hello").await.unwrap();

        assert_eq!(response.session_id, session.id);
        assert!(!response.response.is_empty());
    }

    #[test]
    fn test_session_to_messages() {
        let mut session = Session::new();
        let turn = session.start_turn("Hello");
        turn.complete("Hi there!");

        let messages = session_to_messages(&session);
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0], ("user".to_string(), "Hello".to_string()));
        assert_eq!(
            messages[1],
            ("assistant".to_string(), "Hi there!".to_string())
        );
    }
}
