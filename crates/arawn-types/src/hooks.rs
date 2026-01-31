//! Hook types for plugin lifecycle events.
//!
//! These types are shared between `arawn-plugin` (which implements hook dispatch)
//! and `arawn-agent` (which calls hooks during tool execution).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// A lifecycle event that hooks can listen for (Claude Code compatible).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum HookEvent {
    /// Before a tool is executed. Can block execution.
    PreToolUse,
    /// After a tool has executed successfully. Informational only.
    PostToolUse,
    /// After a tool has failed. Informational only.
    PostToolUseFailure,
    /// When a permission request is made.
    PermissionRequest,
    /// When the user submits a prompt.
    UserPromptSubmit,
    /// When a notification is sent.
    Notification,
    /// When a subagent stops.
    SubagentStop,
    /// Before context compaction.
    PreCompact,
    /// When a new session or turn begins.
    SessionStart,
    /// When a session or turn ends.
    SessionEnd,
    /// When the agent produces a final response.
    Stop,
    /// When a background subagent starts execution.
    SubagentStarted,
    /// When a background subagent completes execution (success or failure).
    SubagentCompleted,
}

impl std::fmt::Display for HookEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HookEvent::PreToolUse => write!(f, "PreToolUse"),
            HookEvent::PostToolUse => write!(f, "PostToolUse"),
            HookEvent::PostToolUseFailure => write!(f, "PostToolUseFailure"),
            HookEvent::PermissionRequest => write!(f, "PermissionRequest"),
            HookEvent::UserPromptSubmit => write!(f, "UserPromptSubmit"),
            HookEvent::Notification => write!(f, "Notification"),
            HookEvent::SubagentStop => write!(f, "SubagentStop"),
            HookEvent::PreCompact => write!(f, "PreCompact"),
            HookEvent::SessionStart => write!(f, "SessionStart"),
            HookEvent::SessionEnd => write!(f, "SessionEnd"),
            HookEvent::Stop => write!(f, "Stop"),
            HookEvent::SubagentStarted => write!(f, "SubagentStarted"),
            HookEvent::SubagentCompleted => write!(f, "SubagentCompleted"),
        }
    }
}

/// Hook type (Claude Code compatible).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum HookType {
    /// Execute a shell command.
    #[default]
    Command,
    /// Evaluate with LLM (returns pass/fail/message).
    Prompt,
    /// Run an agentic verifier.
    Agent,
}

/// A single hook action (Claude Code format).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookAction {
    /// Hook type (command, prompt, or agent).
    #[serde(rename = "type", default)]
    pub hook_type: HookType,
    /// Command to execute (for command type).
    #[serde(default)]
    pub command: Option<String>,
    /// Prompt text (for prompt type).
    #[serde(default)]
    pub prompt: Option<String>,
    /// Agent name (for agent type).
    #[serde(default)]
    pub agent: Option<String>,
    /// Timeout in milliseconds.
    #[serde(default)]
    pub timeout: Option<u64>,
}

/// A matcher group containing hooks (Claude Code format).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookMatcherGroup {
    /// Regex pattern to match tool names (for PreToolUse/PostToolUse).
    #[serde(default)]
    pub matcher: Option<String>,
    /// The hook actions to execute.
    pub hooks: Vec<HookAction>,
}

/// The root hooks.json structure (Claude Code format).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HooksConfig {
    /// Hooks grouped by event type.
    #[serde(default)]
    pub hooks: HashMap<HookEvent, Vec<HookMatcherGroup>>,
}

impl HooksConfig {
    /// Check if this config has any hooks defined.
    pub fn is_empty(&self) -> bool {
        self.hooks.is_empty() || self.hooks.values().all(|v| v.is_empty())
    }
}

/// A hook definition (internal format for the dispatcher).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookDef {
    /// Which lifecycle event triggers this hook.
    pub event: HookEvent,
    /// Glob pattern to match tool names (only for PreToolUse/PostToolUse).
    #[serde(default)]
    pub tool_match: Option<String>,
    /// Regex pattern to match against serialized tool params.
    #[serde(default)]
    pub match_pattern: Option<String>,
    /// Command to execute (relative to plugin dir).
    pub command: PathBuf,
}

/// Outcome of dispatching hooks for an event.
#[derive(Debug, Clone)]
pub enum HookOutcome {
    /// All hooks passed (or no hooks matched). Proceed normally.
    Allow,
    /// A hook blocked the action (PreToolUse only).
    Block { reason: String },
    /// Informational output from hooks (SessionStart context injection, etc.).
    Info { output: String },
}

/// Trait for hook dispatch that can be implemented by different hook systems.
///
/// This trait is object-safe and can be used with `Arc<dyn HookDispatch>` to
/// avoid cyclic dependencies between arawn-agent and arawn-plugin.
#[async_trait::async_trait]
pub trait HookDispatch: Send + Sync {
    /// Dispatch hooks for a PreToolUse event.
    ///
    /// Returns `Block` if any hook exits non-zero (first blocker wins).
    async fn dispatch_pre_tool_use(
        &self,
        tool_name: &str,
        params: &serde_json::Value,
    ) -> HookOutcome;

    /// Dispatch hooks for a PostToolUse event.
    async fn dispatch_post_tool_use(
        &self,
        tool_name: &str,
        params: &serde_json::Value,
        result: &serde_json::Value,
    ) -> HookOutcome;

    /// Dispatch hooks for a SessionStart event.
    async fn dispatch_session_start(&self, session_id: &str) -> HookOutcome;

    /// Dispatch hooks for a SessionEnd event.
    async fn dispatch_session_end(&self, session_id: &str, turn_count: usize) -> HookOutcome;

    /// Dispatch hooks for a Stop event.
    async fn dispatch_stop(&self, response: &str) -> HookOutcome;

    /// Dispatch hooks for a SubagentStarted event.
    ///
    /// Called when a background subagent begins execution.
    async fn dispatch_subagent_started(
        &self,
        parent_session_id: &str,
        subagent_name: &str,
        task_preview: &str,
    ) -> HookOutcome;

    /// Dispatch hooks for a SubagentCompleted event.
    ///
    /// Called when a background subagent finishes execution (success or failure).
    async fn dispatch_subagent_completed(
        &self,
        parent_session_id: &str,
        subagent_name: &str,
        result_preview: &str,
        duration_ms: u64,
        success: bool,
    ) -> HookOutcome;

    /// Get the number of registered hooks.
    fn len(&self) -> usize;

    /// Check if the dispatcher has no hooks.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Shared hook dispatcher type.
pub type SharedHookDispatcher = std::sync::Arc<dyn HookDispatch>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_event_display() {
        assert_eq!(HookEvent::PreToolUse.to_string(), "PreToolUse");
        assert_eq!(HookEvent::SessionEnd.to_string(), "SessionEnd");
        assert_eq!(HookEvent::SubagentStarted.to_string(), "SubagentStarted");
        assert_eq!(
            HookEvent::SubagentCompleted.to_string(),
            "SubagentCompleted"
        );
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
    fn test_subagent_events_serde() {
        // Test SubagentStarted
        let started = HookEvent::SubagentStarted;
        let json = serde_json::to_string(&started).unwrap();
        assert_eq!(json, "\"SubagentStarted\"");
        let parsed: HookEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, HookEvent::SubagentStarted);

        // Test SubagentCompleted
        let completed = HookEvent::SubagentCompleted;
        let json = serde_json::to_string(&completed).unwrap();
        assert_eq!(json, "\"SubagentCompleted\"");
        let parsed: HookEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, HookEvent::SubagentCompleted);
    }

    #[test]
    fn test_hooks_config_empty() {
        let config = HooksConfig::default();
        assert!(config.is_empty());
    }

    #[test]
    fn test_hook_type_default() {
        let action: HookAction = serde_json::from_str(r#"{"command": "./test.sh"}"#).unwrap();
        assert_eq!(action.hook_type, HookType::Command);
    }
}
