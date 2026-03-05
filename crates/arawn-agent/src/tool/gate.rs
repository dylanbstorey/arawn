//! Filesystem gate enforcement for tool execution.
//!
//! Validates file paths and routes shell commands through the OS-level sandbox.

use arawn_types::SharedFsGate;

use super::command_validator::{CommandValidation, CommandValidator};
use super::context::{Tool, ToolContext, ToolResult};
use super::output::OutputConfig;
use super::registry::ToolRegistry;
use crate::error::Result;

impl ToolRegistry {
    /// Validate and rewrite file paths in tool params against the filesystem gate.
    ///
    /// Returns `Ok(params)` with canonicalized paths on success, or
    /// `Err(ToolResult)` with an error message if access is denied.
    pub(super) fn validate_tool_paths(
        &self,
        tool_name: &str,
        mut params: serde_json::Value,
        gate: &SharedFsGate,
    ) -> std::result::Result<serde_json::Value, ToolResult> {
        match tool_name {
            "file_read" | "glob" | "grep" => {
                if let Some(path_str) = params.get("path").and_then(|v| v.as_str()) {
                    let path = std::path::Path::new(path_str);
                    match gate.validate_read(path) {
                        Ok(canonical) => {
                            params["path"] =
                                serde_json::Value::String(canonical.to_string_lossy().to_string());
                        }
                        Err(e) => {
                            return Err(ToolResult::error(format!("Access denied: {}", e)));
                        }
                    }
                }
            }
            "file_write" => {
                if let Some(path_str) = params.get("path").and_then(|v| v.as_str()) {
                    let path = std::path::Path::new(path_str);
                    match gate.validate_write(path) {
                        Ok(validated) => {
                            params["path"] =
                                serde_json::Value::String(validated.to_string_lossy().to_string());
                        }
                        Err(e) => {
                            return Err(ToolResult::error(format!("Access denied: {}", e)));
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(params)
    }

    /// Execute a shell tool through the OS-level sandbox.
    ///
    /// Validates the command against blocked patterns before passing it to
    /// the OS-level sandbox for execution.
    pub(super) async fn execute_shell_sandboxed(
        &self,
        _tool: &dyn Tool,
        params: &serde_json::Value,
        _ctx: &ToolContext,
        gate: &SharedFsGate,
        output_config: &OutputConfig,
    ) -> Result<ToolResult> {
        let command = params
            .get("command")
            .and_then(|v| v.as_str())
            .unwrap_or_default();

        // Validate command before sandbox execution
        let validator = CommandValidator::default();
        if let CommandValidation::Blocked(reason) = validator.validate(command) {
            return Ok(ToolResult::error(format!(
                "Command not allowed: {}",
                reason
            )));
        }

        let timeout = params
            .get("timeout")
            .and_then(|v| v.as_u64())
            .map(std::time::Duration::from_secs);

        match gate.sandbox_execute(command, timeout).await {
            Ok(output) => {
                let content = if output.stderr.is_empty() {
                    output.stdout
                } else if output.stdout.is_empty() {
                    output.stderr
                } else {
                    format!("{}\n\n--- stderr ---\n{}", output.stdout, output.stderr)
                };

                let result = if output.success {
                    ToolResult::text(content)
                } else {
                    ToolResult::error(format!(
                        "Command exited with code {}\n{}",
                        output.exit_code, content
                    ))
                };
                Ok(result.sanitize(output_config))
            }
            Err(e) => Ok(ToolResult::error(format!("Sandbox error: {}", e))),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;

    use super::super::context::{ToolContext, ToolResult};
    use super::super::output::OutputConfig;
    use super::super::registry::{MockTool, ToolRegistry};
    use arawn_types::is_gated_tool;

    // ─────────────────────────────────────────────────────────────────────────
    // Filesystem Gate Enforcement Tests
    // ─────────────────────────────────────────────────────────────────────────

    /// Mock filesystem gate for testing enforcement logic.
    struct MockFsGate {
        /// Paths that are allowed for read access.
        allowed_read: Vec<std::path::PathBuf>,
        /// Paths that are allowed for write access.
        allowed_write: Vec<std::path::PathBuf>,
        /// Working directory to report.
        work_dir: std::path::PathBuf,
        /// Shell command results to return.
        shell_result: std::sync::Mutex<Option<arawn_types::SandboxOutput>>,
    }

    impl MockFsGate {
        fn new(work_dir: impl Into<std::path::PathBuf>) -> Self {
            Self {
                allowed_read: Vec::new(),
                allowed_write: Vec::new(),
                work_dir: work_dir.into(),
                shell_result: std::sync::Mutex::new(None),
            }
        }

        fn allow_read(mut self, path: impl Into<std::path::PathBuf>) -> Self {
            self.allowed_read.push(path.into());
            self
        }

        fn allow_write(mut self, path: impl Into<std::path::PathBuf>) -> Self {
            self.allowed_write.push(path.into());
            self
        }

        fn with_shell_result(self, result: arawn_types::SandboxOutput) -> Self {
            *self.shell_result.lock().unwrap() = Some(result);
            self
        }
    }

    #[async_trait]
    impl arawn_types::FsGate for MockFsGate {
        fn validate_read(
            &self,
            path: &std::path::Path,
        ) -> std::result::Result<std::path::PathBuf, arawn_types::FsGateError> {
            for allowed in &self.allowed_read {
                if path.starts_with(allowed) || path == allowed {
                    return Ok(path.to_path_buf());
                }
            }
            Err(arawn_types::FsGateError::AccessDenied {
                path: path.to_path_buf(),
                reason: "path outside allowed read paths".to_string(),
            })
        }

        fn validate_write(
            &self,
            path: &std::path::Path,
        ) -> std::result::Result<std::path::PathBuf, arawn_types::FsGateError> {
            for allowed in &self.allowed_write {
                if path.starts_with(allowed) || path == allowed {
                    return Ok(path.to_path_buf());
                }
            }
            Err(arawn_types::FsGateError::AccessDenied {
                path: path.to_path_buf(),
                reason: "path outside allowed write paths".to_string(),
            })
        }

        fn working_dir(&self) -> &std::path::Path {
            &self.work_dir
        }

        async fn sandbox_execute(
            &self,
            _command: &str,
            _timeout: Option<std::time::Duration>,
        ) -> std::result::Result<arawn_types::SandboxOutput, arawn_types::FsGateError> {
            match self.shell_result.lock().unwrap().take() {
                Some(result) => Ok(result),
                None => Ok(arawn_types::SandboxOutput {
                    stdout: "sandboxed output".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                    success: true,
                }),
            }
        }
    }

    fn ctx_with_gate(gate: impl arawn_types::FsGate + 'static) -> ToolContext {
        ToolContext {
            fs_gate: Some(Arc::new(gate)),
            ..ToolContext::default()
        }
    }

    #[test]
    fn test_is_gated_tool() {
        assert!(is_gated_tool("file_read"));
        assert!(is_gated_tool("file_write"));
        assert!(is_gated_tool("glob"));
        assert!(is_gated_tool("grep"));
        assert!(is_gated_tool("shell"));

        assert!(!is_gated_tool("think"));
        assert!(!is_gated_tool("web_search"));
        assert!(!is_gated_tool("delegate"));
        assert!(!is_gated_tool("memory_store"));
        assert!(!is_gated_tool(""));
    }

    #[tokio::test]
    async fn test_gate_deny_by_default_no_gate() {
        let mut registry = ToolRegistry::new();
        registry.register(MockTool::new("file_read").with_response(ToolResult::text("secret")));

        // ToolContext with no fs_gate (default)
        let ctx = ToolContext::default();
        let params = serde_json::json!({"path": "/etc/passwd"});

        let result = registry
            .execute_with_config("file_read", params, &ctx, &OutputConfig::default())
            .await
            .unwrap();

        assert!(result.is_error());
        assert!(
            result
                .to_llm_content()
                .contains("requires a filesystem gate")
        );
    }

    #[tokio::test]
    async fn test_gate_deny_by_default_all_gated_tools() {
        let ctx = ToolContext::default();

        for tool_name in arawn_types::GATED_TOOLS {
            let mut registry = ToolRegistry::new();
            registry.register(MockTool::new(*tool_name));

            let result = registry
                .execute_with_config(
                    tool_name,
                    serde_json::json!({}),
                    &ctx,
                    &OutputConfig::default(),
                )
                .await
                .unwrap();

            assert!(
                result.is_error(),
                "Tool '{}' should be denied without gate",
                tool_name
            );
            assert!(
                result
                    .to_llm_content()
                    .contains("requires a filesystem gate"),
                "Tool '{}' error should mention gate requirement",
                tool_name
            );
        }
    }

    #[tokio::test]
    async fn test_gate_non_gated_tool_passes_through_without_gate() {
        let mut registry = ToolRegistry::new();
        registry.register(MockTool::new("think").with_response(ToolResult::text("thought")));

        // No gate set — non-gated tool should work fine
        let ctx = ToolContext::default();
        let result = registry
            .execute_with_config(
                "think",
                serde_json::json!({}),
                &ctx,
                &OutputConfig::default(),
            )
            .await
            .unwrap();

        assert!(result.is_success());
        assert_eq!(result.to_llm_content(), "thought");
    }

    #[tokio::test]
    async fn test_gate_file_read_allowed() {
        let mut registry = ToolRegistry::new();
        registry
            .register(MockTool::new("file_read").with_response(ToolResult::text("file contents")));

        let gate = MockFsGate::new("/work").allow_read("/work");
        let ctx = ctx_with_gate(gate);
        let params = serde_json::json!({"path": "/work/src/main.rs"});

        let result = registry
            .execute_with_config("file_read", params, &ctx, &OutputConfig::default())
            .await
            .unwrap();

        assert!(result.is_success());
        assert_eq!(result.to_llm_content(), "file contents");
    }

    #[tokio::test]
    async fn test_gate_file_read_denied() {
        let mut registry = ToolRegistry::new();
        registry
            .register(MockTool::new("file_read").with_response(ToolResult::text("should not see")));

        let gate = MockFsGate::new("/work").allow_read("/work");
        let ctx = ctx_with_gate(gate);
        let params = serde_json::json!({"path": "/etc/passwd"});

        let result = registry
            .execute_with_config("file_read", params, &ctx, &OutputConfig::default())
            .await
            .unwrap();

        assert!(result.is_error());
        assert!(result.to_llm_content().contains("Access denied"));
    }

    #[tokio::test]
    async fn test_gate_file_write_allowed() {
        let mut registry = ToolRegistry::new();
        registry.register(MockTool::new("file_write").with_response(ToolResult::text("written")));

        let gate = MockFsGate::new("/work").allow_write("/work");
        let ctx = ctx_with_gate(gate);
        let params = serde_json::json!({"path": "/work/output.txt", "content": "hello"});

        let result = registry
            .execute_with_config("file_write", params, &ctx, &OutputConfig::default())
            .await
            .unwrap();

        assert!(result.is_success());
    }

    #[tokio::test]
    async fn test_gate_file_write_denied() {
        let mut registry = ToolRegistry::new();
        registry
            .register(MockTool::new("file_write").with_response(ToolResult::text("should not")));

        let gate = MockFsGate::new("/work").allow_write("/work");
        let ctx = ctx_with_gate(gate);
        let params = serde_json::json!({"path": "/etc/shadow", "content": "malicious"});

        let result = registry
            .execute_with_config("file_write", params, &ctx, &OutputConfig::default())
            .await
            .unwrap();

        assert!(result.is_error());
        assert!(result.to_llm_content().contains("Access denied"));
    }

    #[tokio::test]
    async fn test_gate_glob_allowed() {
        let mut registry = ToolRegistry::new();
        registry
            .register(MockTool::new("glob").with_response(ToolResult::text("file1.rs\nfile2.rs")));

        let gate = MockFsGate::new("/work").allow_read("/work");
        let ctx = ctx_with_gate(gate);
        let params = serde_json::json!({"path": "/work/src"});

        let result = registry
            .execute_with_config("glob", params, &ctx, &OutputConfig::default())
            .await
            .unwrap();

        assert!(result.is_success());
    }

    #[tokio::test]
    async fn test_gate_glob_denied() {
        let mut registry = ToolRegistry::new();
        registry.register(MockTool::new("glob").with_response(ToolResult::text("should not")));

        let gate = MockFsGate::new("/work").allow_read("/work");
        let ctx = ctx_with_gate(gate);
        let params = serde_json::json!({"path": "/home/user/.ssh"});

        let result = registry
            .execute_with_config("glob", params, &ctx, &OutputConfig::default())
            .await
            .unwrap();

        assert!(result.is_error());
        assert!(result.to_llm_content().contains("Access denied"));
    }

    #[tokio::test]
    async fn test_gate_grep_denied() {
        let mut registry = ToolRegistry::new();
        registry.register(MockTool::new("grep").with_response(ToolResult::text("should not")));

        let gate = MockFsGate::new("/work").allow_read("/work");
        let ctx = ctx_with_gate(gate);
        let params = serde_json::json!({"path": "/var/log/secure", "pattern": "password"});

        let result = registry
            .execute_with_config("grep", params, &ctx, &OutputConfig::default())
            .await
            .unwrap();

        assert!(result.is_error());
        assert!(result.to_llm_content().contains("Access denied"));
    }

    #[tokio::test]
    async fn test_gate_shell_routed_through_sandbox() {
        let mut registry = ToolRegistry::new();
        // The mock tool should NOT be called — shell goes through sandbox
        registry.register(MockTool::new("shell").with_response(ToolResult::text("SHOULD NOT SEE")));

        let gate = MockFsGate::new("/work").with_shell_result(arawn_types::SandboxOutput {
            stdout: "sandboxed ls output".to_string(),
            stderr: String::new(),
            exit_code: 0,
            success: true,
        });
        let ctx = ctx_with_gate(gate);
        let params = serde_json::json!({"command": "ls -la"});

        let result = registry
            .execute_with_config("shell", params, &ctx, &OutputConfig::default())
            .await
            .unwrap();

        assert!(result.is_success());
        assert!(result.to_llm_content().contains("sandboxed ls output"));
        // Verify the mock tool was NOT called directly — sandbox bypasses tool.execute()
        assert!(
            !result.to_llm_content().contains("SHOULD NOT SEE"),
            "Shell should route through sandbox, not direct execution"
        );
    }

    #[tokio::test]
    async fn test_gate_shell_sandbox_failure() {
        let mut registry = ToolRegistry::new();
        registry.register(MockTool::new("shell"));

        let gate = MockFsGate::new("/work").with_shell_result(arawn_types::SandboxOutput {
            stdout: String::new(),
            stderr: "permission denied".to_string(),
            exit_code: 1,
            success: false,
        });
        let ctx = ctx_with_gate(gate);
        let params = serde_json::json!({"command": "cat /etc/shadow"});

        let result = registry
            .execute_with_config("shell", params, &ctx, &OutputConfig::default())
            .await
            .unwrap();

        assert!(result.is_error());
        assert!(result.to_llm_content().contains("permission denied"));
    }

    #[tokio::test]
    async fn test_gate_execute_raw_deny_by_default() {
        let mut registry = ToolRegistry::new();
        registry.register(MockTool::new("file_read"));

        let ctx = ToolContext::default();
        let result = registry
            .execute_raw(
                "file_read",
                serde_json::json!({"path": "/etc/passwd"}),
                &ctx,
            )
            .await
            .unwrap();

        assert!(result.is_error());
        assert!(
            result
                .to_llm_content()
                .contains("requires a filesystem gate")
        );
    }

    #[tokio::test]
    async fn test_gate_execute_raw_allowed_with_gate() {
        let mut registry = ToolRegistry::new();
        registry
            .register(MockTool::new("file_read").with_response(ToolResult::text("raw contents")));

        let gate = MockFsGate::new("/work").allow_read("/work");
        let ctx = ctx_with_gate(gate);
        let params = serde_json::json!({"path": "/work/file.txt"});

        let result = registry
            .execute_raw("file_read", params, &ctx)
            .await
            .unwrap();

        assert!(result.is_success());
        // execute_raw skips sanitization, so raw content comes through
        assert_eq!(result.to_llm_content(), "raw contents");
    }

    #[tokio::test]
    async fn test_gate_execute_raw_non_gated_passes_through() {
        let mut registry = ToolRegistry::new();
        registry.register(MockTool::new("think").with_response(ToolResult::text("deep thought")));

        let ctx = ToolContext::default();
        let result = registry
            .execute_raw("think", serde_json::json!({}), &ctx)
            .await
            .unwrap();

        assert!(result.is_success());
        assert_eq!(result.to_llm_content(), "deep thought");
    }

    #[tokio::test]
    async fn test_gate_file_read_no_path_param_passes_through() {
        let mut registry = ToolRegistry::new();
        registry.register(MockTool::new("file_read").with_response(ToolResult::text("ok")));

        // Gate is present but params have no "path" key — validation is skipped
        let gate = MockFsGate::new("/work").allow_read("/work");
        let ctx = ctx_with_gate(gate);
        let params = serde_json::json!({"content": "something"});

        let result = registry
            .execute_with_config("file_read", params, &ctx, &OutputConfig::default())
            .await
            .unwrap();

        // Tool executes without path validation (no path to validate)
        assert!(result.is_success());
    }

    #[tokio::test]
    async fn test_gate_shell_sandbox_combined_output() {
        let mut registry = ToolRegistry::new();
        registry.register(MockTool::new("shell"));

        let gate = MockFsGate::new("/work").with_shell_result(arawn_types::SandboxOutput {
            stdout: "stdout content".to_string(),
            stderr: "stderr content".to_string(),
            exit_code: 0,
            success: true,
        });
        let ctx = ctx_with_gate(gate);
        let params = serde_json::json!({"command": "make build"});

        let result = registry
            .execute_with_config("shell", params, &ctx, &OutputConfig::default())
            .await
            .unwrap();

        assert!(result.is_success());
        let content = result.to_llm_content();
        assert!(content.contains("stdout content"));
        assert!(content.contains("stderr content"));
        assert!(content.contains("--- stderr ---"));
    }

    #[tokio::test]
    async fn test_gate_shell_timeout_passed() {
        let mut registry = ToolRegistry::new();
        registry.register(MockTool::new("shell"));

        let gate = MockFsGate::new("/work");
        let ctx = ctx_with_gate(gate);
        // The timeout param is extracted from params and passed to sandbox_execute
        let params = serde_json::json!({"command": "sleep 100", "timeout": 30});

        let result = registry
            .execute_with_config("shell", params, &ctx, &OutputConfig::default())
            .await
            .unwrap();

        // Default MockFsGate returns success
        assert!(result.is_success());
    }

    // ─── Secret handle resolution tests ────────────────────────────────

    struct MockSecretResolver {
        secrets: std::collections::HashMap<String, String>,
    }

    impl MockSecretResolver {
        fn new(pairs: &[(&str, &str)]) -> Self {
            Self {
                secrets: pairs
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
            }
        }
    }

    impl arawn_types::SecretResolver for MockSecretResolver {
        fn resolve(&self, name: &str) -> Option<String> {
            self.secrets.get(name).cloned()
        }
        fn names(&self) -> Vec<String> {
            self.secrets.keys().cloned().collect()
        }
    }

    fn ctx_with_resolver(resolver: MockSecretResolver) -> ToolContext {
        ToolContext {
            secret_resolver: Some(Arc::new(resolver)),
            ..ToolContext::default()
        }
    }

    #[tokio::test]
    async fn test_gate_shell_blocked_command_rejected() {
        let mut registry = ToolRegistry::new();
        registry.register(MockTool::new("shell"));

        let gate = MockFsGate::new("/work").with_shell_result(arawn_types::SandboxOutput {
            stdout: "SHOULD NOT REACH".to_string(),
            stderr: String::new(),
            exit_code: 0,
            success: true,
        });
        let ctx = ctx_with_gate(gate);
        let params = serde_json::json!({"command": "rm -rf /"});

        let result = registry
            .execute_with_config("shell", params, &ctx, &OutputConfig::default())
            .await
            .unwrap();

        assert!(result.is_error());
        assert!(result.to_llm_content().contains("not allowed"));
        assert!(!result.to_llm_content().contains("SHOULD NOT REACH"));
    }

    #[tokio::test]
    async fn test_gate_shell_blocked_command_case_bypass() {
        let mut registry = ToolRegistry::new();
        registry.register(MockTool::new("shell"));

        let gate = MockFsGate::new("/work");
        let ctx = ctx_with_gate(gate);
        // Try to bypass with mixed case
        let params = serde_json::json!({"command": "RM -RF /"});

        let result = registry
            .execute_with_config("shell", params, &ctx, &OutputConfig::default())
            .await
            .unwrap();

        assert!(result.is_error());
        assert!(result.to_llm_content().contains("not allowed"));
    }

    #[tokio::test]
    async fn test_gate_shell_blocked_command_whitespace_bypass() {
        let mut registry = ToolRegistry::new();
        registry.register(MockTool::new("shell"));

        let gate = MockFsGate::new("/work");
        let ctx = ctx_with_gate(gate);
        // Try to bypass with extra whitespace
        let params = serde_json::json!({"command": "rm   -rf   /"});

        let result = registry
            .execute_with_config("shell", params, &ctx, &OutputConfig::default())
            .await
            .unwrap();

        assert!(result.is_error());
        assert!(result.to_llm_content().contains("not allowed"));
    }
}
