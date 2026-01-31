//! Prompt mode definitions.
//!
//! Different modes control which sections are included in the generated prompt.

use serde::{Deserialize, Serialize};

/// Mode controlling prompt verbosity and sections.
///
/// Different modes are appropriate for different agent contexts:
/// - `Full` for main conversational agents with all capabilities
/// - `Minimal` for subagents or task-focused agents
/// - `Identity` for lightweight agents that just need an identity line
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PromptMode {
    /// Full prompt with all sections enabled.
    ///
    /// Includes: identity, tools, workspace, datetime, memory hints, bootstrap context.
    /// Use for main conversational agents.
    #[default]
    Full,

    /// Minimal prompt with reduced sections.
    ///
    /// Includes: identity, tools (names only), workspace.
    /// Use for subagents or task-focused operations.
    Minimal,

    /// Identity-only prompt.
    ///
    /// Includes: just the identity line ("You are {name}, {description}").
    /// Use for very lightweight agents or when prompt space is limited.
    Identity,
}

impl PromptMode {
    /// Check if this mode includes tool descriptions.
    pub fn include_tool_descriptions(&self) -> bool {
        matches!(self, Self::Full)
    }

    /// Check if this mode includes datetime information.
    pub fn include_datetime(&self) -> bool {
        matches!(self, Self::Full)
    }

    /// Check if this mode includes memory hints.
    pub fn include_memory_hints(&self) -> bool {
        matches!(self, Self::Full)
    }

    /// Check if this mode includes bootstrap context.
    pub fn include_bootstrap(&self) -> bool {
        matches!(self, Self::Full)
    }

    /// Check if this mode includes workspace information.
    pub fn include_workspace(&self) -> bool {
        matches!(self, Self::Full | Self::Minimal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_mode_is_full() {
        assert_eq!(PromptMode::default(), PromptMode::Full);
    }

    #[test]
    fn test_full_mode_includes_all() {
        let mode = PromptMode::Full;
        assert!(mode.include_tool_descriptions());
        assert!(mode.include_datetime());
        assert!(mode.include_memory_hints());
        assert!(mode.include_bootstrap());
        assert!(mode.include_workspace());
    }

    #[test]
    fn test_minimal_mode_includes_subset() {
        let mode = PromptMode::Minimal;
        assert!(!mode.include_tool_descriptions());
        assert!(!mode.include_datetime());
        assert!(!mode.include_memory_hints());
        assert!(!mode.include_bootstrap());
        assert!(mode.include_workspace());
    }

    #[test]
    fn test_identity_mode_includes_nothing() {
        let mode = PromptMode::Identity;
        assert!(!mode.include_tool_descriptions());
        assert!(!mode.include_datetime());
        assert!(!mode.include_memory_hints());
        assert!(!mode.include_bootstrap());
        assert!(!mode.include_workspace());
    }

    #[test]
    fn test_serialization() {
        let mode = PromptMode::Full;
        let json = serde_json::to_string(&mode).unwrap();
        assert_eq!(json, "\"full\"");

        let restored: PromptMode = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, PromptMode::Full);
    }
}
