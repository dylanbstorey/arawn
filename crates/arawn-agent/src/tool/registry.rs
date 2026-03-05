//! Tool registry for managing available tools.

use std::collections::HashMap;
use std::sync::Arc;

use super::context::Tool;
#[cfg(test)]
use super::context::{ToolContext, ToolResult};
use super::output::OutputConfig;
#[cfg(test)]
use crate::error::Result;
#[cfg(test)]
use async_trait::async_trait;

// ─────────────────────────────────────────────────────────────────────────────
// Tool Registry
// ─────────────────────────────────────────────────────────────────────────────

/// Registry for managing available tools.
///
/// The registry maintains a collection of tools that can be used by the agent.
/// It provides lookup by name and conversion to LLM tool definitions.
#[derive(Default, Clone)]
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
    /// Per-tool output config overrides from user configuration.
    output_overrides: HashMap<String, OutputConfig>,
}

impl ToolRegistry {
    /// Create a new empty registry.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use arawn_agent::tool::ToolRegistry;
    ///
    /// let registry = ToolRegistry::new();
    /// assert!(registry.is_empty());
    /// ```
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            output_overrides: HashMap::new(),
        }
    }

    /// Set a per-tool output config override.
    ///
    /// This override takes precedence over the hardcoded defaults in
    /// `output_config_for()`. Multiple tool names can map to the same
    /// config (e.g., "shell" and "bash" share a limit).
    pub fn set_output_config(&mut self, name: impl Into<String>, config: OutputConfig) {
        self.output_overrides.insert(name.into(), config);
    }

    /// Register a tool.
    ///
    /// If a tool with the same name already exists, it will be replaced.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use arawn_agent::tool::ToolRegistry;
    ///
    /// let mut registry = ToolRegistry::new();
    /// registry.register(my_custom_tool);
    /// assert!(registry.contains("my_custom_tool"));
    /// ```
    pub fn register<T: Tool + 'static>(&mut self, tool: T) {
        let name = tool.name().to_string();
        self.tools.insert(name, Arc::new(tool));
    }

    /// Register a tool from an Arc.
    pub fn register_arc(&mut self, tool: Arc<dyn Tool>) {
        let name = tool.name().to_string();
        self.tools.insert(name, tool);
    }

    /// Get a tool by name.
    pub fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(name).cloned()
    }

    /// Check if a tool exists.
    pub fn contains(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    /// Get all tool names.
    pub fn names(&self) -> Vec<&str> {
        self.tools.keys().map(|s| s.as_str()).collect()
    }

    /// Get the number of registered tools.
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }

    /// Convert all tools to LLM tool definitions.
    pub fn to_llm_definitions(&self) -> Vec<arawn_llm::ToolDefinition> {
        self.tools
            .values()
            .map(|tool| {
                arawn_llm::ToolDefinition::new(tool.name(), tool.description(), tool.parameters())
            })
            .collect()
    }

    /// Create a new registry containing only tools whose names are in the allowlist.
    ///
    /// Returns a new `ToolRegistry` with cloned `Arc` refs for matching tools.
    /// Names not matching any registered tool are silently ignored.
    /// Output config overrides for matching tools are also carried over.
    pub fn filtered_by_names(&self, names: &[&str]) -> ToolRegistry {
        let tools: HashMap<String, Arc<dyn Tool>> = names
            .iter()
            .filter_map(|&name| {
                self.tools
                    .get(name)
                    .map(|tool| (name.to_string(), Arc::clone(tool)))
            })
            .collect();

        let output_overrides: HashMap<String, OutputConfig> = names
            .iter()
            .filter_map(|&name| {
                self.output_overrides
                    .get(name)
                    .map(|config| (name.to_string(), config.clone()))
            })
            .collect();

        ToolRegistry {
            tools,
            output_overrides,
        }
    }

    /// Get the output config for a tool by name.
    ///
    /// Checks user-configured overrides first, then falls back to
    /// hardcoded per-tool defaults.
    pub fn output_config_for(&self, name: &str) -> OutputConfig {
        // Check overrides first
        if let Some(config) = self.output_overrides.get(name) {
            return config.clone();
        }

        // Fall back to hardcoded per-tool defaults
        match name {
            "shell" | "bash" => OutputConfig::for_shell(),
            "file_read" | "read_file" => OutputConfig::for_file_read(),
            "web_fetch" | "fetch" => OutputConfig::for_web_fetch(),
            "grep" | "glob" | "search" | "memory_search" => OutputConfig::for_search(),
            _ => OutputConfig::default(),
        }
    }
}

impl std::fmt::Debug for ToolRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolRegistry")
            .field("tools", &self.names())
            .finish()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Mock Tool (for testing)
// ─────────────────────────────────────────────────────────────────────────────

/// A mock tool for testing.
///
/// Returns configurable responses and tracks calls for verification.
#[cfg(test)]
#[derive(Debug)]
pub struct MockTool {
    name: String,
    description: String,
    parameters: serde_json::Value,
    response: std::sync::Mutex<Option<ToolResult>>,
    calls: std::sync::Mutex<Vec<serde_json::Value>>,
}

#[cfg(test)]
impl MockTool {
    /// Create a new mock tool.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: "A mock tool for testing".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
            response: std::sync::Mutex::new(None),
            calls: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// Set the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Set the parameters schema.
    pub fn with_parameters(mut self, parameters: serde_json::Value) -> Self {
        self.parameters = parameters;
        self
    }

    /// Set the response to return.
    pub fn with_response(self, response: ToolResult) -> Self {
        *self.response.lock().unwrap() = Some(response);
        self
    }

    /// Get the calls that were made to this tool.
    pub fn calls(&self) -> Vec<serde_json::Value> {
        self.calls.lock().unwrap().clone()
    }

    /// Get the number of calls made.
    pub fn call_count(&self) -> usize {
        self.calls.lock().unwrap().len()
    }

    /// Clear recorded calls.
    pub fn clear_calls(&self) {
        self.calls.lock().unwrap().clear();
    }
}

#[cfg(test)]
#[async_trait]
impl Tool for MockTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn parameters(&self) -> serde_json::Value {
        self.parameters.clone()
    }

    async fn execute(&self, params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult> {
        // Record the call
        self.calls.lock().unwrap().push(params);

        // Return configured response or default
        Ok(self
            .response
            .lock()
            .unwrap()
            .clone()
            .unwrap_or_else(|| ToolResult::text("mock response")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::AgentError;
    use crate::tool::output::DEFAULT_MAX_OUTPUT_SIZE;

    #[test]
    fn test_registry_empty() {
        let registry = ToolRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
        assert!(registry.names().is_empty());
    }

    #[test]
    fn test_registry_register_and_get() {
        let mut registry = ToolRegistry::new();

        let tool = MockTool::new("test_tool");
        registry.register(tool);

        assert!(!registry.is_empty());
        assert_eq!(registry.len(), 1);
        assert!(registry.contains("test_tool"));
        assert!(!registry.contains("other"));

        let retrieved = registry.get("test_tool");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name(), "test_tool");
    }

    #[test]
    fn test_registry_names() {
        let mut registry = ToolRegistry::new();
        registry.register(MockTool::new("tool_a"));
        registry.register(MockTool::new("tool_b"));

        let names = registry.names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"tool_a"));
        assert!(names.contains(&"tool_b"));
    }

    #[test]
    fn test_registry_to_llm_definitions() {
        let mut registry = ToolRegistry::new();
        registry.register(
            MockTool::new("read_file")
                .with_description("Read a file from disk")
                .with_parameters(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string", "description": "File path"}
                    },
                    "required": ["path"]
                })),
        );

        let definitions = registry.to_llm_definitions();
        assert_eq!(definitions.len(), 1);
        assert_eq!(definitions[0].name, "read_file");
        assert_eq!(definitions[0].description, "Read a file from disk");
    }

    #[tokio::test]
    async fn test_mock_tool_execution() {
        let tool = MockTool::new("test").with_response(ToolResult::text("custom response"));

        let ctx = ToolContext::default();
        let params = serde_json::json!({"arg": "value"});

        let result = tool.execute(params.clone(), &ctx).await.unwrap();
        assert!(matches!(result, ToolResult::Text { content } if content == "custom response"));

        assert_eq!(tool.call_count(), 1);
        assert_eq!(tool.calls()[0], params);
    }

    #[tokio::test]
    async fn test_registry_execute() {
        let mut registry = ToolRegistry::new();
        registry.register(MockTool::new("my_tool").with_response(ToolResult::text("result")));

        let ctx = ToolContext::default();
        let params = serde_json::json!({});

        // Execute existing tool
        let result = registry.execute("my_tool", params, &ctx).await;
        assert!(result.is_ok());

        // Execute non-existent tool
        let result = registry
            .execute("unknown", serde_json::json!({}), &ctx)
            .await;
        assert!(matches!(result, Err(AgentError::ToolNotFound(_))));
    }

    #[test]
    fn test_mock_tool_clear_calls() {
        let tool = MockTool::new("test");
        tool.calls.lock().unwrap().push(serde_json::json!({}));
        assert_eq!(tool.call_count(), 1);

        tool.clear_calls();
        assert_eq!(tool.call_count(), 0);
    }

    fn test_registry_output_config_for() {
        let registry = ToolRegistry::new();

        let shell_config = registry.output_config_for("shell");
        assert_eq!(shell_config.max_size_bytes, 100 * 1024);

        let file_config = registry.output_config_for("file_read");
        assert_eq!(file_config.max_size_bytes, 500 * 1024);

        let unknown_config = registry.output_config_for("unknown_tool");
        assert_eq!(unknown_config.max_size_bytes, DEFAULT_MAX_OUTPUT_SIZE);
    }

    #[test]
    fn test_registry_output_config_override() {
        let mut registry = ToolRegistry::new();

        // Set a custom override for shell
        registry.set_output_config("shell", OutputConfig::with_max_size(256 * 1024));

        // Override should take precedence
        let shell_config = registry.output_config_for("shell");
        assert_eq!(shell_config.max_size_bytes, 256 * 1024);

        // "bash" alias still uses hardcoded default (no override set for it)
        let bash_config = registry.output_config_for("bash");
        assert_eq!(bash_config.max_size_bytes, 100 * 1024);

        // Unoverridden tools still use defaults
        let file_config = registry.output_config_for("file_read");
        assert_eq!(file_config.max_size_bytes, 500 * 1024);
    }

    #[test]
    fn test_registry_output_config_override_all_aliases() {
        let mut registry = ToolRegistry::new();

        // Override both shell aliases
        let config = OutputConfig::with_max_size(200 * 1024);
        registry.set_output_config("shell", config.clone());
        registry.set_output_config("bash", config);

        assert_eq!(
            registry.output_config_for("shell").max_size_bytes,
            200 * 1024
        );
        assert_eq!(
            registry.output_config_for("bash").max_size_bytes,
            200 * 1024
        );
    }

    fn test_filtered_by_names_includes_matching() {
        let mut registry = ToolRegistry::new();
        registry.register(MockTool::new("glob"));
        registry.register(MockTool::new("grep"));
        registry.register(MockTool::new("file_read"));
        registry.register(MockTool::new("shell"));

        let filtered = registry.filtered_by_names(&["glob", "grep", "file_read"]);

        assert_eq!(filtered.len(), 3);
        assert!(filtered.contains("glob"));
        assert!(filtered.contains("grep"));
        assert!(filtered.contains("file_read"));
        assert!(!filtered.contains("shell"));
    }

    #[test]
    fn test_filtered_by_names_excludes_non_matching() {
        let mut registry = ToolRegistry::new();
        registry.register(MockTool::new("shell"));
        registry.register(MockTool::new("file_write"));

        let filtered = registry.filtered_by_names(&["glob", "grep"]);

        assert!(filtered.is_empty());
    }

    #[test]
    fn test_filtered_by_names_ignores_unknown() {
        let mut registry = ToolRegistry::new();
        registry.register(MockTool::new("glob"));

        let filtered = registry.filtered_by_names(&["glob", "nonexistent", "also_missing"]);

        assert_eq!(filtered.len(), 1);
        assert!(filtered.contains("glob"));
    }

    #[test]
    fn test_filtered_by_names_preserves_original() {
        let mut registry = ToolRegistry::new();
        registry.register(MockTool::new("glob"));
        registry.register(MockTool::new("shell"));

        let _filtered = registry.filtered_by_names(&["glob"]);

        // Original unchanged
        assert_eq!(registry.len(), 2);
        assert!(registry.contains("glob"));
        assert!(registry.contains("shell"));
    }

    #[test]
    fn test_filtered_by_names_carries_output_overrides() {
        let mut registry = ToolRegistry::new();
        registry.register(MockTool::new("glob"));
        registry.register(MockTool::new("shell"));
        registry.set_output_config("glob", OutputConfig::with_max_size(999));

        let filtered = registry.filtered_by_names(&["glob"]);

        let config = filtered.output_config_for("glob");
        assert_eq!(config.max_size_bytes, 999);
    }

    #[test]
    fn test_filtered_by_names_llm_definitions() {
        let mut registry = ToolRegistry::new();
        registry.register(MockTool::new("glob").with_description("Search files"));
        registry.register(MockTool::new("shell").with_description("Run commands"));

        let filtered = registry.filtered_by_names(&["glob"]);
        let defs = filtered.to_llm_definitions();

        assert_eq!(defs.len(), 1);
        assert_eq!(defs[0].name, "glob");
    }

    #[test]
    fn test_filtered_by_names_empty_allowlist() {
        let mut registry = ToolRegistry::new();
        registry.register(MockTool::new("glob"));

        let filtered = registry.filtered_by_names(&[]);

        assert!(filtered.is_empty());
    }
}
