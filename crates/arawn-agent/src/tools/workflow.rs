//! Workflow management tool for agent-driven workflow CRUD.
//!
//! Lets the agent create, run, schedule, list, cancel, and check status of
//! workflows via the pipeline engine.

use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{Value, json};
use tokio::sync::RwLock;
use tracing::debug;

use arawn_pipeline::sandbox::ScriptExecutor;
use arawn_pipeline::{PipelineEngine, RuntimeCatalog, WorkflowFile, build_executor_factory};

use crate::error::Result;
use crate::tool::{Tool, ToolContext, ToolResult};

/// Validate a workflow name for safe use as a filename component.
fn validate_name(name: &str) -> std::result::Result<(), String> {
    if name.is_empty() {
        return Err("Name cannot be empty".into());
    }
    if name.contains('/') || name.contains('\\') {
        return Err(format!("Name '{name}' must not contain path separators"));
    }
    if name.contains("..") {
        return Err(format!("Name '{name}' must not contain '..'"));
    }
    if name.starts_with('.') {
        return Err(format!("Name '{name}' must not start with '.'"));
    }
    if name.chars().any(|c| c.is_control()) {
        return Err(format!("Name '{name}' must not contain control characters"));
    }
    Ok(())
}

/// Agent-facing tool for workflow management.
///
/// Provides six actions: `create`, `run`, `schedule`, `list`, `cancel`, `status`.
pub struct WorkflowTool {
    engine: Arc<PipelineEngine>,
    workflow_dir: PathBuf,
    executor: Arc<ScriptExecutor>,
    catalog: Arc<RwLock<RuntimeCatalog>>,
}

impl WorkflowTool {
    /// Create a new workflow tool backed by the given engine, executor, and catalog.
    pub fn new(
        engine: Arc<PipelineEngine>,
        workflow_dir: PathBuf,
        executor: Arc<ScriptExecutor>,
        catalog: Arc<RwLock<RuntimeCatalog>>,
    ) -> Self {
        Self {
            engine,
            workflow_dir,
            executor,
            catalog,
        }
    }

    async fn action_create(&self, params: &Value) -> ToolResult {
        let name = match params.get("name").and_then(|v| v.as_str()) {
            Some(n) => n,
            None => return ToolResult::error("Missing required parameter 'name'"),
        };

        if let Err(e) = validate_name(name) {
            return ToolResult::error(e);
        }

        let definition = match params.get("definition").and_then(|v| v.as_str()) {
            Some(d) => d,
            None => return ToolResult::error("Missing required parameter 'definition'"),
        };

        // Validate TOML parses and the workflow definition is valid
        let wf = match WorkflowFile::from_toml(definition) {
            Ok(wf) => wf,
            Err(e) => return ToolResult::error(format!("Invalid workflow TOML: {e}")),
        };

        if let Err(e) = wf.workflow.validate() {
            return ToolResult::error(format!("Workflow validation failed: {e}"));
        }

        // Write to workflow directory
        if let Err(e) = std::fs::create_dir_all(&self.workflow_dir) {
            return ToolResult::error(format!("Failed to create workflow directory: {e}"));
        }

        let file_path = self.workflow_dir.join(format!("{name}.toml"));
        if let Err(e) = std::fs::write(&file_path, definition) {
            return ToolResult::error(format!("Failed to write workflow file: {e}"));
        }

        // Register with the engine using the real WASM runtime factory
        let factory = build_executor_factory(self.executor.clone(), self.catalog.clone());

        match wf.workflow.to_dynamic_tasks(&factory) {
            Ok(tasks) => {
                let desc = &wf.workflow.description;
                if let Err(e) = self
                    .engine
                    .register_dynamic_workflow(name, desc, tasks)
                    .await
                {
                    return ToolResult::error(format!(
                        "Workflow file written but registration failed: {e}"
                    ));
                }
            }
            Err(e) => {
                return ToolResult::error(format!(
                    "Workflow file written but task conversion failed: {e}"
                ));
            }
        }

        debug!(name = %name, path = %file_path.display(), "Workflow created and registered");

        ToolResult::json(json!({
            "created": name,
            "path": file_path.display().to_string(),
            "registered": true,
        }))
    }

    async fn action_run(&self, params: &Value) -> ToolResult {
        let workflow_name = params
            .get("name")
            .or_else(|| params.get("workflow_name"))
            .and_then(|v| v.as_str());
        let workflow_name = match workflow_name {
            Some(n) => n,
            None => return ToolResult::error("Missing required parameter 'name'"),
        };

        let context_val = params.get("context").cloned().unwrap_or(json!({}));

        // Build Cloacina context
        let mut ctx = cloacina_workflow::context::Context::new();
        if let Err(e) = ctx.insert("input".to_string(), context_val) {
            return ToolResult::error(format!("Failed to build context: {e}"));
        }

        match self.engine.execute(workflow_name, ctx).await {
            Ok(result) => {
                let status_str = match &result.status {
                    arawn_pipeline::ExecutionStatus::Completed => "Completed".to_string(),
                    arawn_pipeline::ExecutionStatus::Failed(msg) => format!("Failed: {msg}"),
                    arawn_pipeline::ExecutionStatus::Running => "Running".to_string(),
                    arawn_pipeline::ExecutionStatus::TimedOut => "TimedOut".to_string(),
                };

                let response = json!({
                    "execution_id": result.execution_id,
                    "status": status_str,
                    "output": result.output,
                });

                match &result.status {
                    arawn_pipeline::ExecutionStatus::Completed => ToolResult::json(response),
                    _ => ToolResult::error(format!(
                        "Workflow execution did not complete successfully: {}",
                        serde_json::to_string_pretty(&response)
                            .unwrap_or_else(|e| format!("(serialization error: {e})"))
                    )),
                }
            }
            Err(e) => ToolResult::error(format!("Execution failed: {e}")),
        }
    }

    async fn action_schedule(&self, params: &Value) -> ToolResult {
        let workflow_name = params
            .get("name")
            .or_else(|| params.get("workflow_name"))
            .and_then(|v| v.as_str());
        let workflow_name = match workflow_name {
            Some(n) => n,
            None => return ToolResult::error("Missing required parameter 'name'"),
        };

        let cron = match params.get("cron").and_then(|v| v.as_str()) {
            Some(c) => c,
            None => return ToolResult::error("Missing required parameter 'cron'"),
        };

        let timezone = params
            .get("timezone")
            .and_then(|v| v.as_str())
            .unwrap_or("UTC");

        match self
            .engine
            .schedule_cron(workflow_name, cron, timezone)
            .await
        {
            Ok(schedule_id) => ToolResult::json(json!({
                "schedule_id": schedule_id,
                "workflow_name": workflow_name,
                "cron": cron,
                "timezone": timezone,
            })),
            Err(e) => ToolResult::error(format!("Scheduling failed: {e}")),
        }
    }

    async fn action_list(&self) -> ToolResult {
        let workflows = self.engine.list_workflows().await;

        // Schedules may not be available if cron is disabled
        let schedule_list: Vec<Value> = match self.engine.list_schedules().await {
            Ok(schedules) => schedules
                .iter()
                .map(|s| {
                    json!({
                        "id": s.id,
                        "workflow_name": s.workflow_name,
                        "cron_expr": s.cron_expr,
                        "enabled": s.enabled,
                    })
                })
                .collect(),
            Err(e) => {
                debug!("Failed to list schedules (cron may be disabled): {e}");
                vec![]
            }
        };

        ToolResult::json(json!({
            "workflows": workflows,
            "schedules": schedule_list,
        }))
    }

    async fn action_cancel(&self, params: &Value) -> ToolResult {
        let schedule_id = match params.get("schedule_id").and_then(|v| v.as_str()) {
            Some(id) => id,
            None => return ToolResult::error("Missing required parameter 'schedule_id'"),
        };

        match self.engine.cancel_schedule(schedule_id).await {
            Ok(()) => ToolResult::json(json!({
                "cancelled": schedule_id,
            })),
            Err(e) => ToolResult::error(format!("Cancel failed: {e}")),
        }
    }

    async fn action_status(&self, params: &Value) -> ToolResult {
        let workflow_name = params
            .get("name")
            .or_else(|| params.get("workflow_name"))
            .and_then(|v| v.as_str());
        let workflow_name = match workflow_name {
            Some(n) => n,
            None => return ToolResult::error("Missing required parameter 'name'"),
        };

        let registered = self.engine.has_workflow(workflow_name).await;

        let matching: Vec<Value> = match self.engine.list_schedules().await {
            Ok(schedules) => schedules
                .iter()
                .filter(|s| s.workflow_name == workflow_name)
                .map(|s| {
                    json!({
                        "id": s.id,
                        "cron_expr": s.cron_expr,
                        "enabled": s.enabled,
                    })
                })
                .collect(),
            Err(e) => {
                debug!("Failed to list schedules for status check: {e}");
                vec![]
            }
        };

        ToolResult::json(json!({
            "workflow_name": workflow_name,
            "registered": registered,
            "schedules": matching,
        }))
    }
}

#[async_trait]
impl Tool for WorkflowTool {
    fn name(&self) -> &str {
        "workflow"
    }

    fn description(&self) -> &str {
        "Manage workflows: create TOML definitions, run workflows immediately, \
         schedule cron jobs, list registered workflows and schedules, cancel schedules, \
         or check workflow status. Use the 'action' parameter to select an operation."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["create", "run", "schedule", "list", "cancel", "status"],
                    "description": "The operation to perform"
                },
                "name": {
                    "type": "string",
                    "description": "Workflow name (used by all actions except 'list' and 'cancel')"
                },
                "definition": {
                    "type": "string",
                    "description": "TOML workflow definition string (for 'create' action)"
                },
                "context": {
                    "type": "object",
                    "description": "JSON context to pass to the workflow (for 'run' action)"
                },
                "cron": {
                    "type": "string",
                    "description": "Cron expression, e.g. '0 9 * * *' (for 'schedule' action)"
                },
                "timezone": {
                    "type": "string",
                    "description": "IANA timezone, e.g. 'America/New_York'. Defaults to 'UTC' (for 'schedule' action)"
                },
                "schedule_id": {
                    "type": "string",
                    "description": "Schedule UUID to cancel (for 'cancel' action)"
                }
            },
            "required": ["action"]
        })
    }

    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        if ctx.is_cancelled() {
            return Ok(ToolResult::error("Operation cancelled"));
        }

        let action = match params.get("action").and_then(|v| v.as_str()) {
            Some(a) => a,
            None => return Ok(ToolResult::error("Missing required parameter 'action'")),
        };

        let result = match action {
            "create" => self.action_create(&params).await,
            "run" => self.action_run(&params).await,
            "schedule" => self.action_schedule(&params).await,
            "list" => self.action_list().await,
            "cancel" => self.action_cancel(&params).await,
            "status" => self.action_status(&params).await,
            _ => ToolResult::error(format!(
                "Unknown action '{action}'. Valid actions: create, run, schedule, list, cancel, status"
            )),
        };

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool::ToolContext;
    use arawn_pipeline::PipelineConfig;
    use tempfile::TempDir;

    async fn setup() -> (WorkflowTool, TempDir) {
        let tmp = TempDir::new().unwrap();
        let db_path = tmp.path().join("test.db");
        let workflow_dir = tmp.path().join("workflows");

        let config = PipelineConfig {
            max_concurrent_tasks: 2,
            task_timeout_secs: 30,
            pipeline_timeout_secs: 60,
            cron_enabled: false,
            triggers_enabled: false,
        };

        let engine = PipelineEngine::new(&db_path, config).await.unwrap();
        let catalog = RuntimeCatalog::load(&tmp.path().join("runtimes")).unwrap();
        let executor =
            ScriptExecutor::new(tmp.path().join("cache"), std::time::Duration::from_secs(30))
                .unwrap();

        let tool = WorkflowTool::new(
            Arc::new(engine),
            workflow_dir,
            Arc::new(executor),
            Arc::new(RwLock::new(catalog)),
        );
        (tool, tmp)
    }

    #[tokio::test]
    async fn test_parameters_schema() {
        let (tool, _tmp) = setup().await;
        let schema = tool.parameters();
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["action"].is_object());
        let required = schema["required"].as_array().unwrap();
        assert!(required.contains(&json!("action")));
    }

    #[tokio::test]
    async fn test_create_writes_toml() {
        let (tool, _tmp) = setup().await;
        let ctx = ToolContext::default();

        let definition = r#"
[workflow]
name = "test_wf"
[[workflow.tasks]]
id = "step1"
action = { type = "tool", name = "echo" }
"#;

        let result = tool
            .execute(
                json!({
                    "action": "create",
                    "name": "test_wf",
                    "definition": definition,
                }),
                &ctx,
            )
            .await
            .unwrap();

        assert!(result.is_success(), "got: {:?}", result);

        // Verify file was written
        let file_path = tool.workflow_dir.join("test_wf.toml");
        assert!(file_path.exists());
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("test_wf"));
    }

    #[tokio::test]
    async fn test_create_invalid_toml() {
        let (tool, _tmp) = setup().await;
        let ctx = ToolContext::default();

        let result = tool
            .execute(
                json!({
                    "action": "create",
                    "name": "bad",
                    "definition": "this is not valid toml {{{",
                }),
                &ctx,
            )
            .await
            .unwrap();

        assert!(result.is_error());
        assert!(result.to_llm_content().contains("Invalid workflow TOML"));
    }

    #[tokio::test]
    async fn test_create_invalid_workflow() {
        let (tool, _tmp) = setup().await;
        let ctx = ToolContext::default();

        // Valid TOML but empty workflow name
        let definition = r#"
[workflow]
name = ""
[[workflow.tasks]]
id = "t1"
action = { type = "tool", name = "echo" }
"#;

        let result = tool
            .execute(
                json!({
                    "action": "create",
                    "name": "bad_wf",
                    "definition": definition,
                }),
                &ctx,
            )
            .await
            .unwrap();

        assert!(result.is_error());
        assert!(result.to_llm_content().contains("validation failed"));
    }

    #[tokio::test]
    async fn test_create_missing_params() {
        let (tool, _tmp) = setup().await;
        let ctx = ToolContext::default();

        let result = tool
            .execute(json!({"action": "create"}), &ctx)
            .await
            .unwrap();

        assert!(result.is_error());
        assert!(result.to_llm_content().contains("name"));
    }

    #[tokio::test]
    async fn test_list_empty() {
        let (tool, _tmp) = setup().await;
        let ctx = ToolContext::default();

        let result = tool.execute(json!({"action": "list"}), &ctx).await.unwrap();

        assert!(result.is_success(), "got: {:?}", result);
        if let ToolResult::Json { content } = &result {
            assert!(content["workflows"].as_array().unwrap().is_empty());
        } else {
            panic!("Expected JSON result");
        }
    }

    #[tokio::test]
    async fn test_run_unknown_workflow() {
        let (tool, _tmp) = setup().await;
        let ctx = ToolContext::default();

        let result = tool
            .execute(
                json!({
                    "action": "run",
                    "workflow_name": "nonexistent",
                }),
                &ctx,
            )
            .await
            .unwrap();

        assert!(result.is_error());
        assert!(
            result.to_llm_content().contains("not found")
                || result.to_llm_content().contains("failed"),
            "got: {}",
            result.to_llm_content()
        );
    }

    #[tokio::test]
    async fn test_cancel_invalid_id() {
        let (tool, _tmp) = setup().await;
        let ctx = ToolContext::default();

        let result = tool
            .execute(
                json!({
                    "action": "cancel",
                    "schedule_id": "not-a-uuid",
                }),
                &ctx,
            )
            .await
            .unwrap();

        assert!(result.is_error());
    }

    #[tokio::test]
    async fn test_status_unregistered() {
        let (tool, _tmp) = setup().await;
        let ctx = ToolContext::default();

        let result = tool
            .execute(
                json!({
                    "action": "status",
                    "workflow_name": "nonexistent",
                }),
                &ctx,
            )
            .await
            .unwrap();

        assert!(result.is_success());
        if let ToolResult::Json { content } = &result {
            assert_eq!(content["registered"], false);
            assert!(content["schedules"].as_array().unwrap().is_empty());
        } else {
            panic!("Expected JSON result");
        }
    }

    #[tokio::test]
    async fn test_unknown_action() {
        let (tool, _tmp) = setup().await;
        let ctx = ToolContext::default();

        let result = tool
            .execute(json!({"action": "explode"}), &ctx)
            .await
            .unwrap();

        assert!(result.is_error());
        assert!(result.to_llm_content().contains("Unknown action"));
    }

    #[tokio::test]
    async fn test_missing_action() {
        let (tool, _tmp) = setup().await;
        let ctx = ToolContext::default();

        let result = tool.execute(json!({}), &ctx).await.unwrap();

        assert!(result.is_error());
        assert!(result.to_llm_content().contains("action"));
    }

    // --- Sad path / boundary tests ---

    #[tokio::test]
    async fn test_create_name_with_path_traversal() {
        let (tool, _tmp) = setup().await;
        let ctx = ToolContext::default();

        let result = tool
            .execute(
                json!({
                    "action": "create",
                    "name": "../../../etc/evil",
                    "definition": "[workflow]\nname = \"x\"\n[[workflow.tasks]]\nid = \"t1\"\naction = { type = \"tool\", name = \"echo\" }",
                }),
                &ctx,
            )
            .await
            .unwrap();
        assert!(result.is_error());
        assert!(result.to_llm_content().contains("path separator"));
    }

    #[tokio::test]
    async fn test_create_empty_name() {
        let (tool, _tmp) = setup().await;
        let ctx = ToolContext::default();

        let result = tool
            .execute(
                json!({
                    "action": "create",
                    "name": "",
                    "definition": "[workflow]\nname = \"x\"",
                }),
                &ctx,
            )
            .await
            .unwrap();
        assert!(result.is_error());
        assert!(result.to_llm_content().contains("empty"));
    }

    #[tokio::test]
    async fn test_create_name_with_control_chars() {
        let (tool, _tmp) = setup().await;
        let ctx = ToolContext::default();

        let result = tool
            .execute(
                json!({
                    "action": "create",
                    "name": "bad\x00name",
                    "definition": "[workflow]\nname = \"x\"",
                }),
                &ctx,
            )
            .await
            .unwrap();
        assert!(result.is_error());
        assert!(result.to_llm_content().contains("control characters"));
    }

    #[tokio::test]
    async fn test_run_missing_name() {
        let (tool, _tmp) = setup().await;
        let ctx = ToolContext::default();

        let result = tool.execute(json!({"action": "run"}), &ctx).await.unwrap();
        assert!(result.is_error());
        assert!(result.to_llm_content().contains("name"));
    }

    #[tokio::test]
    async fn test_run_accepts_name_param() {
        let (tool, _tmp) = setup().await;
        let ctx = ToolContext::default();

        // Should fail because workflow doesn't exist, but not because of missing param
        let result = tool
            .execute(json!({"action": "run", "name": "nonexistent"}), &ctx)
            .await
            .unwrap();
        assert!(result.is_error());
        // Error should be about workflow not found, not about missing param
        assert!(
            !result
                .to_llm_content()
                .contains("Missing required parameter")
        );
    }

    #[tokio::test]
    async fn test_schedule_missing_name() {
        let (tool, _tmp) = setup().await;
        let ctx = ToolContext::default();

        let result = tool
            .execute(json!({"action": "schedule", "cron": "0 9 * * *"}), &ctx)
            .await
            .unwrap();
        assert!(result.is_error());
        assert!(result.to_llm_content().contains("name"));
    }

    #[tokio::test]
    async fn test_schedule_missing_cron() {
        let (tool, _tmp) = setup().await;
        let ctx = ToolContext::default();

        let result = tool
            .execute(json!({"action": "schedule", "name": "wf1"}), &ctx)
            .await
            .unwrap();
        assert!(result.is_error());
        assert!(result.to_llm_content().contains("cron"));
    }

    #[tokio::test]
    async fn test_cancel_missing_schedule_id() {
        let (tool, _tmp) = setup().await;
        let ctx = ToolContext::default();

        let result = tool
            .execute(json!({"action": "cancel"}), &ctx)
            .await
            .unwrap();
        assert!(result.is_error());
        assert!(result.to_llm_content().contains("schedule_id"));
    }

    #[tokio::test]
    async fn test_status_missing_name() {
        let (tool, _tmp) = setup().await;
        let ctx = ToolContext::default();

        let result = tool
            .execute(json!({"action": "status"}), &ctx)
            .await
            .unwrap();
        assert!(result.is_error());
        assert!(result.to_llm_content().contains("name"));
    }

    #[tokio::test]
    async fn test_action_is_number() {
        let (tool, _tmp) = setup().await;
        let ctx = ToolContext::default();

        let result = tool.execute(json!({"action": 42}), &ctx).await.unwrap();
        assert!(result.is_error());
        assert!(result.to_llm_content().contains("action"));
    }

    #[tokio::test]
    async fn test_create_empty_definition() {
        let (tool, _tmp) = setup().await;
        let ctx = ToolContext::default();

        let result = tool
            .execute(
                json!({
                    "action": "create",
                    "name": "test",
                    "definition": "",
                }),
                &ctx,
            )
            .await
            .unwrap();
        assert!(result.is_error());
    }
}
