//! System prompt builder implementation.
//!
//! Provides a fluent builder for assembling system prompts from modular sections.

use std::path::{Path, PathBuf};

use chrono::{Local, Utc};

use crate::tool::ToolRegistry;

use super::bootstrap::BootstrapContext;
use super::mode::PromptMode;

/// A tool summary for prompt generation.
#[derive(Debug, Clone)]
pub struct ToolSummary {
    /// Tool name.
    pub name: String,
    /// Tool description.
    pub description: String,
}

/// Builder for generating system prompts.
///
/// Assembles prompts from modular sections based on configuration.
/// Sections are joined with double newlines.
///
/// # Example
///
/// ```rust,ignore
/// let prompt = SystemPromptBuilder::new()
///     .with_identity("Arawn", "A helpful AI assistant")
///     .with_tools(&registry)
///     .with_workspace("/project")
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct SystemPromptBuilder {
    mode: PromptMode,
    identity: Option<(String, String)>,
    tools: Option<Vec<ToolSummary>>,
    workspace_path: Option<PathBuf>,
    datetime_enabled: bool,
    timezone: Option<String>,
    memory_enabled: bool,
    think_enabled: bool,
    bootstrap_context: Option<BootstrapContext>,
    plugin_prompts: Vec<(String, String)>,
}

impl Default for SystemPromptBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemPromptBuilder {
    /// Create a new builder with default settings.
    pub fn new() -> Self {
        Self {
            mode: PromptMode::Full,
            identity: None,
            tools: None,
            workspace_path: None,
            datetime_enabled: false,
            timezone: None,
            memory_enabled: false,
            think_enabled: false,
            bootstrap_context: None,
            plugin_prompts: Vec::new(),
        }
    }

    /// Set the prompt mode.
    pub fn with_mode(mut self, mode: PromptMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set the agent identity.
    ///
    /// # Arguments
    /// * `name` - The agent's name (e.g., "Arawn")
    /// * `description` - A brief description (e.g., "A helpful AI assistant")
    pub fn with_identity(
        mut self,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        self.identity = Some((name.into(), description.into()));
        self
    }

    /// Add tools from a registry.
    ///
    /// Extracts tool names and descriptions for the prompt.
    pub fn with_tools(mut self, registry: &ToolRegistry) -> Self {
        let summaries: Vec<ToolSummary> = registry
            .names()
            .iter()
            .filter_map(|name| {
                registry.get(name).map(|tool| ToolSummary {
                    name: tool.name().to_string(),
                    description: tool.description().to_string(),
                })
            })
            .collect();

        self.think_enabled = summaries.iter().any(|t| t.name == "think");
        self.tools = Some(summaries);
        self
    }

    /// Add tool summaries directly.
    ///
    /// Use this when you have pre-computed tool summaries.
    pub fn with_tool_summaries(mut self, summaries: Vec<ToolSummary>) -> Self {
        self.think_enabled = summaries.iter().any(|t| t.name == "think");
        self.tools = Some(summaries);
        self
    }

    /// Set the workspace path.
    ///
    /// The workspace is the root directory for file operations.
    pub fn with_workspace(mut self, path: impl AsRef<Path>) -> Self {
        self.workspace_path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Enable datetime section with optional timezone.
    ///
    /// # Arguments
    /// * `timezone` - Optional timezone string (e.g., "America/New_York").
    ///   If None, uses local time.
    pub fn with_datetime(mut self, timezone: Option<&str>) -> Self {
        self.datetime_enabled = true;
        self.timezone = timezone.map(|s| s.to_string());
        self
    }

    /// Enable memory hints section.
    ///
    /// Adds guidance for the agent on using memory search.
    pub fn with_memory_hints(mut self) -> Self {
        self.memory_enabled = true;
        self
    }

    /// Add bootstrap context from workspace files.
    ///
    /// Bootstrap context includes files like SOUL.md, BOOTSTRAP.md, etc.
    pub fn with_bootstrap(mut self, context: BootstrapContext) -> Self {
        self.bootstrap_context = Some(context);
        self
    }

    /// Add plugin prompt fragments.
    ///
    /// Each fragment is a `(plugin_name, prompt_text)` pair.
    /// Empty texts are skipped during build.
    pub fn with_plugin_prompts(mut self, fragments: Vec<(String, String)>) -> Self {
        self.plugin_prompts = fragments;
        self
    }

    /// Build the final system prompt string.
    ///
    /// Sections are assembled based on the configured mode and
    /// joined with double newlines.
    pub fn build(self) -> String {
        let mut sections: Vec<String> = Vec::new();

        // Identity section (always included if set)
        if let Some(identity) = self.build_identity_section() {
            sections.push(identity);
        }

        // Tools section
        if let Some(tools) = self.build_tools_section() {
            sections.push(tools);
        }

        // Workspace section
        if self.mode.include_workspace()
            && let Some(workspace) = self.build_workspace_section()
        {
            sections.push(workspace);
        }

        // DateTime section
        if self.mode.include_datetime()
            && self.datetime_enabled
            && let Some(datetime) = self.build_datetime_section()
        {
            sections.push(datetime);
        }

        // Memory hints section
        if self.mode.include_memory_hints() && self.memory_enabled {
            sections.push(self.build_memory_section());
        }

        // Think tool guidance (auto-detected from tool list)
        if self.mode.include_memory_hints() && self.think_enabled {
            sections.push(Self::build_think_section());
        }

        // Bootstrap context section
        if self.mode.include_bootstrap()
            && let Some(bootstrap) = self.build_bootstrap_section()
        {
            sections.push(bootstrap);
        }

        // Plugin prompt fragments
        for (plugin_name, text) in &self.plugin_prompts {
            if !text.is_empty() {
                sections.push(format!("## Plugin: {}\n\n{}", plugin_name, text));
            }
        }

        sections.join("\n\n")
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Section Builders
    // ─────────────────────────────────────────────────────────────────────────

    fn build_identity_section(&self) -> Option<String> {
        self.identity
            .as_ref()
            .map(|(name, description)| format!("You are {}, {}.", name, description))
    }

    fn build_tools_section(&self) -> Option<String> {
        let tools = self.tools.as_ref()?;
        if tools.is_empty() {
            return None;
        }

        let mut lines = vec!["# Available Tools".to_string()];

        if self.mode.include_tool_descriptions() {
            // Full mode: include descriptions
            for tool in tools {
                lines.push(format!("- **{}**: {}", tool.name, tool.description));
            }
        } else {
            // Minimal mode: just names
            let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
            lines.push(format!("Tools: {}", names.join(", ")));
        }

        Some(lines.join("\n"))
    }

    fn build_workspace_section(&self) -> Option<String> {
        let path = self.workspace_path.as_ref()?;
        let path_str = path.display();

        Some(format!("# Workspace\n\nWorking directory: {}", path_str))
    }

    fn build_datetime_section(&self) -> Option<String> {
        // Always include datetime in Full mode if we've called with_datetime
        // or if a timezone was set
        let now = if self.timezone.is_some() {
            // When timezone is specified, show both local and UTC
            let local = Local::now();
            let utc = Utc::now();
            format!(
                "# Current Time\n\nLocal: {}\nUTC: {}",
                local.format("%Y-%m-%d %H:%M:%S %Z"),
                utc.format("%Y-%m-%d %H:%M:%S UTC")
            )
        } else {
            let local = Local::now();
            format!(
                "# Current Time\n\nLocal: {}",
                local.format("%Y-%m-%d %H:%M:%S %Z")
            )
        };

        Some(now)
    }

    fn build_memory_section(&self) -> String {
        r#"# Memory

You have access to a memory search tool. Use it to:
- Recall information from previous conversations
- Look up notes and context the user has saved
- Find relevant past decisions or discussions

When searching, use specific keywords related to what you're looking for."#
            .to_string()
    }

    fn build_think_section() -> String {
        r#"# Thinking

Use the `think` tool to record your reasoning before answering complex questions. Think through multi-step problems, note assumptions, and correct yourself. Your thoughts are stored permanently and may be recalled in future sessions.

When to think:
- Complex or multi-step questions that benefit from structured reasoning
- Planning a sequence of tool calls or actions
- When you need to weigh trade-offs or consider multiple approaches
- Correcting or refining your understanding mid-response"#
            .to_string()
    }

    fn build_bootstrap_section(&self) -> Option<String> {
        self.bootstrap_context
            .as_ref()
            .map(|ctx| ctx.to_prompt_section())
            .filter(|s| !s.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool::MockTool;

    #[test]
    fn test_builder_default_empty() {
        let prompt = SystemPromptBuilder::new().build();
        assert!(prompt.is_empty());
    }

    #[test]
    fn test_builder_with_identity() {
        let prompt = SystemPromptBuilder::new()
            .with_identity("Arawn", "a helpful AI assistant")
            .build();

        assert!(prompt.contains("You are Arawn, a helpful AI assistant."));
    }

    #[test]
    fn test_builder_with_tools_full_mode() {
        let mut registry = ToolRegistry::new();
        registry.register(MockTool::new("read_file").with_description("Read a file from disk"));
        registry.register(MockTool::new("write_file").with_description("Write content to a file"));

        let prompt = SystemPromptBuilder::new()
            .with_mode(PromptMode::Full)
            .with_tools(&registry)
            .build();

        assert!(prompt.contains("# Available Tools"));
        assert!(prompt.contains("**read_file**"));
        assert!(prompt.contains("Read a file from disk"));
    }

    #[test]
    fn test_builder_with_tools_minimal_mode() {
        let mut registry = ToolRegistry::new();
        registry.register(MockTool::new("read_file").with_description("Read a file from disk"));
        registry.register(MockTool::new("write_file").with_description("Write content to a file"));

        let prompt = SystemPromptBuilder::new()
            .with_mode(PromptMode::Minimal)
            .with_tools(&registry)
            .build();

        assert!(prompt.contains("# Available Tools"));
        assert!(prompt.contains("Tools:"));
        // Should not contain full descriptions in minimal mode
        assert!(!prompt.contains("Read a file from disk"));
    }

    #[test]
    fn test_builder_with_workspace() {
        let prompt = SystemPromptBuilder::new()
            .with_mode(PromptMode::Full)
            .with_workspace("/home/user/project")
            .build();

        assert!(prompt.contains("# Workspace"));
        assert!(prompt.contains("/home/user/project"));
    }

    #[test]
    fn test_builder_with_datetime() {
        let prompt = SystemPromptBuilder::new()
            .with_mode(PromptMode::Full)
            .with_datetime(None)
            .build();

        assert!(prompt.contains("# Current Time"));
        assert!(prompt.contains("Local:"));
    }

    #[test]
    fn test_builder_with_memory_hints() {
        let prompt = SystemPromptBuilder::new()
            .with_mode(PromptMode::Full)
            .with_memory_hints()
            .build();

        assert!(prompt.contains("# Memory"));
        assert!(prompt.contains("memory search tool"));
    }

    #[test]
    fn test_builder_identity_mode() {
        let mut registry = ToolRegistry::new();
        registry.register(MockTool::new("test_tool"));

        let prompt = SystemPromptBuilder::new()
            .with_mode(PromptMode::Identity)
            .with_identity("Assistant", "a test agent")
            .with_tools(&registry)
            .with_workspace("/project")
            .with_memory_hints()
            .build();

        // Identity mode should only include identity
        assert!(prompt.contains("You are Assistant"));
        // Should have tools (always included if set)
        assert!(prompt.contains("# Available Tools"));
        // Should not have workspace, memory, etc.
        assert!(!prompt.contains("# Workspace"));
        assert!(!prompt.contains("# Memory"));
    }

    #[test]
    fn test_sections_joined_with_double_newline() {
        let prompt = SystemPromptBuilder::new()
            .with_identity("Test", "agent")
            .with_workspace("/test")
            .build();

        assert!(prompt.contains("\n\n"));
    }

    #[test]
    fn test_think_section_included_when_tool_registered() {
        let mut registry = ToolRegistry::new();
        registry.register(MockTool::new("think").with_description("Record reasoning"));

        let prompt = SystemPromptBuilder::new()
            .with_mode(PromptMode::Full)
            .with_tools(&registry)
            .build();

        assert!(prompt.contains("# Thinking"));
        assert!(prompt.contains("`think` tool"));
    }

    #[test]
    fn test_think_section_omitted_when_no_think_tool() {
        let mut registry = ToolRegistry::new();
        registry.register(MockTool::new("read_file").with_description("Read a file"));

        let prompt = SystemPromptBuilder::new()
            .with_mode(PromptMode::Full)
            .with_tools(&registry)
            .build();

        assert!(!prompt.contains("# Thinking"));
    }

    #[test]
    fn test_think_section_omitted_in_minimal_mode() {
        let mut registry = ToolRegistry::new();
        registry.register(MockTool::new("think").with_description("Record reasoning"));

        let prompt = SystemPromptBuilder::new()
            .with_mode(PromptMode::Minimal)
            .with_tools(&registry)
            .build();

        assert!(!prompt.contains("# Thinking"));
    }

    #[test]
    fn test_tool_summaries_direct() {
        let summaries = vec![ToolSummary {
            name: "custom_tool".to_string(),
            description: "A custom tool".to_string(),
        }];

        let prompt = SystemPromptBuilder::new()
            .with_tool_summaries(summaries)
            .build();

        assert!(prompt.contains("custom_tool"));
        assert!(prompt.contains("A custom tool"));
    }

    #[test]
    fn test_plugin_prompts_included() {
        let prompt = SystemPromptBuilder::new()
            .with_identity("Test", "agent")
            .with_plugin_prompts(vec![
                (
                    "journal".to_string(),
                    "You can create journal entries.".to_string(),
                ),
                ("git".to_string(), "You can manage git repos.".to_string()),
            ])
            .build();

        assert!(prompt.contains("## Plugin: journal"));
        assert!(prompt.contains("You can create journal entries."));
        assert!(prompt.contains("## Plugin: git"));
        assert!(prompt.contains("You can manage git repos."));
    }

    #[test]
    fn test_plugin_prompts_empty_skipped() {
        let prompt = SystemPromptBuilder::new()
            .with_plugin_prompts(vec![
                ("empty".to_string(), "".to_string()),
                ("notempty".to_string(), "Has content.".to_string()),
            ])
            .build();

        assert!(!prompt.contains("Plugin: empty"));
        assert!(prompt.contains("## Plugin: notempty"));
    }

    #[test]
    fn test_plugin_prompts_none() {
        let prompt = SystemPromptBuilder::new()
            .with_identity("Test", "agent")
            .build();

        assert!(!prompt.contains("Plugin:"));
    }
}
