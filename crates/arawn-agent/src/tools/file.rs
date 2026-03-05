//! File operation tools.
//!
//! Provides tools for reading and writing files.

use async_trait::async_trait;
use serde_json::{Value, json};
use std::path::{Component, Path, PathBuf};
use tokio::fs;

use crate::error::Result;
use crate::tool::{FileReadParams, FileWriteParams, Tool, ToolContext, ToolResult};

/// Reject paths that contain `..` (parent directory) traversal components.
///
/// This is a defense-in-depth check. In the normal flow, the FsGate
/// canonicalizes paths before they reach the tool, so legitimate paths
/// never contain `..` sequences. Any remaining traversal indicates a
/// bypass attempt.
fn reject_traversal(path: &Path) -> std::result::Result<(), crate::error::AgentError> {
    for component in path.components() {
        if matches!(component, Component::ParentDir) {
            return Err(crate::error::AgentError::Tool(format!(
                "Path traversal not allowed: {}",
                path.display()
            )));
        }
    }
    Ok(())
}

/// Resolve `..` and `.` components lexically (without filesystem access).
///
/// Used as a fallback when the filesystem path doesn't exist yet and
/// `canonicalize()` can't be called.
fn normalize_path(path: &Path) -> PathBuf {
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                // Pop the last normal component (if any)
                if let Some(Component::Normal(_)) = components.last() {
                    components.pop();
                } else {
                    components.push(component);
                }
            }
            Component::CurDir => {} // skip `.`
            _ => components.push(component),
        }
    }
    components.iter().collect()
}

// ─────────────────────────────────────────────────────────────────────────────
// File Read Tool
// ─────────────────────────────────────────────────────────────────────────────

/// Tool for reading file contents.
#[derive(Debug, Clone, Default)]
pub struct FileReadTool {
    /// Optional base directory to restrict file access.
    base_dir: Option<String>,
}

impl FileReadTool {
    /// Create a new file read tool.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a file read tool restricted to a base directory.
    pub fn with_base_dir(base_dir: impl Into<String>) -> Self {
        Self {
            base_dir: Some(base_dir.into()),
        }
    }

    /// Validate and resolve the file path.
    fn resolve_path(&self, path: &str) -> Result<std::path::PathBuf> {
        let path = Path::new(path);

        // Defense-in-depth: reject traversal sequences
        reject_traversal(path)?;

        // If we have a base directory, ensure the path is within it
        if let Some(ref base) = self.base_dir {
            let base_path = Path::new(base).canonicalize().map_err(|e| {
                crate::error::AgentError::Tool(format!("Invalid base directory: {}", e))
            })?;

            let full_path = if path.is_absolute() {
                path.to_path_buf()
            } else {
                base_path.join(path)
            };

            let canonical = full_path.canonicalize().map_err(|e| {
                crate::error::AgentError::Tool(format!("Cannot resolve path: {}", e))
            })?;

            if !canonical.starts_with(&base_path) {
                return Err(crate::error::AgentError::Tool(
                    "Path is outside allowed directory".to_string(),
                ));
            }

            Ok(canonical)
        } else {
            Ok(path.to_path_buf())
        }
    }
}

#[async_trait]
impl Tool for FileReadTool {
    fn name(&self) -> &str {
        "file_read"
    }

    fn description(&self) -> &str {
        "Read the contents of a file. Returns the file content as text."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The path to the file to read"
                }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        // Check cancellation
        if ctx.is_cancelled() {
            return Ok(ToolResult::error("Operation cancelled"));
        }

        // Parse and validate parameters using typed struct
        let file_params = match FileReadParams::try_from(params) {
            Ok(p) => p,
            Err(e) => return Ok(ToolResult::error(e.to_string())),
        };

        // Resolve and validate path
        let resolved_path = self.resolve_path(&file_params.path)?;

        // Check if file exists
        if !resolved_path.exists() {
            return Ok(ToolResult::error(format!(
                "File not found: {}",
                resolved_path.display()
            )));
        }

        // Check if it's a file (not a directory)
        if !resolved_path.is_file() {
            return Ok(ToolResult::error(format!(
                "Path is not a file: {}",
                resolved_path.display()
            )));
        }

        // Read the file
        match fs::read_to_string(&resolved_path).await {
            Ok(content) => Ok(ToolResult::text(content)),
            Err(e) => Ok(ToolResult::error(format!("Failed to read file: {}", e))),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// File Write Tool
// ─────────────────────────────────────────────────────────────────────────────

/// Tool for writing file contents.
#[derive(Debug, Clone, Default)]
pub struct FileWriteTool {
    /// Optional base directory to restrict file access.
    base_dir: Option<String>,
    /// Whether to allow creating new files.
    allow_create: bool,
    /// Whether to allow overwriting existing files.
    allow_overwrite: bool,
}

impl FileWriteTool {
    /// Create a new file write tool with default settings.
    pub fn new() -> Self {
        Self {
            base_dir: None,
            allow_create: true,
            allow_overwrite: true,
        }
    }

    /// Create a file write tool restricted to a base directory.
    pub fn with_base_dir(mut self, base_dir: impl Into<String>) -> Self {
        self.base_dir = Some(base_dir.into());
        self
    }

    /// Set whether creating new files is allowed.
    pub fn allow_create(mut self, allow: bool) -> Self {
        self.allow_create = allow;
        self
    }

    /// Set whether overwriting existing files is allowed.
    pub fn allow_overwrite(mut self, allow: bool) -> Self {
        self.allow_overwrite = allow;
        self
    }

    /// Validate and resolve the file path for writing.
    fn resolve_path(&self, path: &str) -> Result<std::path::PathBuf> {
        let path = Path::new(path);

        // Defense-in-depth: reject traversal sequences
        reject_traversal(path)?;

        if let Some(ref base) = self.base_dir {
            let base_path = Path::new(base).canonicalize().map_err(|e| {
                crate::error::AgentError::Tool(format!("Invalid base directory: {}", e))
            })?;

            let full_path = if path.is_absolute() {
                path.to_path_buf()
            } else {
                base_path.join(path)
            };

            // For write, we can't canonicalize if the file doesn't exist yet
            // So we canonicalize the parent directory instead
            if let Some(parent) = full_path.parent()
                && parent.exists()
            {
                let canonical_parent = parent.canonicalize().map_err(|e| {
                    crate::error::AgentError::Tool(format!("Cannot resolve parent path: {}", e))
                })?;

                if !canonical_parent.starts_with(&base_path) {
                    return Err(crate::error::AgentError::Tool(
                        "Path is outside allowed directory".to_string(),
                    ));
                }

                let file_name = full_path.file_name().ok_or_else(|| {
                    crate::error::AgentError::Tool("Invalid file path".to_string())
                })?;

                return Ok(canonical_parent.join(file_name));
            }

            // If parent doesn't exist, normalize the path lexically to
            // resolve any remaining traversal sequences before checking
            // the prefix. This prevents bypass via paths like
            // "/base/nonexistent/../../etc/passwd".
            let normalized = normalize_path(&full_path);
            if normalized.starts_with(&base_path) {
                Ok(normalized)
            } else {
                Err(crate::error::AgentError::Tool(
                    "Path is outside allowed directory".to_string(),
                ))
            }
        } else {
            Ok(path.to_path_buf())
        }
    }
}

#[async_trait]
impl Tool for FileWriteTool {
    fn name(&self) -> &str {
        "file_write"
    }

    fn description(&self) -> &str {
        "Write content to a file. Can create new files or overwrite existing ones."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The path to the file to write"
                },
                "content": {
                    "type": "string",
                    "description": "The content to write to the file"
                },
                "append": {
                    "type": "boolean",
                    "description": "If true, append to the file instead of overwriting. Defaults to false.",
                    "default": false
                }
            },
            "required": ["path", "content"]
        })
    }

    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        // Check cancellation
        if ctx.is_cancelled() {
            return Ok(ToolResult::error("Operation cancelled"));
        }

        // Parse and validate parameters using typed struct
        let file_params = match FileWriteParams::try_from(params) {
            Ok(p) => p,
            Err(e) => return Ok(ToolResult::error(e.to_string())),
        };

        let append = file_params.append;
        let content = &file_params.content;

        // Resolve and validate path
        let resolved_path = self.resolve_path(&file_params.path)?;

        // Check permissions
        let file_exists = resolved_path.exists();

        if !file_exists && !self.allow_create {
            return Ok(ToolResult::error("Creating new files is not allowed"));
        }

        if file_exists && !append && !self.allow_overwrite {
            return Ok(ToolResult::error(
                "Overwriting existing files is not allowed",
            ));
        }

        // Create parent directories if needed
        if let Some(parent) = resolved_path.parent()
            && !parent.exists()
            && let Err(e) = fs::create_dir_all(parent).await
        {
            return Ok(ToolResult::error(format!(
                "Failed to create directories: {}",
                e
            )));
        }

        // Write the file
        let result = if append && file_exists {
            // Read existing content and append
            match fs::read_to_string(&resolved_path).await {
                Ok(existing) => {
                    let new_content = format!("{}{}", existing, content);
                    fs::write(&resolved_path, new_content).await
                }
                Err(e) => Err(e),
            }
        } else {
            fs::write(&resolved_path, content).await
        };

        match result {
            Ok(()) => {
                let action = if append { "appended to" } else { "written to" };
                Ok(ToolResult::text(format!(
                    "Successfully {} {}",
                    action,
                    resolved_path.display()
                )))
            }
            Err(e) => Ok(ToolResult::error(format!("Failed to write file: {}", e))),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_file_read_tool_metadata() {
        let tool = FileReadTool::new();
        assert_eq!(tool.name(), "file_read");
        assert!(!tool.description().is_empty());

        let params = tool.parameters();
        assert!(params.get("properties").is_some());
        assert!(params["properties"].get("path").is_some());
    }

    #[test]
    fn test_file_write_tool_metadata() {
        let tool = FileWriteTool::new();
        assert_eq!(tool.name(), "file_write");
        assert!(!tool.description().is_empty());

        let params = tool.parameters();
        assert!(params.get("properties").is_some());
        assert!(params["properties"].get("path").is_some());
        assert!(params["properties"].get("content").is_some());
    }

    #[tokio::test]
    async fn test_file_read_success() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "Hello, World!").unwrap();

        let tool = FileReadTool::new();
        let ctx = ToolContext::default();

        let result = tool
            .execute(json!({"path": file_path.to_str().unwrap()}), &ctx)
            .await
            .unwrap();

        assert!(result.is_success());
        assert_eq!(result.to_llm_content(), "Hello, World!");
    }

    #[tokio::test]
    async fn test_file_read_not_found() {
        let tool = FileReadTool::new();
        let ctx = ToolContext::default();

        let result = tool
            .execute(json!({"path": "/nonexistent/file.txt"}), &ctx)
            .await
            .unwrap();

        assert!(result.is_error());
        assert!(result.to_llm_content().contains("not found"));
    }

    #[tokio::test]
    async fn test_file_read_with_base_dir() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "Content").unwrap();

        let tool = FileReadTool::with_base_dir(temp_dir.path().to_str().unwrap());
        let ctx = ToolContext::default();

        // Should succeed for file in base dir
        let result = tool
            .execute(json!({"path": "test.txt"}), &ctx)
            .await
            .unwrap();
        assert!(result.is_success());
    }

    #[tokio::test]
    async fn test_file_write_success() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("output.txt");

        let tool = FileWriteTool::new();
        let ctx = ToolContext::default();

        let result = tool
            .execute(
                json!({
                    "path": file_path.to_str().unwrap(),
                    "content": "Test content"
                }),
                &ctx,
            )
            .await
            .unwrap();

        assert!(result.is_success());
        assert!(file_path.exists());
        assert_eq!(std::fs::read_to_string(&file_path).unwrap(), "Test content");
    }

    #[tokio::test]
    async fn test_file_write_append() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("append.txt");
        std::fs::write(&file_path, "First ").unwrap();

        let tool = FileWriteTool::new();
        let ctx = ToolContext::default();

        let result = tool
            .execute(
                json!({
                    "path": file_path.to_str().unwrap(),
                    "content": "Second",
                    "append": true
                }),
                &ctx,
            )
            .await
            .unwrap();

        assert!(result.is_success());
        assert_eq!(std::fs::read_to_string(&file_path).unwrap(), "First Second");
    }

    #[tokio::test]
    async fn test_file_write_no_create() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("new_file.txt");

        let tool = FileWriteTool::new().allow_create(false);
        let ctx = ToolContext::default();

        let result = tool
            .execute(
                json!({
                    "path": file_path.to_str().unwrap(),
                    "content": "Content"
                }),
                &ctx,
            )
            .await
            .unwrap();

        assert!(result.is_error());
        assert!(result.to_llm_content().contains("not allowed"));
    }

    #[tokio::test]
    async fn test_file_write_no_overwrite() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("existing.txt");
        std::fs::write(&file_path, "Original").unwrap();

        let tool = FileWriteTool::new().allow_overwrite(false);
        let ctx = ToolContext::default();

        let result = tool
            .execute(
                json!({
                    "path": file_path.to_str().unwrap(),
                    "content": "New content"
                }),
                &ctx,
            )
            .await
            .unwrap();

        assert!(result.is_error());
        assert!(result.to_llm_content().contains("not allowed"));
    }

    // ─────────────────────────────────────────────────────────────────
    // Path traversal tests
    // ─────────────────────────────────────────────────────────────────

    #[test]
    fn test_reject_traversal_blocks_dotdot() {
        assert!(reject_traversal(Path::new("/tmp/../etc/passwd")).is_err());
        assert!(reject_traversal(Path::new("../secret.txt")).is_err());
        assert!(reject_traversal(Path::new("foo/../../bar")).is_err());
    }

    #[test]
    fn test_reject_traversal_allows_normal_paths() {
        assert!(reject_traversal(Path::new("/tmp/file.txt")).is_ok());
        assert!(reject_traversal(Path::new("relative/path.rs")).is_ok());
        assert!(reject_traversal(Path::new("/a/b/c/d.txt")).is_ok());
    }

    #[test]
    fn test_normalize_path_resolves_dotdot() {
        assert_eq!(
            normalize_path(Path::new("/a/b/../c")),
            PathBuf::from("/a/c")
        );
        assert_eq!(
            normalize_path(Path::new("/a/b/c/../../d")),
            PathBuf::from("/a/d")
        );
        assert_eq!(
            normalize_path(Path::new("/a/./b/./c")),
            PathBuf::from("/a/b/c")
        );
    }

    #[tokio::test]
    async fn test_file_write_traversal_rejected_no_base() {
        let tool = FileWriteTool::new();
        let ctx = ToolContext::default();

        let result = tool
            .execute(
                json!({
                    "path": "/tmp/../etc/passwd",
                    "content": "malicious"
                }),
                &ctx,
            )
            .await;

        // Should fail with traversal error
        assert!(
            result.is_err() || {
                let r = result.unwrap();
                r.is_error() && r.to_llm_content().contains("traversal")
            }
        );
    }

    #[tokio::test]
    async fn test_file_write_traversal_rejected_with_base() {
        let temp_dir = TempDir::new().unwrap();
        let tool = FileWriteTool::new().with_base_dir(temp_dir.path().to_str().unwrap());
        let ctx = ToolContext::default();

        let result = tool
            .execute(
                json!({
                    "path": "../../etc/passwd",
                    "content": "malicious"
                }),
                &ctx,
            )
            .await;

        // Should fail — either traversal rejection or path-outside-base
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_file_read_traversal_rejected() {
        let tool = FileReadTool::new();
        let ctx = ToolContext::default();

        let result = tool
            .execute(json!({"path": "/tmp/../etc/passwd"}), &ctx)
            .await;

        // Should fail with traversal error
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_file_write_base_dir_traversal_nonexistent_parent() {
        let temp_dir = TempDir::new().unwrap();
        let tool = FileWriteTool::new().with_base_dir(temp_dir.path().to_str().unwrap());
        let ctx = ToolContext::default();

        // Attempt traversal through a nonexistent directory
        let result = tool
            .execute(
                json!({
                    "path": "nonexistent/../../etc/passwd",
                    "content": "malicious"
                }),
                &ctx,
            )
            .await;

        // Should fail
        assert!(result.is_err());
    }
}
