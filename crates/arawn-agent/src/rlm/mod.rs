//! RLM (Recursive Language Model) exploration module.
//!
//! The RLM module provides an isolated sub-agent that explores information
//! spaces and returns compressed findings. It ties together:
//!
//! - [`ToolRegistry::filtered_by_names`] for read-only tool access
//! - [`CompactionOrchestrator`] for iterative explore→compact→continue cycles
//! - A research-focused system prompt
//!
//! # Usage
//!
//! ```rust,ignore
//! let spawner = RlmSpawner::new(backend, tool_registry);
//! let result = spawner.explore("How does authentication work?").await?;
//! println!("{}", result.summary);
//! ```

mod prompt;
pub mod types;

pub use prompt::RLM_SYSTEM_PROMPT;
pub use types::{ExplorationMetadata, ExplorationResult, RlmConfig};

use arawn_llm::SharedBackend;

use crate::agent::Agent;
use crate::compaction::{CompactorConfig, SessionCompactor};
use crate::error::Result;
use crate::orchestrator::{CompactionOrchestrator, OrchestratorConfig};
use crate::tool::ToolRegistry;

/// Default set of read-only tool names available to the RLM agent.
///
/// These tools allow exploration without side effects. Notably, `shell`,
/// `file_write`, `delegate`, and `note` are excluded to prevent mutations
/// and recursive sub-agent spawning.
pub const DEFAULT_READ_ONLY_TOOLS: &[&str] = &[
    "file_read",
    "glob",
    "grep",
    "web_fetch",
    "web_search",
    "memory_search",
    "think",
];

// ─────────────────────────────────────────────────────────────────────────────
// RlmSpawner
// ─────────────────────────────────────────────────────────────────────────────

/// Spawns isolated RLM exploration agents.
///
/// The spawner holds shared resources (backend, tools) and creates a fresh
/// agent + orchestrator for each [`explore`](RlmSpawner::explore) call.
/// This ensures each exploration is independent with its own session.
pub struct RlmSpawner {
    /// LLM backend for the exploration agent.
    backend: SharedBackend,
    /// Optional separate backend for compaction (cheaper/faster model).
    compaction_backend: Option<SharedBackend>,
    /// Full tool registry from which read-only tools are filtered.
    tools: ToolRegistry,
    /// Configuration for exploration runs.
    config: RlmConfig,
}

impl RlmSpawner {
    /// Create a new spawner with default configuration.
    pub fn new(backend: SharedBackend, tools: ToolRegistry) -> Self {
        Self {
            backend,
            compaction_backend: None,
            tools,
            config: RlmConfig::default(),
        }
    }

    /// Set the exploration configuration.
    pub fn with_config(mut self, config: RlmConfig) -> Self {
        self.config = config;
        self
    }

    /// Set a separate backend for compaction (e.g., a cheaper model).
    pub fn with_compaction_backend(mut self, backend: SharedBackend) -> Self {
        self.compaction_backend = Some(backend);
        self
    }

    /// Run an exploration for the given query.
    ///
    /// Creates a fresh agent with read-only tools, wraps it in a
    /// [`CompactionOrchestrator`], and runs until completion or budget
    /// exhaustion.
    pub async fn explore(&self, query: &str) -> Result<ExplorationResult> {
        // Filter tools to read-only subset
        let read_only_tools = self.tools.filtered_by_names(DEFAULT_READ_ONLY_TOOLS);

        // Build agent
        let mut builder = Agent::builder()
            .with_shared_backend(self.backend.clone())
            .with_tools(read_only_tools)
            .with_system_prompt(RLM_SYSTEM_PROMPT)
            .with_max_iterations(self.config.max_iterations_per_turn);

        if !self.config.model.is_empty() {
            builder = builder.with_model(&self.config.model);
        }

        if let Some(max_tokens) = self.config.max_total_tokens {
            builder = builder.with_max_total_tokens(max_tokens);
        }

        let agent = builder.build()?;
        let model_used = agent.config().model.clone();

        // Build compactor
        let compaction_backend = self
            .compaction_backend
            .clone()
            .unwrap_or_else(|| self.backend.clone());

        let compactor_config = CompactorConfig {
            summary_prompt: self.config.compaction_prompt.clone(),
            ..CompactorConfig::default()
        };

        let compactor = SessionCompactor::new(compaction_backend, compactor_config);

        // Build orchestrator
        let orchestrator_config = OrchestratorConfig {
            max_context_tokens: self.config.max_context_tokens,
            compaction_threshold: self.config.compaction_threshold,
            max_compactions: self.config.max_compactions,
            max_turns: self.config.max_turns,
        };

        let orchestrator = CompactionOrchestrator::new(agent, compactor, orchestrator_config);

        // Run exploration
        let result = orchestrator.run(query).await?;

        Ok(ExplorationResult {
            summary: result.text,
            truncated: result.truncated,
            metadata: ExplorationMetadata {
                iterations_used: result.metadata.total_iterations,
                input_tokens: result.metadata.total_input_tokens,
                output_tokens: result.metadata.total_output_tokens,
                compactions_performed: result.metadata.compactions_performed,
                model_used,
            },
        })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool::MockTool;
    use arawn_llm::{CompletionResponse, ContentBlock, MockBackend, StopReason, Usage};

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

    fn make_full_registry() -> ToolRegistry {
        let mut registry = ToolRegistry::new();
        // Register read-only tools that should be available
        registry.register(MockTool::new("file_read"));
        registry.register(MockTool::new("glob"));
        registry.register(MockTool::new("grep"));
        registry.register(MockTool::new("think"));
        // Register write tools that should be excluded
        registry.register(MockTool::new("file_write"));
        registry.register(MockTool::new("shell"));
        registry.register(MockTool::new("delegate"));
        registry
    }

    #[tokio::test]
    async fn test_explore_simple_query() {
        // Agent answers immediately without tool calls
        let backend = MockBackend::new(vec![mock_text_response(
            "Authentication uses JWT tokens stored in cookies.",
        )]);

        let spawner = RlmSpawner::new(std::sync::Arc::new(backend), make_full_registry());

        let result = spawner.explore("How does auth work?").await.unwrap();

        assert_eq!(
            result.summary,
            "Authentication uses JWT tokens stored in cookies."
        );
        assert!(!result.truncated);
        assert_eq!(result.metadata.compactions_performed, 0);
        assert_eq!(result.metadata.iterations_used, 1);
        // Model comes from AgentConfig default, not mock response
        assert_eq!(result.metadata.model_used, "claude-sonnet-4-20250514");
    }

    #[tokio::test]
    async fn test_explore_with_tool_calls() {
        // Agent uses a tool then responds
        let backend = MockBackend::new(vec![
            mock_tool_use_response("call_1", "grep", serde_json::json!({"pattern": "auth"})),
            mock_text_response("Found auth module at src/auth.rs."),
        ]);

        let spawner = RlmSpawner::new(std::sync::Arc::new(backend), make_full_registry());

        let result = spawner.explore("Find the auth module").await.unwrap();

        assert_eq!(result.summary, "Found auth module at src/auth.rs.");
        assert!(!result.truncated);
        assert_eq!(result.metadata.compactions_performed, 0);
    }

    #[tokio::test]
    async fn test_explore_filters_tools() {
        // Verify that write tools are NOT available to the exploration agent.
        // The agent tries to use "shell" (a write tool) — since it's not in
        // the filtered registry, the agent won't have it in its tool definitions.
        //
        // We verify this indirectly: the spawner creates an agent with only
        // read-only tools from DEFAULT_READ_ONLY_TOOLS.
        let backend = MockBackend::new(vec![mock_text_response("Done.")]);

        let spawner = RlmSpawner::new(std::sync::Arc::new(backend), make_full_registry());

        // Verify the default read-only tool list doesn't include write tools
        assert!(!DEFAULT_READ_ONLY_TOOLS.contains(&"shell"));
        assert!(!DEFAULT_READ_ONLY_TOOLS.contains(&"file_write"));
        assert!(!DEFAULT_READ_ONLY_TOOLS.contains(&"delegate"));
        assert!(!DEFAULT_READ_ONLY_TOOLS.contains(&"note"));

        // Verify it includes expected read-only tools
        assert!(DEFAULT_READ_ONLY_TOOLS.contains(&"file_read"));
        assert!(DEFAULT_READ_ONLY_TOOLS.contains(&"glob"));
        assert!(DEFAULT_READ_ONLY_TOOLS.contains(&"grep"));
        assert!(DEFAULT_READ_ONLY_TOOLS.contains(&"think"));

        // Exploration should work fine with read-only tools
        let result = spawner.explore("test").await.unwrap();
        assert!(!result.truncated);
    }

    #[tokio::test]
    async fn test_explore_with_custom_config() {
        let backend = MockBackend::new(vec![mock_text_response("Configured result.")]);

        let config = RlmConfig {
            max_context_tokens: 10_000,
            compaction_threshold: 0.5,
            max_compactions: 3,
            max_turns: 10,
            ..RlmConfig::default()
        };

        let spawner =
            RlmSpawner::new(std::sync::Arc::new(backend), make_full_registry()).with_config(config);

        let result = spawner.explore("test with config").await.unwrap();
        assert_eq!(result.summary, "Configured result.");
    }

    #[tokio::test]
    async fn test_explore_metadata_tokens() {
        let backend = MockBackend::new(vec![mock_text_response("Token check.")]);

        let spawner = RlmSpawner::new(std::sync::Arc::new(backend), make_full_registry());

        let result = spawner.explore("count tokens").await.unwrap();

        // Each mock response has Usage::new(100, 50)
        assert_eq!(result.metadata.input_tokens, 100);
        assert_eq!(result.metadata.output_tokens, 50);
        assert_eq!(result.metadata.total_tokens(), 150);
    }

    #[tokio::test]
    async fn test_system_prompt_is_set() {
        // Verify the RLM system prompt constant is non-empty and contains
        // key instructional phrases.
        assert!(!RLM_SYSTEM_PROMPT.is_empty());
        assert!(RLM_SYSTEM_PROMPT.contains("research exploration agent"));
        assert!(RLM_SYSTEM_PROMPT.contains("read-only"));
        assert!(RLM_SYSTEM_PROMPT.contains("Cite sources"));
    }
}
