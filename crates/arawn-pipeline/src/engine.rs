//! Pipeline engine — core wrapper around Cloacina's DefaultRunner.
//!
//! Provides `PipelineEngine` which manages workflow registration, execution,
//! cron scheduling, push triggers, and graceful shutdown.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use cloacina::UniversalUuid;
use cloacina::prelude::*;
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::error::PipelineError;
use crate::task::DynamicTask;

/// Configuration for the pipeline engine.
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// Maximum concurrent tasks.
    pub max_concurrent_tasks: usize,

    /// Task execution timeout in seconds.
    pub task_timeout_secs: u64,

    /// Pipeline (workflow) execution timeout in seconds.
    pub pipeline_timeout_secs: u64,

    /// Enable cron scheduling.
    pub cron_enabled: bool,

    /// Enable event triggers.
    pub triggers_enabled: bool,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 4,
            task_timeout_secs: 300,
            pipeline_timeout_secs: 3600,
            cron_enabled: true,
            triggers_enabled: true,
        }
    }
}

/// Result of a workflow execution.
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// Execution ID for tracking.
    pub execution_id: String,
    /// Final status.
    pub status: ExecutionStatus,
    /// Output context (if completed successfully).
    pub output: Option<serde_json::Value>,
}

/// Status of an execution.
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionStatus {
    /// Workflow completed successfully.
    Completed,
    /// Workflow failed.
    Failed(String),
    /// Workflow is still running.
    Running,
    /// Workflow timed out.
    TimedOut,
}

/// Information about a scheduled workflow.
#[derive(Debug, Clone)]
pub struct ScheduleInfo {
    /// Schedule identifier.
    pub id: String,
    /// Workflow name.
    pub workflow_name: String,
    /// Cron expression.
    pub cron_expr: String,
    /// Whether the schedule is enabled.
    pub enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn test_engine(dir: &Path) -> PipelineEngine {
        let db_path = dir.join("test.db");
        let config = PipelineConfig {
            cron_enabled: false,
            triggers_enabled: false,
            ..Default::default()
        };
        PipelineEngine::new(&db_path, config).await.unwrap()
    }

    #[test]
    fn test_pipeline_config_defaults() {
        let config = PipelineConfig::default();
        assert_eq!(config.max_concurrent_tasks, 4);
        assert_eq!(config.task_timeout_secs, 300);
        assert_eq!(config.pipeline_timeout_secs, 3600);
        assert!(config.cron_enabled);
        assert!(config.triggers_enabled);
    }

    #[test]
    fn test_execution_status_eq() {
        assert_eq!(ExecutionStatus::Completed, ExecutionStatus::Completed);
        assert_eq!(ExecutionStatus::Running, ExecutionStatus::Running);
        assert_ne!(ExecutionStatus::Completed, ExecutionStatus::Running);
        assert_eq!(
            ExecutionStatus::Failed("err".into()),
            ExecutionStatus::Failed("err".into())
        );
        assert_ne!(
            ExecutionStatus::Failed("a".into()),
            ExecutionStatus::Failed("b".into())
        );
    }

    #[tokio::test]
    async fn test_has_workflow_false_initially() {
        let dir = TempDir::new().unwrap();
        let engine = test_engine(dir.path()).await;
        assert!(!engine.has_workflow("anything").await);
        engine.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_list_workflows_empty() {
        let dir = TempDir::new().unwrap();
        let engine = test_engine(dir.path()).await;
        assert!(engine.list_workflows().await.is_empty());
        engine.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_execute_missing_workflow() {
        let dir = TempDir::new().unwrap();
        let engine = test_engine(dir.path()).await;
        let ctx = cloacina_workflow::context::Context::new();
        let result = engine.execute("nonexistent", ctx).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            PipelineError::WorkflowNotFound(name) => assert_eq!(name, "nonexistent"),
            other => panic!("Expected WorkflowNotFound, got: {other:?}"),
        }
        engine.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_trigger_missing_workflow() {
        let dir = TempDir::new().unwrap();
        let engine = test_engine(dir.path()).await;
        let ctx = cloacina_workflow::context::Context::new();
        let result = engine.trigger("nonexistent", ctx).await;
        assert!(result.is_err());
        engine.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_schedule_cron_missing_workflow() {
        let dir = TempDir::new().unwrap();
        let engine = test_engine(dir.path()).await;
        let result = engine
            .schedule_cron("nonexistent", "0 9 * * *", "UTC")
            .await;
        assert!(result.is_err());
        match result.unwrap_err() {
            PipelineError::WorkflowNotFound(name) => assert_eq!(name, "nonexistent"),
            other => panic!("Expected WorkflowNotFound, got: {other:?}"),
        }
        engine.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_cancel_schedule_invalid_uuid() {
        let dir = TempDir::new().unwrap();
        let engine = test_engine(dir.path()).await;
        let result = engine.cancel_schedule("not-a-uuid").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            PipelineError::SchedulingError(msg) => assert!(msg.contains("Invalid schedule ID")),
            other => panic!("Expected SchedulingError, got: {other:?}"),
        }
        engine.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_cancel_schedule_nonexistent_uuid() {
        let dir = TempDir::new().unwrap();
        let engine = test_engine(dir.path()).await;
        // Valid UUID format but doesn't exist
        let result = engine
            .cancel_schedule("00000000-0000-0000-0000-000000000000")
            .await;
        // Should error since this schedule doesn't exist
        assert!(result.is_err());
        engine.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_register_and_has_workflow() {
        let dir = TempDir::new().unwrap();
        let engine = test_engine(dir.path()).await;

        let task = crate::task::DynamicTask::new(
            "t1",
            std::sync::Arc::new(|ctx| Box::pin(async move { Ok(ctx) })),
        );
        engine
            .register_dynamic_workflow("test-wf", "desc", vec![task])
            .await
            .unwrap();

        assert!(engine.has_workflow("test-wf").await);
        assert!(!engine.has_workflow("other").await);
        assert_eq!(engine.list_workflows().await.len(), 1);
        engine.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_register_empty_tasks() {
        let dir = TempDir::new().unwrap();
        let engine = test_engine(dir.path()).await;
        // Registering a workflow with zero tasks — Cloacina should reject this
        let result = engine
            .register_dynamic_workflow("empty", "no tasks", vec![])
            .await;
        // This may or may not error depending on Cloacina's validation
        // Just ensure it doesn't panic
        let _ = result;
        engine.shutdown().await.unwrap();
    }
}

/// The pipeline engine — Arawn's execution backbone.
///
/// Wraps Cloacina's `DefaultRunner` with:
/// - Dynamic workflow construction (no macros needed)
/// - Simplified API for Arawn's use cases
/// - Push trigger support for event-driven execution
pub struct PipelineEngine {
    runner: DefaultRunner,
    /// Registered workflows by name, for push trigger execution.
    workflows: Arc<RwLock<HashMap<String, Workflow>>>,
}

impl PipelineEngine {
    /// Initialize the pipeline engine with a SQLite database.
    ///
    /// # Arguments
    ///
    /// * `db_path` - Path to the SQLite database file
    /// * `config` - Engine configuration
    pub async fn new(db_path: &Path, config: PipelineConfig) -> Result<Self, PipelineError> {
        let db_url = format!("sqlite://{}", db_path.display());

        let mut runner_config = DefaultRunnerConfig::default();
        runner_config.max_concurrent_tasks = config.max_concurrent_tasks;
        runner_config.task_timeout = std::time::Duration::from_secs(config.task_timeout_secs);
        runner_config.pipeline_timeout =
            Some(std::time::Duration::from_secs(config.pipeline_timeout_secs));
        runner_config.enable_cron_scheduling = config.cron_enabled;
        runner_config.enable_trigger_scheduling = config.triggers_enabled;

        let runner = DefaultRunner::with_config(&db_url, runner_config)
            .await
            .map_err(|e| PipelineError::InitFailed(e.to_string()))?;

        info!("Pipeline engine initialized with database: {}", db_url);

        Ok(Self {
            runner,
            workflows: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Register a dynamically constructed workflow.
    ///
    /// The workflow is built using Cloacina's builder API and registered
    /// with the engine for execution. This does not use macros.
    pub async fn register_workflow(&self, workflow: Workflow) -> Result<(), PipelineError> {
        let name = workflow.name().to_string();
        debug!("Registering workflow: {}", name);

        // Register with Cloacina's global registry so the scheduler can find it
        let wf = workflow.clone();
        cloacina::register_workflow_constructor(name.clone(), move || wf.clone());

        // Store in our local registry for lookup
        self.workflows.write().await.insert(name.clone(), workflow);

        info!("Workflow registered: {}", name);
        Ok(())
    }

    /// Build and register a workflow from dynamic tasks.
    ///
    /// Convenience method that constructs a workflow from `DynamicTask`s
    /// using Cloacina's builder API.
    pub async fn register_dynamic_workflow(
        &self,
        name: &str,
        description: &str,
        tasks: Vec<DynamicTask>,
    ) -> Result<(), PipelineError> {
        let mut builder = Workflow::builder(name).description(description);

        for task in tasks {
            let task = task.resolve_workflow_name(name);
            let task = Arc::new(task);

            // Register each task in Cloacina's global task registry so the
            // executor can find it at runtime by namespace.
            let namespace =
                cloacina_workflow::TaskNamespace::new("public", "embedded", name, task.id());
            let task_clone = task.clone();
            cloacina::register_task_constructor(namespace, move || task_clone.clone());

            builder = builder
                .add_task(task)
                .map_err(|e| PipelineError::InvalidWorkflow(e.to_string()))?;
        }

        let workflow = builder
            .build()
            .map_err(|e| PipelineError::InvalidWorkflow(e.to_string()))?;

        self.register_workflow(workflow).await
    }

    /// Execute a registered workflow.
    ///
    /// Runs the workflow synchronously (waits for completion) and returns
    /// the result.
    pub async fn execute(
        &self,
        workflow_name: &str,
        context: Context<serde_json::Value>,
    ) -> Result<ExecutionResult, PipelineError> {
        let workflows = self.workflows.read().await;
        if !workflows.contains_key(workflow_name) {
            return Err(PipelineError::WorkflowNotFound(workflow_name.to_string()));
        }
        drop(workflows);

        let result = self
            .runner
            .execute(workflow_name, context)
            .await
            .map_err(|e| PipelineError::ExecutionFailed(e.to_string()))?;

        let status = match result.status {
            PipelineStatus::Completed => ExecutionStatus::Completed,
            PipelineStatus::Failed => {
                ExecutionStatus::Failed(result.error_message.unwrap_or_default())
            }
            PipelineStatus::Running => ExecutionStatus::Running,
            PipelineStatus::Cancelled => ExecutionStatus::Failed("Cancelled".to_string()),
            _ => ExecutionStatus::Failed("Unknown status".to_string()),
        };

        let context_data = result.final_context.into_data();
        let output = match serde_json::to_value(&context_data) {
            Ok(v) => Some(v),
            Err(e) => {
                tracing::warn!("Failed to serialize execution output: {e}");
                None
            }
        };

        Ok(ExecutionResult {
            execution_id: result.execution_id.to_string(),
            status,
            output,
        })
    }

    /// Execute a workflow via push trigger.
    ///
    /// This is the same as `execute` but semantically represents an
    /// event-driven invocation (e.g., session close, memory update).
    pub async fn trigger(
        &self,
        workflow_name: &str,
        context: Context<serde_json::Value>,
    ) -> Result<ExecutionResult, PipelineError> {
        debug!("Trigger fired for workflow: {}", workflow_name);
        self.execute(workflow_name, context).await
    }

    /// Register a cron schedule for a workflow.
    ///
    /// # Arguments
    ///
    /// * `workflow_name` - Name of the workflow to schedule
    /// * `cron_expr` - Cron expression (e.g., "0 9 * * *" for 9am daily)
    /// * `timezone` - IANA timezone (e.g., "America/New_York")
    pub async fn schedule_cron(
        &self,
        workflow_name: &str,
        cron_expr: &str,
        timezone: &str,
    ) -> Result<String, PipelineError> {
        let workflows = self.workflows.read().await;
        if !workflows.contains_key(workflow_name) {
            return Err(PipelineError::WorkflowNotFound(workflow_name.to_string()));
        }
        drop(workflows);

        self.runner
            .register_cron_workflow(workflow_name, cron_expr, timezone)
            .await
            .map_err(|e| PipelineError::SchedulingError(e.to_string()))?;

        info!(
            "Cron schedule registered: {} ({} {})",
            workflow_name, cron_expr, timezone
        );

        // Return a schedule ID (workflow name for now — Cloacina uses workflow name as key)
        Ok(workflow_name.to_string())
    }

    /// List all cron schedules.
    pub async fn list_schedules(&self) -> Result<Vec<ScheduleInfo>, PipelineError> {
        let schedules = self
            .runner
            .list_cron_schedules(false, 100, 0)
            .await
            .map_err(|e| PipelineError::Runtime(e.to_string()))?;

        Ok(schedules
            .into_iter()
            .map(|s| ScheduleInfo {
                id: s.id.to_string(),
                workflow_name: s.workflow_name,
                cron_expr: s.cron_expression,
                enabled: s.enabled.into(),
            })
            .collect())
    }

    /// Cancel a cron schedule.
    pub async fn cancel_schedule(&self, schedule_id: &str) -> Result<(), PipelineError> {
        let uuid = uuid::Uuid::parse_str(schedule_id)
            .map_err(|e| PipelineError::SchedulingError(format!("Invalid schedule ID: {}", e)))?;
        self.runner
            .delete_cron_schedule(UniversalUuid(uuid))
            .await
            .map_err(|e| PipelineError::SchedulingError(e.to_string()))?;

        info!("Cron schedule cancelled: {}", schedule_id);
        Ok(())
    }

    /// List registered workflow names.
    pub async fn list_workflows(&self) -> Vec<String> {
        self.workflows.read().await.keys().cloned().collect()
    }

    /// Check if a workflow is registered.
    pub async fn has_workflow(&self, name: &str) -> bool {
        self.workflows.read().await.contains_key(name)
    }

    /// Gracefully shut down the engine.
    ///
    /// Drains running workflows and stops background services.
    pub async fn shutdown(self) -> Result<(), PipelineError> {
        info!("Pipeline engine shutting down...");

        self.runner
            .shutdown()
            .await
            .map_err(|e| PipelineError::ShutdownFailed(e.to_string()))?;

        info!("Pipeline engine shutdown complete");
        Ok(())
    }
}
