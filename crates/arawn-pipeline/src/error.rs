//! Error types for the pipeline engine.

use thiserror::Error;

/// Result type for pipeline operations.
pub type Result<T> = std::result::Result<T, PipelineError>;

/// Errors that can occur during pipeline operations.
#[derive(Debug, Error)]
pub enum PipelineError {
    /// Failed to initialize the pipeline engine.
    #[error("Engine initialization failed: {0}")]
    InitFailed(String),

    /// Workflow not found.
    #[error("Workflow not found: {0}")]
    WorkflowNotFound(String),

    /// Workflow execution failed.
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    /// Invalid workflow definition.
    #[error("Invalid workflow: {0}")]
    InvalidWorkflow(String),

    /// Scheduling error.
    #[error("Scheduling error: {0}")]
    SchedulingError(String),

    /// Cloacina runtime error.
    #[error("Runtime error: {0}")]
    Runtime(String),

    /// Script compilation error (rustc stderr).
    #[error("Script compilation failed: {0}")]
    CompilationFailed(String),

    /// Script execution error (Wasmtime).
    #[error("Script execution failed: {0}")]
    ScriptFailed(String),

    /// Shutdown error.
    #[error("Shutdown error: {0}")]
    ShutdownFailed(String),
}

impl From<cloacina::PipelineError> for PipelineError {
    fn from(err: cloacina::PipelineError) -> Self {
        PipelineError::Runtime(err.to_string())
    }
}
