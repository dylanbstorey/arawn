//! Real `ActionExecutorFactory` implementation that bridges workflow task
//! definitions to WASM runtime execution via `ScriptExecutor`.

use std::sync::Arc;

use serde_json::Value;
use tokio::sync::RwLock;
use tracing::debug;

use crate::catalog::RuntimeCatalog;
use crate::definition::{ActionDefinition, ActionExecutorFactory};
use crate::protocol::RuntimeInput;
use crate::sandbox::ScriptExecutor;
use crate::task::TaskFn;

/// Build an `ActionExecutorFactory` that dispatches to WASM runtimes via
/// `ScriptExecutor::execute_runtime`.
///
/// Each produced `TaskFn` will:
/// 1. Snapshot the current pipeline context as a JSON `Value`
/// 2. Build a `RuntimeInput { config, context }`
/// 3. Call `execute_runtime` on the `ScriptExecutor`
/// 4. Store the `RuntimeOutput.output` under `context[task_id]`
pub fn build_executor_factory(
    executor: Arc<ScriptExecutor>,
    catalog: Arc<RwLock<RuntimeCatalog>>,
) -> ActionExecutorFactory {
    Arc::new(move |task_id: &str, action: &ActionDefinition| -> TaskFn {
        let executor = executor.clone();
        let catalog = catalog.clone();
        let task_id = task_id.to_string();

        // Derive runtime name and config from the action definition.
        // In the new schema, callers will pass a synthetic Tool action
        // where name = runtime name and params = config (see definition.rs).
        let (runtime_name, config) = match action {
            ActionDefinition::Tool { name, params } => {
                let cfg = serde_json::to_value(params).unwrap_or_default();
                (name.clone(), cfg)
            }
            ActionDefinition::Script {
                source_file,
                language,
            } => (
                "script".to_string(),
                serde_json::json!({"source_file": source_file, "language": language}),
            ),
            ActionDefinition::Llm { prompt, model } => (
                "llm".to_string(),
                serde_json::json!({"prompt": prompt, "model": model}),
            ),
        };

        Arc::new(move |ctx| {
            let executor = executor.clone();
            let catalog = catalog.clone();
            let task_id = task_id.clone();
            let runtime_name = runtime_name.clone();
            let config = config.clone();

            Box::pin(async move {
                // Snapshot the current context as a JSON Value
                let context_snapshot = match ctx.to_json() {
                    Ok(json_str) => serde_json::from_str::<Value>(&json_str)
                        .unwrap_or(Value::Object(Default::default())),
                    Err(_) => Value::Object(Default::default()),
                };

                let input = RuntimeInput {
                    config: config.clone(),
                    context: context_snapshot,
                };

                debug!(
                    task_id = %task_id,
                    runtime = %runtime_name,
                    "Executing runtime for task"
                );

                let make_err = |msg: String| cloacina_workflow::error::TaskError::ExecutionFailed {
                    timestamp: chrono::Utc::now(),
                    task_id: task_id.clone(),
                    message: msg,
                };

                let catalog_guard = catalog.read().await;
                let output = executor
                    .execute_runtime(&runtime_name, &input, &catalog_guard)
                    .await
                    .map_err(|e| make_err(format!("Runtime '{}' failed: {}", runtime_name, e)))?;

                if !output.is_ok() {
                    let err_msg = output.error.unwrap_or_else(|| "unknown error".to_string());
                    return Err(make_err(format!(
                        "Runtime '{}' returned error: {}",
                        runtime_name, err_msg
                    )));
                }

                // Store output under context[task_id]
                let mut result_ctx = ctx;
                let output_val = output.output.unwrap_or(Value::Null);

                if result_ctx.get(&task_id).is_some() {
                    result_ctx
                        .update(&task_id, output_val)
                        .map_err(|e| make_err(format!("Context update failed: {e}")))?;
                } else {
                    result_ctx
                        .insert(&task_id, output_val)
                        .map_err(|e| make_err(format!("Context insert failed: {e}")))?;
                }

                debug!(task_id = %task_id, "Task output stored in context");

                Ok(result_ctx)
            })
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::{CatalogEntry, RuntimeCategory};
    use crate::protocol::RuntimeOutput;
    use cloacina_workflow::context::Context;
    use std::time::Duration;
    use tempfile::TempDir;

    /// Helper: set up executor, compile a simple passthrough wasm, register in catalog.
    async fn setup_with_passthrough() -> (Arc<ScriptExecutor>, Arc<RwLock<RuntimeCatalog>>, TempDir)
    {
        let tmp = TempDir::new().unwrap();
        let executor =
            ScriptExecutor::new(tmp.path().join("cache"), Duration::from_secs(30)).unwrap();

        // Compile a mini passthrough: reads stdin JSON, wraps as RuntimeOutput
        let source = r#"
use std::io::Read;
fn main() {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).unwrap();
    print!("{{\"status\":\"ok\",\"output\":{}}}", input);
}
"#;
        let cr = executor.compile(source).await.unwrap();

        // Set up catalog with the compiled wasm
        let runtimes_dir = tmp.path().join("runtimes");
        let builtin_dir = runtimes_dir.join("builtin");
        std::fs::create_dir_all(&builtin_dir).unwrap();
        std::fs::copy(&cr.wasm_path, builtin_dir.join("passthrough.wasm")).unwrap();

        let mut catalog = RuntimeCatalog::load(&runtimes_dir).unwrap();
        catalog
            .add(
                "passthrough",
                CatalogEntry {
                    description: "Test passthrough".into(),
                    path: "builtin/passthrough.wasm".into(),
                    category: RuntimeCategory::Builtin,
                },
            )
            .unwrap();

        (Arc::new(executor), Arc::new(RwLock::new(catalog)), tmp)
    }

    fn can_compile_wasm() -> bool {
        // Quick check: if wasm32-wasip1 target is not installed, skip
        std::process::Command::new("rustup")
            .args(["target", "list", "--installed"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).contains("wasm32-wasip1"))
            .unwrap_or(false)
    }

    #[tokio::test]
    async fn test_factory_produces_working_task_fn() {
        if !can_compile_wasm() {
            eprintln!("Skipping: wasm32-wasip1 not installed");
            return;
        }

        let (executor, catalog, _tmp) = setup_with_passthrough().await;
        let factory = build_executor_factory(executor, catalog);

        let action = ActionDefinition::Tool {
            name: "passthrough".into(),
            params: [("key".into(), serde_json::json!("value"))].into(),
        };

        let task_fn = factory("step1", &action);

        let ctx = Context::<Value>::new();
        let result = task_fn(ctx).await.unwrap();

        // Output should be stored under "step1"
        let step1_output = result.get("step1").expect("step1 should be in context");
        // The passthrough echoes the full RuntimeInput
        assert_eq!(step1_output["config"]["key"], "value");
    }

    #[tokio::test]
    async fn test_factory_context_propagation() {
        if !can_compile_wasm() {
            eprintln!("Skipping: wasm32-wasip1 not installed");
            return;
        }

        let (executor, catalog, _tmp) = setup_with_passthrough().await;
        let factory = build_executor_factory(executor, catalog);

        let action1 = ActionDefinition::Tool {
            name: "passthrough".into(),
            params: [("stage".into(), serde_json::json!("first"))].into(),
        };
        let action2 = ActionDefinition::Tool {
            name: "passthrough".into(),
            params: [("stage".into(), serde_json::json!("second"))].into(),
        };

        let task_fn1 = factory("step1", &action1);
        let task_fn2 = factory("step2", &action2);

        // Execute step1
        let ctx = Context::<Value>::new();
        let ctx = task_fn1(ctx).await.unwrap();

        // step1 output should exist in context
        assert!(ctx.get("step1").is_some());

        // Execute step2 — should see step1's output in context snapshot
        let ctx = task_fn2(ctx).await.unwrap();

        // Both steps should be in context
        assert!(ctx.get("step1").is_some());
        assert!(ctx.get("step2").is_some());

        // step2's output should contain the context that includes step1
        let step2_output = ctx.get("step2").unwrap();
        assert_eq!(step2_output["context"]["step1"]["config"]["stage"], "first");
    }

    #[tokio::test]
    async fn test_factory_unknown_runtime_error() {
        if !can_compile_wasm() {
            eprintln!("Skipping: wasm32-wasip1 not installed");
            return;
        }

        let (executor, catalog, _tmp) = setup_with_passthrough().await;
        let factory = build_executor_factory(executor, catalog);

        let action = ActionDefinition::Tool {
            name: "nonexistent_runtime".into(),
            params: Default::default(),
        };

        let task_fn = factory("bad_task", &action);
        let ctx = Context::<Value>::new();
        let result = task_fn(ctx).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            cloacina_workflow::error::TaskError::ExecutionFailed {
                task_id, message, ..
            } => {
                assert_eq!(task_id, "bad_task");
                assert!(message.contains("nonexistent_runtime"), "msg: {message}");
            }
            other => panic!("Expected ExecutionFailed, got: {other}"),
        }
    }
}
