//! Hook dispatcher for plugin lifecycle events.
//!
//! Hooks are shell commands that fire at lifecycle events in the agent turn
//! loop. They receive JSON context on stdin and can block tool execution
//! (PreToolUse) or provide informational side effects.

use arawn_types::{HookDef, HookDispatch, HookEvent, HookOutcome};
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use tokio::io::AsyncWriteExt;

/// Default timeout for hook subprocesses.
const DEFAULT_HOOK_TIMEOUT: Duration = Duration::from_secs(10);

/// A compiled hook ready for matching and execution.
#[derive(Debug, Clone)]
struct CompiledHook {
    /// The original hook definition.
    def: HookDef,
    /// Compiled glob pattern for tool_match.
    tool_pattern: Option<glob::Pattern>,
    /// Compiled regex for match_pattern.
    param_regex: Option<regex::Regex>,
    /// Plugin directory (working directory for subprocess).
    plugin_dir: PathBuf,
}

/// Dispatches hooks at lifecycle events.
#[derive(Debug)]
pub struct HookDispatcher {
    /// Hooks grouped by event type.
    hooks: HashMap<HookEvent, Vec<CompiledHook>>,
    /// Subprocess timeout.
    timeout: Duration,
}

impl HookDispatcher {
    /// Create an empty dispatcher.
    pub fn new() -> Self {
        Self {
            hooks: HashMap::new(),
            timeout: DEFAULT_HOOK_TIMEOUT,
        }
    }

    /// Set the subprocess timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Register a hook from a plugin.
    pub fn register(&mut self, def: HookDef, plugin_dir: PathBuf) {
        let tool_pattern = def.tool_match.as_ref().and_then(|p| {
            glob::Pattern::new(p)
                .map_err(|e| {
                    tracing::warn!(
                        pattern = %p,
                        error = %e,
                        "invalid tool_match glob pattern, hook will never match"
                    );
                    e
                })
                .ok()
        });

        let param_regex = def.match_pattern.as_ref().and_then(|p| {
            regex::Regex::new(p)
                .map_err(|e| {
                    tracing::warn!(
                        pattern = %p,
                        error = %e,
                        "invalid match_pattern regex, hook will never match"
                    );
                    e
                })
                .ok()
        });

        let compiled = CompiledHook {
            def: def.clone(),
            tool_pattern,
            param_regex,
            plugin_dir,
        };

        self.hooks.entry(def.event).or_default().push(compiled);
    }

    /// Get the number of registered hooks.
    pub fn len(&self) -> usize {
        self.hooks.values().map(|v| v.len()).sum()
    }

    /// Check if the dispatcher has no hooks.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the number of hooks for a specific event.
    pub fn count_for_event(&self, event: HookEvent) -> usize {
        self.hooks.get(&event).map_or(0, |v| v.len())
    }

    /// Dispatch hooks for a PreToolUse event.
    ///
    /// Returns `Block` if any hook exits non-zero (first blocker wins).
    pub async fn dispatch_pre_tool_use(
        &self,
        tool_name: &str,
        params: &serde_json::Value,
    ) -> HookOutcome {
        let context = PreToolUseContext {
            tool: tool_name,
            params,
        };
        self.dispatch_blocking(
            HookEvent::PreToolUse,
            &context,
            Some(tool_name),
            Some(params),
        )
        .await
    }

    /// Dispatch hooks for a PostToolUse event.
    pub async fn dispatch_post_tool_use(
        &self,
        tool_name: &str,
        params: &serde_json::Value,
        result: &serde_json::Value,
    ) -> HookOutcome {
        let context = PostToolUseContext {
            tool: tool_name,
            params,
            result,
        };
        self.dispatch_info(
            HookEvent::PostToolUse,
            &context,
            Some(tool_name),
            Some(params),
        )
        .await
    }

    /// Dispatch hooks for a SessionStart event.
    pub async fn dispatch_session_start(&self, session_id: &str) -> HookOutcome {
        let context = SessionContext { session_id };
        self.dispatch_info(HookEvent::SessionStart, &context, None, None)
            .await
    }

    /// Dispatch hooks for a SessionEnd event.
    pub async fn dispatch_session_end(&self, session_id: &str, turn_count: usize) -> HookOutcome {
        let context = SessionEndContext {
            session_id,
            turn_count,
        };
        self.dispatch_info(HookEvent::SessionEnd, &context, None, None)
            .await
    }

    /// Dispatch hooks for a Stop event.
    pub async fn dispatch_stop(&self, response: &str) -> HookOutcome {
        let context = StopContext { response };
        self.dispatch_info(HookEvent::Stop, &context, None, None)
            .await
    }

    /// Dispatch hooks for a SubagentStarted event.
    pub async fn dispatch_subagent_started(
        &self,
        parent_session_id: &str,
        subagent_name: &str,
        task_preview: &str,
    ) -> HookOutcome {
        let context = SubagentStartedContext {
            parent_session_id,
            subagent_name,
            task_preview,
        };
        self.dispatch_info(HookEvent::SubagentStarted, &context, None, None)
            .await
    }

    /// Dispatch hooks for a SubagentCompleted event.
    pub async fn dispatch_subagent_completed(
        &self,
        parent_session_id: &str,
        subagent_name: &str,
        result_preview: &str,
        duration_ms: u64,
        success: bool,
    ) -> HookOutcome {
        let context = SubagentCompletedContext {
            parent_session_id,
            subagent_name,
            result_preview,
            duration_ms,
            success,
        };
        self.dispatch_info(HookEvent::SubagentCompleted, &context, None, None)
            .await
    }

    /// Dispatch hooks that can block (PreToolUse).
    async fn dispatch_blocking<C: Serialize>(
        &self,
        event: HookEvent,
        context: &C,
        tool_name: Option<&str>,
        params: Option<&serde_json::Value>,
    ) -> HookOutcome {
        let Some(hooks) = self.hooks.get(&event) else {
            return HookOutcome::Allow;
        };

        let context_json = serde_json::to_string(context).unwrap_or_default();

        for hook in hooks {
            if !matches_hook(hook, tool_name, params) {
                continue;
            }

            match run_hook_command(
                &hook.def.command,
                &hook.plugin_dir,
                &context_json,
                self.timeout,
            )
            .await
            {
                HookRunResult::Success(output) => {
                    tracing::debug!(
                        event = %event,
                        command = %hook.def.command.display(),
                        "hook passed"
                    );
                    if !output.is_empty() {
                        tracing::debug!(output = %output, "hook output");
                    }
                }
                HookRunResult::Blocked(reason) => {
                    tracing::info!(
                        event = %event,
                        command = %hook.def.command.display(),
                        reason = %reason,
                        "hook blocked action"
                    );
                    return HookOutcome::Block { reason };
                }
                HookRunResult::Error(e) => {
                    tracing::warn!(
                        event = %event,
                        command = %hook.def.command.display(),
                        error = %e,
                        "hook execution failed"
                    );
                }
            }
        }

        HookOutcome::Allow
    }

    /// Dispatch informational hooks (PostToolUse, SessionStart, SessionEnd, Stop).
    async fn dispatch_info<C: Serialize>(
        &self,
        event: HookEvent,
        context: &C,
        tool_name: Option<&str>,
        params: Option<&serde_json::Value>,
    ) -> HookOutcome {
        let Some(hooks) = self.hooks.get(&event) else {
            return HookOutcome::Allow;
        };

        let context_json = serde_json::to_string(context).unwrap_or_default();
        let mut combined_output = String::new();

        for hook in hooks {
            if !matches_hook(hook, tool_name, params) {
                continue;
            }

            match run_hook_command(
                &hook.def.command,
                &hook.plugin_dir,
                &context_json,
                self.timeout,
            )
            .await
            {
                HookRunResult::Success(output) => {
                    if !output.is_empty() {
                        if !combined_output.is_empty() {
                            combined_output.push('\n');
                        }
                        combined_output.push_str(&output);
                    }
                }
                HookRunResult::Blocked(_) | HookRunResult::Error(_) => {
                    // Informational hooks: non-zero exit is logged but doesn't block
                    tracing::debug!(
                        event = %event,
                        command = %hook.def.command.display(),
                        "informational hook returned non-zero exit"
                    );
                }
            }
        }

        if combined_output.is_empty() {
            HookOutcome::Allow
        } else {
            HookOutcome::Info {
                output: combined_output,
            }
        }
    }
}

impl Default for HookDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl HookDispatcher {
    /// Register hooks from a Claude-format `HooksConfig`.
    ///
    /// This converts the Claude `HooksConfig` format (matcher groups with actions)
    /// into `HookDef` entries that the dispatcher can execute.
    ///
    /// Only `HookType::Command` hooks are supported; `Prompt` and `Agent` hooks
    /// are logged and skipped.
    pub fn register_from_config(
        &mut self,
        config: &crate::HooksConfig,
        plugin_dir: &std::path::Path,
    ) {
        for (event, matcher_groups) in &config.hooks {
            for group in matcher_groups {
                for action in &group.hooks {
                    // Only support command hooks for now
                    if action.hook_type != crate::HookType::Command {
                        tracing::debug!(
                            event = %event,
                            hook_type = ?action.hook_type,
                            "skipping non-command hook (not yet supported)"
                        );
                        continue;
                    }

                    let Some(ref command_str) = action.command else {
                        tracing::warn!(
                            event = %event,
                            "command hook missing 'command' field, skipping"
                        );
                        continue;
                    };

                    let def = crate::HookDef {
                        event: *event,
                        tool_match: group.matcher.clone(),
                        match_pattern: None, // Claude format uses matcher as regex for tool name
                        command: std::path::PathBuf::from(command_str),
                    };

                    self.register(def, plugin_dir.to_path_buf());
                }
            }
        }
    }
}

/// Implement the HookDispatch trait for HookDispatcher.
///
/// This allows HookDispatcher to be used via the trait object pattern,
/// avoiding cyclic dependencies between arawn-agent and arawn-plugin.
#[async_trait::async_trait]
impl HookDispatch for HookDispatcher {
    async fn dispatch_pre_tool_use(
        &self,
        tool_name: &str,
        params: &serde_json::Value,
    ) -> HookOutcome {
        HookDispatcher::dispatch_pre_tool_use(self, tool_name, params).await
    }

    async fn dispatch_post_tool_use(
        &self,
        tool_name: &str,
        params: &serde_json::Value,
        result: &serde_json::Value,
    ) -> HookOutcome {
        HookDispatcher::dispatch_post_tool_use(self, tool_name, params, result).await
    }

    async fn dispatch_session_start(&self, session_id: &str) -> HookOutcome {
        HookDispatcher::dispatch_session_start(self, session_id).await
    }

    async fn dispatch_session_end(&self, session_id: &str, turn_count: usize) -> HookOutcome {
        HookDispatcher::dispatch_session_end(self, session_id, turn_count).await
    }

    async fn dispatch_stop(&self, response: &str) -> HookOutcome {
        HookDispatcher::dispatch_stop(self, response).await
    }

    async fn dispatch_subagent_started(
        &self,
        parent_session_id: &str,
        subagent_name: &str,
        task_preview: &str,
    ) -> HookOutcome {
        HookDispatcher::dispatch_subagent_started(
            self,
            parent_session_id,
            subagent_name,
            task_preview,
        )
        .await
    }

    async fn dispatch_subagent_completed(
        &self,
        parent_session_id: &str,
        subagent_name: &str,
        result_preview: &str,
        duration_ms: u64,
        success: bool,
    ) -> HookOutcome {
        HookDispatcher::dispatch_subagent_completed(
            self,
            parent_session_id,
            subagent_name,
            result_preview,
            duration_ms,
            success,
        )
        .await
    }

    fn len(&self) -> usize {
        HookDispatcher::len(self)
    }

    fn is_empty(&self) -> bool {
        HookDispatcher::is_empty(self)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Context types (serialized to JSON for hook stdin)
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct PreToolUseContext<'a> {
    tool: &'a str,
    params: &'a serde_json::Value,
}

#[derive(Serialize)]
struct PostToolUseContext<'a> {
    tool: &'a str,
    params: &'a serde_json::Value,
    result: &'a serde_json::Value,
}

#[derive(Serialize)]
struct SessionContext<'a> {
    session_id: &'a str,
}

#[derive(Serialize)]
struct SessionEndContext<'a> {
    session_id: &'a str,
    turn_count: usize,
}

#[derive(Serialize)]
struct StopContext<'a> {
    response: &'a str,
}

#[derive(Serialize)]
struct SubagentStartedContext<'a> {
    parent_session_id: &'a str,
    subagent_name: &'a str,
    task_preview: &'a str,
}

#[derive(Serialize)]
struct SubagentCompletedContext<'a> {
    parent_session_id: &'a str,
    subagent_name: &'a str,
    result_preview: &'a str,
    duration_ms: u64,
    success: bool,
}

// ─────────────────────────────────────────────────────────────────────────────
// Hook matching
// ─────────────────────────────────────────────────────────────────────────────

fn matches_hook(
    hook: &CompiledHook,
    tool_name: Option<&str>,
    params: Option<&serde_json::Value>,
) -> bool {
    // Check tool_match glob
    if let Some(ref pattern) = hook.tool_pattern {
        match tool_name {
            Some(name) => {
                if !pattern.matches(name) {
                    return false;
                }
            }
            None => return false,
        }
    }

    // Check match_pattern regex against serialized params
    if let Some(ref regex) = hook.param_regex {
        match params {
            Some(p) => {
                let serialized = serde_json::to_string(p).unwrap_or_default();
                if !regex.is_match(&serialized) {
                    return false;
                }
            }
            None => return false,
        }
    }

    true
}

// ─────────────────────────────────────────────────────────────────────────────
// Hook execution
// ─────────────────────────────────────────────────────────────────────────────

enum HookRunResult {
    /// Hook exited 0, with optional stdout.
    Success(String),
    /// Hook exited non-zero. Stdout is the reason.
    Blocked(String),
    /// Hook failed to run.
    Error(String),
}

async fn run_hook_command(
    command: &std::path::Path,
    plugin_dir: &std::path::Path,
    stdin_data: &str,
    timeout: Duration,
) -> HookRunResult {
    // Expand ${CLAUDE_PLUGIN_ROOT} in the command path
    let expanded_command = crate::expand_plugin_root_path(command, plugin_dir);

    let child = tokio::process::Command::new(&expanded_command)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .current_dir(plugin_dir)
        .env("ARAWN_PLUGIN_DIR", plugin_dir)
        .env(crate::CLAUDE_PLUGIN_ROOT_VAR, plugin_dir)
        .spawn();

    let mut child = match child {
        Ok(c) => c,
        Err(e) => return HookRunResult::Error(format!("failed to spawn: {}", e)),
    };

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(stdin_data.as_bytes()).await;
        let _ = stdin.shutdown().await;
    }

    let output = match tokio::time::timeout(timeout, child.wait_with_output()).await {
        Ok(Ok(output)) => output,
        Ok(Err(e)) => return HookRunResult::Error(format!("process error: {}", e)),
        Err(_) => return HookRunResult::Error(format!("timed out after {}s", timeout.as_secs())),
    };

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        tracing::debug!(stderr = %stderr.trim(), "hook stderr");
    }

    if output.status.success() {
        HookRunResult::Success(stdout)
    } else {
        let reason = if stdout.is_empty() {
            stderr.trim().to_string()
        } else {
            stdout
        };
        HookRunResult::Blocked(reason)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use tempfile::TempDir;

    fn create_hook_script(dir: &std::path::Path, name: &str, script: &str) -> PathBuf {
        let path = dir.join(name);
        fs::write(&path, script).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        path
    }

    fn make_hook(event: HookEvent, command: PathBuf) -> HookDef {
        HookDef {
            event,
            tool_match: None,
            match_pattern: None,
            command,
        }
    }

    #[tokio::test]
    async fn test_pre_tool_use_allow() {
        let tmp = TempDir::new().unwrap();
        let script = create_hook_script(tmp.path(), "allow.sh", "#!/bin/bash\nexit 0\n");

        let mut dispatcher = HookDispatcher::new();
        dispatcher.register(
            make_hook(HookEvent::PreToolUse, script),
            tmp.path().to_path_buf(),
        );

        let outcome = dispatcher
            .dispatch_pre_tool_use("shell", &serde_json::json!({"cmd": "ls"}))
            .await;
        assert!(matches!(outcome, HookOutcome::Allow));
    }

    #[tokio::test]
    async fn test_pre_tool_use_block() {
        let tmp = TempDir::new().unwrap();
        let script = create_hook_script(
            tmp.path(),
            "block.sh",
            "#!/bin/bash\necho 'dangerous command blocked'\nexit 1\n",
        );

        let mut dispatcher = HookDispatcher::new();
        dispatcher.register(
            make_hook(HookEvent::PreToolUse, script),
            tmp.path().to_path_buf(),
        );

        let outcome = dispatcher
            .dispatch_pre_tool_use("shell", &serde_json::json!({"cmd": "rm -rf /"}))
            .await;
        match outcome {
            HookOutcome::Block { reason } => {
                assert!(reason.contains("dangerous command blocked"));
            }
            other => panic!("expected Block, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_tool_match_glob() {
        let tmp = TempDir::new().unwrap();
        let script = create_hook_script(tmp.path(), "block.sh", "#!/bin/bash\nexit 1\n");

        let mut dispatcher = HookDispatcher::new();
        let mut hook = make_hook(HookEvent::PreToolUse, script);
        hook.tool_match = Some("shell*".to_string());
        dispatcher.register(hook, tmp.path().to_path_buf());

        // Should match "shell"
        let outcome = dispatcher
            .dispatch_pre_tool_use("shell", &serde_json::json!({}))
            .await;
        assert!(matches!(outcome, HookOutcome::Block { .. }));

        // Should not match "file_read"
        let outcome = dispatcher
            .dispatch_pre_tool_use("file_read", &serde_json::json!({}))
            .await;
        assert!(matches!(outcome, HookOutcome::Allow));
    }

    #[tokio::test]
    async fn test_match_pattern_regex() {
        let tmp = TempDir::new().unwrap();
        let script = create_hook_script(tmp.path(), "block.sh", "#!/bin/bash\nexit 1\n");

        let mut dispatcher = HookDispatcher::new();
        let mut hook = make_hook(HookEvent::PreToolUse, script);
        hook.match_pattern = Some("git commit".to_string());
        dispatcher.register(hook, tmp.path().to_path_buf());

        // Should match params containing "git commit"
        let outcome = dispatcher
            .dispatch_pre_tool_use("shell", &serde_json::json!({"cmd": "git commit -m 'test'"}))
            .await;
        assert!(matches!(outcome, HookOutcome::Block { .. }));

        // Should not match other commands
        let outcome = dispatcher
            .dispatch_pre_tool_use("shell", &serde_json::json!({"cmd": "git status"}))
            .await;
        assert!(matches!(outcome, HookOutcome::Allow));
    }

    #[tokio::test]
    async fn test_session_start_info() {
        let tmp = TempDir::new().unwrap();
        let script = create_hook_script(
            tmp.path(),
            "start.sh",
            "#!/bin/bash\necho 'Welcome to the session'\n",
        );

        let mut dispatcher = HookDispatcher::new();
        dispatcher.register(
            make_hook(HookEvent::SessionStart, script),
            tmp.path().to_path_buf(),
        );

        let outcome = dispatcher.dispatch_session_start("sess-123").await;
        match outcome {
            HookOutcome::Info { output } => {
                assert!(output.contains("Welcome to the session"));
            }
            other => panic!("expected Info, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_session_end() {
        let tmp = TempDir::new().unwrap();
        let script = create_hook_script(tmp.path(), "end.sh", "#!/bin/bash\nexit 0\n");

        let mut dispatcher = HookDispatcher::new();
        dispatcher.register(
            make_hook(HookEvent::SessionEnd, script),
            tmp.path().to_path_buf(),
        );

        let outcome = dispatcher.dispatch_session_end("sess-123", 5).await;
        assert!(matches!(outcome, HookOutcome::Allow));
    }

    #[tokio::test]
    async fn test_stop_hook() {
        let tmp = TempDir::new().unwrap();
        let script = create_hook_script(
            tmp.path(),
            "stop.sh",
            "#!/bin/bash\necho 'Response validated'\n",
        );

        let mut dispatcher = HookDispatcher::new();
        dispatcher.register(make_hook(HookEvent::Stop, script), tmp.path().to_path_buf());

        let outcome = dispatcher.dispatch_stop("Here is my response").await;
        match outcome {
            HookOutcome::Info { output } => {
                assert!(output.contains("Response validated"));
            }
            other => panic!("expected Info, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_no_hooks_registered() {
        let dispatcher = HookDispatcher::new();
        let outcome = dispatcher
            .dispatch_pre_tool_use("shell", &serde_json::json!({}))
            .await;
        assert!(matches!(outcome, HookOutcome::Allow));
    }

    #[tokio::test]
    async fn test_post_tool_use() {
        let tmp = TempDir::new().unwrap();
        let script = create_hook_script(tmp.path(), "post.sh", "#!/bin/bash\necho 'logged'\n");

        let mut dispatcher = HookDispatcher::new();
        dispatcher.register(
            make_hook(HookEvent::PostToolUse, script),
            tmp.path().to_path_buf(),
        );

        let outcome = dispatcher
            .dispatch_post_tool_use(
                "shell",
                &serde_json::json!({"cmd": "ls"}),
                &serde_json::json!({"output": "file1\nfile2"}),
            )
            .await;
        match outcome {
            HookOutcome::Info { output } => assert!(output.contains("logged")),
            other => panic!("expected Info, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_hook_receives_stdin() {
        let tmp = TempDir::new().unwrap();
        // Script that reads stdin and echoes the tool name from it
        let script = create_hook_script(
            tmp.path(),
            "echo.sh",
            "#!/bin/bash\nread input\necho \"$input\"\n",
        );

        let mut dispatcher = HookDispatcher::new();
        dispatcher.register(
            make_hook(HookEvent::SessionStart, script),
            tmp.path().to_path_buf(),
        );

        let outcome = dispatcher.dispatch_session_start("test-session").await;
        match outcome {
            HookOutcome::Info { output } => {
                assert!(output.contains("test-session"));
            }
            other => panic!("expected Info, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_dispatcher_len() {
        let tmp = TempDir::new().unwrap();
        let script = create_hook_script(tmp.path(), "hook.sh", "#!/bin/bash\nexit 0\n");

        let mut dispatcher = HookDispatcher::new();
        assert!(dispatcher.is_empty());
        assert_eq!(dispatcher.len(), 0);

        dispatcher.register(
            make_hook(HookEvent::PreToolUse, script.clone()),
            tmp.path().to_path_buf(),
        );
        dispatcher.register(
            make_hook(HookEvent::SessionEnd, script),
            tmp.path().to_path_buf(),
        );

        assert_eq!(dispatcher.len(), 2);
        assert!(!dispatcher.is_empty());
        assert_eq!(dispatcher.count_for_event(HookEvent::PreToolUse), 1);
        assert_eq!(dispatcher.count_for_event(HookEvent::SessionEnd), 1);
        assert_eq!(dispatcher.count_for_event(HookEvent::Stop), 0);
    }

    #[tokio::test]
    async fn test_hook_timeout() {
        let tmp = TempDir::new().unwrap();
        let script = create_hook_script(tmp.path(), "slow.sh", "#!/bin/bash\nsleep 60\n");

        let mut dispatcher = HookDispatcher::new().with_timeout(Duration::from_millis(100));
        dispatcher.register(
            make_hook(HookEvent::SessionStart, script),
            tmp.path().to_path_buf(),
        );

        // Should not hang — timeout kicks in
        let outcome = dispatcher.dispatch_session_start("sess").await;
        // Timeout results in Allow for info hooks (error is logged)
        assert!(matches!(outcome, HookOutcome::Allow));
    }

    #[test]
    fn test_matches_hook_no_filters() {
        let hook = CompiledHook {
            def: make_hook(HookEvent::PreToolUse, PathBuf::from("test.sh")),
            tool_pattern: None,
            param_regex: None,
            plugin_dir: PathBuf::from("."),
        };
        // No filters = always matches
        assert!(matches_hook(
            &hook,
            Some("anything"),
            Some(&serde_json::json!({}))
        ));
        assert!(matches_hook(&hook, None, None));
    }

    #[test]
    fn test_matches_hook_tool_pattern_no_tool_name() {
        let hook = CompiledHook {
            def: make_hook(HookEvent::PreToolUse, PathBuf::from("test.sh")),
            tool_pattern: Some(glob::Pattern::new("shell").unwrap()),
            param_regex: None,
            plugin_dir: PathBuf::from("."),
        };
        // tool_match set but no tool_name provided = no match
        assert!(!matches_hook(&hook, None, None));
    }

    #[tokio::test]
    async fn test_subagent_started_event() {
        let tmp = TempDir::new().unwrap();
        let script = create_hook_script(
            tmp.path(),
            "started.sh",
            "#!/bin/bash\nread input\necho \"$input\"\n",
        );

        let mut dispatcher = HookDispatcher::new();
        dispatcher.register(
            make_hook(HookEvent::SubagentStarted, script),
            tmp.path().to_path_buf(),
        );

        let outcome = dispatcher
            .dispatch_subagent_started("parent-123", "researcher", "Find papers on RAG")
            .await;

        match outcome {
            HookOutcome::Info { output } => {
                assert!(output.contains("parent-123"));
                assert!(output.contains("researcher"));
                assert!(output.contains("Find papers on RAG"));
            }
            other => panic!("expected Info, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_subagent_completed_event() {
        let tmp = TempDir::new().unwrap();
        let script = create_hook_script(
            tmp.path(),
            "completed.sh",
            "#!/bin/bash\nread input\necho \"$input\"\n",
        );

        let mut dispatcher = HookDispatcher::new();
        dispatcher.register(
            make_hook(HookEvent::SubagentCompleted, script),
            tmp.path().to_path_buf(),
        );

        let outcome = dispatcher
            .dispatch_subagent_completed(
                "parent-123",
                "researcher",
                "Found 5 relevant papers",
                1500,
                true,
            )
            .await;

        match outcome {
            HookOutcome::Info { output } => {
                assert!(output.contains("parent-123"));
                assert!(output.contains("researcher"));
                assert!(output.contains("Found 5 relevant papers"));
                assert!(output.contains("1500"));
                assert!(output.contains("true"));
            }
            other => panic!("expected Info, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_subagent_completed_failure_event() {
        let tmp = TempDir::new().unwrap();
        let script = create_hook_script(
            tmp.path(),
            "completed.sh",
            "#!/bin/bash\nread input\necho \"$input\"\n",
        );

        let mut dispatcher = HookDispatcher::new();
        dispatcher.register(
            make_hook(HookEvent::SubagentCompleted, script),
            tmp.path().to_path_buf(),
        );

        let outcome = dispatcher
            .dispatch_subagent_completed("parent-456", "reviewer", "Connection timeout", 500, false)
            .await;

        match outcome {
            HookOutcome::Info { output } => {
                assert!(output.contains("parent-456"));
                assert!(output.contains("reviewer"));
                assert!(output.contains("Connection timeout"));
                assert!(output.contains("false"));
            }
            other => panic!("expected Info, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_subagent_events_no_hooks_registered() {
        let dispatcher = HookDispatcher::new();

        // Should return Allow when no hooks are registered
        let outcome = dispatcher
            .dispatch_subagent_started("sess-1", "agent", "task")
            .await;
        assert!(matches!(outcome, HookOutcome::Allow));

        let outcome = dispatcher
            .dispatch_subagent_completed("sess-1", "agent", "result", 100, true)
            .await;
        assert!(matches!(outcome, HookOutcome::Allow));
    }
}
