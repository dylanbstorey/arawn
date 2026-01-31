//! Runtime catalog management tool for agent-driven WASM runtime CRUD.
//!
//! Lets the agent list, register, inspect, and remove WASM runtimes
//! from the runtime catalog.

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{Value, json};
use tokio::sync::RwLock;
use tracing::debug;

use arawn_pipeline::sandbox::ScriptExecutor;
use arawn_pipeline::{CatalogEntry, RuntimeCatalog, RuntimeCategory};

use crate::error::Result;
use crate::tool::{Tool, ToolContext, ToolResult};

/// Validate a runtime or workflow name for safe use as a filename component.
///
/// Rejects path separators, parent directory references, hidden file prefixes,
/// and control characters.
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

/// Agent-facing tool for runtime catalog management.
///
/// Provides five actions: `list`, `compile`, `register`, `inspect`, `remove`.
pub struct CatalogTool {
    catalog: Arc<RwLock<RuntimeCatalog>>,
    executor: Arc<ScriptExecutor>,
}

impl CatalogTool {
    /// Create a new catalog tool backed by the given catalog and executor.
    pub fn new(catalog: Arc<RwLock<RuntimeCatalog>>, executor: Arc<ScriptExecutor>) -> Self {
        Self { catalog, executor }
    }

    async fn action_list(&self) -> ToolResult {
        let catalog = self.catalog.read().await;
        let entries: Vec<Value> = catalog
            .list()
            .iter()
            .map(|(name, entry)| {
                json!({
                    "name": name,
                    "description": entry.description,
                    "path": entry.path,
                    "category": match entry.category {
                        RuntimeCategory::Builtin => "builtin",
                        RuntimeCategory::Custom => "custom",
                    },
                })
            })
            .collect();

        ToolResult::json(json!({
            "runtimes": entries,
            "count": entries.len(),
        }))
    }

    async fn action_compile(&self, params: &Value) -> ToolResult {
        let name = match params.get("name").and_then(|v| v.as_str()) {
            Some(n) => n,
            None => return ToolResult::error("Missing required parameter 'name'"),
        };

        if let Err(e) = validate_name(name) {
            return ToolResult::error(e);
        }

        // Read source from a .rs file (agent writes the file first with file_write)
        let source_path = match params.get("source_path").and_then(|v| v.as_str()) {
            Some(p) => p,
            None => {
                return ToolResult::error(
                    "Missing required parameter 'source_path'. Write a .rs file first using file_write, then pass the path here.",
                );
            }
        };

        let source = match std::fs::read_to_string(source_path) {
            Ok(s) => s,
            Err(e) => {
                return ToolResult::error(format!(
                    "Failed to read source file '{source_path}': {e}"
                ));
            }
        };

        let description = params
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // Compile Rust source to WASM
        let compile_result = match self.executor.compile(&source).await {
            Ok(cr) => cr,
            Err(e) => return ToolResult::error(format!("Compilation failed: {e}")),
        };

        // Copy .wasm into catalog custom/ directory and register
        let mut catalog = self.catalog.write().await;
        let dest_filename = format!("{name}.wasm");
        let custom_dir = catalog.root().join("custom");
        if let Err(e) = std::fs::create_dir_all(&custom_dir) {
            return ToolResult::error(format!("Failed to create custom dir: {e}"));
        }

        let dest = custom_dir.join(&dest_filename);
        if let Err(e) = std::fs::copy(&compile_result.wasm_path, &dest) {
            return ToolResult::error(format!("Failed to copy WASM file: {e}"));
        }

        let entry = CatalogEntry {
            description,
            path: format!("custom/{dest_filename}"),
            category: RuntimeCategory::Custom,
        };

        if let Err(e) = catalog.add(name, entry) {
            return ToolResult::error(format!("Failed to register runtime: {e}"));
        }

        debug!(name = %name, cached = compile_result.cached, "Runtime compiled and registered");

        ToolResult::json(json!({
            "compiled": name,
            "path": dest.display().to_string(),
            "cached": compile_result.cached,
            "category": "custom",
        }))
    }

    async fn action_register(&self, params: &Value) -> ToolResult {
        let name = match params.get("name").and_then(|v| v.as_str()) {
            Some(n) => n,
            None => return ToolResult::error("Missing required parameter 'name'"),
        };

        if let Err(e) = validate_name(name) {
            return ToolResult::error(e);
        }

        let wasm_path = match params.get("wasm_path").and_then(|v| v.as_str()) {
            Some(p) => p,
            None => return ToolResult::error("Missing required parameter 'wasm_path'"),
        };

        let description = params
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // Verify the source .wasm file exists
        let source = std::path::Path::new(wasm_path);
        if !source.exists() {
            return ToolResult::error(format!("WASM file not found: {wasm_path}"));
        }

        let mut catalog = self.catalog.write().await;

        // Copy into custom/ directory
        let dest_filename = format!("{name}.wasm");
        let custom_dir = catalog.root().join("custom");
        if let Err(e) = std::fs::create_dir_all(&custom_dir) {
            return ToolResult::error(format!("Failed to create custom dir: {e}"));
        }

        let dest = custom_dir.join(&dest_filename);
        if let Err(e) = std::fs::copy(source, &dest) {
            return ToolResult::error(format!("Failed to copy WASM file: {e}"));
        }

        let entry = CatalogEntry {
            description,
            path: format!("custom/{dest_filename}"),
            category: RuntimeCategory::Custom,
        };

        if let Err(e) = catalog.add(name, entry) {
            return ToolResult::error(format!("Failed to register runtime: {e}"));
        }

        debug!(name = %name, dest = %dest.display(), "Runtime registered");

        ToolResult::json(json!({
            "registered": name,
            "path": dest.display().to_string(),
            "category": "custom",
        }))
    }

    async fn action_inspect(&self, params: &Value) -> ToolResult {
        let name = match params.get("name").and_then(|v| v.as_str()) {
            Some(n) => n,
            None => return ToolResult::error("Missing required parameter 'name'"),
        };

        let catalog = self.catalog.read().await;
        match catalog.get(name) {
            Some(entry) => {
                let resolved = catalog.resolve_path(name);
                let exists = resolved.as_ref().is_some_and(|p| p.exists());

                ToolResult::json(json!({
                    "name": name,
                    "description": entry.description,
                    "path": entry.path,
                    "category": match entry.category {
                        RuntimeCategory::Builtin => "builtin",
                        RuntimeCategory::Custom => "custom",
                    },
                    "resolved_path": resolved.map(|p| p.display().to_string()),
                    "wasm_exists": exists,
                }))
            }
            None => ToolResult::error(format!("Runtime '{name}' not found in catalog")),
        }
    }

    async fn action_remove(&self, params: &Value) -> ToolResult {
        let name = match params.get("name").and_then(|v| v.as_str()) {
            Some(n) => n,
            None => return ToolResult::error("Missing required parameter 'name'"),
        };

        let mut catalog = self.catalog.write().await;

        // Check if it exists and refuse to remove builtins
        match catalog.get(name) {
            Some(entry) if entry.category == RuntimeCategory::Builtin => {
                return ToolResult::error(format!(
                    "Cannot remove builtin runtime '{name}'. Only custom runtimes can be removed."
                ));
            }
            None => {
                return ToolResult::error(format!("Runtime '{name}' not found in catalog"));
            }
            _ => {}
        }

        // Delete the .wasm file
        if let Some(wasm_path) = catalog.resolve_path(name) {
            if wasm_path.exists() {
                if let Err(e) = std::fs::remove_file(&wasm_path) {
                    return ToolResult::error(format!("Failed to delete WASM file: {e}"));
                }
            }
        }

        match catalog.remove(name) {
            Ok(Some(_)) => {
                debug!(name = %name, "Runtime removed");
                ToolResult::json(json!({ "removed": name }))
            }
            Ok(None) => ToolResult::error(format!("Runtime '{name}' not found")),
            Err(e) => ToolResult::error(format!("Failed to remove runtime: {e}")),
        }
    }
}

#[async_trait]
impl Tool for CatalogTool {
    fn name(&self) -> &str {
        "catalog"
    }

    fn description(&self) -> &str {
        "Manage WASM runtime catalog: list available runtimes, compile Rust source into a new \
         custom WASM runtime, register pre-built .wasm files, inspect runtime details, or \
         remove custom runtimes. Use the 'action' parameter to select an operation."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["list", "compile", "register", "inspect", "remove"],
                    "description": "The operation to perform"
                },
                "name": {
                    "type": "string",
                    "description": "Runtime name (for 'compile', 'register', 'inspect', 'remove' actions)"
                },
                "source_path": {
                    "type": "string",
                    "description": "Path to a .rs file to compile into a WASM runtime (for 'compile' action). Write the file first with file_write, then pass the path here. The Rust code must read JSON from stdin and print {\"status\":\"ok\",\"output\":{...}} to stdout."
                },
                "wasm_path": {
                    "type": "string",
                    "description": "Path to .wasm file to register (for 'register' action)"
                },
                "description": {
                    "type": "string",
                    "description": "Human-readable description (for 'compile' and 'register' actions)"
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
            "list" => self.action_list().await,
            "compile" => self.action_compile(&params).await,
            "register" => self.action_register(&params).await,
            "inspect" => self.action_inspect(&params).await,
            "remove" => self.action_remove(&params).await,
            _ => ToolResult::error(format!(
                "Unknown action '{action}'. Valid actions: list, compile, register, inspect, remove"
            )),
        };

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool::ToolContext;
    use std::time::Duration;
    use tempfile::TempDir;

    fn make_executor(tmp: &TempDir) -> Arc<ScriptExecutor> {
        Arc::new(ScriptExecutor::new(tmp.path().join("cache"), Duration::from_secs(30)).unwrap())
    }

    async fn setup() -> (CatalogTool, TempDir) {
        let tmp = TempDir::new().unwrap();
        let catalog = RuntimeCatalog::load(&tmp.path().join("runtimes")).unwrap();
        let executor = make_executor(&tmp);
        let tool = CatalogTool::new(Arc::new(RwLock::new(catalog)), executor);
        (tool, tmp)
    }

    async fn setup_with_entries() -> (CatalogTool, TempDir) {
        let tmp = TempDir::new().unwrap();
        let runtimes_dir = tmp.path().join("runtimes");
        let mut catalog = RuntimeCatalog::load(&runtimes_dir).unwrap();

        // Add a builtin entry
        catalog
            .add(
                "passthrough",
                CatalogEntry {
                    description: "Echoes context unchanged".into(),
                    path: "builtin/passthrough.wasm".into(),
                    category: RuntimeCategory::Builtin,
                },
            )
            .unwrap();

        // Create a dummy custom wasm file and register it
        let custom_dir = runtimes_dir.join("custom");
        std::fs::create_dir_all(&custom_dir).unwrap();
        std::fs::write(custom_dir.join("my_transform.wasm"), b"fake wasm").unwrap();

        catalog
            .add(
                "my_transform",
                CatalogEntry {
                    description: "Custom transform".into(),
                    path: "custom/my_transform.wasm".into(),
                    category: RuntimeCategory::Custom,
                },
            )
            .unwrap();

        let executor = make_executor(&tmp);
        let tool = CatalogTool::new(Arc::new(RwLock::new(catalog)), executor);
        (tool, tmp)
    }

    #[tokio::test]
    async fn test_list_empty() {
        let (tool, _tmp) = setup().await;
        let ctx = ToolContext::default();
        let result = tool.execute(json!({"action": "list"}), &ctx).await.unwrap();
        assert!(result.is_success());
        if let ToolResult::Json { content } = &result {
            assert_eq!(content["count"], 0);
            assert!(content["runtimes"].as_array().unwrap().is_empty());
        }
    }

    #[tokio::test]
    async fn test_list_with_entries() {
        let (tool, _tmp) = setup_with_entries().await;
        let ctx = ToolContext::default();
        let result = tool.execute(json!({"action": "list"}), &ctx).await.unwrap();
        assert!(result.is_success());
        if let ToolResult::Json { content } = &result {
            assert_eq!(content["count"], 2);
            let runtimes = content["runtimes"].as_array().unwrap();
            let names: Vec<&str> = runtimes
                .iter()
                .map(|r| r["name"].as_str().unwrap())
                .collect();
            assert!(names.contains(&"passthrough"));
            assert!(names.contains(&"my_transform"));
        }
    }

    #[tokio::test]
    async fn test_inspect_existing() {
        let (tool, _tmp) = setup_with_entries().await;
        let ctx = ToolContext::default();
        let result = tool
            .execute(json!({"action": "inspect", "name": "passthrough"}), &ctx)
            .await
            .unwrap();
        assert!(result.is_success());
        if let ToolResult::Json { content } = &result {
            assert_eq!(content["name"], "passthrough");
            assert_eq!(content["category"], "builtin");
        }
    }

    #[tokio::test]
    async fn test_inspect_nonexistent() {
        let (tool, _tmp) = setup().await;
        let ctx = ToolContext::default();
        let result = tool
            .execute(json!({"action": "inspect", "name": "nope"}), &ctx)
            .await
            .unwrap();
        assert!(result.is_error());
        assert!(result.to_llm_content().contains("not found"));
    }

    #[tokio::test]
    async fn test_register_new_runtime() {
        let (tool, tmp) = setup().await;
        let ctx = ToolContext::default();

        // Create a dummy wasm file to register
        let wasm_file = tmp.path().join("my_runtime.wasm");
        std::fs::write(&wasm_file, b"fake wasm binary").unwrap();

        let result = tool
            .execute(
                json!({
                    "action": "register",
                    "name": "my_runtime",
                    "wasm_path": wasm_file.display().to_string(),
                    "description": "My custom runtime",
                }),
                &ctx,
            )
            .await
            .unwrap();

        assert!(result.is_success(), "got: {:?}", result);
        if let ToolResult::Json { content } = &result {
            assert_eq!(content["registered"], "my_runtime");
            assert_eq!(content["category"], "custom");
        }

        // Verify it shows up in list
        let list = tool.execute(json!({"action": "list"}), &ctx).await.unwrap();
        if let ToolResult::Json { content } = &list {
            assert_eq!(content["count"], 1);
        }
    }

    #[tokio::test]
    async fn test_register_missing_wasm() {
        let (tool, _tmp) = setup().await;
        let ctx = ToolContext::default();
        let result = tool
            .execute(
                json!({
                    "action": "register",
                    "name": "bad",
                    "wasm_path": "/nonexistent/path.wasm",
                }),
                &ctx,
            )
            .await
            .unwrap();
        assert!(result.is_error());
        assert!(result.to_llm_content().contains("not found"));
    }

    #[tokio::test]
    async fn test_remove_custom_runtime() {
        let (tool, _tmp) = setup_with_entries().await;
        let ctx = ToolContext::default();
        let result = tool
            .execute(json!({"action": "remove", "name": "my_transform"}), &ctx)
            .await
            .unwrap();
        assert!(result.is_success());
        if let ToolResult::Json { content } = &result {
            assert_eq!(content["removed"], "my_transform");
        }

        // Verify it's gone
        let inspect = tool
            .execute(json!({"action": "inspect", "name": "my_transform"}), &ctx)
            .await
            .unwrap();
        assert!(inspect.is_error());
    }

    #[tokio::test]
    async fn test_remove_builtin_refused() {
        let (tool, _tmp) = setup_with_entries().await;
        let ctx = ToolContext::default();
        let result = tool
            .execute(json!({"action": "remove", "name": "passthrough"}), &ctx)
            .await
            .unwrap();
        assert!(result.is_error());
        assert!(result.to_llm_content().contains("Cannot remove builtin"));
    }

    #[tokio::test]
    async fn test_remove_nonexistent() {
        let (tool, _tmp) = setup().await;
        let ctx = ToolContext::default();
        let result = tool
            .execute(json!({"action": "remove", "name": "nope"}), &ctx)
            .await
            .unwrap();
        assert!(result.is_error());
        assert!(result.to_llm_content().contains("not found"));
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

    #[tokio::test]
    async fn test_parameters_schema() {
        let (tool, _tmp) = setup().await;
        let schema = tool.parameters();
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["action"].is_object());
        let required = schema["required"].as_array().unwrap();
        assert!(required.contains(&json!("action")));
    }

    // --- Sad path / boundary tests ---

    #[tokio::test]
    async fn test_register_name_with_path_separator() {
        let (tool, tmp) = setup().await;
        let ctx = ToolContext::default();
        let wasm_file = tmp.path().join("test.wasm");
        std::fs::write(&wasm_file, b"fake").unwrap();

        let result = tool
            .execute(
                json!({
                    "action": "register",
                    "name": "../etc/evil",
                    "wasm_path": wasm_file.display().to_string(),
                }),
                &ctx,
            )
            .await
            .unwrap();
        assert!(result.is_error());
        assert!(result.to_llm_content().contains("path separator"));
    }

    #[tokio::test]
    async fn test_register_name_with_dotdot() {
        let (tool, tmp) = setup().await;
        let ctx = ToolContext::default();
        let wasm_file = tmp.path().join("test.wasm");
        std::fs::write(&wasm_file, b"fake").unwrap();

        let result = tool
            .execute(
                json!({
                    "action": "register",
                    "name": "foo..bar",
                    "wasm_path": wasm_file.display().to_string(),
                }),
                &ctx,
            )
            .await
            .unwrap();
        assert!(result.is_error());
        assert!(result.to_llm_content().contains(".."));
    }

    #[tokio::test]
    async fn test_register_name_starting_with_dot() {
        let (tool, tmp) = setup().await;
        let ctx = ToolContext::default();
        let wasm_file = tmp.path().join("test.wasm");
        std::fs::write(&wasm_file, b"fake").unwrap();

        let result = tool
            .execute(
                json!({
                    "action": "register",
                    "name": ".hidden",
                    "wasm_path": wasm_file.display().to_string(),
                }),
                &ctx,
            )
            .await
            .unwrap();
        assert!(result.is_error());
        assert!(result.to_llm_content().contains("must not start with '.'"));
    }

    #[tokio::test]
    async fn test_register_empty_name() {
        let (tool, tmp) = setup().await;
        let ctx = ToolContext::default();
        let wasm_file = tmp.path().join("test.wasm");
        std::fs::write(&wasm_file, b"fake").unwrap();

        let result = tool
            .execute(
                json!({
                    "action": "register",
                    "name": "",
                    "wasm_path": wasm_file.display().to_string(),
                }),
                &ctx,
            )
            .await
            .unwrap();
        assert!(result.is_error());
        assert!(result.to_llm_content().contains("empty"));
    }

    #[tokio::test]
    async fn test_register_missing_name() {
        let (tool, _tmp) = setup().await;
        let ctx = ToolContext::default();
        let result = tool
            .execute(
                json!({
                    "action": "register",
                    "wasm_path": "/some/path.wasm",
                }),
                &ctx,
            )
            .await
            .unwrap();
        assert!(result.is_error());
        assert!(result.to_llm_content().contains("name"));
    }

    #[tokio::test]
    async fn test_inspect_missing_name() {
        let (tool, _tmp) = setup().await;
        let ctx = ToolContext::default();
        let result = tool
            .execute(json!({"action": "inspect"}), &ctx)
            .await
            .unwrap();
        assert!(result.is_error());
        assert!(result.to_llm_content().contains("name"));
    }

    #[tokio::test]
    async fn test_remove_missing_name() {
        let (tool, _tmp) = setup().await;
        let ctx = ToolContext::default();
        let result = tool
            .execute(json!({"action": "remove"}), &ctx)
            .await
            .unwrap();
        assert!(result.is_error());
        assert!(result.to_llm_content().contains("name"));
    }

    #[tokio::test]
    async fn test_compile_missing_name() {
        let (tool, _tmp) = setup().await;
        let ctx = ToolContext::default();
        let result = tool
            .execute(
                json!({"action": "compile", "source_path": "/some/file.rs"}),
                &ctx,
            )
            .await
            .unwrap();
        assert!(result.is_error());
        assert!(result.to_llm_content().contains("name"));
    }

    #[tokio::test]
    async fn test_compile_missing_source_path() {
        let (tool, _tmp) = setup().await;
        let ctx = ToolContext::default();
        let result = tool
            .execute(json!({"action": "compile", "name": "test_rt"}), &ctx)
            .await
            .unwrap();
        assert!(result.is_error());
        assert!(result.to_llm_content().contains("source_path"));
    }

    #[tokio::test]
    async fn test_compile_nonexistent_source() {
        let (tool, _tmp) = setup().await;
        let ctx = ToolContext::default();
        let result = tool
            .execute(
                json!({
                    "action": "compile",
                    "name": "test_rt",
                    "source_path": "/nonexistent/file.rs"
                }),
                &ctx,
            )
            .await
            .unwrap();
        assert!(result.is_error());
        assert!(
            result
                .to_llm_content()
                .contains("Failed to read source file")
        );
    }

    #[tokio::test]
    async fn test_action_is_number() {
        let (tool, _tmp) = setup().await;
        let ctx = ToolContext::default();
        let result = tool.execute(json!({"action": 42}), &ctx).await.unwrap();
        assert!(result.is_error());
        assert!(result.to_llm_content().contains("action"));
    }
}
