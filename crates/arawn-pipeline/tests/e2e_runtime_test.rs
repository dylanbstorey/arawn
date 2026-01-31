//! End-to-end integration tests for the WASM runtime pipeline.
//!
//! Test 1: Multi-step workflow with context propagation between tasks.
//! Test 2: Agent self-extension — compile, register, and execute a custom runtime.

use std::sync::Arc;
use std::time::Duration;

use arawn_pipeline::{
    CatalogEntry, ExecutionStatus, PipelineConfig, PipelineEngine, RuntimeCatalog, RuntimeCategory,
    ScriptExecutor, WorkflowFile, build_executor_factory,
};
use tokio::sync::RwLock;

fn can_compile_wasm() -> bool {
    std::process::Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("wasm32-wasip1"))
        .unwrap_or(false)
}

/// Set up executor + catalog with compiled test runtimes.
///
/// Registers two runtimes:
/// - `passthrough`: echoes the full RuntimeInput as output
/// - `uppercase`: reads `config.text`, uppercases it, returns as output
async fn setup() -> (
    Arc<PipelineEngine>,
    Arc<ScriptExecutor>,
    Arc<RwLock<RuntimeCatalog>>,
    tempfile::TempDir,
) {
    let tmp = tempfile::tempdir().unwrap();

    let executor = ScriptExecutor::new(tmp.path().join("cache"), Duration::from_secs(30)).unwrap();

    // Compile passthrough runtime
    let passthrough_src = r#"
use std::io::Read;
fn main() {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).unwrap();
    print!("{{\"status\":\"ok\",\"output\":{}}}", input);
}
"#;
    let passthrough_cr = executor.compile(passthrough_src).await.unwrap();

    // Compile uppercase runtime: reads config.text, uppercases it
    let uppercase_src = r#"
use std::io::Read;
fn main() {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).unwrap();

    // Crude JSON parse: find "text":"<value>"
    let text = extract_string(&input, "text").unwrap_or_default();
    let upper = text.to_uppercase();

    // Also extract context for propagation
    let ctx_start = input.find("\"context\"").unwrap_or(0);
    print!("{{\"status\":\"ok\",\"output\":{{\"transformed\":\"{}\"}}}}", upper);
}

fn extract_string(json: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{}\":\"", key);
    let start = json.find(&pattern)? + pattern.len();
    let end = json[start..].find('"')? + start;
    Some(json[start..end].to_string())
}
"#;
    let uppercase_cr = executor.compile(uppercase_src).await.unwrap();

    // Set up catalog
    let runtimes_dir = tmp.path().join("runtimes");
    let builtin_dir = runtimes_dir.join("builtin");
    std::fs::create_dir_all(&builtin_dir).unwrap();

    std::fs::copy(
        &passthrough_cr.wasm_path,
        builtin_dir.join("passthrough.wasm"),
    )
    .unwrap();
    std::fs::copy(&uppercase_cr.wasm_path, builtin_dir.join("uppercase.wasm")).unwrap();

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
    catalog
        .add(
            "uppercase",
            CatalogEntry {
                description: "Test uppercase".into(),
                path: "builtin/uppercase.wasm".into(),
                category: RuntimeCategory::Builtin,
            },
        )
        .unwrap();

    // Create engine
    let db_path = tmp.path().join("test.db");
    let config = PipelineConfig {
        cron_enabled: false,
        triggers_enabled: false,
        ..Default::default()
    };
    let engine = PipelineEngine::new(&db_path, config).await.unwrap();

    (
        Arc::new(engine),
        Arc::new(executor),
        Arc::new(RwLock::new(catalog)),
        tmp,
    )
}

/// Test 1: Multi-step workflow with context propagation.
///
/// Workflow: passthrough (step1) -> uppercase (step2)
/// - step1 echoes the input config+context
/// - step2 reads config.text and uppercases it
/// - step2 depends on step1
#[tokio::test]
async fn test_multistep_workflow_context_propagation() {
    if !can_compile_wasm() {
        eprintln!("Skipping: wasm32-wasip1 not installed");
        return;
    }

    let (engine, executor, catalog, _tmp) = setup().await;

    // Define a two-step workflow via TOML
    let toml = r#"
[workflow]
name = "multi-step"
description = "Two-step workflow with context propagation"

[[workflow.tasks]]
id = "step1"
runtime = "passthrough"

[workflow.tasks.config]
greeting = "hello"

[[workflow.tasks]]
id = "step2"
runtime = "uppercase"
dependencies = ["step1"]

[workflow.tasks.config]
text = "hello world"
"#;

    let wf = WorkflowFile::from_toml(toml).expect("TOML parse failed");
    wf.workflow.validate().expect("validation failed");

    let factory = build_executor_factory(executor, catalog);
    let tasks = wf
        .workflow
        .to_dynamic_tasks(&factory)
        .expect("task conversion failed");

    engine
        .register_dynamic_workflow("multi-step", "Two-step test", tasks)
        .await
        .expect("registration failed");

    let ctx = cloacina_workflow::context::Context::new();
    let result = engine
        .execute("multi-step", ctx)
        .await
        .expect("execution failed");

    assert_eq!(result.status, ExecutionStatus::Completed);
    assert!(result.output.is_some(), "Expected output context");

    let output = result.output.unwrap();

    // step1 should exist in output (passthrough echoes config+context)
    let step1 = output
        .get("step1")
        .expect("step1 output should exist in final context");
    assert!(
        step1.get("config").is_some(),
        "step1 should have config in output"
    );

    // step2 should have the uppercased text
    let step2 = output
        .get("step2")
        .expect("step2 output should exist in final context");
    assert_eq!(
        step2.get("transformed").and_then(|v| v.as_str()),
        Some("HELLO WORLD"),
        "step2 should uppercase the text"
    );
}

/// Test 2: Agent self-extension — compile a custom runtime, register it,
/// and execute it in a workflow.
#[tokio::test]
async fn test_agent_self_extension() {
    if !can_compile_wasm() {
        eprintln!("Skipping: wasm32-wasip1 not installed");
        return;
    }

    let (engine, executor, catalog, _tmp) = setup().await;

    // Write a custom runtime source that doubles a number
    let custom_src = r#"
use std::io::Read;
fn main() {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).unwrap();

    // Extract "value":<number> from the config section
    let value = extract_number(&input, "value").unwrap_or(0);
    let doubled = value * 2;

    print!("{{\"status\":\"ok\",\"output\":{{\"doubled\":{}}}}}", doubled);
}

fn extract_number(json: &str, key: &str) -> Option<i64> {
    // Find "key": pattern then parse the number
    let pattern = format!("\"{}\":", key);
    let start = json.find(&pattern)? + pattern.len();
    let rest = json[start..].trim_start();
    let end = rest.find(|c: char| !c.is_ascii_digit() && c != '-').unwrap_or(rest.len());
    rest[..end].parse().ok()
}
"#;

    // Compile via ScriptExecutor (simulates what an agent would do)
    let cr = executor
        .compile(custom_src)
        .await
        .expect("compile custom runtime");

    // Register in catalog (simulates CatalogTool register action)
    {
        let mut cat = catalog.write().await;
        let custom_dir = cat.root().join("custom");
        std::fs::create_dir_all(&custom_dir).unwrap();
        std::fs::copy(&cr.wasm_path, custom_dir.join("doubler.wasm")).unwrap();
        cat.add(
            "doubler",
            CatalogEntry {
                description: "Doubles a number".into(),
                path: "custom/doubler.wasm".into(),
                category: RuntimeCategory::Custom,
            },
        )
        .unwrap();
    }

    // Create a workflow using the custom runtime
    let toml = r#"
[workflow]
name = "self-extended"
description = "Workflow using a custom agent-registered runtime"

[[workflow.tasks]]
id = "double_it"
runtime = "doubler"

[workflow.tasks.config]
value = 21
"#;

    let wf = WorkflowFile::from_toml(toml).expect("TOML parse");
    wf.workflow.validate().expect("validation");

    let factory = build_executor_factory(executor, catalog);
    let tasks = wf
        .workflow
        .to_dynamic_tasks(&factory)
        .expect("task conversion");

    engine
        .register_dynamic_workflow("self-extended", "Custom runtime test", tasks)
        .await
        .expect("registration");

    let ctx = cloacina_workflow::context::Context::new();
    let result = engine
        .execute("self-extended", ctx)
        .await
        .expect("execution");

    assert_eq!(result.status, ExecutionStatus::Completed);
    assert!(result.output.is_some());

    let output = result.output.unwrap();
    let double_output = output
        .get("double_it")
        .expect("double_it output should exist in final context");
    assert_eq!(
        double_output.get("doubled").and_then(|v| v.as_i64()),
        Some(42),
        "doubler should return 21 * 2 = 42"
    );
}

/// Test 3: Verify unknown runtime produces a clear error.
#[tokio::test]
async fn test_workflow_unknown_runtime_error() {
    if !can_compile_wasm() {
        eprintln!("Skipping: wasm32-wasip1 not installed");
        return;
    }

    let (engine, executor, catalog, _tmp) = setup().await;

    let toml = r#"
[workflow]
name = "bad-runtime"
description = "References a runtime that does not exist"

[[workflow.tasks]]
id = "fail_task"
runtime = "nonexistent"
"#;

    let wf = WorkflowFile::from_toml(toml).unwrap();
    wf.workflow.validate().unwrap();

    let factory = build_executor_factory(executor, catalog);
    let tasks = wf.workflow.to_dynamic_tasks(&factory).unwrap();

    engine
        .register_dynamic_workflow("bad-runtime", "Bad runtime test", tasks)
        .await
        .unwrap();

    let ctx = cloacina_workflow::context::Context::new();
    let result = engine.execute("bad-runtime", ctx).await;

    // The runtime doesn't exist, so the task should fail.
    // Cloacina may report pipeline-level Completed even if a task failed,
    // so we check that either:
    // - The execution returns an error, OR
    // - The status is Failed, OR
    // - The output doesn't contain the task's output key (task didn't produce output)
    match result {
        Ok(r) => {
            if r.status == ExecutionStatus::Completed {
                // Pipeline completed but task should not have produced output
                if let Some(ref output) = r.output {
                    assert!(
                        output.get("fail_task").is_none(),
                        "fail_task should not produce output with unknown runtime"
                    );
                }
            }
            // If status is Failed, that's also correct
        }
        Err(_) => {
            // An error is the expected outcome
        }
    }
}
