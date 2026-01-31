//! Shell command execution tool.
//!
//! Provides a tool for executing shell commands with safety controls.
//! Supports both standard process execution and PTY (pseudo-terminal) mode
//! for commands that need terminal emulation (colored output, interactive prompts).

use async_trait::async_trait;
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use serde_json::{Value, json};
use std::io::Read;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

use crate::error::Result;
use crate::tool::{ShellParams, Tool, ToolContext, ToolResult};

// ─────────────────────────────────────────────────────────────────────────────
// Shell Configuration
// ─────────────────────────────────────────────────────────────────────────────

/// Configuration for shell command execution.
#[derive(Debug, Clone)]
pub struct ShellConfig {
    /// Maximum execution time for commands.
    pub timeout: Duration,
    /// Working directory for command execution.
    pub working_dir: Option<String>,
    /// List of allowed command prefixes (if empty, all commands allowed).
    pub allowed_commands: Vec<String>,
    /// List of blocked command prefixes.
    pub blocked_commands: Vec<String>,
    /// Maximum output size in bytes.
    pub max_output_size: usize,
    /// PTY terminal size (rows, cols).
    pub pty_size: (u16, u16),
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            working_dir: None,
            allowed_commands: Vec::new(),
            blocked_commands: vec![
                "rm -rf /".to_string(),
                "rm -rf /*".to_string(),
                ":(){ :|:& };:".to_string(), // Fork bomb
                "dd if=/dev".to_string(),
                "> /dev/sda".to_string(),
                "mkfs".to_string(),
                "shutdown".to_string(),
                "reboot".to_string(),
                "halt".to_string(),
                "poweroff".to_string(),
            ],
            max_output_size: 1024 * 1024, // 1MB
            pty_size: (24, 80),           // Standard terminal size
        }
    }
}

impl ShellConfig {
    /// Create a new shell configuration with defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the command timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the working directory.
    pub fn with_working_dir(mut self, dir: impl Into<String>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }

    /// Set allowed commands (whitelist).
    pub fn with_allowed_commands(mut self, commands: Vec<String>) -> Self {
        self.allowed_commands = commands;
        self
    }

    /// Add a blocked command.
    pub fn block_command(mut self, command: impl Into<String>) -> Self {
        self.blocked_commands.push(command.into());
        self
    }

    /// Set maximum output size.
    pub fn with_max_output_size(mut self, size: usize) -> Self {
        self.max_output_size = size;
        self
    }

    /// Set PTY terminal size.
    pub fn with_pty_size(mut self, rows: u16, cols: u16) -> Self {
        self.pty_size = (rows, cols);
        self
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Shell Tool
// ─────────────────────────────────────────────────────────────────────────────

/// Shared working directory state across sessions.
pub type SharedWorkingDirs = Arc<Mutex<std::collections::HashMap<String, PathBuf>>>;

/// Tool for executing shell commands.
#[derive(Debug, Clone)]
pub struct ShellTool {
    config: ShellConfig,
    /// Per-session working directories for persistence.
    working_dirs: SharedWorkingDirs,
}

impl ShellTool {
    /// Create a new shell tool with default configuration.
    pub fn new() -> Self {
        Self {
            config: ShellConfig::default(),
            working_dirs: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    /// Create a shell tool with custom configuration.
    pub fn with_config(config: ShellConfig) -> Self {
        Self {
            config,
            working_dirs: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    /// Get the working directory for a session.
    fn get_working_dir(&self, session_id: &str) -> Option<PathBuf> {
        // First check session-specific working directory
        if let Ok(dirs) = self.working_dirs.lock() {
            if let Some(dir) = dirs.get(session_id) {
                return Some(dir.clone());
            }
        }
        // Fall back to config working directory
        self.config.working_dir.as_ref().map(PathBuf::from)
    }

    /// Set the working directory for a session.
    fn set_working_dir(&self, session_id: &str, dir: PathBuf) {
        if let Ok(mut dirs) = self.working_dirs.lock() {
            dirs.insert(session_id.to_string(), dir);
        }
    }

    /// Execute command in PTY mode with optional streaming callback.
    fn execute_pty_with_callback<F>(
        &self,
        command: &str,
        working_dir: Option<&PathBuf>,
        timeout_duration: Duration,
        mut on_output: F,
    ) -> std::result::Result<(String, bool), String>
    where
        F: FnMut(&str),
    {
        let pty_system = NativePtySystem::default();

        let (rows, cols) = self.config.pty_size;
        let pair = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| format!("Failed to open PTY: {}", e))?;

        // Build command
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        let mut cmd = CommandBuilder::new(&shell);
        cmd.arg("-c");
        cmd.arg(command);

        // Set working directory if specified
        if let Some(dir) = working_dir {
            cmd.cwd(dir);
        }

        // Spawn the command
        let mut child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| format!("Failed to spawn PTY command: {}", e))?;

        // Drop the slave so we get EOF when the child exits
        drop(pair.slave);

        // Read output with timeout
        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| format!("Failed to clone PTY reader: {}", e))?;

        let start = std::time::Instant::now();
        let mut output = Vec::new();
        let mut buf = [0u8; 4096];

        loop {
            // Check timeout
            if start.elapsed() > timeout_duration {
                let _ = child.kill();
                let output_str = String::from_utf8_lossy(&output).to_string();
                return Ok((
                    format!(
                        "{}\n\n[Command timed out after {:?}]",
                        output_str, timeout_duration
                    ),
                    false,
                ));
            }

            // Try to read with a short timeout
            match reader.read(&mut buf) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    let chunk = String::from_utf8_lossy(&buf[..n]);
                    on_output(&chunk);
                    output.extend_from_slice(&buf[..n]);
                    // Check output size limit
                    if output.len() > self.config.max_output_size {
                        let _ = child.kill();
                        let output_str = String::from_utf8_lossy(&output).to_string();
                        return Ok((
                            format!(
                                "{}\n\n[Output truncated at {} bytes]",
                                output_str, self.config.max_output_size
                            ),
                            false,
                        ));
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No data available, check if process exited
                    if let Ok(Some(_)) = child.try_wait() {
                        break;
                    }
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(e) => {
                    return Err(format!("Failed to read PTY output: {}", e));
                }
            }
        }

        // Wait for child to exit
        let status = child
            .wait()
            .map_err(|e| format!("Failed to wait for PTY child: {}", e))?;

        let output_str = String::from_utf8_lossy(&output).to_string();
        let success = status.exit_code() == 0;

        Ok((output_str, success))
    }

    /// Execute command in PTY mode (non-streaming).
    fn execute_pty(
        &self,
        command: &str,
        working_dir: Option<&PathBuf>,
        timeout_duration: Duration,
    ) -> std::result::Result<(String, bool), String> {
        self.execute_pty_with_callback(command, working_dir, timeout_duration, |_| {})
    }

    /// Check if a command is allowed.
    fn is_command_allowed(&self, command: &str) -> bool {
        // Check blocked commands first
        let cmd_lower = command.to_lowercase();
        for blocked in &self.config.blocked_commands {
            if cmd_lower.contains(&blocked.to_lowercase()) {
                return false;
            }
        }

        // If we have an allowlist, check it
        if !self.config.allowed_commands.is_empty() {
            for allowed in &self.config.allowed_commands {
                if cmd_lower.starts_with(&allowed.to_lowercase()) {
                    return true;
                }
            }
            return false;
        }

        true
    }

    /// Truncate output if it exceeds the maximum size.
    fn truncate_output(&self, output: String) -> String {
        if output.len() > self.config.max_output_size {
            let truncated = &output[..self.config.max_output_size];
            format!(
                "{}\n\n[Output truncated at {} bytes]",
                truncated, self.config.max_output_size
            )
        } else {
            output
        }
    }
}

impl Default for ShellTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for ShellTool {
    fn name(&self) -> &str {
        "shell"
    }

    fn description(&self) -> &str {
        "Execute a shell command and return its output. Use this for running system commands, scripts, or interacting with the operating system."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The shell command to execute"
                },
                "pty": {
                    "type": "boolean",
                    "description": "Run in PTY (pseudo-terminal) mode for commands needing terminal features (colored output, interactive prompts). Default: false"
                },
                "stream": {
                    "type": "boolean",
                    "description": "Stream output in real-time as it becomes available. Useful for long-running commands. Default: false"
                },
                "cwd": {
                    "type": "string",
                    "description": "Working directory for this command. If not specified, uses session's current directory or config default"
                },
                "timeout_secs": {
                    "type": "integer",
                    "description": "Command timeout in seconds. Default: 30"
                }
            },
            "required": ["command"]
        })
    }

    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        // Check cancellation
        if ctx.is_cancelled() {
            return Ok(ToolResult::error("Operation cancelled"));
        }

        // Parse and validate parameters using typed struct
        let shell_params = match ShellParams::try_from(params) {
            Ok(p) => p,
            Err(e) => return Ok(ToolResult::error(e.to_string())),
        };

        // Extract validated parameters
        let command = &shell_params.command;
        let use_pty = shell_params.pty;
        let use_stream = shell_params.stream;
        let explicit_cwd = shell_params.cwd.map(PathBuf::from);
        let timeout_secs = shell_params
            .timeout_secs
            .map(Duration::from_secs)
            .unwrap_or(self.config.timeout);

        // Determine if we should stream (stream param + context has sender)
        let should_stream = use_stream && ctx.is_streaming();

        // Check if command is allowed
        if !self.is_command_allowed(command) {
            return Ok(ToolResult::error(format!(
                "Command not allowed: {}",
                command
            )));
        }

        // Determine working directory
        let session_id = ctx.session_id.to_string();
        let working_dir = explicit_cwd.or_else(|| self.get_working_dir(&session_id));

        // Handle 'cd' commands specially for working directory persistence
        if let Some(target) = self.extract_cd_target(command) {
            // Resolve the target path
            let new_dir = self.resolve_cd_path(&target, &working_dir);

            // Verify directory exists
            if new_dir.is_dir() {
                self.set_working_dir(&session_id, new_dir.clone());
                return Ok(ToolResult::text(format!(
                    "Changed directory to: {}",
                    new_dir.display()
                )));
            } else {
                return Ok(ToolResult::error(format!(
                    "Directory not found: {}",
                    new_dir.display()
                )));
            }
        }

        // Execute the command
        if use_pty {
            // PTY mode execution (synchronous due to portable-pty API)
            let wd = working_dir.clone();
            let timeout_dur = timeout_secs;
            let tool = self.clone();
            let cmd = command.to_string();

            if should_stream {
                // Streaming PTY execution
                let sender = ctx.output_sender.clone();
                let result = tokio::task::spawn_blocking(move || {
                    tool.execute_pty_with_callback(&cmd, wd.as_ref(), timeout_dur, |chunk| {
                        if let Some(ref tx) = sender {
                            let _ = tx.send(chunk.to_string());
                        }
                    })
                })
                .await
                .map_err(|e| crate::error::AgentError::Tool(format!("PTY task failed: {}", e)))?;

                match result {
                    Ok((output, success)) => {
                        let output = self.truncate_output(output);
                        if success {
                            Ok(ToolResult::text(output))
                        } else {
                            Ok(ToolResult::error(format!("Command failed\n{}", output)))
                        }
                    }
                    Err(e) => Ok(ToolResult::error(e)),
                }
            } else {
                // Non-streaming PTY execution
                let result = tokio::task::spawn_blocking(move || {
                    tool.execute_pty(&cmd, wd.as_ref(), timeout_dur)
                })
                .await
                .map_err(|e| crate::error::AgentError::Tool(format!("PTY task failed: {}", e)))?;

                match result {
                    Ok((output, success)) => {
                        let output = self.truncate_output(output);
                        if success {
                            Ok(ToolResult::text(output))
                        } else {
                            Ok(ToolResult::error(format!("Command failed\n{}", output)))
                        }
                    }
                    Err(e) => Ok(ToolResult::error(e)),
                }
            }
        } else if should_stream {
            // Streaming standard process execution
            self.execute_standard_streaming(command, working_dir.as_ref(), timeout_secs, ctx)
                .await
        } else {
            // Standard process execution
            self.execute_standard(command, working_dir.as_ref(), timeout_secs)
                .await
        }
    }
}

impl ShellTool {
    /// Extract the target path from a cd command, if it is one.
    /// Returns None if this is not a cd command.
    fn extract_cd_target(&self, command: &str) -> Option<String> {
        let command = command.trim();

        if command == "cd" {
            return Some("~".to_string());
        }

        if let Some(path) = command.strip_prefix("cd ") {
            let path = path.trim();
            if path.is_empty() {
                return Some("~".to_string());
            }
            return Some(path.to_string());
        }

        None
    }

    /// Resolve a cd target path to an absolute path.
    fn resolve_cd_path(&self, target: &str, current_dir: &Option<PathBuf>) -> PathBuf {
        if target == "~" || target.is_empty() {
            return dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        }

        if target.starts_with('~') {
            // Handle ~ expansion
            if let Some(home) = dirs::home_dir() {
                if target == "~" {
                    return home;
                } else if let Some(rest) = target.strip_prefix("~/") {
                    return home.join(rest);
                }
            }
            // Fall through to treat as literal path
        }

        if target.starts_with('/') {
            // Absolute path
            PathBuf::from(target)
        } else {
            // Relative path
            let base = current_dir
                .clone()
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
            let joined = base.join(target);
            // Try to canonicalize to resolve .. and ., but fall back to joined path
            joined.canonicalize().unwrap_or(joined)
        }
    }

    /// Check if this is a cd command and return the new directory path.
    /// Deprecated: Use extract_cd_target + resolve_cd_path instead.
    #[cfg(test)]
    fn parse_cd_command(&self, command: &str, current_dir: &Option<PathBuf>) -> Option<PathBuf> {
        self.extract_cd_target(command)
            .map(|target| self.resolve_cd_path(&target, current_dir))
            .filter(|p| p.is_dir())
    }

    /// Standard process execution (non-PTY).
    async fn execute_standard(
        &self,
        command: &str,
        working_dir: Option<&PathBuf>,
        timeout_duration: Duration,
    ) -> Result<ToolResult> {
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg(command);

        // Set working directory if specified
        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        // Configure stdio
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        // Execute with timeout
        let result = timeout(timeout_duration, cmd.output()).await;

        match result {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                let mut result_text = String::new();

                if !stdout.is_empty() {
                    result_text.push_str(&stdout);
                }

                if !stderr.is_empty() {
                    if !result_text.is_empty() {
                        result_text.push_str("\n\n--- stderr ---\n");
                    }
                    result_text.push_str(&stderr);
                }

                if result_text.is_empty() {
                    result_text = "(no output)".to_string();
                }

                let result_text = self.truncate_output(result_text);

                if output.status.success() {
                    Ok(ToolResult::text(result_text))
                } else {
                    let exit_code = output.status.code().unwrap_or(-1);
                    Ok(ToolResult::error(format!(
                        "Command failed with exit code {}\n{}",
                        exit_code, result_text
                    )))
                }
            }
            Ok(Err(e)) => Ok(ToolResult::error(format!(
                "Failed to execute command: {}",
                e
            ))),
            Err(_) => Ok(ToolResult::error(format!(
                "Command timed out after {:?}",
                timeout_duration
            ))),
        }
    }

    /// Streaming standard process execution.
    /// Sends output chunks to the context's output sender as they arrive.
    async fn execute_standard_streaming(
        &self,
        command: &str,
        working_dir: Option<&PathBuf>,
        timeout_duration: Duration,
        ctx: &ToolContext,
    ) -> Result<ToolResult> {
        use tokio::io::{AsyncBufReadExt, BufReader};

        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg(command);

        // Set working directory if specified
        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        // Configure stdio for streaming
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        // Spawn the process
        let mut child = cmd.spawn().map_err(|e| {
            crate::error::AgentError::Tool(format!("Failed to spawn command: {}", e))
        })?;

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        let mut all_output = String::new();
        let start = std::time::Instant::now();
        let max_size = self.config.max_output_size;

        // Stream stdout
        if let Some(stdout) = stdout {
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = timeout(
                timeout_duration.saturating_sub(start.elapsed()),
                reader.next_line(),
            )
            .await
            .unwrap_or(Ok(None))
            {
                // Check timeout
                if start.elapsed() > timeout_duration {
                    let _ = child.kill().await;
                    all_output.push_str("\n\n[Command timed out]");
                    return Ok(ToolResult::error(self.truncate_output(all_output)));
                }

                // Send to stream
                ctx.send_output(&line);
                ctx.send_output("\n");

                // Accumulate for final result
                all_output.push_str(&line);
                all_output.push('\n');

                // Check size limit
                if all_output.len() > max_size {
                    let _ = child.kill().await;
                    all_output.push_str("\n[Output truncated]");
                    break;
                }
            }
        }

        // Stream stderr
        if let Some(stderr) = stderr {
            let mut reader = BufReader::new(stderr).lines();
            let mut has_stderr = false;
            while let Ok(Some(line)) = timeout(
                timeout_duration.saturating_sub(start.elapsed()),
                reader.next_line(),
            )
            .await
            .unwrap_or(Ok(None))
            {
                if !has_stderr {
                    has_stderr = true;
                    ctx.send_output("\n--- stderr ---\n");
                    all_output.push_str("\n--- stderr ---\n");
                }

                ctx.send_output(&line);
                ctx.send_output("\n");

                all_output.push_str(&line);
                all_output.push('\n');

                if all_output.len() > max_size {
                    let _ = child.kill().await;
                    break;
                }
            }
        }

        // Wait for process to complete
        let status = timeout(
            timeout_duration.saturating_sub(start.elapsed()),
            child.wait(),
        )
        .await
        .map_err(|_| {
            crate::error::AgentError::Tool(format!(
                "Command timed out after {:?}",
                timeout_duration
            ))
        })?
        .map_err(|e| {
            crate::error::AgentError::Tool(format!("Failed to wait for command: {}", e))
        })?;

        if all_output.is_empty() {
            all_output = "(no output)".to_string();
        }

        let all_output = self.truncate_output(all_output);

        if status.success() {
            Ok(ToolResult::text(all_output))
        } else {
            let exit_code = status.code().unwrap_or(-1);
            Ok(ToolResult::error(format!(
                "Command failed with exit code {}\n{}",
                exit_code, all_output
            )))
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_tool_metadata() {
        let tool = ShellTool::new();
        assert_eq!(tool.name(), "shell");
        assert!(!tool.description().is_empty());

        let params = tool.parameters();
        assert!(params.get("properties").is_some());
        assert!(params["properties"].get("command").is_some());
        assert!(params["properties"].get("pty").is_some());
        assert!(params["properties"].get("stream").is_some());
        assert!(params["properties"].get("cwd").is_some());
        assert!(params["properties"].get("timeout_secs").is_some());
    }

    #[test]
    fn test_shell_config_defaults() {
        let config = ShellConfig::default();
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert!(config.working_dir.is_none());
        assert!(!config.blocked_commands.is_empty());
        assert_eq!(config.pty_size, (24, 80));
    }

    #[test]
    fn test_command_blocking() {
        let tool = ShellTool::new();

        // These should be blocked
        assert!(!tool.is_command_allowed("rm -rf /"));
        assert!(!tool.is_command_allowed("RM -RF /"));
        assert!(!tool.is_command_allowed("shutdown now"));

        // These should be allowed
        assert!(tool.is_command_allowed("ls -la"));
        assert!(tool.is_command_allowed("echo hello"));
        assert!(tool.is_command_allowed("cat file.txt"));
    }

    #[test]
    fn test_command_whitelist() {
        let config = ShellConfig::new().with_allowed_commands(vec![
            "ls".to_string(),
            "cat".to_string(),
            "echo".to_string(),
        ]);
        let tool = ShellTool::with_config(config);

        // Only whitelisted commands should be allowed
        assert!(tool.is_command_allowed("ls -la"));
        assert!(tool.is_command_allowed("cat file.txt"));
        assert!(tool.is_command_allowed("echo hello"));

        // Others should be blocked
        assert!(!tool.is_command_allowed("rm file.txt"));
        assert!(!tool.is_command_allowed("wget http://example.com"));
    }

    #[tokio::test]
    async fn test_shell_echo() {
        let tool = ShellTool::new();
        let ctx = ToolContext::default();

        let result = tool
            .execute(json!({"command": "echo 'Hello, World!'"}), &ctx)
            .await
            .unwrap();

        assert!(result.is_success());
        assert!(result.to_llm_content().contains("Hello, World!"));
    }

    #[tokio::test]
    async fn test_shell_pwd() {
        let tool = ShellTool::new();
        let ctx = ToolContext::default();

        let result = tool.execute(json!({"command": "pwd"}), &ctx).await.unwrap();

        assert!(result.is_success());
        assert!(!result.to_llm_content().is_empty());
    }

    #[tokio::test]
    async fn test_shell_working_dir() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let config = ShellConfig::new().with_working_dir(temp_dir.path().to_str().unwrap());
        let tool = ShellTool::with_config(config);
        let ctx = ToolContext::default();

        let result = tool.execute(json!({"command": "pwd"}), &ctx).await.unwrap();

        assert!(result.is_success());
        // The output should contain the temp directory path
        let output = result.to_llm_content();
        assert!(output.contains(temp_dir.path().file_name().unwrap().to_str().unwrap()));
    }

    #[tokio::test]
    async fn test_shell_explicit_cwd() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let tool = ShellTool::new();
        let ctx = ToolContext::default();

        let result = tool
            .execute(
                json!({
                    "command": "pwd",
                    "cwd": temp_dir.path().to_str().unwrap()
                }),
                &ctx,
            )
            .await
            .unwrap();

        assert!(result.is_success());
        let output = result.to_llm_content();
        assert!(output.contains(temp_dir.path().file_name().unwrap().to_str().unwrap()));
    }

    #[tokio::test]
    async fn test_shell_cd_persistence() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let tool = ShellTool::new();
        let ctx = ToolContext::default();

        // cd to temp directory
        let cd_result = tool
            .execute(
                json!({"command": format!("cd {}", temp_dir.path().display())}),
                &ctx,
            )
            .await
            .unwrap();

        assert!(cd_result.is_success());
        assert!(cd_result.to_llm_content().contains("Changed directory"));

        // Subsequent command should use the new directory
        let pwd_result = tool.execute(json!({"command": "pwd"}), &ctx).await.unwrap();

        assert!(pwd_result.is_success());
        let output = pwd_result.to_llm_content();
        assert!(output.contains(temp_dir.path().file_name().unwrap().to_str().unwrap()));
    }

    #[tokio::test]
    async fn test_shell_cd_nonexistent() {
        let tool = ShellTool::new();
        let ctx = ToolContext::default();

        let result = tool
            .execute(
                json!({"command": "cd /nonexistent/path/that/doesnt/exist"}),
                &ctx,
            )
            .await
            .unwrap();

        assert!(result.is_error());
        assert!(result.to_llm_content().contains("not found"));
    }

    #[tokio::test]
    async fn test_shell_blocked_command() {
        let tool = ShellTool::new();
        let ctx = ToolContext::default();

        let result = tool
            .execute(json!({"command": "rm -rf /"}), &ctx)
            .await
            .unwrap();

        assert!(result.is_error());
        assert!(result.to_llm_content().contains("not allowed"));
    }

    #[tokio::test]
    async fn test_shell_failed_command() {
        let tool = ShellTool::new();
        let ctx = ToolContext::default();

        let result = tool
            .execute(json!({"command": "exit 1"}), &ctx)
            .await
            .unwrap();

        assert!(result.is_error());
        assert!(result.to_llm_content().contains("exit code 1"));
    }

    #[tokio::test]
    async fn test_shell_timeout() {
        let config = ShellConfig::new().with_timeout(Duration::from_millis(100));
        let tool = ShellTool::with_config(config);
        let ctx = ToolContext::default();

        let result = tool
            .execute(json!({"command": "sleep 5"}), &ctx)
            .await
            .unwrap();

        assert!(result.is_error());
        assert!(result.to_llm_content().contains("timed out"));
    }

    #[tokio::test]
    async fn test_shell_custom_timeout() {
        let tool = ShellTool::new();
        let ctx = ToolContext::default();

        let result = tool
            .execute(
                json!({
                    "command": "sleep 2",
                    "timeout_secs": 1
                }),
                &ctx,
            )
            .await
            .unwrap();

        assert!(result.is_error());
        assert!(result.to_llm_content().contains("timed out"));
    }

    #[tokio::test]
    async fn test_shell_pty_echo() {
        let tool = ShellTool::new();
        let ctx = ToolContext::default();

        let result = tool
            .execute(
                json!({
                    "command": "echo 'PTY Test'",
                    "pty": true
                }),
                &ctx,
            )
            .await
            .unwrap();

        assert!(result.is_success());
        assert!(result.to_llm_content().contains("PTY Test"));
    }

    #[tokio::test]
    async fn test_shell_pty_colored_output() {
        let tool = ShellTool::new();
        let ctx = ToolContext::default();

        // This command produces ANSI color codes in a terminal
        let result = tool
            .execute(
                json!({
                    "command": "printf '\\033[31mRed\\033[0m'",
                    "pty": true
                }),
                &ctx,
            )
            .await
            .unwrap();

        assert!(result.is_success());
        // PTY mode should capture the escape sequences
        let output = result.to_llm_content();
        assert!(output.contains("Red") || output.contains("\x1b[31m"));
    }

    #[tokio::test]
    async fn test_shell_streaming() {
        use tokio::sync::mpsc;

        let tool = ShellTool::new();
        let (tx, mut rx) = mpsc::unbounded_channel();

        let ctx = ToolContext::default().with_streaming(tx, "test-call-id");

        // Run command with streaming
        let result = tool
            .execute(
                json!({
                    "command": "echo 'Line 1' && echo 'Line 2' && echo 'Line 3'",
                    "stream": true
                }),
                &ctx,
            )
            .await
            .unwrap();

        assert!(result.is_success());

        // Collect streamed output
        let mut streamed = String::new();
        while let Ok(chunk) = rx.try_recv() {
            streamed.push_str(&chunk);
        }

        // Should have received the lines
        assert!(streamed.contains("Line 1"));
        assert!(streamed.contains("Line 2"));
        assert!(streamed.contains("Line 3"));
    }

    #[tokio::test]
    async fn test_shell_streaming_pty() {
        use tokio::sync::mpsc;

        let tool = ShellTool::new();
        let (tx, mut rx) = mpsc::unbounded_channel();

        let ctx = ToolContext::default().with_streaming(tx, "test-call-id");

        // Run PTY command with streaming
        let result = tool
            .execute(
                json!({
                    "command": "echo 'PTY Streamed'",
                    "pty": true,
                    "stream": true
                }),
                &ctx,
            )
            .await
            .unwrap();

        assert!(result.is_success());

        // Collect streamed output
        let mut streamed = String::new();
        while let Ok(chunk) = rx.try_recv() {
            streamed.push_str(&chunk);
        }

        // Should have received the output
        assert!(streamed.contains("PTY Streamed"));
    }

    #[test]
    fn test_output_truncation() {
        let config = ShellConfig::new().with_max_output_size(100);
        let tool = ShellTool::with_config(config);

        let long_output = "x".repeat(200);
        let truncated = tool.truncate_output(long_output);

        assert!(truncated.len() < 200);
        assert!(truncated.contains("[Output truncated"));
    }

    #[test]
    fn test_parse_cd_command() {
        let tool = ShellTool::new();

        // Basic cd parsing
        assert!(tool.parse_cd_command("cd", &None).is_some());
        assert!(tool.parse_cd_command("cd ~", &None).is_some());
        assert!(tool.parse_cd_command("cd /tmp", &None).is_some());

        // Not cd commands
        assert!(tool.parse_cd_command("echo cd", &None).is_none());
        assert!(tool.parse_cd_command("ls", &None).is_none());
    }

    #[test]
    fn test_pty_size_config() {
        let config = ShellConfig::new().with_pty_size(40, 120);
        assert_eq!(config.pty_size, (40, 120));
    }
}
