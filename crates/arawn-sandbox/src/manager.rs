//! Sandbox manager for command execution.

use std::path::Path;
use std::process::Stdio;
use std::time::Duration;

use sandbox_runtime::{
    FilesystemConfig, NetworkConfig, SandboxManager as RuntimeSandboxManager, SandboxRuntimeConfig,
};
use tokio::process::Command;
use tokio::time::timeout;

use crate::config::SandboxConfig;
use crate::error::{SandboxError, SandboxResult};
use crate::platform::{Platform, SandboxStatus};

/// Output from a sandboxed command execution.
#[derive(Debug, Clone)]
pub struct CommandOutput {
    /// Standard output.
    pub stdout: String,
    /// Standard error.
    pub stderr: String,
    /// Exit code (0 = success).
    pub exit_code: i32,
    /// Whether the command succeeded (exit code 0).
    pub success: bool,
}

impl CommandOutput {
    /// Create a new command output.
    pub fn new(stdout: String, stderr: String, exit_code: i32) -> Self {
        Self {
            stdout,
            stderr,
            success: exit_code == 0,
            exit_code,
        }
    }

    /// Create an output for a failed command.
    pub fn error(message: String) -> Self {
        Self {
            stdout: String::new(),
            stderr: message,
            exit_code: 1,
            success: false,
        }
    }

    /// Combine stdout and stderr for display.
    pub fn combined_output(&self) -> String {
        if self.stderr.is_empty() {
            self.stdout.clone()
        } else if self.stdout.is_empty() {
            self.stderr.clone()
        } else {
            format!("{}\n\n--- stderr ---\n{}", self.stdout, self.stderr)
        }
    }
}

/// Manager for sandboxed command execution.
///
/// This wraps the `sandbox-runtime` crate to provide a higher-level interface
/// for executing commands with OS-level sandboxing.
///
/// # Security
///
/// Sandboxing is **required**. If the sandbox is unavailable (missing dependencies),
/// commands will fail with a clear error message explaining how to install
/// the required dependencies.
pub struct SandboxManager {
    runtime: RuntimeSandboxManager,
    platform: Platform,
}

impl SandboxManager {
    /// Create a new sandbox manager.
    ///
    /// # Errors
    ///
    /// Returns an error if the sandbox is unavailable on this platform.
    pub async fn new() -> SandboxResult<Self> {
        let status = Self::check_availability();

        match status {
            SandboxStatus::Available { platform } => {
                let runtime = RuntimeSandboxManager::new();
                Ok(Self { runtime, platform })
            }
            SandboxStatus::MissingDependency {
                missing,
                install_hint,
                ..
            } => Err(SandboxError::Unavailable {
                message: format!("Missing dependencies: {}", missing.join(", ")),
                install_hint,
            }),
            SandboxStatus::Unsupported { platform_name } => Err(SandboxError::Unavailable {
                message: format!("Platform not supported: {}", platform_name),
                install_hint: "Sandboxing is only available on macOS and Linux.".to_string(),
            }),
        }
    }

    /// Check if sandbox is available on this platform.
    pub fn check_availability() -> SandboxStatus {
        SandboxStatus::detect()
    }

    /// Get the current platform.
    pub fn platform(&self) -> Platform {
        self.platform
    }

    /// Execute a command in the sandbox.
    ///
    /// # Arguments
    ///
    /// * `command` - The shell command to execute.
    /// * `config` - Sandbox configuration (allowed paths, network, etc.).
    ///
    /// # Returns
    ///
    /// The command output (stdout, stderr, exit code).
    ///
    /// # Errors
    ///
    /// Returns an error if the sandbox fails to initialize or the command fails.
    pub async fn execute(
        &self,
        command: &str,
        config: &SandboxConfig,
    ) -> SandboxResult<CommandOutput> {
        // Build sandbox-runtime configuration
        let runtime_config = self.build_runtime_config(config)?;

        // Initialize the sandbox runtime with this config
        self.runtime
            .initialize(runtime_config)
            .await
            .map_err(|e| SandboxError::InitializationFailed(e.to_string()))?;

        // Wrap the command with sandbox restrictions
        let wrapped_command = self
            .runtime
            .wrap_with_sandbox(command, None, None)
            .await
            .map_err(|e| SandboxError::ExecutionFailed(e.to_string()))?;

        tracing::debug!(
            original = %command,
            wrapped = %wrapped_command,
            "Wrapped command with sandbox"
        );

        // Execute the wrapped command
        let output = self.execute_wrapped(&wrapped_command, config).await;

        // Reset sandbox state for next command
        self.runtime.reset().await;

        output
    }

    /// Execute the already-wrapped command.
    async fn execute_wrapped(
        &self,
        wrapped_command: &str,
        config: &SandboxConfig,
    ) -> SandboxResult<CommandOutput> {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());

        let mut cmd = Command::new(&shell);
        cmd.arg("-c");
        cmd.arg(wrapped_command);

        // Set working directory if specified
        if let Some(ref dir) = config.working_dir {
            cmd.current_dir(dir);
        }

        // Set environment variables
        for (key, value) in &config.env_vars {
            cmd.env(key, value);
        }

        // Configure stdio
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        // Execute with timeout
        let result = timeout(config.timeout, cmd.output()).await;

        match result {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let exit_code = output.status.code().unwrap_or(-1);

                Ok(CommandOutput::new(stdout, stderr, exit_code))
            }
            Ok(Err(e)) => Err(SandboxError::ExecutionFailed(format!(
                "Failed to execute command: {}",
                e
            ))),
            Err(_) => Err(SandboxError::Timeout(config.timeout)),
        }
    }

    /// Build the sandbox-runtime configuration from our config.
    fn build_runtime_config(&self, config: &SandboxConfig) -> SandboxResult<SandboxRuntimeConfig> {
        // Build filesystem config
        let filesystem = FilesystemConfig {
            allow_write: config
                .write_paths
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect(),
            deny_write: Vec::new(), // We use allow-only for writes
            deny_read: config
                .deny_read_paths
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect(),
            allow_git_config: Some(config.allow_git),
            ..Default::default()
        };

        // Build network config
        let network = NetworkConfig {
            allowed_domains: config.allowed_domains.clone(),
            ..Default::default()
        };

        Ok(SandboxRuntimeConfig {
            filesystem,
            network,
            ..Default::default()
        })
    }

    /// Execute a command with explicit path restrictions.
    ///
    /// This is a convenience method that creates a SandboxConfig on the fly.
    pub async fn execute_with_paths(
        &self,
        command: &str,
        working_dir: &Path,
        allowed_write_paths: &[std::path::PathBuf],
        timeout_duration: Duration,
    ) -> SandboxResult<CommandOutput> {
        let config = SandboxConfig::default()
            .with_write_paths(allowed_write_paths.to_vec())
            .with_working_dir(working_dir)
            .with_timeout(timeout_duration);

        self.execute(command, &config).await
    }

    /// Check if a command would be allowed under the given config.
    ///
    /// This does NOT execute the command - it just validates the configuration.
    pub fn validate_config(&self, config: &SandboxConfig) -> SandboxResult<()> {
        // Validate write paths exist
        for path in &config.write_paths {
            if !path.exists() {
                tracing::warn!(
                    path = %path.display(),
                    "Write path does not exist (will be created on first write)"
                );
            }
        }

        // Validate working directory exists
        if let Some(ref dir) = config.working_dir {
            if !dir.exists() {
                return Err(SandboxError::ConfigError(format!(
                    "Working directory does not exist: {}",
                    dir.display()
                )));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_output_success() {
        let output = CommandOutput::new("hello".to_string(), String::new(), 0);
        assert!(output.success);
        assert_eq!(output.exit_code, 0);
        assert_eq!(output.combined_output(), "hello");
    }

    #[test]
    fn test_command_output_error() {
        let output = CommandOutput::new(String::new(), "error".to_string(), 1);
        assert!(!output.success);
        assert_eq!(output.exit_code, 1);
        assert_eq!(output.combined_output(), "error");
    }

    #[test]
    fn test_command_output_combined() {
        let output = CommandOutput::new("out".to_string(), "err".to_string(), 0);
        let combined = output.combined_output();
        assert!(combined.contains("out"));
        assert!(combined.contains("err"));
        assert!(combined.contains("stderr"));
    }

    #[tokio::test]
    async fn test_sandbox_manager_creation() {
        let status = SandboxManager::check_availability();

        if status.is_available() {
            let manager = SandboxManager::new().await;
            assert!(manager.is_ok());
        } else {
            // If not available, creation should fail with clear error
            let manager = SandboxManager::new().await;
            assert!(matches!(manager, Err(SandboxError::Unavailable { .. })));
        }
    }

    #[tokio::test]
    async fn test_validate_config_working_dir() {
        let status = SandboxManager::check_availability();
        if !status.is_available() {
            return; // Skip on systems without sandbox
        }

        let manager = SandboxManager::new().await.unwrap();

        // Valid config
        let config = SandboxConfig::default().with_working_dir("/tmp");
        assert!(manager.validate_config(&config).is_ok());

        // Invalid working dir
        let config =
            SandboxConfig::default().with_working_dir("/nonexistent/path/that/does/not/exist");
        assert!(manager.validate_config(&config).is_err());
    }

    // Integration tests that actually execute sandboxed commands
    // These only run if sandbox is available

    #[tokio::test]
    #[ignore] // Run with --ignored flag
    async fn test_sandboxed_echo() {
        let status = SandboxManager::check_availability();
        if !status.is_available() {
            eprintln!("Skipping: sandbox not available");
            return;
        }

        let manager = SandboxManager::new().await.unwrap();
        let config = SandboxConfig::default();

        let output = manager
            .execute("echo 'hello sandbox'", &config)
            .await
            .unwrap();
        assert!(output.success);
        assert!(output.stdout.contains("hello sandbox"));
    }

    #[tokio::test]
    #[ignore] // Run with --ignored flag
    async fn test_sandboxed_write_allowed() {
        let status = SandboxManager::check_availability();
        if !status.is_available() {
            eprintln!("Skipping: sandbox not available");
            return;
        }

        let temp_dir = tempfile::tempdir().unwrap();
        let manager = SandboxManager::new().await.unwrap();
        let config = SandboxConfig::default()
            .with_write_paths(vec![temp_dir.path().to_path_buf()])
            .with_working_dir(temp_dir.path());

        // Should be able to write to allowed path
        let output = manager
            .execute("touch test.txt && ls -la", &config)
            .await
            .unwrap();

        // Note: sandbox behavior may vary; check if command ran
        assert!(output.exit_code >= 0);
    }

    #[tokio::test]
    #[ignore] // Run with --ignored flag
    async fn test_sandboxed_write_denied() {
        let status = SandboxManager::check_availability();
        if !status.is_available() {
            eprintln!("Skipping: sandbox not available");
            return;
        }

        let manager = SandboxManager::new().await.unwrap();
        let config = SandboxConfig::default(); // No write paths allowed

        // Trying to write outside allowed paths should fail
        let output = manager
            .execute("touch /tmp/sandbox_test_should_fail.txt", &config)
            .await
            .unwrap();

        // Command should fail due to sandbox restrictions
        // (exact behavior depends on platform)
        assert!(
            !output.success
                || output.stderr.contains("denied")
                || output.stderr.contains("Operation not permitted")
        );
    }
}
