//! Plugin system for Arawn.
//!
//! Plugins bundle skills, hooks, agents, CLI tools, and prompt fragments
//! together with a JSON manifest. This crate provides the core types,
//! manifest parsing, and plugin loading infrastructure.
//!
//! Compatible with Claude Code's plugin format.
//!
//! # Plugin Structure
//!
//! ```text
//! my-plugin/
//!   .claude-plugin/
//!     plugin.json            # manifest
//!   skills/
//!     my-skill/
//!       SKILL.md             # skill: prompt template with YAML frontmatter
//!   hooks/
//!     hooks.json             # hook configuration
//!   agents/
//!     my-agent.md            # agent: markdown with YAML frontmatter
//!   commands/
//!     my-command.sh          # CLI wrapper (JSON stdin/stdout)
//! ```

pub mod agent_spawner;
pub mod cli_tool;
pub mod hooks;
pub mod manager;
pub mod manifest;
pub mod skill;
pub mod subscription;
pub mod types;
pub mod validation;
pub mod watcher;

pub use agent_spawner::{AgentSpawner, PluginSubagentSpawner};
pub use arawn_types::HookOutcome;
pub use cli_tool::CliPluginTool;
pub use hooks::HookDispatcher;
pub use manager::{LoadedAgent, LoadedPlugin, LoadedSkill, PluginManager};
pub use manifest::{CapabilitySummary, PluginManifest};
pub use skill::{Skill, SkillInvocation, SkillRegistry};
pub use subscription::{GitOps, RuntimePluginsConfig, SubscriptionManager, SyncAction, SyncResult};
pub use types::{
    AgentConstraints, AgentSection, AgentSystemPrompt, CliToolDef, HookAction, HookDef, HookEvent,
    HookMatcherGroup, HookType, HooksConfig, PluginAgentConfig, PluginAgentDef, PromptFragment,
    SkillArg, SkillDef,
};
pub use validation::{ManifestValidationError, ValidationResult};
pub use watcher::{PluginEvent, PluginState, PluginWatcher, WatcherHandle};

/// Plugin error type.
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    /// Failed to parse a plugin manifest.
    #[error("failed to parse plugin manifest: {reason}")]
    ManifestParse { reason: String },

    /// Validation error in manifest.
    #[error("validation error in {field}: {message}")]
    Validation { field: String, message: String },

    /// IO error reading plugin files.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// TOML parsing error for agent configs.
    #[error("failed to parse agent config: {reason}")]
    AgentConfigParse { reason: String },
}

/// Result type for plugin operations.
pub type Result<T> = std::result::Result<T, PluginError>;

/// The environment variable name for the plugin root directory.
///
/// This is set when executing plugin scripts and can be used in paths
/// within plugin configurations. Claude Code uses this same variable name.
pub const CLAUDE_PLUGIN_ROOT_VAR: &str = "CLAUDE_PLUGIN_ROOT";

/// Expand `${CLAUDE_PLUGIN_ROOT}` in a string to the actual plugin directory path.
///
/// This enables plugins to use portable paths that work after being cloned
/// or cached to different locations.
///
/// # Example
///
/// ```
/// use std::path::Path;
/// use arawn_plugin::expand_plugin_root;
///
/// let plugin_dir = Path::new("/home/user/.arawn/plugins/my-plugin");
/// let cmd = "${CLAUDE_PLUGIN_ROOT}/scripts/format.sh";
/// let expanded = expand_plugin_root(cmd, plugin_dir);
/// assert_eq!(expanded, "/home/user/.arawn/plugins/my-plugin/scripts/format.sh");
/// ```
pub fn expand_plugin_root(s: &str, plugin_dir: &std::path::Path) -> String {
    s.replace("${CLAUDE_PLUGIN_ROOT}", &plugin_dir.display().to_string())
}

/// Expand `${CLAUDE_PLUGIN_ROOT}` in a PathBuf.
pub fn expand_plugin_root_path(
    path: &std::path::Path,
    plugin_dir: &std::path::Path,
) -> std::path::PathBuf {
    let path_str = path.to_string_lossy();
    if path_str.contains("${CLAUDE_PLUGIN_ROOT}") {
        std::path::PathBuf::from(expand_plugin_root(&path_str, plugin_dir))
    } else {
        path.to_path_buf()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_expand_plugin_root() {
        let plugin_dir = Path::new("/home/user/plugins/my-plugin");

        // Test basic expansion
        let input = "${CLAUDE_PLUGIN_ROOT}/scripts/test.sh";
        let expanded = expand_plugin_root(input, plugin_dir);
        assert_eq!(expanded, "/home/user/plugins/my-plugin/scripts/test.sh");
    }

    #[test]
    fn test_expand_plugin_root_multiple() {
        let plugin_dir = Path::new("/plugins/test");

        // Test multiple occurrences
        let input = "${CLAUDE_PLUGIN_ROOT}/bin:${CLAUDE_PLUGIN_ROOT}/lib";
        let expanded = expand_plugin_root(input, plugin_dir);
        assert_eq!(expanded, "/plugins/test/bin:/plugins/test/lib");
    }

    #[test]
    fn test_expand_plugin_root_no_variable() {
        let plugin_dir = Path::new("/plugins/test");

        // Test string without variable
        let input = "./relative/path.sh";
        let expanded = expand_plugin_root(input, plugin_dir);
        assert_eq!(expanded, "./relative/path.sh");
    }

    #[test]
    fn test_expand_plugin_root_path() {
        let plugin_dir = Path::new("/home/user/plugins/my-plugin");

        // Test path expansion
        let path = Path::new("${CLAUDE_PLUGIN_ROOT}/scripts/test.sh");
        let expanded = expand_plugin_root_path(path, plugin_dir);
        assert_eq!(
            expanded,
            Path::new("/home/user/plugins/my-plugin/scripts/test.sh")
        );
    }

    #[test]
    fn test_expand_plugin_root_path_no_variable() {
        let plugin_dir = Path::new("/plugins/test");

        // Test path without variable (should return unchanged)
        let path = Path::new("./scripts/test.sh");
        let expanded = expand_plugin_root_path(path, plugin_dir);
        assert_eq!(expanded, Path::new("./scripts/test.sh"));
    }

    #[test]
    fn test_claude_plugin_root_var_name() {
        assert_eq!(CLAUDE_PLUGIN_ROOT_VAR, "CLAUDE_PLUGIN_ROOT");
    }
}
