//! Error types for sandbox operations.

use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during sandbox operations.
#[derive(Debug, Error)]
pub enum SandboxError {
    /// Sandbox is not available on this system.
    #[error("Sandbox unavailable: {message}\n\n{install_hint}")]
    Unavailable {
        message: String,
        install_hint: String,
    },

    /// Sandbox initialization failed.
    #[error("Failed to initialize sandbox: {0}")]
    InitializationFailed(String),

    /// Command execution failed within sandbox.
    #[error("Sandbox execution failed: {0}")]
    ExecutionFailed(String),

    /// Path is not allowed for the requested operation.
    #[error("Path not allowed: {path}")]
    PathNotAllowed { path: PathBuf },

    /// Configuration error.
    #[error("Invalid sandbox configuration: {0}")]
    ConfigError(String),

    /// Timeout waiting for command.
    #[error("Command timed out after {0:?}")]
    Timeout(std::time::Duration),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// sandbox-runtime crate error.
    #[error("Sandbox runtime error: {0}")]
    Runtime(String),
}

/// Result type for sandbox operations.
pub type SandboxResult<T> = std::result::Result<T, SandboxError>;
