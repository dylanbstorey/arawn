//! Session compaction for mid-session context management.
//!
//! The [`SessionCompactor`] summarizes older turns in a session while preserving
//! recent turns verbatim, enabling context management before hitting hard limits.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use arawn_llm::{CompletionRequest, Message, SharedBackend};

use crate::Result;
use crate::context::estimate_tokens;
use crate::types::{Session, Turn};

// ─────────────────────────────────────────────────────────────────────────────
// Constants
// ─────────────────────────────────────────────────────────────────────────────

/// Default number of recent turns to preserve verbatim.
const DEFAULT_PRESERVE_RECENT: usize = 3;

/// System prompt for mid-session summarization.
const MID_SESSION_SUMMARY_PROMPT: &str = "\
Summarize the earlier portion of this conversation concisely. Focus on:
- Key topics discussed and decisions made
- Important context needed for the ongoing conversation
- Any pending items or questions raised

Provide a clear, factual summary in 1-2 paragraphs. The summary will replace \
the earlier turns while the most recent exchanges are preserved verbatim.";

// ─────────────────────────────────────────────────────────────────────────────
// Types
// ─────────────────────────────────────────────────────────────────────────────

/// Configuration for session compaction.
#[derive(Debug, Clone)]
pub struct CompactorConfig {
    /// Model to use for summarization.
    pub model: String,
    /// Max tokens for summary generation.
    pub max_summary_tokens: u32,
    /// Number of recent turns to preserve verbatim.
    pub preserve_recent: usize,
    /// Custom summary prompt. When `None`, uses the default `MID_SESSION_SUMMARY_PROMPT`.
    pub summary_prompt: Option<String>,
}

impl Default for CompactorConfig {
    fn default() -> Self {
        Self {
            model: String::new(),
            max_summary_tokens: 1024,
            preserve_recent: DEFAULT_PRESERVE_RECENT,
            summary_prompt: None,
        }
    }
}

/// Result of a compaction operation.
#[derive(Debug, Clone)]
pub struct CompactionResult {
    /// Number of turns that were compacted into the summary.
    pub turns_compacted: usize,
    /// Estimated tokens before compaction.
    pub tokens_before: usize,
    /// Estimated tokens after compaction.
    pub tokens_after: usize,
    /// The generated summary text.
    pub summary: String,
}

impl CompactionResult {
    /// Estimate tokens freed by compaction.
    pub fn tokens_freed(&self) -> usize {
        self.tokens_before.saturating_sub(self.tokens_after)
    }

    /// Get compression ratio (smaller is better).
    pub fn compression_ratio(&self) -> f32 {
        if self.tokens_before == 0 {
            return 1.0;
        }
        (self.tokens_after as f32) / (self.tokens_before as f32)
    }
}

/// Progress callback for compaction operations.
pub type ProgressCallback = Box<dyn Fn(CompactionProgress) + Send + Sync>;

/// Progress updates during compaction.
#[derive(Debug, Clone)]
pub enum CompactionProgress {
    /// Starting compaction.
    Started {
        /// Total turns to compact.
        turns_to_compact: usize,
    },
    /// Generating summary.
    Summarizing,
    /// Compaction completed.
    Completed {
        /// Result of the compaction.
        result: CompactionResult,
    },
    /// Compaction was cancelled.
    Cancelled,
}

/// Token for cancelling compaction operations.
#[derive(Debug, Clone, Default)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    /// Create a new cancellation token.
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Signal cancellation.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    /// Check if cancellation was requested.
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SessionCompactor
// ─────────────────────────────────────────────────────────────────────────────

/// Compacts sessions by summarizing older turns while preserving recent ones.
///
/// The compactor:
/// 1. Identifies turns eligible for compaction (all but last N)
/// 2. Generates an LLM summary of those turns
/// 3. Returns the summary and statistics
///
/// The caller is responsible for applying the compaction to the session.
pub struct SessionCompactor {
    backend: SharedBackend,
    config: CompactorConfig,
}

impl SessionCompactor {
    /// Create a new session compactor.
    pub fn new(backend: SharedBackend, config: CompactorConfig) -> Self {
        Self { backend, config }
    }

    /// Set the number of recent turns to preserve.
    pub fn with_preserve_recent(mut self, count: usize) -> Self {
        self.config.preserve_recent = count;
        self
    }

    /// Set a custom summary prompt for compaction.
    ///
    /// When set, this prompt is used instead of the default `MID_SESSION_SUMMARY_PROMPT`.
    /// This allows different agent types (e.g., RLM exploration agents) to provide
    /// domain-specific compaction strategies.
    pub fn with_summary_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.config.summary_prompt = Some(prompt.into());
        self
    }

    /// Compact a session, generating a summary of older turns.
    ///
    /// Returns `None` if there aren't enough turns to compact (less than preserve_recent + 1).
    pub async fn compact(&self, session: &Session) -> Result<Option<CompactionResult>> {
        self.compact_with_options(session, None, None).await
    }

    /// Compact with progress callback.
    pub async fn compact_with_progress(
        &self,
        session: &Session,
        progress: Option<&ProgressCallback>,
    ) -> Result<Option<CompactionResult>> {
        self.compact_with_options(session, progress, None).await
    }

    /// Compact with full options: progress callback and cancellation token.
    pub async fn compact_with_options(
        &self,
        session: &Session,
        progress: Option<&ProgressCallback>,
        cancel: Option<&CancellationToken>,
    ) -> Result<Option<CompactionResult>> {
        // Check for cancellation before starting
        if cancel.is_some_and(|c| c.is_cancelled()) {
            if let Some(cb) = progress {
                cb(CompactionProgress::Cancelled);
            }
            return Err(crate::AgentError::Cancelled);
        }

        let turns = session.all_turns();
        let total_turns = turns.len();

        // Need at least preserve_recent + 1 turns to compact anything
        if total_turns <= self.config.preserve_recent {
            return Ok(None);
        }

        let turns_to_compact = total_turns - self.config.preserve_recent;

        // Report start
        if let Some(cb) = progress {
            cb(CompactionProgress::Started { turns_to_compact });
        }

        // Check for cancellation after reporting start
        if cancel.is_some_and(|c| c.is_cancelled()) {
            if let Some(cb) = progress {
                cb(CompactionProgress::Cancelled);
            }
            return Err(crate::AgentError::Cancelled);
        }

        // Split turns into compactable and preserved
        let (old_turns, _recent_turns) = turns.split_at(turns_to_compact);

        // Calculate tokens before
        let tokens_before = self.estimate_turns_tokens(old_turns);

        // Report summarizing
        if let Some(cb) = progress {
            cb(CompactionProgress::Summarizing);
        }

        // Generate summary (cancellation during LLM call is handled by the backend)
        let summary = self.summarize_turns(old_turns).await?;

        // Check for cancellation after summarization
        if cancel.is_some_and(|c| c.is_cancelled()) {
            if let Some(cb) = progress {
                cb(CompactionProgress::Cancelled);
            }
            return Err(crate::AgentError::Cancelled);
        }

        let tokens_after = estimate_tokens(&summary);

        let result = CompactionResult {
            turns_compacted: turns_to_compact,
            tokens_before,
            tokens_after,
            summary,
        };

        // Report completion
        if let Some(cb) = progress {
            cb(CompactionProgress::Completed {
                result: result.clone(),
            });
        }

        Ok(Some(result))
    }

    /// Check if a session needs compaction based on turn count.
    ///
    /// Returns true if there are more than `threshold` turns beyond preserved.
    pub fn needs_compaction(&self, session: &Session, threshold: usize) -> bool {
        let compactable = session
            .turn_count()
            .saturating_sub(self.config.preserve_recent);
        compactable >= threshold
    }

    /// Estimate tokens for a slice of turns.
    fn estimate_turns_tokens(&self, turns: &[Turn]) -> usize {
        turns
            .iter()
            .map(|t| {
                let mut tokens = estimate_tokens(&t.user_message);
                if let Some(ref response) = t.assistant_response {
                    tokens += estimate_tokens(response);
                }
                for tc in &t.tool_calls {
                    tokens += estimate_tokens(&tc.name);
                    tokens += estimate_tokens(&tc.arguments.to_string());
                }
                for tr in &t.tool_results {
                    tokens += estimate_tokens(&tr.content);
                }
                tokens
            })
            .sum()
    }

    /// Generate a summary of the given turns.
    async fn summarize_turns(&self, turns: &[Turn]) -> Result<String> {
        // Format turns as a conversation transcript
        let transcript = turns
            .iter()
            .map(|t| {
                let mut parts = vec![format!("User: {}", t.user_message)];

                for tc in &t.tool_calls {
                    parts.push(format!("Tool call: {} ({})", tc.name, tc.id));
                }

                for tr in &t.tool_results {
                    let status = if tr.success { "success" } else { "error" };
                    // Truncate long tool results
                    let content = if tr.content.len() > 500 {
                        format!("{}... [truncated]", &tr.content[..500])
                    } else {
                        tr.content.clone()
                    };
                    parts.push(format!("Tool result ({}): {}", status, content));
                }

                if let Some(ref response) = t.assistant_response {
                    parts.push(format!("Assistant: {}", response));
                }

                parts.join("\n")
            })
            .collect::<Vec<_>>()
            .join("\n\n---\n\n");

        let request = CompletionRequest::new(
            &self.config.model,
            vec![Message::user(transcript)],
            self.config.max_summary_tokens,
        )
        .with_system(
            self.config
                .summary_prompt
                .as_deref()
                .unwrap_or(MID_SESSION_SUMMARY_PROMPT),
        );

        let response =
            self.backend.complete(request).await.map_err(|e| {
                crate::AgentError::internal(format!("Compaction LLM call failed: {e}"))
            })?;

        Ok(response.text())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use arawn_llm::MockBackend;
    use std::sync::Arc;

    fn create_test_session(turn_count: usize) -> Session {
        let mut session = Session::new();
        for i in 0..turn_count {
            let turn = session.start_turn(format!("Message {}", i + 1));
            turn.complete(format!("Response {}", i + 1));
        }
        session
    }

    fn test_config() -> CompactorConfig {
        CompactorConfig {
            model: "test-model".to_string(),
            ..Default::default()
        }
    }

    fn test_compactor(backend: SharedBackend) -> SessionCompactor {
        SessionCompactor::new(backend, test_config())
    }

    #[test]
    fn test_compactor_config_defaults() {
        let config = CompactorConfig::default();
        assert_eq!(config.preserve_recent, 3);
        assert_eq!(config.max_summary_tokens, 1024);
    }

    #[test]
    fn test_compaction_result_tokens_freed() {
        let result = CompactionResult {
            turns_compacted: 5,
            tokens_before: 1000,
            tokens_after: 200,
            summary: "Summary".to_string(),
        };
        assert_eq!(result.tokens_freed(), 800);
    }

    #[test]
    fn test_compaction_result_compression_ratio() {
        let result = CompactionResult {
            turns_compacted: 5,
            tokens_before: 1000,
            tokens_after: 250,
            summary: "Summary".to_string(),
        };
        assert!((result.compression_ratio() - 0.25).abs() < 0.001);
    }

    #[test]
    fn test_compaction_result_zero_tokens_before() {
        let result = CompactionResult {
            turns_compacted: 0,
            tokens_before: 0,
            tokens_after: 0,
            summary: String::new(),
        };
        assert_eq!(result.compression_ratio(), 1.0);
    }

    #[test]
    fn test_needs_compaction_below_threshold() {
        let backend = Arc::new(MockBackend::with_text("summary"));
        let compactor = test_compactor(backend);

        // 5 turns, preserve 3, so 2 compactable - below threshold of 3
        let session = create_test_session(5);
        assert!(!compactor.needs_compaction(&session, 3));
    }

    #[test]
    fn test_needs_compaction_at_threshold() {
        let backend = Arc::new(MockBackend::with_text("summary"));
        let compactor = test_compactor(backend);

        // 6 turns, preserve 3, so 3 compactable - at threshold of 3
        let session = create_test_session(6);
        assert!(compactor.needs_compaction(&session, 3));
    }

    #[test]
    fn test_needs_compaction_above_threshold() {
        let backend = Arc::new(MockBackend::with_text("summary"));
        let compactor = test_compactor(backend);

        // 10 turns, preserve 3, so 7 compactable - above threshold of 3
        let session = create_test_session(10);
        assert!(compactor.needs_compaction(&session, 3));
    }

    #[tokio::test]
    async fn test_compact_empty_session() {
        let backend = Arc::new(MockBackend::with_text("summary"));
        let compactor = test_compactor(backend);

        let session = Session::new();
        let result = compactor.compact(&session).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_compact_insufficient_turns() {
        let backend = Arc::new(MockBackend::with_text("summary"));
        let compactor = test_compactor(backend); // preserve 3

        // Only 3 turns - nothing to compact
        let session = create_test_session(3);
        let result = compactor.compact(&session).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_compact_preserves_recent_turns() {
        let backend = Arc::new(MockBackend::with_text("Summary of earlier conversation."));
        let compactor = test_compactor(backend); // preserve 3

        // 6 turns - should compact 3
        let session = create_test_session(6);
        let result = compactor.compact(&session).await.unwrap();

        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.turns_compacted, 3);
        assert_eq!(result.summary, "Summary of earlier conversation.");
    }

    #[tokio::test]
    async fn test_compact_custom_preserve_count() {
        let backend = Arc::new(MockBackend::with_text("Summary"));
        let compactor = test_compactor(backend).with_preserve_recent(5);

        // 8 turns, preserve 5, compact 3
        let session = create_test_session(8);
        let result = compactor.compact(&session).await.unwrap();

        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.turns_compacted, 3);
    }

    #[tokio::test]
    async fn test_compact_custom_summary_prompt() {
        let custom_prompt = "Summarize research findings, preserving sources and key facts.";
        let backend = Arc::new(MockBackend::with_text("Research summary."));
        let compactor = test_compactor(backend.clone()).with_summary_prompt(custom_prompt);

        let session = create_test_session(6);
        let result = compactor.compact(&session).await.unwrap();
        assert!(result.is_some());

        // Verify the custom prompt was passed to the LLM
        let requests = backend.requests();
        assert_eq!(requests.len(), 1);
        let system = requests[0]
            .system
            .as_ref()
            .expect("should have system prompt");
        assert!(system.to_text().contains("research findings"));
    }

    #[tokio::test]
    async fn test_compact_result_stats() {
        let backend = Arc::new(MockBackend::with_text("Short summary."));
        let compactor = test_compactor(backend);

        let session = create_test_session(6);
        let result = compactor.compact(&session).await.unwrap().unwrap();

        assert_eq!(result.turns_compacted, 3);
        assert!(result.tokens_before > 0);
        assert!(result.tokens_after > 0);
        assert!(result.tokens_after < result.tokens_before);
    }

    #[tokio::test]
    async fn test_compact_with_progress_callback() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let backend = Arc::new(MockBackend::with_text("Summary"));
        let compactor = test_compactor(backend);

        let progress_count = Arc::new(AtomicUsize::new(0));
        let progress_count_clone = progress_count.clone();

        let callback: ProgressCallback = Box::new(move |_progress| {
            progress_count_clone.fetch_add(1, Ordering::SeqCst);
        });

        let session = create_test_session(6);
        compactor
            .compact_with_progress(&session, Some(&callback))
            .await
            .unwrap();

        // Should have received: Started, Summarizing, Completed
        assert_eq!(progress_count.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn test_estimate_turns_tokens() {
        let backend = Arc::new(MockBackend::with_text("summary"));
        let compactor = test_compactor(backend);

        let session = create_test_session(2);
        let tokens = compactor.estimate_turns_tokens(session.all_turns());

        // Each turn has "Message N" (~9 chars) + "Response N" (~10 chars)
        // With 4 chars/token, ~4-5 tokens per turn
        assert!(tokens > 0);
    }

    #[test]
    fn test_cancellation_token() {
        let token = CancellationToken::new();
        assert!(!token.is_cancelled());

        token.cancel();
        assert!(token.is_cancelled());
    }

    #[tokio::test]
    async fn test_compact_cancelled_before_start() {
        let backend = Arc::new(MockBackend::with_text("Summary"));
        let compactor = test_compactor(backend);

        let session = create_test_session(6);
        let cancel = CancellationToken::new();
        cancel.cancel(); // Cancel before starting

        let result = compactor
            .compact_with_options(&session, None, Some(&cancel))
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::AgentError::Cancelled));
    }

    #[tokio::test]
    async fn test_compact_cancelled_reports_progress() {
        use std::sync::atomic::{AtomicBool, Ordering};

        let backend = Arc::new(MockBackend::with_text("Summary"));
        let compactor = test_compactor(backend);

        let session = create_test_session(6);
        let cancel = CancellationToken::new();
        cancel.cancel();

        let cancelled_reported = Arc::new(AtomicBool::new(false));
        let cancelled_reported_clone = cancelled_reported.clone();

        let callback: ProgressCallback = Box::new(move |progress| {
            if matches!(progress, CompactionProgress::Cancelled) {
                cancelled_reported_clone.store(true, Ordering::SeqCst);
            }
        });

        let _ = compactor
            .compact_with_options(&session, Some(&callback), Some(&cancel))
            .await;

        assert!(cancelled_reported.load(Ordering::SeqCst));
    }
}
