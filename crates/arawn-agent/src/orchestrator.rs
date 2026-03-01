//! Compaction orchestrator for long-running agent tasks.
//!
//! The [`CompactionOrchestrator`] manages the explore→compact→continue cycle
//! that allows agents to work beyond context window limits. When context grows
//! past a configurable threshold, the orchestrator pauses the agent, compresses
//! the conversation history via [`SessionCompactor`], and resumes with a fresh
//! context containing the original query plus compacted findings.
//!
//! This is generic infrastructure — any long-running agent can use it.

use arawn_llm::SharedBackend;

use crate::agent::Agent;
use crate::compaction::{CompactorConfig, SessionCompactor};
use crate::context::estimate_tokens;
use crate::error::Result;
use crate::types::Session;

// ─────────────────────────────────────────────────────────────────────────────
// Configuration
// ─────────────────────────────────────────────────────────────────────────────

/// Configuration for the compaction orchestrator.
#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    /// Maximum estimated tokens before triggering compaction (e.g., 50_000).
    pub max_context_tokens: usize,
    /// Fraction of `max_context_tokens` that triggers compaction (0.0–1.0).
    /// Default: 0.7 (compact at 70% of max).
    pub compaction_threshold: f32,
    /// Maximum number of compaction cycles before stopping.
    /// Prevents infinite loops. Default: 10.
    pub max_compactions: u32,
    /// Maximum number of `agent.turn()` calls before stopping.
    /// Safety valve against infinite loops when the agent keeps getting
    /// truncated but context never triggers compaction. Default: 50.
    pub max_turns: u32,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            max_context_tokens: 50_000,
            compaction_threshold: 0.7,
            max_compactions: 10,
            max_turns: 50,
        }
    }
}

impl OrchestratorConfig {
    /// Token count that triggers compaction.
    fn threshold_tokens(&self) -> usize {
        (self.max_context_tokens as f64 * self.compaction_threshold as f64) as usize
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Result
// ─────────────────────────────────────────────────────────────────────────────

/// Result of an orchestrated run.
#[derive(Debug, Clone)]
pub struct OrchestrationResult {
    /// The final response text from the agent.
    pub text: String,
    /// Whether the run was truncated (budget or compaction limit exceeded).
    pub truncated: bool,
    /// Metadata about the orchestration run.
    pub metadata: OrchestrationMetadata,
}

/// Metadata from an orchestration run.
#[derive(Debug, Clone)]
pub struct OrchestrationMetadata {
    /// Total LLM iterations across all compaction cycles.
    pub total_iterations: u32,
    /// Number of compaction cycles performed.
    pub compactions_performed: u32,
    /// Total input tokens consumed across all cycles.
    pub total_input_tokens: u32,
    /// Total output tokens generated across all cycles.
    pub total_output_tokens: u32,
}

impl OrchestrationMetadata {
    /// Total tokens used (input + output).
    pub fn total_tokens(&self) -> u32 {
        self.total_input_tokens + self.total_output_tokens
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Orchestrator
// ─────────────────────────────────────────────────────────────────────────────

/// Manages the explore→compact→continue cycle for long-running agent tasks.
///
/// The orchestrator wraps an [`Agent`] and a [`SessionCompactor`], running
/// the agent in a loop. When the context grows beyond the configured threshold,
/// it compacts the conversation and continues with fresh context.
///
/// # Example
///
/// ```rust,ignore
/// let orchestrator = CompactionOrchestrator::new(agent, compactor, config);
/// let result = orchestrator.run("How does authentication work?").await?;
/// println!("{}", result.text);
/// println!("Compactions: {}", result.metadata.compactions_performed);
/// ```
pub struct CompactionOrchestrator {
    agent: Agent,
    compactor: SessionCompactor,
    config: OrchestratorConfig,
}

impl CompactionOrchestrator {
    /// Create a new orchestrator.
    pub fn new(agent: Agent, compactor: SessionCompactor, config: OrchestratorConfig) -> Self {
        Self {
            agent,
            compactor,
            config,
        }
    }

    /// Create an orchestrator with a compaction backend that may differ from the agent's.
    ///
    /// This allows using a cheaper/faster model for compaction while using
    /// a more capable model for exploration.
    pub fn with_compaction_backend(
        agent: Agent,
        compaction_backend: SharedBackend,
        compaction_prompt: Option<String>,
        config: OrchestratorConfig,
    ) -> Self {
        let compactor_config = CompactorConfig {
            model: String::new(), // Model comes from the backend
            summary_prompt: compaction_prompt,
            ..CompactorConfig::default()
        };

        let compactor = SessionCompactor::new(compaction_backend, compactor_config);

        Self {
            agent,
            compactor,
            config,
        }
    }

    /// Run the agent with compaction-managed context.
    ///
    /// The agent processes the query, calling tools as needed. When context
    /// grows beyond the threshold, it's compacted and the agent continues.
    /// The run stops when:
    /// - The agent produces a response without tool calls (done)
    /// - The agent's token budget is exceeded (truncated)
    /// - The maximum compaction limit is reached (truncated)
    pub async fn run(&self, query: &str) -> Result<OrchestrationResult> {
        let mut session = Session::new();
        let mut compactions_performed = 0u32;
        let mut cumulative_input_tokens = 0u32;
        let mut cumulative_output_tokens = 0u32;
        let mut cumulative_iterations = 0u32;
        #[allow(unused_assignments)]
        let mut last_text = String::new();
        let mut truncated = false;
        let mut turns = 0u32;

        // The effective query may change after compaction (original + summary)
        let mut effective_query = query.to_string();

        loop {
            // Run a turn
            let response = self.agent.turn(&mut session, &effective_query).await?;

            // Accumulate stats
            cumulative_input_tokens += response.usage.input_tokens;
            cumulative_output_tokens += response.usage.output_tokens;
            cumulative_iterations += response.iterations;
            last_text = response.text.clone();
            turns += 1;

            // Agent finished naturally (no tool calls in final iteration) → done.
            // Note: response.tool_calls accumulates ALL tool calls from the turn,
            // so we check truncated alone — a non-truncated response means the LLM
            // produced a final text response without requesting more tool calls.
            if !response.truncated {
                break;
            }

            // Max turns safety valve — prevents infinite loops when the agent
            // keeps getting truncated but context never triggers compaction.
            if turns >= self.config.max_turns {
                tracing::warn!(
                    turns,
                    max = self.config.max_turns,
                    "Orchestrator: max turns reached"
                );
                truncated = true;
                break;
            }

            // Check if we need compaction
            let context_tokens = self.estimate_session_tokens(&session);
            let threshold = self.config.threshold_tokens();

            tracing::debug!(
                context_tokens,
                threshold,
                compactions = compactions_performed,
                "Orchestrator: checking compaction threshold"
            );

            if context_tokens >= threshold {
                // Check compaction limit
                if compactions_performed >= self.config.max_compactions {
                    tracing::warn!(
                        compactions = compactions_performed,
                        max = self.config.max_compactions,
                        "Orchestrator: max compaction limit reached"
                    );
                    truncated = true;
                    break;
                }

                // Compact
                tracing::info!(
                    context_tokens,
                    threshold,
                    compaction = compactions_performed + 1,
                    "Orchestrator: compacting context"
                );

                match self.compactor.compact(&session).await {
                    Ok(Some(compaction_result)) => {
                        compactions_performed += 1;

                        tracing::info!(
                            turns_compacted = compaction_result.turns_compacted,
                            tokens_before = compaction_result.tokens_before,
                            tokens_after = compaction_result.tokens_after,
                            "Orchestrator: compaction complete"
                        );

                        // Build a new session with the compacted context
                        session = Session::new();
                        effective_query = format!(
                            "Original query: {}\n\n\
                             Previous findings (compacted):\n{}\n\n\
                             Continue exploring to answer the original query. \
                             Build on the findings above — do not repeat work already done.",
                            query, compaction_result.summary
                        );
                    }
                    Ok(None) => {
                        // Not enough turns to compact — continue without compaction
                        tracing::debug!("Orchestrator: not enough turns to compact, continuing");
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, "Orchestrator: compaction failed, continuing without");
                    }
                }
            }

            // Agent was truncated (hit max_iterations or token budget).
            // Loop back: if compaction happened, the next turn starts fresh.
            // If not, the next turn picks up where the session left off.
        }

        Ok(OrchestrationResult {
            text: last_text,
            truncated,
            metadata: OrchestrationMetadata {
                total_iterations: cumulative_iterations,
                compactions_performed,
                total_input_tokens: cumulative_input_tokens,
                total_output_tokens: cumulative_output_tokens,
            },
        })
    }

    /// Estimate total tokens in a session's conversation history.
    fn estimate_session_tokens(&self, session: &Session) -> usize {
        session
            .all_turns()
            .iter()
            .map(|turn| {
                let user_tokens = estimate_tokens(&turn.user_message);
                let assistant_tokens = turn
                    .assistant_response
                    .as_deref()
                    .map(estimate_tokens)
                    .unwrap_or(0);
                let tool_tokens: usize = turn
                    .tool_results
                    .iter()
                    .map(|r| estimate_tokens(&r.content))
                    .sum();
                user_tokens + assistant_tokens + tool_tokens
            })
            .sum()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool::{MockTool, ToolRegistry, ToolResult};
    use arawn_llm::{CompletionResponse, ContentBlock, MockBackend, StopReason, Usage};
    use std::sync::Arc;

    fn mock_text_response(text: &str) -> CompletionResponse {
        CompletionResponse::new(
            "msg_1",
            "test-model",
            vec![ContentBlock::Text {
                text: text.to_string(),
                cache_control: None,
            }],
            StopReason::EndTurn,
            Usage::new(100, 50),
        )
    }

    fn mock_tool_use_response(
        tool_id: &str,
        tool_name: &str,
        args: serde_json::Value,
    ) -> CompletionResponse {
        CompletionResponse::new(
            "msg_1",
            "test-model",
            vec![ContentBlock::ToolUse {
                id: tool_id.to_string(),
                name: tool_name.to_string(),
                input: args,
                cache_control: None,
            }],
            StopReason::ToolUse,
            Usage::new(100, 50),
        )
    }

    /// Build an agent with max_iterations=1 so the orchestrator controls
    /// the iteration flow (one LLM call per turn, compaction between turns).
    fn make_agent(backend: MockBackend, tools: ToolRegistry) -> Agent {
        Agent::builder()
            .with_backend(backend)
            .with_tools(tools)
            .with_max_iterations(1)
            .build()
            .unwrap()
    }

    fn make_compactor(backend: MockBackend) -> SessionCompactor {
        // preserve_recent=0 so the compactor can summarize even a single turn.
        // With max_iterations=1 on the agent, each orchestrator turn produces
        // just 1 session turn, so we need preserve_recent=0 to compact it.
        let config = CompactorConfig {
            preserve_recent: 0,
            ..CompactorConfig::default()
        };
        SessionCompactor::new(Arc::new(backend), config)
    }

    #[tokio::test]
    async fn test_simple_run_no_tools() {
        // Agent responds immediately without tool calls → done
        let agent_backend = MockBackend::new(vec![mock_text_response("Here are my findings.")]);
        let compactor_backend = MockBackend::new(vec![]);

        let agent = make_agent(agent_backend, ToolRegistry::new());
        let compactor = make_compactor(compactor_backend);
        let config = OrchestratorConfig::default();

        let orchestrator = CompactionOrchestrator::new(agent, compactor, config);
        let result = orchestrator.run("What is this?").await.unwrap();

        assert_eq!(result.text, "Here are my findings.");
        assert!(!result.truncated);
        assert_eq!(result.metadata.compactions_performed, 0);
        assert_eq!(result.metadata.total_iterations, 1);
    }

    #[tokio::test]
    async fn test_run_with_tool_calls_then_done() {
        // With max_iterations=1, each tool call is a separate turn:
        //   Turn 1: LLM → tool_use → execute → hits max_iterations → truncated
        //   Turn 2: LLM → text → natural end
        let agent_backend = MockBackend::new(vec![
            mock_tool_use_response("call_1", "search", serde_json::json!({})),
            mock_text_response("Found the answer."),
        ]);
        let compactor_backend = MockBackend::new(vec![]);

        let mut tools = ToolRegistry::new();
        tools.register(MockTool::new("search"));

        let agent = make_agent(agent_backend, tools);
        let compactor = make_compactor(compactor_backend);
        let config = OrchestratorConfig::default();

        let orchestrator = CompactionOrchestrator::new(agent, compactor, config);
        let result = orchestrator.run("Search for auth").await.unwrap();

        assert_eq!(result.text, "Found the answer.");
        assert!(!result.truncated);
        assert_eq!(result.metadata.compactions_performed, 0);
        // Turn 1: iterations=2 (loop runs twice: call LLM + hit limit)
        // Turn 2: iterations=1 (call LLM, natural end)
        assert_eq!(result.metadata.total_iterations, 3);
    }

    #[tokio::test]
    async fn test_compaction_triggered_at_threshold() {
        // With max_iterations=1 and low threshold:
        //   Turn 1: tool_use (large output) → truncated → context exceeds threshold → compact
        //   Turn 2: text → natural end
        let large_output = "x".repeat(2000); // ~500 tokens at 4 chars/token

        let agent_backend = MockBackend::new(vec![
            mock_tool_use_response("call_1", "search", serde_json::json!({})),
            mock_text_response("Final answer after compaction."),
        ]);

        // Compactor returns a summary
        let compactor_backend =
            MockBackend::new(vec![mock_text_response("Compacted summary of findings.")]);

        let mut tools = ToolRegistry::new();
        tools.register(MockTool::new("search").with_response(ToolResult::text(&large_output)));

        let agent = make_agent(agent_backend, tools);
        let compactor = make_compactor(compactor_backend);

        let config = OrchestratorConfig {
            max_context_tokens: 1000,
            compaction_threshold: 0.1, // Very low — triggers easily
            max_compactions: 10,
            max_turns: 50,
        };

        let orchestrator = CompactionOrchestrator::new(agent, compactor, config);
        let result = orchestrator.run("Search for stuff").await.unwrap();

        assert_eq!(result.text, "Final answer after compaction.");
        assert!(!result.truncated);
        assert_eq!(result.metadata.compactions_performed, 1);
    }

    #[tokio::test]
    async fn test_no_compaction_when_under_threshold() {
        // Context stays small — no compaction needed.
        // With max_iterations=1:
        //   Turn 1: tool_use (small output) → truncated → context below threshold → continue
        //   Turn 2: text → natural end
        let agent_backend = MockBackend::new(vec![
            mock_tool_use_response("call_1", "search", serde_json::json!({})),
            mock_text_response("Done."),
        ]);
        let compactor_backend = MockBackend::new(vec![]);

        let mut tools = ToolRegistry::new();
        tools.register(MockTool::new("search").with_response(ToolResult::text("small result")));

        let agent = make_agent(agent_backend, tools);
        let compactor = make_compactor(compactor_backend);

        let config = OrchestratorConfig {
            max_context_tokens: 100_000, // Very high — never triggers
            compaction_threshold: 0.7,
            max_compactions: 10,
            max_turns: 50,
        };

        let orchestrator = CompactionOrchestrator::new(agent, compactor, config);
        let result = orchestrator.run("Quick question").await.unwrap();

        assert_eq!(result.text, "Done.");
        assert!(!result.truncated);
        assert_eq!(result.metadata.compactions_performed, 0);
    }

    #[tokio::test]
    async fn test_max_compactions_exceeded() {
        // With max_iterations=1, max_compactions=2:
        //   Turn 1: tool_use (large) → truncated → compact (1)
        //   Turn 2: tool_use (large) → truncated → compact (2)
        //   Turn 3: tool_use (large) → truncated → compactions >= max → stop
        let large_output = "x".repeat(2000);

        let agent_backend = MockBackend::new(vec![
            mock_tool_use_response("call_0", "search", serde_json::json!({})),
            mock_tool_use_response("call_1", "search", serde_json::json!({})),
            mock_tool_use_response("call_2", "search", serde_json::json!({})),
        ]);

        // Compactor responses for each compaction cycle
        let compactor_backend = MockBackend::new(vec![
            mock_text_response("Summary 1."),
            mock_text_response("Summary 2."),
        ]);

        let mut tools = ToolRegistry::new();
        tools.register(MockTool::new("search").with_response(ToolResult::text(&large_output)));

        let agent = make_agent(agent_backend, tools);
        let compactor = make_compactor(compactor_backend);

        let config = OrchestratorConfig {
            max_context_tokens: 1000,
            compaction_threshold: 0.1,
            max_compactions: 2,
            max_turns: 50,
        };

        let orchestrator = CompactionOrchestrator::new(agent, compactor, config);
        let result = orchestrator.run("Keep searching").await.unwrap();

        assert!(result.truncated);
        assert_eq!(result.metadata.compactions_performed, 2);
    }

    #[tokio::test]
    async fn test_max_turns_stops_cleanly() {
        // Agent keeps calling tools but context stays small (no compaction).
        // The max_turns safety valve should fire.
        let agent_backend = MockBackend::new(vec![
            mock_tool_use_response("call_0", "search", serde_json::json!({})),
            mock_tool_use_response("call_1", "search", serde_json::json!({})),
            mock_tool_use_response("call_2", "search", serde_json::json!({})),
        ]);
        let compactor_backend = MockBackend::new(vec![]);

        let mut tools = ToolRegistry::new();
        tools.register(MockTool::new("search"));

        let agent = make_agent(agent_backend, tools);
        let compactor = make_compactor(compactor_backend);

        let config = OrchestratorConfig {
            max_context_tokens: 100_000, // High — no compaction
            compaction_threshold: 0.7,
            max_compactions: 10,
            max_turns: 2, // Fire after 2 turns
        };

        let orchestrator = CompactionOrchestrator::new(agent, compactor, config);
        let result = orchestrator.run("Search forever").await.unwrap();

        assert!(result.truncated);
        assert_eq!(result.metadata.compactions_performed, 0);
    }

    #[tokio::test]
    async fn test_cumulative_stats() {
        // With max_iterations=1, two tool calls then text = 3 turns:
        //   Turn 1: tool_use → truncated (iterations=2)
        //   Turn 2: tool_use → truncated (iterations=2)
        //   Turn 3: text → natural end (iterations=1)
        let agent_backend = MockBackend::new(vec![
            mock_tool_use_response("call_1", "search", serde_json::json!({})),
            mock_tool_use_response("call_2", "search", serde_json::json!({})),
            mock_text_response("All done."),
        ]);
        let compactor_backend = MockBackend::new(vec![]);

        let mut tools = ToolRegistry::new();
        tools.register(MockTool::new("search"));

        let agent = make_agent(agent_backend, tools);
        let compactor = make_compactor(compactor_backend);
        let config = OrchestratorConfig::default();

        let orchestrator = CompactionOrchestrator::new(agent, compactor, config);
        let result = orchestrator.run("Search twice").await.unwrap();

        assert_eq!(result.text, "All done.");
        // Turn 1: iterations=2 (100 in, 50 out)
        // Turn 2: iterations=2 (100 in, 50 out)
        // Turn 3: iterations=1 (100 in, 50 out)
        assert_eq!(result.metadata.total_iterations, 5);
        assert_eq!(result.metadata.total_input_tokens, 300);
        assert_eq!(result.metadata.total_output_tokens, 150);
        assert_eq!(result.metadata.total_tokens(), 450);
    }
}
