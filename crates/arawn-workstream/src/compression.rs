use arawn_llm::{CompletionRequest, Message, SharedBackend};

use crate::manager::WorkstreamManager;
use crate::store::Session;
use crate::types::WorkstreamMessage;
use crate::{Result, WorkstreamError};

/// Prompts used for compression.
const SESSION_SUMMARY_PROMPT: &str = "\
Summarize this conversation session concisely. Focus on:
- What was discussed and decided
- Action items or outcomes
- Current state of any work in progress
- Open questions or blockers

Provide a clear, factual summary in 2-4 paragraphs. Do not use bullet points.";

const WORKSTREAM_REDUCE_PROMPT: &str = "\
Given these session summaries for a workstream, produce a unified summary covering:
- The workstream's purpose and objective
- Current progress and state
- Key decisions made across sessions
- Open items and next steps

Provide a clear, factual summary in 2-4 paragraphs. Do not repeat information across sections.";

/// Configuration for the compressor.
#[derive(Debug, Clone)]
pub struct CompressorConfig {
    /// Model to use for summarization calls.
    pub model: String,
    /// Max tokens for summary generation.
    pub max_summary_tokens: u32,
    /// Token threshold (in estimated chars) that triggers mid-session compression.
    pub token_threshold_chars: usize,
}

impl Default for CompressorConfig {
    fn default() -> Self {
        Self {
            model: "claude-sonnet".to_string(),
            max_summary_tokens: 1024,
            // ~8k tokens ≈ 32k chars
            token_threshold_chars: 32_000,
        }
    }
}

/// Map-reduce context compressor.
///
/// - **Map**: Summarize individual sessions.
/// - **Reduce**: Merge session summaries into a workstream summary.
pub struct Compressor {
    backend: SharedBackend,
    config: CompressorConfig,
}

impl Compressor {
    pub fn new(backend: SharedBackend, config: CompressorConfig) -> Self {
        Self { backend, config }
    }

    /// Compress a single session's messages into a summary.
    ///
    /// Reads messages from JSONL for the session's time range,
    /// sends them to the LLM, and stores the summary in SQLite.
    pub async fn compress_session(
        &self,
        manager: &WorkstreamManager,
        session_id: &str,
    ) -> Result<String> {
        let session = manager.store().get_session(session_id)?;

        if session.ended_at.is_none() {
            return Err(WorkstreamError::Migration(
                "Cannot compress an active session".to_string(),
            ));
        }

        let messages = manager.get_messages(&session.workstream_id)?;
        let session_messages = filter_session_messages(&messages, &session);

        if session_messages.is_empty() {
            return Ok("Empty session.".to_string());
        }

        let owned: Vec<_> = session_messages.into_iter().cloned().collect();
        let summary = self.summarize(&owned, SESSION_SUMMARY_PROMPT).await?;

        // Store summary in SQLite
        manager
            .store()
            .update_session_summary(session_id, &summary)?;

        Ok(summary)
    }

    /// Reduce all session summaries for a workstream into a single workstream summary.
    ///
    /// Reads session summaries from SQLite, sends them to the LLM,
    /// and stores the result as the workstream summary.
    pub async fn compress_workstream(
        &self,
        manager: &WorkstreamManager,
        workstream_id: &str,
    ) -> Result<String> {
        let sessions = manager.list_sessions(workstream_id)?;

        let summaries: Vec<String> = sessions.iter().filter_map(|s| s.summary.clone()).collect();

        if summaries.is_empty() {
            return Ok("No session summaries available.".to_string());
        }

        // Build a synthetic conversation with session summaries
        let combined = summaries
            .iter()
            .enumerate()
            .map(|(i, s)| format!("Session {}:\n{}", i + 1, s))
            .collect::<Vec<_>>()
            .join("\n\n---\n\n");

        let fake_messages = vec![WorkstreamMessage {
            id: String::new(),
            workstream_id: workstream_id.to_string(),
            session_id: None,
            role: crate::types::MessageRole::User,
            content: combined,
            timestamp: chrono::Utc::now(),
            metadata: None,
        }];

        let summary = self
            .summarize(&fake_messages, WORKSTREAM_REDUCE_PROMPT)
            .await?;

        // Store as workstream summary
        manager
            .store()
            .update_workstream(workstream_id, None, Some(&summary), None, None)?;

        Ok(summary)
    }

    /// Check if a workstream's current session exceeds the token threshold.
    ///
    /// Note: Uses character count as a rough approximation for tokens.
    /// Actual token count varies by model (~4 chars/token for English text),
    /// but character count provides a fast, conservative estimate without
    /// requiring tokenization. The threshold is set accordingly (~8k tokens ≈ 32k chars).
    pub fn needs_compression(&self, messages: &[WorkstreamMessage]) -> bool {
        let total_chars: usize = messages.iter().map(|m| m.content.len()).sum();
        total_chars > self.config.token_threshold_chars
    }

    /// Send messages to LLM with a system prompt for summarization.
    async fn summarize(
        &self,
        messages: &[WorkstreamMessage],
        system_prompt: &str,
    ) -> Result<String> {
        // Format messages as a conversation transcript
        let transcript = messages
            .iter()
            .map(|m| format!("[{}] {}", m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n\n");

        let request = CompletionRequest::new(
            &self.config.model,
            vec![Message::user(transcript)],
            self.config.max_summary_tokens,
        )
        .with_system(system_prompt);

        let response = self
            .backend
            .complete(request)
            .await
            .map_err(|e| WorkstreamError::Migration(format!("LLM compression failed: {e}")))?;

        Ok(response.text())
    }
}

/// Filter messages that belong to a specific session's time range.
fn filter_session_messages<'a>(
    messages: &'a [WorkstreamMessage],
    session: &Session,
) -> Vec<&'a WorkstreamMessage> {
    let start = session.started_at;
    let end = session.ended_at.unwrap_or_else(chrono::Utc::now);

    messages
        .iter()
        .filter(|m| m.timestamp >= start && m.timestamp <= end)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message_store::MessageStore;
    use crate::store::WorkstreamStore;
    use crate::types::MessageRole;

    fn test_manager() -> (tempfile::TempDir, WorkstreamManager) {
        let dir = tempfile::tempdir().unwrap();
        let store = WorkstreamStore::open_in_memory().unwrap();
        let msg_store = MessageStore::new(dir.path());
        let mgr = WorkstreamManager::from_parts(store, msg_store, 30);
        (dir, mgr)
    }

    #[test]
    fn test_needs_compression_below_threshold() {
        let backend = arawn_llm::MockBackend::with_text("summary");
        let compressor = Compressor::new(
            std::sync::Arc::new(backend),
            CompressorConfig {
                token_threshold_chars: 100,
                ..Default::default()
            },
        );

        let messages = vec![WorkstreamMessage {
            id: "1".into(),
            workstream_id: "ws".into(),
            session_id: None,
            role: MessageRole::User,
            content: "short".into(),
            timestamp: chrono::Utc::now(),
            metadata: None,
        }];

        assert!(!compressor.needs_compression(&messages));
    }

    #[test]
    fn test_needs_compression_above_threshold() {
        let backend = arawn_llm::MockBackend::with_text("summary");
        let compressor = Compressor::new(
            std::sync::Arc::new(backend),
            CompressorConfig {
                token_threshold_chars: 10,
                ..Default::default()
            },
        );

        let messages = vec![WorkstreamMessage {
            id: "1".into(),
            workstream_id: "ws".into(),
            session_id: None,
            role: MessageRole::User,
            content: "this is a longer message that exceeds threshold".into(),
            timestamp: chrono::Utc::now(),
            metadata: None,
        }];

        assert!(compressor.needs_compression(&messages));
    }

    #[tokio::test]
    async fn test_compress_session() {
        let (_dir, mgr) = test_manager();
        let ws = mgr.create_workstream("Test", None, &[]).unwrap();

        // Send messages and end the session
        mgr.send_message(Some(&ws.id), None, MessageRole::User, "What is Rust?", None)
            .unwrap();
        mgr.send_message(
            Some(&ws.id),
            None,
            MessageRole::Assistant,
            "Rust is a systems programming language.",
            None,
        )
        .unwrap();

        let session = mgr.get_active_session(&ws.id).unwrap().unwrap();
        mgr.end_session(&session.id).unwrap();

        // Compress
        let backend = arawn_llm::MockBackend::with_text("Discussed Rust programming language.");
        let compressor = Compressor::new(std::sync::Arc::new(backend), CompressorConfig::default());

        let summary = compressor
            .compress_session(&mgr, &session.id)
            .await
            .unwrap();
        assert_eq!(summary, "Discussed Rust programming language.");

        // Verify stored in SQLite
        let updated_session = mgr.store().get_session(&session.id).unwrap();
        assert_eq!(
            updated_session.summary.as_deref(),
            Some("Discussed Rust programming language.")
        );
    }

    #[tokio::test]
    async fn test_compress_workstream_reduces_sessions() {
        let (_dir, mgr) = test_manager();
        let ws = mgr.create_workstream("Test", None, &[]).unwrap();

        // Create and end two sessions with summaries
        mgr.send_message(Some(&ws.id), None, MessageRole::User, "msg1", None)
            .unwrap();
        let s1 = mgr.get_active_session(&ws.id).unwrap().unwrap();
        mgr.end_session(&s1.id).unwrap();
        mgr.store()
            .update_session_summary(&s1.id, "Session 1: discussed architecture")
            .unwrap();

        mgr.send_message(Some(&ws.id), None, MessageRole::User, "msg2", None)
            .unwrap();
        let s2 = mgr.get_active_session(&ws.id).unwrap().unwrap();
        mgr.end_session(&s2.id).unwrap();
        mgr.store()
            .update_session_summary(&s2.id, "Session 2: implemented features")
            .unwrap();

        // Compress workstream
        let backend =
            arawn_llm::MockBackend::with_text("Overall: architecture designed and features built.");
        let compressor = Compressor::new(std::sync::Arc::new(backend), CompressorConfig::default());

        let summary = compressor.compress_workstream(&mgr, &ws.id).await.unwrap();
        assert_eq!(
            summary,
            "Overall: architecture designed and features built."
        );

        // Verify stored on workstream
        let updated_ws = mgr.get_workstream(&ws.id).unwrap();
        assert_eq!(
            updated_ws.summary.as_deref(),
            Some("Overall: architecture designed and features built.")
        );
    }

    #[tokio::test]
    async fn test_compress_active_session_fails() {
        let (_dir, mgr) = test_manager();
        let ws = mgr.create_workstream("Test", None, &[]).unwrap();

        mgr.send_message(Some(&ws.id), None, MessageRole::User, "hi", None)
            .unwrap();
        let session = mgr.get_active_session(&ws.id).unwrap().unwrap();

        let backend = arawn_llm::MockBackend::with_text("summary");
        let compressor = Compressor::new(std::sync::Arc::new(backend), CompressorConfig::default());

        let err = compressor
            .compress_session(&mgr, &session.id)
            .await
            .unwrap_err();
        assert!(format!("{err}").contains("active session"));
    }
}
