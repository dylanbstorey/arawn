//! Core types for the plugin system.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// Re-export hook types from arawn-types
pub use arawn_types::{
    HookAction, HookDef, HookEvent, HookMatcherGroup, HookOutcome, HookType, HooksConfig,
};

// ─────────────────────────────────────────────────────────────────────────────
// HooksConfig extension methods
// ─────────────────────────────────────────────────────────────────────────────

/// Extension trait for HooksConfig to add parsing methods.
pub trait HooksConfigExt {
    /// Parse hooks config from JSON string.
    fn from_json(json_str: &str) -> Result<HooksConfig, crate::PluginError>;
    /// Parse hooks config from a file.
    fn from_file(path: &std::path::Path) -> Result<HooksConfig, crate::PluginError>;
}

impl HooksConfigExt for HooksConfig {
    fn from_json(json_str: &str) -> Result<HooksConfig, crate::PluginError> {
        serde_json::from_str(json_str).map_err(|e| crate::PluginError::ManifestParse {
            reason: format!("hooks.json: {}", e),
        })
    }

    fn from_file(path: &std::path::Path) -> Result<HooksConfig, crate::PluginError> {
        let content = std::fs::read_to_string(path)?;
        Self::from_json(&content)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Skill
// ─────────────────────────────────────────────────────────────────────────────

/// A skill definition from a plugin manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDef {
    /// Skill name (used for invocation, e.g., "review-pr").
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Path to the skill markdown file (relative to plugin dir).
    pub file: PathBuf,
    /// Tools this skill uses (informational / optional constraining).
    #[serde(default)]
    pub uses_tools: Vec<String>,
}

/// A skill argument declaration (parsed from skill markdown frontmatter).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillArg {
    /// Argument name.
    pub name: String,
    /// Human-readable description.
    #[serde(default)]
    pub description: String,
    /// Whether this argument is required.
    #[serde(default)]
    pub required: bool,
}

// ─────────────────────────────────────────────────────────────────────────────
// Plugin Agent
// ─────────────────────────────────────────────────────────────────────────────

/// A plugin-defined agent (subagent) definition.
///
/// Parsed from `agents/<name>.md` files with YAML frontmatter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginAgentDef {
    /// Agent name (derived from filename).
    pub name: String,
    /// Human-readable description (from frontmatter).
    pub description: String,
    /// Path to agent markdown file (relative to plugin dir).
    pub file: PathBuf,
    /// Constrained tool set (from frontmatter `tools` field).
    #[serde(default)]
    pub tools: Vec<String>,
}

/// Full agent configuration parsed from an agent markdown file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginAgentConfig {
    /// Agent section.
    pub agent: AgentSection,
}

/// Agent configuration section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSection {
    /// Agent name.
    pub name: String,
    /// Human-readable description.
    #[serde(default)]
    pub description: String,
    /// Optional model override (e.g., "claude-sonnet").
    #[serde(default)]
    pub model: Option<String>,
    /// System prompt configuration.
    #[serde(default)]
    pub system_prompt: Option<AgentSystemPrompt>,
    /// Constraints on the agent.
    #[serde(default)]
    pub constraints: Option<AgentConstraints>,
}

/// System prompt for a plugin agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSystemPrompt {
    /// The system prompt text.
    pub text: String,
}

/// Constraints on a plugin agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConstraints {
    /// Allowed tool names.
    #[serde(default)]
    pub tools: Vec<String>,
    /// Maximum number of turn loop iterations.
    #[serde(default)]
    pub max_iterations: Option<usize>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Prompt Fragment
// ─────────────────────────────────────────────────────────────────────────────

/// Plugin-provided prompt fragment injected into the system prompt.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PromptFragment {
    /// Text to inject into the system prompt.
    #[serde(default)]
    pub system: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_event_display() {
        assert_eq!(HookEvent::PreToolUse.to_string(), "PreToolUse");
        assert_eq!(HookEvent::SessionEnd.to_string(), "SessionEnd");
        assert_eq!(HookEvent::UserPromptSubmit.to_string(), "UserPromptSubmit");
        assert_eq!(HookEvent::PreCompact.to_string(), "PreCompact");
    }

    #[test]
    fn test_hook_event_serde_roundtrip() {
        let event = HookEvent::PostToolUse;
        let json = serde_json::to_string(&event).unwrap();
        assert_eq!(json, "\"PostToolUse\"");
        let parsed: HookEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, event);
    }

    #[test]
    fn test_new_hook_events_serde() {
        let events = vec![
            HookEvent::PostToolUseFailure,
            HookEvent::PermissionRequest,
            HookEvent::UserPromptSubmit,
            HookEvent::Notification,
            HookEvent::SubagentStop,
            HookEvent::PreCompact,
            HookEvent::SubagentStarted,
            HookEvent::SubagentCompleted,
        ];
        for event in events {
            let json = serde_json::to_string(&event).unwrap();
            let parsed: HookEvent = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, event);
        }
    }

    #[test]
    fn test_plugin_agent_config_parse() {
        let toml_str = r#"
[agent]
name = "reviewer"
description = "Code review agent"
model = "claude-sonnet"

[agent.system_prompt]
text = "You are a code reviewer."

[agent.constraints]
tools = ["shell", "file_read"]
max_iterations = 10
"#;
        let config: PluginAgentConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.agent.name, "reviewer");
        assert_eq!(config.agent.model.as_deref(), Some("claude-sonnet"));
        assert_eq!(
            config.agent.system_prompt.unwrap().text,
            "You are a code reviewer."
        );
        let constraints = config.agent.constraints.unwrap();
        assert_eq!(constraints.tools, vec!["shell", "file_read"]);
        assert_eq!(constraints.max_iterations, Some(10));
    }

    #[test]
    fn test_hooks_config_parse() {
        let json = r#"{
            "hooks": {
                "PreToolUse": [
                    {
                        "matcher": "Write|Edit",
                        "hooks": [
                            {
                                "type": "command",
                                "command": "./scripts/validate.sh"
                            }
                        ]
                    }
                ],
                "SessionStart": [
                    {
                        "hooks": [
                            {
                                "type": "prompt",
                                "prompt": "Initialize the session"
                            }
                        ]
                    }
                ]
            }
        }"#;
        let config = <HooksConfig as HooksConfigExt>::from_json(json).unwrap();
        assert!(!config.is_empty());

        let pre_tool = config.hooks.get(&HookEvent::PreToolUse).unwrap();
        assert_eq!(pre_tool.len(), 1);
        assert_eq!(pre_tool[0].matcher, Some("Write|Edit".to_string()));
        assert_eq!(pre_tool[0].hooks[0].hook_type, HookType::Command);
        assert_eq!(
            pre_tool[0].hooks[0].command,
            Some("./scripts/validate.sh".to_string())
        );

        let session_start = config.hooks.get(&HookEvent::SessionStart).unwrap();
        assert_eq!(session_start[0].hooks[0].hook_type, HookType::Prompt);
    }

    #[test]
    fn test_hooks_config_empty() {
        let config = HooksConfig::default();
        assert!(config.is_empty());

        let json = r#"{"hooks": {}}"#;
        let config = <HooksConfig as HooksConfigExt>::from_json(json).unwrap();
        assert!(config.is_empty());
    }

    #[test]
    fn test_hook_type_default() {
        let action: HookAction = serde_json::from_str(r#"{"command": "./test.sh"}"#).unwrap();
        assert_eq!(action.hook_type, HookType::Command);
    }
}
