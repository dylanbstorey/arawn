//! Core Agent implementation.
//!
//! The [`Agent`] struct is the brain of the system - it orchestrates the
//! conversation loop, handles tool execution, and manages context.

use std::sync::Arc;
use std::time::Instant;

use arawn_llm::{
    CompletionRequest, CompletionResponse, ContentBlock, LlmBackend, Message, SharedBackend,
    SharedEmbedder, ToolResultBlock,
    interaction_log::{InteractionLogger, InteractionRecord},
};
use arawn_memory::store::{MemoryStore, RecallQuery};
use arawn_types::{HookOutcome, SharedHookDispatcher};
use tokio_util::sync::CancellationToken;

use crate::stream::{AgentStream, create_turn_stream};

use crate::context::estimate_tokens;
use crate::error::{AgentError, Result};
use crate::prompt::SystemPromptBuilder;
use crate::tool::{ToolContext, ToolRegistry, ToolResult};
use crate::types::{
    AgentConfig, AgentResponse, ResponseUsage, Session, ToolCall, ToolResultRecord,
};

// ─────────────────────────────────────────────────────────────────────────────
// Recall Configuration
// ─────────────────────────────────────────────────────────────────────────────

/// Configuration for active recall behavior in the agent turn loop.
#[derive(Debug, Clone)]
pub struct RecallConfig {
    /// Whether active recall is enabled.
    pub enabled: bool,
    /// Minimum similarity score threshold (0.0–1.0).
    pub threshold: f32,
    /// Maximum number of memories to recall per turn.
    pub limit: usize,
}

impl Default for RecallConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            threshold: 0.6,
            limit: 5,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Agent
// ─────────────────────────────────────────────────────────────────────────────

/// The core agent that orchestrates LLM calls and tool execution.
pub struct Agent {
    /// LLM backend for completions.
    backend: SharedBackend,
    /// Registered tools.
    tools: Arc<ToolRegistry>,
    /// Agent configuration.
    config: AgentConfig,
    /// Optional interaction logger for structured JSONL capture.
    interaction_logger: Option<Arc<InteractionLogger>>,
    /// Optional memory store for active recall.
    memory_store: Option<Arc<MemoryStore>>,
    /// Optional embedder for computing query embeddings.
    embedder: Option<SharedEmbedder>,
    /// Active recall configuration.
    recall_config: RecallConfig,
    /// Optional hook dispatcher for plugin lifecycle events.
    hook_dispatcher: Option<SharedHookDispatcher>,
}

impl Agent {
    /// Create a new agent with the given backend and tools.
    pub fn new(backend: SharedBackend, tools: ToolRegistry, config: AgentConfig) -> Self {
        Self {
            backend,
            tools: Arc::new(tools),
            config,
            interaction_logger: None,
            memory_store: None,
            embedder: None,
            recall_config: RecallConfig::default(),
            hook_dispatcher: None,
        }
    }

    /// Create an agent builder for fluent construction.
    pub fn builder() -> AgentBuilder {
        AgentBuilder::new()
    }

    /// Get the agent configuration.
    pub fn config(&self) -> &AgentConfig {
        &self.config
    }

    /// Get the tool registry.
    pub fn tools(&self) -> &ToolRegistry {
        &self.tools
    }

    /// Get the LLM backend.
    pub fn backend(&self) -> SharedBackend {
        self.backend.clone()
    }

    /// Execute a single turn of conversation.
    ///
    /// Takes a user message, potentially executes multiple tool calls,
    /// and returns the final response.
    pub async fn turn(&self, session: &mut Session, user_message: &str) -> Result<AgentResponse> {
        // Start a new turn
        let turn = session.start_turn(user_message);
        let turn_id = turn.id;
        let session_id = session.id;

        tracing::info!(
            %session_id,
            %turn_id,
            message_len = user_message.len(),
            "Turn started"
        );

        // Build initial messages from session history
        let mut messages = self.build_messages(session);

        // Log initial context size
        let initial_context_tokens = self.estimate_messages_tokens(&messages);
        tracing::debug!(
            %session_id,
            %turn_id,
            message_count = messages.len(),
            estimated_tokens = initial_context_tokens,
            "Context: initial history loaded"
        );

        // Active recall: inject relevant memories before first LLM call
        if let Some(context_msg) = self.perform_recall(user_message).await {
            // Insert as second message (after first user message, or at start)
            let insert_pos = 1.min(messages.len());
            messages.insert(insert_pos, context_msg);
        }

        // Track usage and iterations
        let mut total_input_tokens = 0u32;
        let mut total_output_tokens = 0u32;
        let mut iterations = 0u32;
        let mut all_tool_calls = Vec::new();
        let mut all_tool_results = Vec::new();

        // Tool execution loop
        loop {
            iterations += 1;

            if iterations > self.config.max_iterations {
                tracing::warn!(%session_id, %turn_id, iterations, "Max iterations exceeded");
                // Mark turn as truncated
                let turn = session.current_turn_mut().unwrap();
                turn.complete("[Response truncated: max iterations exceeded]");
                turn.tool_calls = all_tool_calls.clone();
                turn.tool_results = all_tool_results.clone();

                return Ok(AgentResponse {
                    text: "[Response truncated: max iterations exceeded]".to_string(),
                    tool_calls: all_tool_calls,
                    tool_results: all_tool_results,
                    iterations,
                    usage: ResponseUsage::new(total_input_tokens, total_output_tokens),
                    truncated: true,
                });
            }

            // Build completion request
            let request = self.build_request(&messages, session.context_preamble());

            tracing::debug!(
                %session_id,
                iteration = iterations,
                messages = messages.len(),
                tools = self.tools.names().len(),
                model = %request.model,
                "Calling LLM"
            );

            // Call LLM with timing
            let call_start = Instant::now();
            let response = match self.backend.complete(request.clone()).await {
                Ok(r) => r,
                Err(e) => {
                    // Check if this is a tool validation error (LLM hallucinated a tool name)
                    // If so, inject feedback and retry instead of failing
                    if e.is_tool_validation_error() {
                        let invalid_tool = e.invalid_tool_name().unwrap_or("unknown");
                        let available_tools = self.tools.names().join(", ");

                        tracing::warn!(
                            %session_id,
                            %turn_id,
                            iteration = iterations,
                            invalid_tool = %invalid_tool,
                            "Tool validation error - injecting feedback and retrying"
                        );

                        // Add feedback as a user message so the LLM can correct itself
                        let feedback = format!(
                            "Error: The tool '{}' does not exist. Available tools are: {}. Please use the exact tool name from this list.",
                            invalid_tool, available_tools
                        );
                        messages.push(Message::user(feedback));

                        // Continue to retry (counts against iteration limit)
                        continue;
                    }

                    tracing::error!(%session_id, %turn_id, iteration = iterations, error = %e, "LLM call failed");
                    return Err(e.into());
                }
            };
            let duration_ms = call_start.elapsed().as_millis() as u64;

            // Update usage
            total_input_tokens += response.usage.input_tokens;
            total_output_tokens += response.usage.output_tokens;

            tracing::debug!(
                %session_id,
                iteration = iterations,
                input_tokens = response.usage.input_tokens,
                output_tokens = response.usage.output_tokens,
                stop_reason = ?response.stop_reason,
                has_tool_use = response.has_tool_use(),
                duration_ms,
                "LLM response received"
            );

            // Write structured interaction record
            if let Some(ref logger) = self.interaction_logger {
                let record = InteractionRecord::from_exchange(&request, &response, duration_ms);
                if let Err(e) = logger.log(&record) {
                    tracing::warn!(error = %e, "Failed to write interaction log");
                }
            }

            // Check for tool use
            if response.has_tool_use() {
                let tool_uses = response.tool_uses();
                tracing::info!(
                    %session_id,
                    iteration = iterations,
                    tool_count = tool_uses.len(),
                    tools = %tool_uses.iter().map(|t| t.name.as_str()).collect::<Vec<_>>().join(", "),
                    "Executing tools"
                );

                // Execute tools
                let (tool_calls, tool_results) =
                    self.execute_tools(&response, session_id, turn_id).await?;

                // Record tool calls and results
                all_tool_calls.extend(tool_calls.clone());
                all_tool_results.extend(tool_results.clone());

                // Add assistant message with tool calls to history
                messages.push(Message::assistant_blocks(response.content.clone()));

                // Add tool results to history
                let tool_result_blocks: Vec<ToolResultBlock> = tool_results
                    .iter()
                    .map(|r| {
                        if r.success {
                            ToolResultBlock::success(&r.tool_call_id, &r.content)
                        } else {
                            ToolResultBlock::error(&r.tool_call_id, &r.content)
                        }
                    })
                    .collect();

                messages.push(Message::tool_results(tool_result_blocks));

                // Log context size after adding tool results
                let context_tokens = self.estimate_messages_tokens(&messages);
                tracing::debug!(
                    %session_id,
                    iteration = iterations,
                    message_count = messages.len(),
                    estimated_tokens = context_tokens,
                    "Context: after tool results"
                );

                // Continue loop for next LLM call
                continue;
            }

            // No tool use - we have the final response
            let text = response.text();

            tracing::info!(
                %session_id,
                %turn_id,
                iterations,
                total_input_tokens,
                total_output_tokens,
                tool_calls = all_tool_calls.len(),
                response_len = text.len(),
                "Turn completed"
            );

            // Complete the turn
            let turn = session.current_turn_mut().unwrap();
            turn.complete(&text);
            turn.tool_calls = all_tool_calls.clone();
            turn.tool_results = all_tool_results.clone();

            return Ok(AgentResponse {
                text,
                tool_calls: all_tool_calls,
                tool_results: all_tool_results,
                iterations,
                usage: ResponseUsage::new(total_input_tokens, total_output_tokens),
                truncated: false,
            });
        }
    }

    /// Execute a single turn of conversation with streaming output.
    ///
    /// Returns a stream of chunks that yield text deltas, tool execution events,
    /// and completion notifications as they occur.
    ///
    /// # Arguments
    /// * `session` - The session to operate on
    /// * `user_message` - The user's message
    /// * `cancellation` - Token to cancel the operation
    ///
    /// # Returns
    /// A stream of `StreamChunk` items
    pub fn turn_stream(
        &self,
        session: &mut Session,
        user_message: &str,
        cancellation: CancellationToken,
    ) -> AgentStream {
        // Start a new turn
        let turn = session.start_turn(user_message);
        let turn_id = turn.id;
        let session_id = session.id;

        // Build initial messages from session history
        let messages = self.build_messages(session);

        create_turn_stream(
            self.backend.clone(),
            self.tools.clone(),
            self.config.clone(),
            messages,
            session_id,
            turn_id,
            cancellation,
        )
    }

    /// Estimate total tokens for a list of messages.
    fn estimate_messages_tokens(&self, messages: &[Message]) -> usize {
        messages
            .iter()
            .map(|m| self.estimate_message_tokens(m))
            .sum()
    }

    /// Estimate tokens for a single message.
    fn estimate_message_tokens(&self, message: &Message) -> usize {
        // Base overhead for message structure
        let mut tokens = 10;

        // Add content tokens
        for block in message.content.blocks() {
            tokens += match block {
                ContentBlock::Text { text, .. } => estimate_tokens(&text),
                ContentBlock::ToolUse { name, input, .. } => {
                    estimate_tokens(&name) + estimate_tokens(&input.to_string())
                }
                ContentBlock::ToolResult { content, .. } => {
                    if let Some(c) = content {
                        match c {
                            arawn_llm::ToolResultContent::Text(text) => estimate_tokens(&text),
                            arawn_llm::ToolResultContent::Blocks(blocks) => {
                                estimate_tokens(&serde_json::to_string(&blocks).unwrap_or_default())
                            }
                        }
                    } else {
                        0
                    }
                }
            };
        }

        tokens
    }

    /// Build messages from session history.
    fn build_messages(&self, session: &Session) -> Vec<Message> {
        let mut messages = Vec::new();

        // Add previous turns (excluding current incomplete turn)
        for turn in session.all_turns() {
            // Skip if this turn has no response (current turn)
            if turn.assistant_response.is_none() && turn.tool_calls.is_empty() {
                // This is the current turn - add just the user message
                messages.push(Message::user(&turn.user_message));
                continue;
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
        }

        messages
    }

    /// Build a completion request.
    ///
    /// # Arguments
    /// * `messages` - The conversation messages
    /// * `context_preamble` - Optional session context to prepend to the system prompt
    fn build_request(
        &self,
        messages: &[Message],
        context_preamble: Option<&str>,
    ) -> CompletionRequest {
        let mut request = CompletionRequest::new(
            &self.config.model,
            messages.to_vec(),
            self.config.max_tokens,
        );

        // Build system prompt with optional context preamble
        let system_prompt = match (&self.config.system_prompt, context_preamble) {
            (Some(prompt), Some(preamble)) => {
                // Prepend context preamble to system prompt
                Some(format!(
                    "[Session Context]\n{}\n\n---\n\n{}",
                    preamble, prompt
                ))
            }
            (Some(prompt), None) => Some(prompt.clone()),
            (None, Some(preamble)) => {
                // Context preamble only, no base system prompt
                Some(format!("[Session Context]\n{}", preamble))
            }
            (None, None) => None,
        };

        if let Some(ref prompt) = system_prompt {
            request = request.with_system(prompt);
        }

        // Add temperature
        if let Some(temp) = self.config.temperature {
            request = request.with_temperature(temp);
        }

        // Add tools
        let tool_defs = self.tools.to_llm_definitions();
        if !tool_defs.is_empty() {
            request = request.with_tools(tool_defs);
        }

        request
    }

    /// Execute tool calls from an LLM response.
    async fn execute_tools(
        &self,
        response: &CompletionResponse,
        session_id: crate::types::SessionId,
        turn_id: crate::types::TurnId,
    ) -> Result<(Vec<ToolCall>, Vec<ToolResultRecord>)> {
        let mut tool_calls = Vec::new();
        let mut tool_results = Vec::new();

        let ctx = ToolContext::new(session_id, turn_id);

        for tool_use in response.tool_uses() {
            let tool_call = ToolCall {
                id: tool_use.id.clone(),
                name: tool_use.name.clone(),
                arguments: tool_use.input.clone(),
            };
            tool_calls.push(tool_call);

            // Pre-tool hook: can block tool execution
            if let Some(ref dispatcher) = self.hook_dispatcher {
                match dispatcher
                    .dispatch_pre_tool_use(&tool_use.name, &tool_use.input)
                    .await
                {
                    HookOutcome::Block { reason } => {
                        tracing::info!(
                            tool = %tool_use.name,
                            reason = %reason,
                            "Tool blocked by hook"
                        );
                        tool_results.push(ToolResultRecord {
                            tool_call_id: tool_use.id.clone(),
                            success: false,
                            content: format!("Blocked by hook: {}", reason),
                        });
                        continue;
                    }
                    HookOutcome::Allow | HookOutcome::Info { .. } => {
                        // Proceed with tool execution
                    }
                }
            }

            // Log tool input
            let input_str = tool_use.input.to_string();
            let input_bytes = input_str.len();
            tracing::debug!(
                tool = %tool_use.name,
                tool_call_id = %tool_use.id,
                input_bytes,
                input_tokens = estimate_tokens(&input_str),
                "Tool: executing"
            );

            // Execute the tool with per-tool output limits
            let output_config = self.tools.output_config_for(&tool_use.name);
            let result = match self
                .tools
                .execute_with_config(&tool_use.name, tool_use.input.clone(), &ctx, &output_config)
                .await
            {
                Ok(result) => result,
                Err(e) => {
                    tracing::warn!(
                        tool = %tool_use.name,
                        error = %e,
                        "Tool execution failed"
                    );
                    ToolResult::error(e.to_string())
                }
            };

            // Log tool output size
            let output_content = result.to_llm_content();
            let output_bytes = output_content.len();
            let output_tokens = estimate_tokens(&output_content);
            tracing::debug!(
                tool = %tool_use.name,
                tool_call_id = %tool_use.id,
                success = result.is_success(),
                output_bytes,
                output_tokens,
                "Tool: completed"
            );

            // Post-tool hook: informational only
            if let Some(ref dispatcher) = self.hook_dispatcher {
                let result_json = serde_json::to_value(&result).unwrap_or_default();
                let _ = dispatcher
                    .dispatch_post_tool_use(&tool_use.name, &tool_use.input, &result_json)
                    .await;
            }

            tool_results.push(ToolResultRecord {
                tool_call_id: tool_use.id.clone(),
                success: result.is_success(),
                content: result.to_llm_content(),
            });
        }

        Ok((tool_calls, tool_results))
    }

    /// Perform active recall for a user message.
    ///
    /// Embeds the user message, queries the memory store, and returns
    /// a system message with relevant context if any matches are found.
    /// Returns `None` if recall is disabled, not configured, or finds nothing.
    async fn perform_recall(&self, user_message: &str) -> Option<Message> {
        // Guard: recall must be enabled
        if !self.recall_config.enabled {
            return None;
        }

        // Guard: need both memory store and embedder
        let store = self.memory_store.as_ref()?;
        let embedder = self.embedder.as_ref()?;

        // Guard: skip empty/whitespace messages
        if user_message.trim().is_empty() {
            return None;
        }

        // Guard: vectors must be initialized
        if !store.has_vectors() {
            return None;
        }

        // Embed the user message
        let embedding = match embedder.embed(user_message).await {
            Ok(emb) => emb,
            Err(e) => {
                tracing::debug!(error = %e, "Recall: embedding failed, skipping");
                return None;
            }
        };

        // Build recall query
        let query = RecallQuery::new(embedding)
            .with_limit(self.recall_config.limit)
            .with_min_score(self.recall_config.threshold);

        // Execute recall
        let result = match store.recall(query) {
            Ok(r) => r,
            Err(e) => {
                tracing::debug!(error = %e, "Recall: query failed, skipping");
                return None;
            }
        };

        if result.matches.is_empty() {
            return None;
        }

        let context = format_recall_context(&result.matches);

        tracing::debug!(
            matches = result.matches.len(),
            query_time_ms = result.query_time_ms,
            "Recall: injecting context"
        );

        Some(Message::user(format!(
            "[SYSTEM: Relevant memories recalled for context]\n{}",
            context
        )))
    }
}

/// Format recall matches into a concise context string for injection.
fn format_recall_context(matches: &[arawn_memory::store::RecallMatch]) -> String {
    let mut lines = Vec::new();
    for m in matches {
        let ts = m.memory.created_at.format("%Y-%m-%d %H:%M");
        lines.push(format!(
            "- [{}] ({:.0}%) {}",
            ts,
            m.score * 100.0,
            m.memory.content
        ));
    }
    lines.join("\n")
}

// ─────────────────────────────────────────────────────────────────────────────
// Agent Builder
// ─────────────────────────────────────────────────────────────────────────────

/// Builder for constructing an Agent with fluent API.
pub struct AgentBuilder {
    backend: Option<SharedBackend>,
    tools: ToolRegistry,
    config: AgentConfig,
    prompt_builder: Option<SystemPromptBuilder>,
    bootstrap_context: Option<crate::prompt::BootstrapContext>,
    interaction_logger: Option<Arc<InteractionLogger>>,
    memory_store: Option<Arc<MemoryStore>>,
    embedder: Option<SharedEmbedder>,
    recall_config: RecallConfig,
    plugin_prompts: Vec<(String, String)>,
    hook_dispatcher: Option<SharedHookDispatcher>,
}

impl AgentBuilder {
    /// Create a new builder with defaults.
    pub fn new() -> Self {
        Self {
            backend: None,
            tools: ToolRegistry::new(),
            config: AgentConfig::default(),
            prompt_builder: None,
            bootstrap_context: None,
            interaction_logger: None,
            memory_store: None,
            embedder: None,
            recall_config: RecallConfig::default(),
            plugin_prompts: Vec::new(),
            hook_dispatcher: None,
        }
    }

    /// Set the LLM backend.
    pub fn with_backend(mut self, backend: impl LlmBackend + 'static) -> Self {
        self.backend = Some(Arc::new(backend));
        self
    }

    /// Set the LLM backend from a shared reference.
    pub fn with_shared_backend(mut self, backend: SharedBackend) -> Self {
        self.backend = Some(backend);
        self
    }

    /// Set the tool registry.
    pub fn with_tools(mut self, tools: ToolRegistry) -> Self {
        self.tools = tools;
        self
    }

    /// Register a single tool.
    pub fn with_tool<T: crate::tool::Tool + 'static>(mut self, tool: T) -> Self {
        self.tools.register(tool);
        self
    }

    /// Set the configuration.
    pub fn with_config(mut self, config: AgentConfig) -> Self {
        self.config = config;
        self
    }

    /// Set the model.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.config.model = model.into();
        self
    }

    /// Set the system prompt.
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.config.system_prompt = Some(prompt.into());
        self
    }

    /// Set max tokens.
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.config.max_tokens = max_tokens;
        self
    }

    /// Set max iterations.
    pub fn with_max_iterations(mut self, max_iterations: u32) -> Self {
        self.config.max_iterations = max_iterations;
        self
    }

    /// Set the workspace path.
    ///
    /// The workspace is the root directory for file operations.
    pub fn with_workspace(mut self, path: impl Into<std::path::PathBuf>) -> Self {
        self.config.workspace_path = Some(path.into());
        self
    }

    /// Set a prompt builder for dynamic system prompt generation.
    ///
    /// When set, the builder will be used to generate the system prompt
    /// at build time, incorporating tools and other context.
    ///
    /// This takes precedence over `with_system_prompt()`.
    pub fn with_prompt_builder(mut self, builder: SystemPromptBuilder) -> Self {
        self.prompt_builder = Some(builder);
        self
    }

    /// Load bootstrap context files from a directory.
    ///
    /// Looks for BEHAVIOR.md, BOOTSTRAP.md, MEMORY.md, IDENTITY.md in the
    /// specified directory and adds them to the prompt.
    ///
    /// Can be combined with `with_prompt_file()` to add additional custom files.
    ///
    /// # Example
    /// ```rust,ignore
    /// let agent = Agent::builder()
    ///     .with_backend(backend)
    ///     .with_bootstrap_dir("/path/to/prompts")
    ///     .build()?;
    /// ```
    pub fn with_bootstrap_dir(mut self, path: impl AsRef<std::path::Path>) -> Self {
        use crate::prompt::BootstrapContext;

        match BootstrapContext::load(path.as_ref()) {
            Ok(context) if !context.is_empty() => {
                // Merge with existing context or set new one
                if let Some(ref mut existing) = self.bootstrap_context {
                    for file in context.files() {
                        existing.add_file(&file.filename, &file.content);
                    }
                } else {
                    self.bootstrap_context = Some(context);
                }
            }
            Ok(_) => {
                // Empty context, no files found - that's fine
            }
            Err(e) => {
                tracing::warn!(
                    path = %path.as_ref().display(),
                    error = %e,
                    "Failed to load bootstrap context"
                );
            }
        }
        self
    }

    /// Load a custom prompt file and add it to the bootstrap context.
    ///
    /// Use this for prompt files with non-standard names. Can be called
    /// multiple times to add multiple files. The file content will be
    /// added to the bootstrap context section of the prompt.
    ///
    /// # Example
    /// ```rust,ignore
    /// let agent = Agent::builder()
    ///     .with_backend(backend)
    ///     .with_prompt_file("/path/to/custom_persona.md")
    ///     .with_prompt_file("/path/to/guidelines.md")
    ///     .build()?;
    /// ```
    pub fn with_prompt_file(mut self, path: impl AsRef<std::path::Path>) -> Self {
        use crate::prompt::BootstrapContext;

        let path = path.as_ref();
        match std::fs::read_to_string(path) {
            Ok(content) => {
                let filename = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("custom.md")
                    .to_string();

                // Get or create bootstrap context and add file
                let context = self
                    .bootstrap_context
                    .get_or_insert_with(BootstrapContext::new);
                context.add_file(filename, content);
            }
            Err(e) => {
                tracing::warn!(
                    path = %path.display(),
                    error = %e,
                    "Failed to load prompt file"
                );
            }
        }
        self
    }

    /// Set the memory store for active recall.
    pub fn with_memory_store(mut self, store: Arc<MemoryStore>) -> Self {
        self.memory_store = Some(store);
        self
    }

    /// Set the embedder for active recall.
    pub fn with_embedder(mut self, embedder: SharedEmbedder) -> Self {
        self.embedder = Some(embedder);
        self
    }

    /// Set the recall configuration.
    pub fn with_recall_config(mut self, config: RecallConfig) -> Self {
        self.recall_config = config;
        self
    }

    /// Set the interaction logger for structured JSONL capture.
    pub fn with_interaction_logger(mut self, logger: Arc<InteractionLogger>) -> Self {
        self.interaction_logger = Some(logger);
        self
    }

    /// Add plugin prompt fragments to the system prompt.
    ///
    /// Each fragment is a `(plugin_name, prompt_text)` pair that will be
    /// appended as a `## Plugin: {name}` section in the system prompt.
    pub fn with_plugin_prompts(mut self, prompts: Vec<(String, String)>) -> Self {
        self.plugin_prompts = prompts;
        self
    }

    /// Set the hook dispatcher for plugin lifecycle events.
    ///
    /// The hook dispatcher fires hooks at lifecycle events like PreToolUse,
    /// PostToolUse, SessionStart, and SessionEnd. PreToolUse hooks can block
    /// tool execution.
    ///
    /// Accepts any type implementing `HookDispatch`, wrapped in an Arc.
    pub fn with_hook_dispatcher(mut self, dispatcher: SharedHookDispatcher) -> Self {
        self.hook_dispatcher = Some(dispatcher);
        self
    }

    /// Build the agent.
    pub fn build(mut self) -> Result<Agent> {
        let backend = self
            .backend
            .ok_or_else(|| AgentError::Config("LLM backend is required".to_string()))?;

        // If we have bootstrap context, a prompt builder, or plugin prompts, generate the system prompt
        if self.prompt_builder.is_some()
            || self.bootstrap_context.is_some()
            || !self.plugin_prompts.is_empty()
        {
            let builder = self.prompt_builder.take().unwrap_or_default();

            // Configure builder with tools and workspace
            let builder = builder.with_tools(&self.tools);
            let builder = if let Some(ref path) = self.config.workspace_path {
                builder.with_workspace(path)
            } else {
                builder
            };

            // Add bootstrap context if present
            let builder = if let Some(context) = self.bootstrap_context.take() {
                builder.with_bootstrap(context)
            } else {
                builder
            };

            // Add plugin prompt fragments
            let builder = if !self.plugin_prompts.is_empty() {
                builder.with_plugin_prompts(self.plugin_prompts)
            } else {
                builder
            };

            let system_prompt = builder.build();
            if !system_prompt.is_empty() {
                self.config.system_prompt = Some(system_prompt);
            }
        }

        let mut agent = Agent::new(backend, self.tools, self.config);
        agent.interaction_logger = self.interaction_logger;
        agent.memory_store = self.memory_store;
        agent.embedder = self.embedder;
        agent.recall_config = self.recall_config;
        agent.hook_dispatcher = self.hook_dispatcher;
        Ok(agent)
    }
}

impl Default for AgentBuilder {
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
    use crate::tool::MockTool;
    use arawn_llm::{ContentBlock, MockBackend, MockResponse, StopReason, Usage};

    fn mock_text_response(text: &str) -> CompletionResponse {
        CompletionResponse::new(
            "msg_1",
            "test-model",
            vec![ContentBlock::Text {
                text: text.to_string(),
                cache_control: None,
            }],
            StopReason::EndTurn,
            Usage::new(10, 20),
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
            Usage::new(10, 20),
        )
    }

    #[test]
    fn test_agent_builder_no_backend() {
        let result = Agent::builder().build();
        assert!(result.is_err());
    }

    #[test]
    fn test_agent_builder_with_backend() {
        let backend = MockBackend::with_text("Hello");
        let agent = Agent::builder()
            .with_backend(backend)
            .with_model("test-model")
            .with_system_prompt("You are helpful.")
            .build()
            .unwrap();

        assert_eq!(agent.config().model, "test-model");
        assert_eq!(
            agent.config().system_prompt,
            Some("You are helpful.".to_string())
        );
    }

    #[tokio::test]
    async fn test_simple_turn_no_tools() {
        let backend = MockBackend::with_text("Hello! How can I help?");
        let agent = Agent::builder().with_backend(backend).build().unwrap();

        let mut session = Session::new();
        let response = agent.turn(&mut session, "Hi there").await.unwrap();

        assert_eq!(response.text, "Hello! How can I help?");
        assert!(response.tool_calls.is_empty());
        assert!(!response.truncated);
        assert_eq!(response.iterations, 1);
        assert_eq!(session.turn_count(), 1);
    }

    #[tokio::test]
    async fn test_turn_with_tool_use() {
        // First response: tool call
        // Second response: final text
        let backend = MockBackend::new(vec![
            mock_tool_use_response("call_1", "test_tool", serde_json::json!({"arg": "value"})),
            mock_text_response("Done! I used the tool."),
        ]);

        let mut tools = ToolRegistry::new();
        tools.register(MockTool::new("test_tool").with_response(ToolResult::text("tool output")));

        let agent = Agent::builder()
            .with_backend(backend)
            .with_tools(tools)
            .build()
            .unwrap();

        let mut session = Session::new();
        let response = agent.turn(&mut session, "Use the tool").await.unwrap();

        assert_eq!(response.text, "Done! I used the tool.");
        assert_eq!(response.tool_calls.len(), 1);
        assert_eq!(response.tool_calls[0].name, "test_tool");
        assert_eq!(response.tool_results.len(), 1);
        assert!(response.tool_results[0].success);
        assert_eq!(response.iterations, 2);
    }

    #[tokio::test]
    async fn test_turn_max_iterations() {
        // Keep returning tool calls to hit max iterations
        let responses: Vec<CompletionResponse> = (0..20)
            .map(|i| {
                mock_tool_use_response(&format!("call_{}", i), "test_tool", serde_json::json!({}))
            })
            .collect();

        let backend = MockBackend::new(responses);

        let mut tools = ToolRegistry::new();
        tools.register(MockTool::new("test_tool"));

        let agent = Agent::builder()
            .with_backend(backend)
            .with_tools(tools)
            .with_max_iterations(5)
            .build()
            .unwrap();

        let mut session = Session::new();
        let response = agent.turn(&mut session, "Keep using tools").await.unwrap();

        assert!(response.truncated);
        assert_eq!(response.iterations, 6); // 5 + 1 that exceeded
    }

    #[tokio::test]
    async fn test_turn_tool_error_handling() {
        // First response: tool call
        // Second response: final text
        let backend = MockBackend::new(vec![
            mock_tool_use_response("call_1", "failing_tool", serde_json::json!({})),
            mock_text_response("I see the tool failed."),
        ]);

        let mut tools = ToolRegistry::new();
        tools.register(
            MockTool::new("failing_tool").with_response(ToolResult::error("Something went wrong")),
        );

        let agent = Agent::builder()
            .with_backend(backend)
            .with_tools(tools)
            .build()
            .unwrap();

        let mut session = Session::new();
        let response = agent
            .turn(&mut session, "Try the failing tool")
            .await
            .unwrap();

        assert_eq!(response.text, "I see the tool failed.");
        assert!(!response.tool_results[0].success);
        assert!(
            response.tool_results[0]
                .content
                .contains("Something went wrong")
        );
    }

    #[tokio::test]
    async fn test_turn_unknown_tool() {
        // Request a tool that doesn't exist
        let backend = MockBackend::new(vec![
            mock_tool_use_response("call_1", "nonexistent_tool", serde_json::json!({})),
            mock_text_response("I couldn't find that tool."),
        ]);

        let agent = Agent::builder().with_backend(backend).build().unwrap();

        let mut session = Session::new();
        let response = agent.turn(&mut session, "Use unknown tool").await.unwrap();

        assert!(!response.tool_results[0].success);
        assert!(response.tool_results[0].content.contains("not found"));
    }

    #[tokio::test]
    async fn test_tool_validation_error_retry() {
        // Test that when the backend returns a tool validation error (LLM hallucinated
        // a tool name), the agent injects feedback and retries instead of failing.
        let tool_validation_error = "tool call validation failed: attempted to call tool 'read_file' which was not in request.tools".to_string();

        let backend = MockBackend::with_results(vec![
            // First call: backend rejects with tool validation error
            MockResponse::Error(tool_validation_error),
            // Second call: LLM corrects itself and returns text
            MockResponse::Success(mock_text_response("I'll use the correct tool name.")),
        ]);

        let agent = Agent::builder().with_backend(backend).build().unwrap();

        let mut session = Session::new();
        let response = agent.turn(&mut session, "Read the file").await.unwrap();

        // Should succeed after retry
        assert_eq!(response.text, "I'll use the correct tool name.");
    }

    #[tokio::test]
    async fn test_tool_validation_error_exhausts_retries() {
        // Test that repeated tool validation errors eventually hit the iteration limit
        let tool_validation_error = "tool call validation failed: attempted to call tool 'bad_tool' which was not in request.tools".to_string();

        // Return errors for more iterations than max_iterations
        let errors: Vec<MockResponse> = (0..15)
            .map(|_| MockResponse::Error(tool_validation_error.clone()))
            .collect();

        let backend = MockBackend::with_results(errors);

        let agent = Agent::builder()
            .with_backend(backend)
            .with_max_iterations(3) // Low limit to speed up test
            .build()
            .unwrap();

        let mut session = Session::new();
        let result = agent.turn(&mut session, "Keep failing").await.unwrap();

        // Should hit max iterations and return truncated response
        assert!(result.text.contains("truncated") || result.text.contains("max iterations"));
    }

    #[tokio::test]
    async fn test_multi_turn_conversation() {
        let backend = MockBackend::new(vec![
            mock_text_response("Hello!"),
            mock_text_response("I'm doing great, thanks for asking!"),
        ]);

        let agent = Agent::builder().with_backend(backend).build().unwrap();

        let mut session = Session::new();

        let r1 = agent.turn(&mut session, "Hi").await.unwrap();
        assert_eq!(r1.text, "Hello!");
        assert_eq!(session.turn_count(), 1);

        let r2 = agent.turn(&mut session, "How are you?").await.unwrap();
        assert_eq!(r2.text, "I'm doing great, thanks for asking!");
        assert_eq!(session.turn_count(), 2);
    }

    #[test]
    fn test_agent_with_prompt_builder() {
        use crate::prompt::{PromptMode, SystemPromptBuilder};

        let backend = MockBackend::with_text("Hello");

        let prompt_builder = SystemPromptBuilder::new()
            .with_mode(PromptMode::Full)
            .with_identity("TestAgent", "a test assistant");

        let agent = Agent::builder()
            .with_backend(backend)
            .with_tool(MockTool::new("test_tool").with_description("A test tool"))
            .with_workspace("/test/workspace")
            .with_prompt_builder(prompt_builder)
            .build()
            .unwrap();

        // Verify the system prompt was generated
        let system_prompt = agent.config().system_prompt.as_ref().unwrap();
        assert!(system_prompt.contains("You are TestAgent"));
        assert!(system_prompt.contains("test assistant"));
        assert!(system_prompt.contains("test_tool"));
        assert!(system_prompt.contains("/test/workspace"));
    }

    #[test]
    fn test_agent_prompt_builder_with_static_fallback() {
        // When no prompt builder is set, system_prompt from config should be used
        let backend = MockBackend::with_text("Hello");

        let agent = Agent::builder()
            .with_backend(backend)
            .with_system_prompt("Static system prompt")
            .build()
            .unwrap();

        assert_eq!(
            agent.config().system_prompt,
            Some("Static system prompt".to_string())
        );
    }

    #[test]
    fn test_agent_prompt_builder_overrides_static() {
        use crate::prompt::{PromptMode, SystemPromptBuilder};

        let backend = MockBackend::with_text("Hello");

        let prompt_builder = SystemPromptBuilder::new()
            .with_mode(PromptMode::Full)
            .with_identity("Dynamic", "agent");

        // Set both static and builder - builder should win
        let agent = Agent::builder()
            .with_backend(backend)
            .with_system_prompt("This should be overridden")
            .with_prompt_builder(prompt_builder)
            .build()
            .unwrap();

        let system_prompt = agent.config().system_prompt.as_ref().unwrap();
        assert!(system_prompt.contains("You are Dynamic"));
        assert!(!system_prompt.contains("This should be overridden"));
    }

    #[test]
    fn test_agent_with_bootstrap_dir() {
        use crate::prompt::{PromptMode, SystemPromptBuilder};
        use std::fs;
        use tempfile::TempDir;

        let backend = MockBackend::with_text("Hello");

        // Create temp dir with a BEHAVIOR.md file
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("BEHAVIOR.md"),
            "# Soul\n\nYou are kind and helpful.",
        )
        .unwrap();

        let prompt_builder = SystemPromptBuilder::new()
            .with_mode(PromptMode::Full)
            .with_identity("BootstrapAgent", "an agent with soul");

        let agent = Agent::builder()
            .with_backend(backend)
            .with_prompt_builder(prompt_builder)
            .with_bootstrap_dir(temp_dir.path())
            .build()
            .unwrap();

        let system_prompt = agent.config().system_prompt.as_ref().unwrap();
        assert!(system_prompt.contains("You are BootstrapAgent"));
        assert!(system_prompt.contains("kind and helpful"));
        assert!(system_prompt.contains("BEHAVIOR.md"));
    }

    #[test]
    fn test_agent_bootstrap_dir_creates_builder_if_none() {
        use std::fs;
        use tempfile::TempDir;

        let backend = MockBackend::with_text("Hello");

        // Create temp dir with a BEHAVIOR.md file
        let temp_dir = TempDir::new().unwrap();
        fs::write(
            temp_dir.path().join("BEHAVIOR.md"),
            "Be excellent to each other.",
        )
        .unwrap();

        // No prompt builder set - bootstrap_dir should create one
        let agent = Agent::builder()
            .with_backend(backend)
            .with_bootstrap_dir(temp_dir.path())
            .build()
            .unwrap();

        let system_prompt = agent.config().system_prompt.as_ref().unwrap();
        assert!(system_prompt.contains("Be excellent"));
    }

    #[test]
    fn test_agent_bootstrap_dir_nonexistent_is_ok() {
        let backend = MockBackend::with_text("Hello");

        // Non-existent directory should not cause an error
        let agent = Agent::builder()
            .with_backend(backend)
            .with_bootstrap_dir("/nonexistent/path/to/prompts")
            .build()
            .unwrap();

        // Should build successfully, just with no bootstrap content
        assert!(agent.config().system_prompt.is_none());
    }

    #[test]
    fn test_agent_with_prompt_file() {
        use std::fs;
        use tempfile::TempDir;

        let backend = MockBackend::with_text("Hello");

        // Create temp file
        let temp_dir = TempDir::new().unwrap();
        let custom_file = temp_dir.path().join("custom_persona.md");
        fs::write(&custom_file, "You have a friendly personality.").unwrap();

        let agent = Agent::builder()
            .with_backend(backend)
            .with_prompt_file(&custom_file)
            .build()
            .unwrap();

        let system_prompt = agent.config().system_prompt.as_ref().unwrap();
        assert!(system_prompt.contains("friendly personality"));
        assert!(system_prompt.contains("custom_persona.md"));
    }

    #[test]
    fn test_agent_with_multiple_prompt_files() {
        use std::fs;
        use tempfile::TempDir;

        let backend = MockBackend::with_text("Hello");

        // Create multiple temp files
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("persona.md");
        let file2 = temp_dir.path().join("guidelines.md");
        fs::write(&file1, "Be helpful and kind.").unwrap();
        fs::write(&file2, "Always verify your answers.").unwrap();

        let agent = Agent::builder()
            .with_backend(backend)
            .with_prompt_file(&file1)
            .with_prompt_file(&file2)
            .build()
            .unwrap();

        let system_prompt = agent.config().system_prompt.as_ref().unwrap();
        assert!(system_prompt.contains("helpful and kind"));
        assert!(system_prompt.contains("verify your answers"));
    }

    #[test]
    fn test_agent_combine_bootstrap_dir_and_prompt_file() {
        use std::fs;
        use tempfile::TempDir;

        let backend = MockBackend::with_text("Hello");

        // Create bootstrap dir with BEHAVIOR.md
        let bootstrap_dir = TempDir::new().unwrap();
        fs::write(
            bootstrap_dir.path().join("BEHAVIOR.md"),
            "Core values here.",
        )
        .unwrap();

        // Create custom file elsewhere
        let custom_dir = TempDir::new().unwrap();
        let custom_file = custom_dir.path().join("extra.md");
        fs::write(&custom_file, "Additional guidelines.").unwrap();

        let agent = Agent::builder()
            .with_backend(backend)
            .with_bootstrap_dir(bootstrap_dir.path())
            .with_prompt_file(&custom_file)
            .build()
            .unwrap();

        let system_prompt = agent.config().system_prompt.as_ref().unwrap();
        // Should have both
        assert!(system_prompt.contains("Core values"));
        assert!(system_prompt.contains("Additional guidelines"));
    }

    // ── Active Recall Tests ──────────────────────────────────────────

    mod recall_tests {
        use super::*;
        use arawn_llm::Embedder;
        use arawn_memory::store::MemoryStore;
        use arawn_memory::types::{ContentType, Memory};
        use serial_test::serial;

        /// Simple mock embedder that returns a fixed vector.
        struct FixedEmbedder {
            dims: usize,
        }

        impl FixedEmbedder {
            fn new(dims: usize) -> Self {
                Self { dims }
            }
        }

        #[async_trait::async_trait]
        impl Embedder for FixedEmbedder {
            async fn embed(&self, _text: &str) -> arawn_llm::Result<Vec<f32>> {
                Ok(vec![0.5; self.dims])
            }

            fn dimensions(&self) -> usize {
                self.dims
            }

            fn name(&self) -> &str {
                "fixed"
            }
        }

        fn create_recall_store(dims: usize) -> Arc<MemoryStore> {
            arawn_memory::vector::init_vector_extension();
            let store = MemoryStore::open_in_memory().unwrap();
            store.init_vectors(dims, "mock").unwrap();
            Arc::new(store)
        }

        #[tokio::test]
        #[serial]
        async fn test_recall_injects_context() {
            let store = create_recall_store(4);

            // Insert a memory with embedding
            let mem = Memory::new(ContentType::Note, "Rust has great memory safety");
            store
                .insert_memory_with_embedding(&mem, &[0.5, 0.5, 0.5, 0.5])
                .unwrap();

            let embedder: SharedEmbedder = Arc::new(FixedEmbedder::new(4));

            let backend = MockBackend::with_text("I recall that Rust has great memory safety.");
            let agent = Agent::builder()
                .with_backend(backend)
                .with_memory_store(store)
                .with_embedder(embedder)
                .with_recall_config(super::super::RecallConfig {
                    enabled: true,
                    threshold: 0.0, // low threshold to ensure match
                    limit: 5,
                })
                .build()
                .unwrap();

            let mut session = Session::new();
            let response = agent
                .turn(&mut session, "Tell me about Rust")
                .await
                .unwrap();

            // The agent should respond (recall happens silently)
            assert_eq!(response.text, "I recall that Rust has great memory safety.");
        }

        #[tokio::test]
        #[serial]
        async fn test_recall_no_results() {
            let store = create_recall_store(4);
            // Empty store — no memories to recall

            let embedder: SharedEmbedder = Arc::new(FixedEmbedder::new(4));

            let backend = MockBackend::with_text("No memories found.");
            let agent = Agent::builder()
                .with_backend(backend)
                .with_memory_store(store)
                .with_embedder(embedder)
                .with_recall_config(super::super::RecallConfig {
                    enabled: true,
                    threshold: 0.0,
                    limit: 5,
                })
                .build()
                .unwrap();

            let mut session = Session::new();
            let response = agent.turn(&mut session, "Hello").await.unwrap();
            assert_eq!(response.text, "No memories found.");
        }

        #[tokio::test]
        async fn test_recall_disabled_config() {
            let backend = MockBackend::with_text("Recall disabled.");
            let agent = Agent::builder()
                .with_backend(backend)
                .with_recall_config(super::super::RecallConfig {
                    enabled: false,
                    threshold: 0.6,
                    limit: 5,
                })
                .build()
                .unwrap();

            let mut session = Session::new();
            let response = agent.turn(&mut session, "Hello").await.unwrap();
            assert_eq!(response.text, "Recall disabled.");
        }

        #[tokio::test]
        async fn test_recall_no_embedder() {
            // No embedder configured — recall should be silently skipped
            let backend = MockBackend::with_text("No embedder.");
            let agent = Agent::builder()
                .with_backend(backend)
                .with_recall_config(super::super::RecallConfig {
                    enabled: true,
                    threshold: 0.6,
                    limit: 5,
                })
                .build()
                .unwrap();

            let mut session = Session::new();
            let response = agent.turn(&mut session, "Hello").await.unwrap();
            assert_eq!(response.text, "No embedder.");
        }
    }
}
