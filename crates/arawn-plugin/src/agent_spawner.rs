//! Plugin agent spawning.
//!
//! Creates `Agent` instances from plugin-defined agent configurations.
//! Subagents get constrained tool sets and custom system prompts.
//!
//! This module provides two main types:
//! - [`AgentSpawner`]: Low-level spawner that creates agents from configs
//! - [`PluginSubagentSpawner`]: Implements [`SubagentSpawner`] trait for use with `DelegateTool`

use arawn_agent::Agent;
use arawn_agent::tool::ToolRegistry;
use arawn_agent::types::AgentConfig;
use arawn_config::CompactionConfig;
use arawn_llm::SharedBackend;
use arawn_llm::types::{CompletionRequest, Message};
use arawn_types::{
    DelegationOutcome, SharedHookDispatcher, SubagentInfo, SubagentResult, SubagentSpawner,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use crate::Result;
use crate::types::PluginAgentConfig;

/// Default maximum length for context passed to subagents (in characters).
const DEFAULT_MAX_CONTEXT_LEN: usize = 4000;

/// Default maximum length for subagent results (in characters).
const DEFAULT_MAX_RESULT_LEN: usize = 8000;

/// Truncate context to a maximum length, preserving word boundaries where possible.
fn truncate_context(context: &str, max_len: usize) -> String {
    if context.len() <= max_len {
        return context.to_string();
    }

    // Find a good break point (word boundary) near max_len
    let truncate_at = context[..max_len]
        .rfind(|c: char| c.is_whitespace())
        .unwrap_or(max_len);

    format!("{}...(truncated)", &context[..truncate_at])
}

/// Result of truncating a subagent response.
struct TruncatedResult {
    /// The (possibly truncated) text.
    text: String,
    /// Whether truncation occurred.
    truncated: bool,
    /// Original length if truncated.
    original_len: Option<usize>,
}

/// Truncate a subagent result, preserving beginning and end of the response.
///
/// If the text exceeds `max_len`, keeps the first 60% and last 30% of the budget,
/// with a truncation notice in between.
fn truncate_result(text: &str, max_len: usize) -> TruncatedResult {
    if text.len() <= max_len {
        return TruncatedResult {
            text: text.to_string(),
            truncated: false,
            original_len: None,
        };
    }

    let original_len = text.len();

    // Allocate budget: 60% for beginning, 30% for end, 10% for the notice
    let notice = format!(
        "\n\n[...{} characters omitted...]\n\n",
        original_len - max_len
    );
    let available = max_len.saturating_sub(notice.len());
    let first_len = (available as f64 * 0.65) as usize;
    let last_len = available.saturating_sub(first_len);

    // Find word boundary for first section
    let first_end = text[..first_len]
        .rfind(|c: char| c.is_whitespace())
        .unwrap_or(first_len);

    // Find word boundary for last section
    let last_start = text.len() - last_len;
    let last_start = text[last_start..]
        .find(|c: char| c.is_whitespace())
        .map(|i| last_start + i + 1)
        .unwrap_or(last_start);

    let truncated_text = format!("{}{}{}", &text[..first_end], notice, &text[last_start..]);

    TruncatedResult {
        text: truncated_text,
        truncated: true,
        original_len: Some(original_len),
    }
}

/// System prompt for context compaction.
const COMPACTION_SYSTEM_PROMPT: &str = r#"You are a specialized summarization assistant. Your task is to condense long text while preserving the most important information.

## What to Preserve (Priority Order)
1. **Actionable conclusions and findings** - What was accomplished? What are the results?
2. **Code snippets and examples** - Preserve exact code with context
3. **Specific data, numbers, and citations** - URLs, file paths, line numbers, measurements
4. **Key decisions and their rationale** - Why something was done a certain way
5. **Error messages and warnings** - Full text of any errors encountered
6. **Next steps or recommendations** - What should happen next

## What to Remove
- Conversational filler and pleasantries
- Redundant explanations of the same concept
- Verbose step-by-step narration (summarize the outcome instead)
- Generic caveats and disclaimers
- Repeated information

## Output Format
- Use markdown formatting for structure
- Keep technical terms precise
- Preserve the original's tone (if it was formal, stay formal)
- If the content is a list, keep it as a list (possibly condensed)

Produce a summary that is roughly 40-60% of the original length while retaining the essential information."#;

/// Result of compacting a subagent response.
struct CompactionResult {
    /// The compacted text.
    text: String,
    /// Whether compaction was successful.
    success: bool,
    /// Original length before compaction.
    original_len: usize,
}

/// Compact a long subagent result using LLM summarization.
async fn compact_result(
    text: &str,
    backend: &SharedBackend,
    model: &str,
    target_len: usize,
) -> CompactionResult {
    let original_len = text.len();

    // Build the user message with the content to summarize
    let user_prompt = format!(
        "Summarize the following text to approximately {} characters while preserving the most important information:\n\n---\n\n{}",
        target_len, text
    );

    // Build the completion request
    let request = CompletionRequest::new(model, vec![Message::user(user_prompt)], 4096)
        .with_system(COMPACTION_SYSTEM_PROMPT);

    // Execute the compaction
    match backend.complete(request).await {
        Ok(response) => {
            let compacted_text = response.text();
            tracing::debug!(
                original_len = original_len,
                compacted_len = compacted_text.len(),
                "compacted subagent result"
            );
            CompactionResult {
                text: compacted_text,
                success: true,
                original_len,
            }
        }
        Err(e) => {
            tracing::warn!(error = %e, "compaction failed, falling back to truncation");
            CompactionResult {
                text: text.to_string(),
                success: false,
                original_len,
            }
        }
    }
}

/// Spawns agents from plugin agent configurations.
pub struct AgentSpawner {
    /// The parent agent's tool registry (source for constrained tool sets).
    parent_tools: Arc<ToolRegistry>,
    /// The LLM backend to use for subagents.
    backend: SharedBackend,
    /// Default max_iterations from `[agent.default]` config (fallback for all agents).
    default_max_iterations: Option<u32>,
}

impl AgentSpawner {
    /// Create a new agent spawner.
    pub fn new(parent_tools: Arc<ToolRegistry>, backend: SharedBackend) -> Self {
        Self {
            parent_tools,
            backend,
            default_max_iterations: None,
        }
    }

    /// Create a new agent spawner with a default max_iterations.
    ///
    /// The `default_max_iterations` is applied to all spawned agents unless
    /// they specify their own `constraints.max_iterations`.
    pub fn with_default_max_iterations(mut self, max_iterations: u32) -> Self {
        self.default_max_iterations = Some(max_iterations);
        self
    }

    /// Spawn an agent from a plugin agent configuration.
    ///
    /// The spawned agent gets:
    /// - A constrained tool registry (only tools listed in config)
    /// - The plugin agent's custom system prompt
    /// - Optional max_iterations cap
    ///
    /// The max_iterations resolution order is:
    /// 1. Agent-specific `constraints.max_iterations` (highest priority)
    /// 2. Global `default_max_iterations` from `[agent.default]` config
    /// 3. `AgentConfig::default()` (hardcoded 10)
    pub fn spawn(&self, config: &PluginAgentConfig) -> Result<Agent> {
        // Build constrained tool registry
        let constrained_tools = self.constrain_tools(config);

        // Build agent config
        let mut agent_config = AgentConfig::default();

        // Apply global default from [agent.default] if set
        if let Some(default_max) = self.default_max_iterations {
            agent_config.max_iterations = default_max;
        }

        if let Some(ref prompt) = config.agent.system_prompt {
            agent_config.system_prompt = Some(prompt.text.clone());
        }

        // Agent-specific override takes precedence
        if let Some(ref constraints) = config.agent.constraints {
            if let Some(max_iter) = constraints.max_iterations {
                agent_config.max_iterations = max_iter as u32;
            }
        }

        // Note: model override is stored in config.agent.model but actual
        // backend switching requires the config resolver (deferred to wiring task).

        let agent = Agent::builder()
            .with_shared_backend(self.backend.clone())
            .with_tools(constrained_tools)
            .with_config(agent_config)
            .build()
            .map_err(|e| crate::PluginError::AgentConfigParse {
                reason: format!("failed to build agent '{}': {}", config.agent.name, e),
            })?;

        Ok(agent)
    }

    /// Create a constrained tool registry from the parent's tools.
    fn constrain_tools(&self, config: &PluginAgentConfig) -> ToolRegistry {
        let allowed: Vec<&str> = if let Some(ref constraints) = config.agent.constraints {
            constraints.tools.iter().map(|s| s.as_str()).collect()
        } else {
            // No constraints section — no tools
            Vec::new()
        };

        let mut registry = ToolRegistry::new();
        for name in &allowed {
            if let Some(tool) = self.parent_tools.get(name) {
                registry.register_arc(tool);
            } else {
                tracing::warn!(
                    agent = %config.agent.name,
                    tool = %name,
                    "plugin agent references tool not in parent registry, skipping"
                );
            }
        }
        registry
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Plugin Subagent Spawner (implements SubagentSpawner trait)
// ─────────────────────────────────────────────────────────────────────────────

/// A subagent spawner backed by plugin-defined agent configurations.
///
/// This struct implements the [`SubagentSpawner`] trait from `arawn-types`,
/// enabling use with [`DelegateTool`] for subagent delegation.
///
/// # Example
///
/// ```rust,ignore
/// use arawn_plugin::PluginSubagentSpawner;
///
/// // Collect agent configs from loaded plugins
/// let agent_configs: HashMap<String, PluginAgentConfig> = /* ... */;
///
/// // Create the spawner
/// let spawner = PluginSubagentSpawner::new(parent_tools, backend, agent_configs);
///
/// // Use with DelegateTool
/// let delegate_tool = DelegateTool::new(Arc::new(spawner));
/// ```
pub struct PluginSubagentSpawner {
    /// The underlying agent spawner.
    spawner: AgentSpawner,
    /// Agent configurations keyed by agent name.
    agent_configs: HashMap<String, PluginAgentConfig>,
    /// Source plugin name for each agent (for SubagentInfo).
    agent_sources: HashMap<String, String>,
    /// Optional hook dispatcher for subagent lifecycle events.
    hook_dispatcher: Option<SharedHookDispatcher>,
    /// Optional backend for result compaction.
    compaction_backend: Option<SharedBackend>,
    /// Compaction configuration.
    compaction_config: CompactionConfig,
}

impl PluginSubagentSpawner {
    /// Create a new plugin subagent spawner.
    ///
    /// # Arguments
    /// * `parent_tools` - The parent agent's tool registry
    /// * `backend` - The LLM backend to use for subagents
    /// * `agent_configs` - Map of agent name to configuration
    pub fn new(
        parent_tools: Arc<ToolRegistry>,
        backend: SharedBackend,
        agent_configs: HashMap<String, PluginAgentConfig>,
    ) -> Self {
        Self {
            spawner: AgentSpawner::new(parent_tools, backend),
            agent_configs,
            agent_sources: HashMap::new(),
            hook_dispatcher: None,
            compaction_backend: None,
            compaction_config: CompactionConfig::default(),
        }
    }

    /// Create a spawner with source plugin tracking.
    ///
    /// This variant records which plugin each agent came from,
    /// useful for debugging and the `source` field in [`SubagentInfo`].
    pub fn with_sources(
        parent_tools: Arc<ToolRegistry>,
        backend: SharedBackend,
        agent_configs: HashMap<String, PluginAgentConfig>,
        agent_sources: HashMap<String, String>,
    ) -> Self {
        Self {
            spawner: AgentSpawner::new(parent_tools, backend),
            agent_configs,
            agent_sources,
            hook_dispatcher: None,
            compaction_backend: None,
            compaction_config: CompactionConfig::default(),
        }
    }

    /// Set the hook dispatcher for subagent lifecycle events.
    ///
    /// When set, the spawner will fire `SubagentStarted` and `SubagentCompleted`
    /// events for background subagent executions.
    pub fn with_hook_dispatcher(mut self, dispatcher: SharedHookDispatcher) -> Self {
        self.hook_dispatcher = Some(dispatcher);
        self
    }

    /// Set the compaction backend and configuration.
    ///
    /// When set, long subagent results will be summarized using the provided
    /// LLM backend instead of being truncated.
    pub fn with_compaction(mut self, backend: SharedBackend, config: CompactionConfig) -> Self {
        self.compaction_backend = Some(backend);
        self.compaction_config = config;
        self
    }

    /// Set the default max_iterations for all spawned agents.
    ///
    /// This value is used as a fallback when an agent doesn't specify
    /// its own `constraints.max_iterations`. Mirrors `[agent.default].max_iterations`.
    pub fn with_default_max_iterations(mut self, max_iterations: u32) -> Self {
        self.spawner = self.spawner.with_default_max_iterations(max_iterations);
        self
    }

    /// Get the number of available agents.
    pub fn agent_count(&self) -> usize {
        self.agent_configs.len()
    }

    /// Check if any agents are available.
    pub fn is_empty(&self) -> bool {
        self.agent_configs.is_empty()
    }

    /// Get the names of all available agents.
    pub fn agent_names(&self) -> Vec<&str> {
        self.agent_configs.keys().map(|s| s.as_str()).collect()
    }
}

#[async_trait]
impl SubagentSpawner for PluginSubagentSpawner {
    async fn list_agents(&self) -> Vec<SubagentInfo> {
        self.agent_configs
            .iter()
            .map(|(name, config)| {
                let tools = config
                    .agent
                    .constraints
                    .as_ref()
                    .map(|c| c.tools.clone())
                    .unwrap_or_default();

                SubagentInfo {
                    name: name.clone(),
                    description: config.agent.description.clone(),
                    tools,
                    source: self.agent_sources.get(name).cloned(),
                }
            })
            .collect()
    }

    async fn delegate(
        &self,
        agent_name: &str,
        task: &str,
        context: Option<&str>,
        max_turns: Option<usize>,
    ) -> DelegationOutcome {
        use arawn_agent::types::Session;

        // Look up the agent config
        let config = match self.agent_configs.get(agent_name) {
            Some(c) => c,
            None => {
                return DelegationOutcome::UnknownAgent {
                    name: agent_name.to_string(),
                    available: self.agent_configs.keys().cloned().collect(),
                };
            }
        };

        // Spawn the agent with max_turns override if specified
        let mut agent_config = config.clone();
        if let Some(max) = max_turns {
            if let Some(ref mut constraints) = agent_config.agent.constraints {
                constraints.max_iterations = Some(max);
            }
        }

        let agent = match self.spawner.spawn(&agent_config) {
            Ok(a) => a,
            Err(e) => {
                return DelegationOutcome::Error {
                    message: format!("Failed to spawn agent '{}': {}", agent_name, e),
                };
            }
        };

        // Create session with optional context preamble
        let mut session = Session::new();
        if let Some(ctx) = context {
            let truncated = truncate_context(ctx, DEFAULT_MAX_CONTEXT_LEN);
            session
                .set_context_preamble(format!("## Context from parent session\n\n{}", truncated));
        }

        // Execute the agent turn
        let start = Instant::now();

        match agent.turn(&mut session, task).await {
            Ok(response) => {
                let duration_ms = start.elapsed().as_millis() as u64;
                let response_text = &response.text;
                let response_len = response_text.len();

                // Determine how to handle long results
                let (final_text, truncated, compacted, original_len) = if response_len
                    > self.compaction_config.threshold
                {
                    // Result exceeds threshold - try compaction or truncate
                    if self.compaction_config.enabled {
                        if let Some(ref backend) = self.compaction_backend {
                            // Use LLM compaction
                            let result = compact_result(
                                response_text,
                                backend,
                                &self.compaction_config.model,
                                self.compaction_config.target_len,
                            )
                            .await;

                            if result.success {
                                (result.text, false, true, Some(result.original_len))
                            } else {
                                // Compaction failed, fall back to truncation
                                let truncated =
                                    truncate_result(response_text, DEFAULT_MAX_RESULT_LEN);
                                (
                                    truncated.text,
                                    truncated.truncated,
                                    false,
                                    truncated.original_len,
                                )
                            }
                        } else {
                            // No compaction backend, fall back to truncation
                            let truncated = truncate_result(response_text, DEFAULT_MAX_RESULT_LEN);
                            (
                                truncated.text,
                                truncated.truncated,
                                false,
                                truncated.original_len,
                            )
                        }
                    } else {
                        // Compaction disabled, use truncation
                        let truncated = truncate_result(response_text, DEFAULT_MAX_RESULT_LEN);
                        (
                            truncated.text,
                            truncated.truncated,
                            false,
                            truncated.original_len,
                        )
                    }
                } else {
                    // Result is within limits, no processing needed
                    (response_text.clone(), false, false, None)
                };

                DelegationOutcome::Success(SubagentResult {
                    text: final_text,
                    success: !response.truncated,
                    turns: response.iterations as usize,
                    duration_ms,
                    truncated,
                    compacted,
                    original_len,
                })
            }
            Err(e) => DelegationOutcome::Error {
                message: format!("Agent '{}' execution failed: {}", agent_name, e),
            },
        }
    }

    async fn delegate_background(
        &self,
        agent_name: &str,
        task: &str,
        context: Option<&str>,
        parent_session_id: &str,
    ) -> std::result::Result<(), String> {
        use arawn_agent::types::Session;

        // Verify the agent exists
        let config = self
            .agent_configs
            .get(agent_name)
            .ok_or_else(|| format!("Unknown agent: {}", agent_name))?;

        // Spawn the agent
        let agent = self
            .spawner
            .spawn(config)
            .map_err(|e| format!("Failed to spawn agent '{}': {}", agent_name, e))?;

        // Build context preamble if context is provided
        let context_preamble = context.map(|ctx| {
            let truncated = truncate_context(ctx, DEFAULT_MAX_CONTEXT_LEN);
            format!("## Context from parent session\n\n{}", truncated)
        });

        let task_owned = task.to_string();
        let agent_name_owned = agent_name.to_string();
        let parent_id = parent_session_id.to_string();
        let hook_dispatcher = self.hook_dispatcher.clone();

        // Create task preview for events (truncate to reasonable length)
        let task_preview = if task.len() > 200 {
            format!("{}...", &task[..197])
        } else {
            task.to_string()
        };

        // Spawn background task
        tokio::spawn(async move {
            // Fire SubagentStarted event
            if let Some(ref dispatcher) = hook_dispatcher {
                dispatcher
                    .dispatch_subagent_started(&parent_id, &agent_name_owned, &task_preview)
                    .await;
            }

            let start = Instant::now();
            let mut session = Session::new();

            // Set context preamble if provided
            if let Some(preamble) = context_preamble {
                session.set_context_preamble(preamble);
            }

            let result = agent.turn(&mut session, &task_owned).await;
            let duration_ms = start.elapsed().as_millis() as u64;

            match result {
                Ok(response) => {
                    tracing::info!(
                        agent = %agent_name_owned,
                        parent_session = %parent_id,
                        iterations = response.iterations,
                        duration_ms = duration_ms,
                        "background subagent completed successfully"
                    );

                    // Fire SubagentCompleted event (success)
                    if let Some(ref dispatcher) = hook_dispatcher {
                        let result_preview = if response.text.len() > 200 {
                            format!("{}...", &response.text[..197])
                        } else {
                            response.text.clone()
                        };
                        dispatcher
                            .dispatch_subagent_completed(
                                &parent_id,
                                &agent_name_owned,
                                &result_preview,
                                duration_ms,
                                true,
                            )
                            .await;
                    }
                }
                Err(e) => {
                    tracing::error!(
                        agent = %agent_name_owned,
                        parent_session = %parent_id,
                        error = %e,
                        "background subagent failed"
                    );

                    // Fire SubagentCompleted event (failure)
                    if let Some(ref dispatcher) = hook_dispatcher {
                        dispatcher
                            .dispatch_subagent_completed(
                                &parent_id,
                                &agent_name_owned,
                                &e.to_string(),
                                duration_ms,
                                false,
                            )
                            .await;
                    }
                }
            }
        });

        tracing::info!(
            agent = %agent_name,
            parent_session = %parent_session_id,
            "background subagent spawned"
        );

        Ok(())
    }

    async fn has_agent(&self, name: &str) -> bool {
        self.agent_configs.contains_key(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arawn_agent::tool::{Tool, ToolContext, ToolResult};
    use arawn_llm::{MockBackend, SharedBackend};
    use async_trait::async_trait;

    /// A simple test tool.
    struct TestTool {
        tool_name: String,
    }

    impl TestTool {
        fn new(name: &str) -> Self {
            Self {
                tool_name: name.to_string(),
            }
        }
    }

    #[async_trait]
    impl Tool for TestTool {
        fn name(&self) -> &str {
            &self.tool_name
        }
        fn description(&self) -> &str {
            "test tool"
        }
        fn parameters(&self) -> serde_json::Value {
            serde_json::json!({"type": "object"})
        }
        async fn execute(
            &self,
            _params: serde_json::Value,
            _ctx: &ToolContext,
        ) -> arawn_agent::error::Result<ToolResult> {
            Ok(ToolResult::Text {
                content: "ok".to_string(),
            })
        }
    }

    fn make_parent_tools() -> Arc<ToolRegistry> {
        let mut registry = ToolRegistry::new();
        registry.register(TestTool::new("shell"));
        registry.register(TestTool::new("file_read"));
        registry.register(TestTool::new("journal"));
        Arc::new(registry)
    }

    fn make_agent_config(
        name: &str,
        tools: Vec<&str>,
        max_iter: Option<usize>,
    ) -> PluginAgentConfig {
        use crate::types::{AgentConstraints, AgentSection, AgentSystemPrompt};

        PluginAgentConfig {
            agent: AgentSection {
                name: name.to_string(),
                description: format!("Test agent: {}", name),
                model: None,
                system_prompt: Some(AgentSystemPrompt {
                    text: format!("You are the {} agent.", name),
                }),
                constraints: Some(AgentConstraints {
                    tools: tools.into_iter().map(|s| s.to_string()).collect(),
                    max_iterations: max_iter,
                }),
            },
        }
    }

    #[test]
    fn test_spawn_agent_with_constrained_tools() {
        let parent_tools = make_parent_tools();
        let backend: SharedBackend = Arc::new(MockBackend::with_text("test"));

        let spawner = AgentSpawner::new(parent_tools, backend);
        let config = make_agent_config("reviewer", vec!["shell", "file_read"], None);

        let agent = spawner.spawn(&config).unwrap();
        let tool_names = agent.tools().names();
        assert!(tool_names.contains(&"shell"));
        assert!(tool_names.contains(&"file_read"));
        assert!(!tool_names.contains(&"journal"));
    }

    #[test]
    fn test_spawn_agent_missing_tool_skipped() {
        let parent_tools = make_parent_tools();
        let backend: SharedBackend = Arc::new(MockBackend::with_text("test"));

        let spawner = AgentSpawner::new(parent_tools, backend);
        let config = make_agent_config("test", vec!["shell", "nonexistent_tool"], None);

        let agent = spawner.spawn(&config).unwrap();
        let tool_names = agent.tools().names();
        assert!(tool_names.contains(&"shell"));
        assert!(!tool_names.contains(&"nonexistent_tool"));
    }

    #[test]
    fn test_spawn_agent_max_iterations() {
        let parent_tools = make_parent_tools();
        let backend: SharedBackend = Arc::new(MockBackend::with_text("test"));

        let spawner = AgentSpawner::new(parent_tools, backend);
        let config = make_agent_config("limited", vec!["shell"], Some(5));

        let agent = spawner.spawn(&config).unwrap();
        assert_eq!(agent.config().max_iterations, 5);
    }

    #[test]
    fn test_spawn_agent_system_prompt() {
        let parent_tools = make_parent_tools();
        let backend: SharedBackend = Arc::new(MockBackend::with_text("test"));

        let spawner = AgentSpawner::new(parent_tools, backend);
        let config = make_agent_config("prompted", vec![], None);

        let agent = spawner.spawn(&config).unwrap();
        let prompt = agent.config().system_prompt.as_deref().unwrap();
        assert!(prompt.contains("prompted agent"));
    }

    #[test]
    fn test_spawn_agent_no_constraints() {
        let parent_tools = make_parent_tools();
        let backend: SharedBackend = Arc::new(MockBackend::with_text("test"));

        let spawner = AgentSpawner::new(parent_tools, backend);
        let config = PluginAgentConfig {
            agent: crate::types::AgentSection {
                name: "open".to_string(),
                description: "No constraints".to_string(),
                model: None,
                system_prompt: None,
                constraints: None,
            },
        };

        let agent = spawner.spawn(&config).unwrap();
        // No constraints means no tools
        assert!(agent.tools().names().is_empty());
    }

    // ── PluginSubagentSpawner Tests ─────────────────────────────────────

    #[tokio::test]
    async fn test_plugin_subagent_spawner_list_agents() {
        let parent_tools = make_parent_tools();
        let backend: SharedBackend = Arc::new(MockBackend::with_text("test"));

        let mut configs = HashMap::new();
        configs.insert(
            "researcher".to_string(),
            make_agent_config("researcher", vec!["shell"], None),
        );
        configs.insert(
            "reviewer".to_string(),
            make_agent_config("reviewer", vec!["file_read"], None),
        );

        let mut sources = HashMap::new();
        sources.insert("researcher".to_string(), "test-plugin".to_string());

        let spawner = PluginSubagentSpawner::with_sources(parent_tools, backend, configs, sources);

        let agents = spawner.list_agents().await;
        assert_eq!(agents.len(), 2);

        let researcher = agents.iter().find(|a| a.name == "researcher").unwrap();
        assert_eq!(researcher.description, "Test agent: researcher");
        assert_eq!(researcher.tools, vec!["shell"]);
        assert_eq!(researcher.source.as_deref(), Some("test-plugin"));

        let reviewer = agents.iter().find(|a| a.name == "reviewer").unwrap();
        assert_eq!(reviewer.description, "Test agent: reviewer");
        assert!(reviewer.source.is_none());
    }

    #[tokio::test]
    async fn test_plugin_subagent_spawner_has_agent() {
        let parent_tools = make_parent_tools();
        let backend: SharedBackend = Arc::new(MockBackend::with_text("test"));

        let mut configs = HashMap::new();
        configs.insert(
            "researcher".to_string(),
            make_agent_config("researcher", vec!["shell"], None),
        );

        let spawner = PluginSubagentSpawner::new(parent_tools, backend, configs);

        assert!(spawner.has_agent("researcher").await);
        assert!(!spawner.has_agent("nonexistent").await);
    }

    #[tokio::test]
    async fn test_plugin_subagent_spawner_delegate_unknown_agent() {
        let parent_tools = make_parent_tools();
        let backend: SharedBackend = Arc::new(MockBackend::with_text("test"));

        let spawner = PluginSubagentSpawner::new(parent_tools, backend, HashMap::new());

        let outcome = spawner.delegate("unknown", "task", None, None).await;

        match outcome {
            DelegationOutcome::UnknownAgent { name, available } => {
                assert_eq!(name, "unknown");
                assert!(available.is_empty());
            }
            _ => panic!("Expected UnknownAgent outcome"),
        }
    }

    #[test]
    fn test_plugin_subagent_spawner_agent_count() {
        let parent_tools = make_parent_tools();
        let backend: SharedBackend = Arc::new(MockBackend::with_text("test"));

        let mut configs = HashMap::new();
        configs.insert("a".to_string(), make_agent_config("a", vec![], None));
        configs.insert("b".to_string(), make_agent_config("b", vec![], None));

        let spawner = PluginSubagentSpawner::new(parent_tools, backend, configs);

        assert_eq!(spawner.agent_count(), 2);
        assert!(!spawner.is_empty());

        let names = spawner.agent_names();
        assert!(names.contains(&"a"));
        assert!(names.contains(&"b"));
    }

    #[test]
    fn test_plugin_subagent_spawner_empty() {
        let parent_tools = make_parent_tools();
        let backend: SharedBackend = Arc::new(MockBackend::with_text("test"));

        let spawner = PluginSubagentSpawner::new(parent_tools, backend, HashMap::new());

        assert_eq!(spawner.agent_count(), 0);
        assert!(spawner.is_empty());
        assert!(spawner.agent_names().is_empty());
    }

    // ── Context Truncation Tests ──────────────────────────────────────────

    #[test]
    fn test_truncate_context_short() {
        let context = "Short context";
        let result = super::truncate_context(context, 100);
        assert_eq!(result, "Short context");
    }

    #[test]
    fn test_truncate_context_exact_limit() {
        let context = "x".repeat(100);
        let result = super::truncate_context(&context, 100);
        assert_eq!(result, context);
    }

    #[test]
    fn test_truncate_context_over_limit() {
        let context = "word1 word2 word3 word4 word5";
        let result = super::truncate_context(context, 15);
        // Should truncate at word boundary
        assert!(result.ends_with("...(truncated)"));
        assert!(result.len() < context.len() + 14); // 14 for "...(truncated)"
    }

    #[test]
    fn test_truncate_context_word_boundary() {
        let context = "The quick brown fox jumps over the lazy dog";
        let result = super::truncate_context(context, 20);
        // Should truncate at a space, not mid-word
        assert!(result.starts_with("The quick brown"));
        assert!(result.ends_with("...(truncated)"));
    }

    #[test]
    fn test_truncate_context_no_spaces() {
        let context = "abcdefghijklmnopqrstuvwxyz";
        let result = super::truncate_context(context, 10);
        // No word boundary, truncates at exact limit
        assert_eq!(result, "abcdefghij...(truncated)");
    }

    // ── Result Truncation Tests ──────────────────────────────────────────

    #[test]
    fn test_truncate_result_short() {
        let text = "Short result";
        let result = super::truncate_result(text, 100);
        assert_eq!(result.text, "Short result");
        assert!(!result.truncated);
        assert!(result.original_len.is_none());
    }

    #[test]
    fn test_truncate_result_exact_limit() {
        let text = "x".repeat(100);
        let result = super::truncate_result(&text, 100);
        assert_eq!(result.text, text);
        assert!(!result.truncated);
        assert!(result.original_len.is_none());
    }

    #[test]
    fn test_truncate_result_over_limit() {
        // Create a long text that will be truncated
        let text = "Beginning section. ".repeat(100) + &"Ending section. ".repeat(100);
        let result = super::truncate_result(&text, 500);

        assert!(result.truncated);
        assert_eq!(result.original_len, Some(text.len()));
        assert!(result.text.len() <= 500);
        // Should contain the omission notice
        assert!(result.text.contains("characters omitted"));
        // Should preserve beginning
        assert!(result.text.starts_with("Beginning"));
        // Should preserve end
        assert!(result.text.ends_with("section. "));
    }

    #[test]
    fn test_truncate_result_preserves_beginning_and_end() {
        let beginning = "START_MARKER ".repeat(50);
        let middle = "middle ".repeat(500);
        let ending = "END_MARKER ".repeat(50);
        let text = format!("{}{}{}", beginning, middle, ending);

        let result = super::truncate_result(&text, 1000);

        assert!(result.truncated);
        // Should have beginning marker
        assert!(result.text.contains("START_MARKER"));
        // Should have ending marker
        assert!(result.text.contains("END_MARKER"));
        // Middle should be truncated
        assert!(!result.text.contains(&middle));
    }

    #[test]
    fn test_truncate_result_metadata() {
        let text = "x".repeat(10000);
        let result = super::truncate_result(&text, 1000);

        assert!(result.truncated);
        assert_eq!(result.original_len, Some(10000));
        // Verify the omission notice contains the right count
        let omitted = 10000 - 1000;
        assert!(
            result
                .text
                .contains(&format!("{} characters omitted", omitted))
        );
    }

    #[test]
    fn test_truncate_result_word_boundaries() {
        let text = "The quick brown fox jumps over the lazy dog. ".repeat(100);
        let result = super::truncate_result(&text, 500);

        assert!(result.truncated);
        // Should not end mid-word in beginning section (before the notice)
        let notice_pos = result.text.find("characters omitted").unwrap();
        let before_notice = &result.text[..notice_pos];
        // The beginning section should end with whitespace (word boundary)
        let trimmed = before_notice.trim_end_matches(&['\n', '[', '.'][..]);
        assert!(
            trimmed.ends_with(' ') || trimmed.ends_with('.'),
            "Beginning should end at word boundary: {:?}",
            &trimmed[trimmed.len().saturating_sub(20)..]
        );
    }

    // ── Compaction Tests ─────────────────────────────────────────────────

    #[tokio::test]
    async fn test_compact_result_success() {
        use crate::agent_spawner::compact_result;
        use arawn_llm::MockBackend;

        // Create a mock backend that returns a summarized response
        let backend: SharedBackend = Arc::new(MockBackend::with_text(
            "## Summary\n- Key point 1\n- Key point 2",
        ));

        let long_text = "This is a very long text. ".repeat(500);
        let result = compact_result(&long_text, &backend, "test-model", 1000).await;

        assert!(result.success);
        assert_eq!(result.original_len, long_text.len());
        assert!(result.text.contains("Key point"));
    }

    #[test]
    fn test_compaction_config_default() {
        let config = CompactionConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.threshold, 8000);
        assert_eq!(config.backend, "default");
        assert_eq!(config.model, "gpt-4o-mini");
        assert_eq!(config.target_len, 4000);
    }

    #[test]
    fn test_spawner_with_compaction() {
        let parent_tools = make_parent_tools();
        let backend: SharedBackend = Arc::new(MockBackend::with_text("test"));
        let compaction_backend: SharedBackend = Arc::new(MockBackend::with_text("summary"));

        let config = CompactionConfig {
            enabled: true,
            threshold: 1000,
            backend: "fast".to_string(),
            model: "gpt-4o-mini".to_string(),
            target_len: 500,
        };

        let spawner = PluginSubagentSpawner::new(parent_tools, backend, HashMap::new())
            .with_compaction(compaction_backend, config);

        assert!(spawner.compaction_backend.is_some());
        assert!(spawner.compaction_config.enabled);
        assert_eq!(spawner.compaction_config.threshold, 1000);
        assert_eq!(spawner.compaction_config.model, "gpt-4o-mini");
    }
}
