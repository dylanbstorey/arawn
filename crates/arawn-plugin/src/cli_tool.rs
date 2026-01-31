//! CLI-wrapper tool adapter.
//!
//! Wraps an external CLI executable as an Arawn [`Tool`]. The adapter spawns
//! the subprocess, sends JSON parameters on stdin, reads a JSON response from
//! stdout, and maps it to a [`ToolResult`].
//!
//! ## Protocol
//!
//! **stdin** (JSON): the tool parameters object.
//!
//! **stdout** (JSON): one of:
//! - `{"success": true, "content": "..."}` → `ToolResult::Text`
//! - `{"error": "..."}` → `ToolResult::Error`
//!
//! **stderr**: captured and logged via tracing.
//!
//! **exit code**: non-zero is treated as an error.

use arawn_agent::error::{AgentError, Result as AgentResult};
use arawn_agent::tool::{Tool, ToolContext, ToolResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;
use tokio::io::AsyncWriteExt;

use crate::types::CliToolDef;

/// Default timeout for CLI tool subprocesses.
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// A tool backed by a CLI executable using the JSON stdin/stdout protocol.
#[derive(Debug, Clone)]
pub struct CliPluginTool {
    /// Tool definition from the plugin manifest.
    def: CliToolDef,
    /// Working directory for the subprocess (plugin directory).
    plugin_dir: PathBuf,
    /// Subprocess timeout.
    timeout: Duration,
}

impl CliPluginTool {
    /// Create a new CLI plugin tool from a definition and plugin directory.
    pub fn new(def: CliToolDef, plugin_dir: PathBuf) -> Self {
        Self {
            def,
            plugin_dir,
            timeout: DEFAULT_TIMEOUT,
        }
    }

    /// Set the subprocess timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

/// JSON response from a CLI tool.
#[derive(Debug, Deserialize, Serialize)]
struct CliToolResponse {
    #[serde(default)]
    success: Option<bool>,
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    error: Option<String>,
}

#[async_trait]
impl Tool for CliPluginTool {
    fn name(&self) -> &str {
        &self.def.name
    }

    fn description(&self) -> &str {
        &self.def.description
    }

    fn parameters(&self) -> serde_json::Value {
        self.def.parameters.clone()
    }

    async fn execute(
        &self,
        params: serde_json::Value,
        ctx: &ToolContext,
    ) -> AgentResult<ToolResult> {
        let input_json = serde_json::to_string(&params).unwrap_or_default();

        // Expand ${CLAUDE_PLUGIN_ROOT} in the command path
        let expanded_command = crate::expand_plugin_root_path(&self.def.command, &self.plugin_dir);

        let mut child = tokio::process::Command::new(&expanded_command)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .current_dir(&self.plugin_dir)
            .env("ARAWN_PLUGIN_DIR", &self.plugin_dir)
            .env(crate::CLAUDE_PLUGIN_ROOT_VAR, &self.plugin_dir)
            .env("ARAWN_SESSION_ID", ctx.session_id.to_string())
            .spawn()
            .map_err(|e| {
                AgentError::Tool(format!(
                    "failed to spawn CLI tool '{}': {}",
                    self.def.name, e
                ))
            })?;

        // Write params to stdin
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(input_json.as_bytes()).await;
            let _ = stdin.shutdown().await;
        }

        // Wait with timeout
        let output = match tokio::time::timeout(self.timeout, child.wait_with_output()).await {
            Ok(Ok(output)) => output,
            Ok(Err(e)) => {
                return Ok(ToolResult::Error {
                    message: format!("CLI tool process error: {}", e),
                    recoverable: true,
                });
            }
            Err(_) => {
                return Ok(ToolResult::Error {
                    message: format!(
                        "CLI tool '{}' timed out after {}s",
                        self.def.name,
                        self.timeout.as_secs()
                    ),
                    recoverable: true,
                });
            }
        };

        // Log stderr
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.is_empty() {
            tracing::debug!(
                tool = %self.def.name,
                stderr = %stderr.trim(),
                "CLI tool stderr"
            );
        }

        // Check exit code
        if !output.status.success() {
            let exit_code = output.status.code().unwrap_or(-1);
            let error_msg = if stderr.is_empty() {
                format!(
                    "CLI tool '{}' exited with code {}",
                    self.def.name, exit_code
                )
            } else {
                format!(
                    "CLI tool '{}' exited with code {}: {}",
                    self.def.name,
                    exit_code,
                    stderr.trim()
                )
            };
            return Ok(ToolResult::Error {
                message: error_msg,
                recoverable: true,
            });
        }

        // Parse stdout as JSON
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stdout_trimmed = stdout.trim();

        if stdout_trimmed.is_empty() {
            return Ok(ToolResult::Text {
                content: String::new(),
            });
        }

        match serde_json::from_str::<CliToolResponse>(stdout_trimmed) {
            Ok(response) => {
                if let Some(error) = response.error {
                    Ok(ToolResult::Error {
                        message: error,
                        recoverable: true,
                    })
                } else {
                    Ok(ToolResult::Text {
                        content: response.content.unwrap_or_default(),
                    })
                }
            }
            Err(_) => {
                // If stdout isn't valid JSON protocol, return raw output
                Ok(ToolResult::Text {
                    content: stdout_trimmed.to_string(),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use tempfile::TempDir;

    fn create_tool_script(dir: &std::path::Path, name: &str, script: &str) -> PathBuf {
        let path = dir.join(name);
        fs::write(&path, script).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        path
    }

    fn make_def(name: &str, command: PathBuf) -> CliToolDef {
        CliToolDef {
            name: name.to_string(),
            description: format!("Test tool: {}", name),
            command,
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "input": { "type": "string" }
                }
            }),
        }
    }

    #[tokio::test]
    async fn test_successful_execution() {
        let tmp = TempDir::new().unwrap();
        let script = create_tool_script(
            tmp.path(),
            "echo.sh",
            r#"#!/bin/bash
read input
echo '{"success": true, "content": "hello world"}'
"#,
        );

        let tool = CliPluginTool::new(make_def("echo", script), tmp.path().to_path_buf());
        let ctx = ToolContext::default();
        let result = tool
            .execute(serde_json::json!({"input": "test"}), &ctx)
            .await
            .unwrap();

        match result {
            ToolResult::Text { content } => assert_eq!(content, "hello world"),
            other => panic!("expected Text, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_error_response() {
        let tmp = TempDir::new().unwrap();
        let script = create_tool_script(
            tmp.path(),
            "error.sh",
            r#"#!/bin/bash
read input
echo '{"error": "something went wrong"}'
"#,
        );

        let tool = CliPluginTool::new(make_def("error", script), tmp.path().to_path_buf());
        let ctx = ToolContext::default();
        let result = tool.execute(serde_json::json!({}), &ctx).await.unwrap();

        match result {
            ToolResult::Error { message, .. } => assert_eq!(message, "something went wrong"),
            other => panic!("expected Error, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_nonzero_exit_code() {
        let tmp = TempDir::new().unwrap();
        let script = create_tool_script(
            tmp.path(),
            "fail.sh",
            r#"#!/bin/bash
echo "bad thing happened" >&2
exit 1
"#,
        );

        let tool = CliPluginTool::new(make_def("fail", script), tmp.path().to_path_buf());
        let ctx = ToolContext::default();
        let result = tool.execute(serde_json::json!({}), &ctx).await.unwrap();

        match result {
            ToolResult::Error { message, .. } => {
                assert!(message.contains("exited with code 1"));
                assert!(message.contains("bad thing happened"));
            }
            other => panic!("expected Error, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_timeout() {
        let tmp = TempDir::new().unwrap();
        let script = create_tool_script(
            tmp.path(),
            "slow.sh",
            r#"#!/bin/bash
sleep 60
"#,
        );

        let tool = CliPluginTool::new(make_def("slow", script), tmp.path().to_path_buf())
            .with_timeout(Duration::from_millis(100));
        let ctx = ToolContext::default();
        let result = tool.execute(serde_json::json!({}), &ctx).await.unwrap();

        match result {
            ToolResult::Error { message, .. } => assert!(message.contains("timed out")),
            other => panic!("expected Error, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_malformed_json_output() {
        let tmp = TempDir::new().unwrap();
        let script = create_tool_script(
            tmp.path(),
            "raw.sh",
            r#"#!/bin/bash
echo "just plain text output"
"#,
        );

        let tool = CliPluginTool::new(make_def("raw", script), tmp.path().to_path_buf());
        let ctx = ToolContext::default();
        let result = tool.execute(serde_json::json!({}), &ctx).await.unwrap();

        // Non-JSON output should be returned as raw text
        match result {
            ToolResult::Text { content } => assert_eq!(content, "just plain text output"),
            other => panic!("expected Text, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_tool_trait_methods() {
        let tmp = TempDir::new().unwrap();
        let def = make_def("test-tool", tmp.path().join("test.sh"));
        let tool = CliPluginTool::new(def, tmp.path().to_path_buf());

        assert_eq!(tool.name(), "test-tool");
        assert_eq!(tool.description(), "Test tool: test-tool");
        assert!(tool.parameters()["properties"]["input"]["type"] == "string");
    }

    #[tokio::test]
    async fn test_stdin_passthrough() {
        let tmp = TempDir::new().unwrap();
        let script = create_tool_script(
            tmp.path(),
            "mirror.sh",
            r#"#!/bin/bash
read input
# Echo back the input we received as the content
echo "{\"success\": true, \"content\": $input}"
"#,
        );

        let tool = CliPluginTool::new(make_def("mirror", script), tmp.path().to_path_buf());
        let ctx = ToolContext::default();
        let params = serde_json::json!({"input": "hello"});
        let result = tool.execute(params.clone(), &ctx).await.unwrap();

        match result {
            ToolResult::Text { content } => {
                // The content should contain our input JSON
                assert!(content.contains("hello"));
            }
            other => panic!("expected Text, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_missing_command() {
        let tmp = TempDir::new().unwrap();
        let def = make_def("missing", PathBuf::from("/nonexistent/tool.sh"));
        let tool = CliPluginTool::new(def, tmp.path().to_path_buf());
        let ctx = ToolContext::default();

        let result = tool.execute(serde_json::json!({}), &ctx).await;
        assert!(result.is_err());
    }
}
