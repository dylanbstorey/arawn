//! Dynamic task implementation for runtime-defined workflows.
//!
//! Provides `DynamicTask` — a concrete implementation of Cloacina's `Task` trait
//! that can be constructed at runtime without macros. This is the bridge between
//! declarative workflow definitions (TOML files) and Cloacina's execution engine.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use async_trait::async_trait;
use cloacina_workflow::context::Context;
use cloacina_workflow::error::TaskError;
use cloacina_workflow::namespace::TaskNamespace;
use cloacina_workflow::retry::RetryPolicy;
use cloacina_workflow::task::Task;

/// Type alias for the async function that executes a dynamic task.
///
/// Takes owned context, returns updated context or error.
pub type TaskFn = Arc<
    dyn Fn(
            Context<serde_json::Value>,
        ) -> Pin<
            Box<
                dyn Future<Output = std::result::Result<Context<serde_json::Value>, TaskError>>
                    + Send,
            >,
        > + Send
        + Sync,
>;

/// A task that can be constructed at runtime without macros.
///
/// This implements Cloacina's `Task` trait with all parameters provided at
/// construction time, enabling dynamic workflow definition from TOML files
/// or agent-generated specifications.
pub struct DynamicTask {
    id: String,
    dependencies: Vec<TaskNamespace>,
    retry_policy: RetryPolicy,
    execute_fn: TaskFn,
}

impl DynamicTask {
    /// Create a new dynamic task.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique task identifier within the workflow
    /// * `execute_fn` - Async function to execute
    pub fn new(id: impl Into<String>, execute_fn: TaskFn) -> Self {
        Self {
            id: id.into(),
            dependencies: Vec::new(),
            retry_policy: RetryPolicy::default(),
            execute_fn,
        }
    }

    /// Add a dependency on another task by its namespace.
    pub fn with_dependency(mut self, dep: TaskNamespace) -> Self {
        self.dependencies.push(dep);
        self
    }

    /// Add a dependency on another task by its short ID within the same workflow.
    ///
    /// Requires the workflow name to build the correct 4-part namespace
    /// (tenant::package::workflow::task) that Cloacina uses for resolution.
    pub fn with_dependency_id(mut self, task_id: &str) -> Self {
        // Workflow name will be filled in by register_dynamic_workflow
        self.dependencies.push(TaskNamespace::new(
            "public",
            "embedded",
            "__pending__",
            task_id,
        ));
        self
    }

    /// Resolve pending dependency namespaces with the actual workflow name.
    pub(crate) fn resolve_workflow_name(mut self, workflow_name: &str) -> Self {
        for ns in &mut self.dependencies {
            if ns.workflow_id == "__pending__" {
                ns.workflow_id = workflow_name.to_string();
            }
        }
        self
    }

    /// Set the retry policy for this task.
    pub fn with_retry_policy(mut self, policy: RetryPolicy) -> Self {
        self.retry_policy = policy;
        self
    }
}

impl std::fmt::Debug for DynamicTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DynamicTask")
            .field("id", &self.id)
            .field("dependencies", &self.dependencies)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn noop_fn() -> TaskFn {
        Arc::new(|ctx| Box::pin(async move { Ok(ctx) }))
    }

    fn failing_fn() -> TaskFn {
        Arc::new(|_ctx| {
            Box::pin(async move {
                Err(TaskError::ExecutionFailed {
                    message: "test failure".into(),
                    task_id: "t1".into(),
                    timestamp: chrono::Utc::now(),
                })
            })
        })
    }

    #[test]
    fn test_new_task_id() {
        let task = DynamicTask::new("my_task", noop_fn());
        assert_eq!(task.id(), "my_task");
    }

    #[test]
    fn test_new_task_no_dependencies() {
        let task = DynamicTask::new("t1", noop_fn());
        assert!(task.dependencies().is_empty());
    }

    #[test]
    fn test_new_task_default_retry_policy() {
        let task = DynamicTask::new("t1", noop_fn());
        let policy = task.retry_policy();
        assert_eq!(policy, RetryPolicy::default());
    }

    #[test]
    fn test_with_dependency() {
        let ns = TaskNamespace::new("public", "embedded", "wf1", "step_a");
        let task = DynamicTask::new("t1", noop_fn()).with_dependency(ns);
        assert_eq!(task.dependencies().len(), 1);
        assert_eq!(task.dependencies()[0].task_id, "step_a");
    }

    #[test]
    fn test_with_multiple_dependencies() {
        let task = DynamicTask::new("t1", noop_fn())
            .with_dependency_id("step_a")
            .with_dependency_id("step_b");
        assert_eq!(task.dependencies().len(), 2);
    }

    #[test]
    fn test_with_dependency_id_uses_pending() {
        let task = DynamicTask::new("t1", noop_fn()).with_dependency_id("step_a");
        assert_eq!(task.dependencies()[0].workflow_id, "__pending__");
        assert_eq!(task.dependencies()[0].task_id, "step_a");
    }

    #[test]
    fn test_resolve_workflow_name() {
        let task = DynamicTask::new("t1", noop_fn())
            .with_dependency_id("step_a")
            .resolve_workflow_name("my_workflow");
        assert_eq!(task.dependencies()[0].workflow_id, "my_workflow");
        assert_eq!(task.dependencies()[0].task_id, "step_a");
    }

    #[test]
    fn test_resolve_preserves_non_pending() {
        let explicit_ns = TaskNamespace::new("public", "embedded", "other_wf", "step_x");
        let task = DynamicTask::new("t1", noop_fn())
            .with_dependency(explicit_ns)
            .with_dependency_id("step_a")
            .resolve_workflow_name("my_workflow");
        // Explicit one should be unchanged
        assert_eq!(task.dependencies()[0].workflow_id, "other_wf");
        // Pending one should be resolved
        assert_eq!(task.dependencies()[1].workflow_id, "my_workflow");
    }

    #[test]
    fn test_with_retry_policy() {
        let policy = RetryPolicy {
            max_attempts: 3,
            initial_delay: std::time::Duration::from_secs(1),
            ..RetryPolicy::default()
        };
        let task = DynamicTask::new("t1", noop_fn()).with_retry_policy(policy.clone());
        assert_eq!(task.retry_policy().max_attempts, 3);
        assert_eq!(
            task.retry_policy().initial_delay,
            std::time::Duration::from_secs(1)
        );
    }

    #[tokio::test]
    async fn test_execute_success() {
        let task = DynamicTask::new(
            "t1",
            Arc::new(|mut ctx| {
                Box::pin(async move {
                    ctx.insert("result", json!(42)).unwrap();
                    Ok(ctx)
                })
            }),
        );
        let ctx = Context::new();
        let result = task.execute(ctx).await;
        assert!(result.is_ok());
        let ctx = result.unwrap();
        assert_eq!(ctx.get("result"), Some(&json!(42)));
    }

    #[tokio::test]
    async fn test_execute_failure() {
        let task = DynamicTask::new("t1", failing_fn());
        let ctx = Context::new();
        let result = task.execute(ctx).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_debug_format() {
        let task = DynamicTask::new("my_task", noop_fn()).with_dependency_id("dep1");
        let debug_str = format!("{:?}", task);
        assert!(debug_str.contains("my_task"));
        assert!(debug_str.contains("dep1"));
    }

    #[test]
    fn test_string_id_conversion() {
        let task = DynamicTask::new(String::from("from_string"), noop_fn());
        assert_eq!(task.id(), "from_string");
    }
}

#[async_trait]
impl Task for DynamicTask {
    async fn execute(
        &self,
        context: Context<serde_json::Value>,
    ) -> std::result::Result<Context<serde_json::Value>, TaskError> {
        (self.execute_fn)(context).await
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn dependencies(&self) -> &[TaskNamespace] {
        &self.dependencies
    }

    fn retry_policy(&self) -> RetryPolicy {
        self.retry_policy.clone()
    }
}
