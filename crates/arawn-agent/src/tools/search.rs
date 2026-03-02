//! File search tools.
//!
//! Provides tools for searching files by pattern and content.

use async_trait::async_trait;
use regex::Regex;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::error::Result;
use crate::tool::{Tool, ToolContext, ToolResult};

// ─────────────────────────────────────────────────────────────────────────────
// Glob Tool
// ─────────────────────────────────────────────────────────────────────────────

/// Tool for finding files matching glob patterns.
#[derive(Debug, Clone)]
pub struct GlobTool {
    /// Base directory to restrict searches.
    base_dir: Option<PathBuf>,
    /// Maximum number of results.
    max_results: usize,
    /// Maximum depth to traverse.
    max_depth: usize,
}

impl GlobTool {
    /// Create a new glob tool.
    pub fn new() -> Self {
        Self {
            base_dir: None,
            max_results: 1000,
            max_depth: 20,
        }
    }

    /// Create a glob tool restricted to a base directory.
    pub fn with_base_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.base_dir = Some(dir.into());
        self
    }

    /// Set maximum number of results.
    pub fn with_max_results(mut self, max: usize) -> Self {
        self.max_results = max;
        self
    }

    /// Set maximum traversal depth.
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    /// Resolve the search directory.
    fn resolve_dir(&self, dir: Option<&str>) -> PathBuf {
        if let Some(d) = dir {
            let path = PathBuf::from(d);
            if let Some(ref base) = self.base_dir {
                if path.is_absolute() {
                    path
                } else {
                    base.join(path)
                }
            } else {
                path
            }
        } else {
            self.base_dir.clone().unwrap_or_else(|| PathBuf::from("."))
        }
    }

    /// Calculate the optimal walk depth for a pattern.
    ///
    /// - Patterns with `**` require full recursive walk
    /// - Patterns like `*.txt` only need depth 1
    /// - Patterns like `src/*.txt` only need depth 2
    fn calculate_walk_depth(&self, pattern: &str) -> usize {
        // If pattern contains `**`, we need full recursive walk
        if pattern.contains("**") {
            return self.max_depth;
        }

        // Count directory separators to determine required depth
        // `*.txt` -> depth 1 (just current dir)
        // `src/*.txt` -> depth 2
        // `src/sub/*.txt` -> depth 3
        let separators = pattern.chars().filter(|&c| c == '/' || c == '\\').count();

        // Add 1 because depth 0 is just the root, depth 1 includes immediate children
        (separators + 1).min(self.max_depth)
    }
}

impl Default for GlobTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for GlobTool {
    fn name(&self) -> &str {
        "glob"
    }

    fn description(&self) -> &str {
        "Find files matching a glob pattern (e.g., '**/*.rs', 'src/*.txt'). Returns a list of matching file paths."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "The glob pattern to match (e.g., '**/*.rs', 'src/**/*.py')"
                },
                "directory": {
                    "type": "string",
                    "description": "Directory to search in. Defaults to current directory."
                }
            },
            "required": ["pattern"]
        })
    }

    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        if ctx.is_cancelled() {
            return Ok(ToolResult::error("Operation cancelled"));
        }

        let pattern = params
            .get("pattern")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::AgentError::Tool("Missing 'pattern' parameter".to_string())
            })?;

        let directory = params.get("directory").and_then(|v| v.as_str());
        let search_dir = self.resolve_dir(directory);

        if !search_dir.exists() {
            return Ok(ToolResult::error(format!(
                "Directory not found: {}",
                search_dir.display()
            )));
        }

        // Build glob pattern
        let full_pattern = search_dir.join(pattern);
        let glob_pattern = match glob::Pattern::new(&full_pattern.to_string_lossy()) {
            Ok(p) => p,
            Err(e) => return Ok(ToolResult::error(format!("Invalid glob pattern: {}", e))),
        };

        // Calculate optimal walk depth based on pattern
        // This avoids walking the entire tree for patterns like `*.toml`
        let walk_depth = self.calculate_walk_depth(pattern);

        // Walk directory and match files
        let mut matches = Vec::new();
        let walker = WalkDir::new(&search_dir)
            .max_depth(walk_depth)
            .follow_links(false);

        for entry in walker.into_iter().filter_map(|e| e.ok()) {
            if ctx.is_cancelled() {
                return Ok(ToolResult::error("Operation cancelled"));
            }

            if matches.len() >= self.max_results {
                break;
            }

            let path = entry.path();
            if glob_pattern.matches_path(path) {
                // Return relative path if possible
                let display_path = path
                    .strip_prefix(&search_dir)
                    .unwrap_or(path)
                    .to_string_lossy()
                    .to_string();

                matches.push(json!({
                    "path": display_path,
                    "is_dir": path.is_dir(),
                    "size": path.metadata().map(|m| m.len()).unwrap_or(0)
                }));
            }
        }

        let truncated = matches.len() >= self.max_results;

        Ok(ToolResult::json(json!({
            "pattern": pattern,
            "directory": search_dir.display().to_string(),
            "count": matches.len(),
            "truncated": truncated,
            "files": matches
        })))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Grep Tool
// ─────────────────────────────────────────────────────────────────────────────

/// A single grep match.
#[derive(Debug, Clone)]
struct GrepMatch {
    file: String,
    line_number: usize,
    line: String,
}

/// Tool for searching file contents with regex.
#[derive(Debug, Clone)]
pub struct GrepTool {
    /// Base directory to restrict searches.
    base_dir: Option<PathBuf>,
    /// Maximum number of results.
    max_results: usize,
    /// Maximum depth to traverse.
    max_depth: usize,
    /// Maximum file size to search (bytes).
    max_file_size: u64,
    /// Number of context lines before/after match.
    context_lines: usize,
}

impl GrepTool {
    /// Create a new grep tool.
    pub fn new() -> Self {
        Self {
            base_dir: None,
            max_results: 500,
            max_depth: 20,
            max_file_size: 10 * 1024 * 1024, // 10MB
            context_lines: 0,
        }
    }

    /// Create a grep tool restricted to a base directory.
    pub fn with_base_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.base_dir = Some(dir.into());
        self
    }

    /// Set maximum number of results.
    pub fn with_max_results(mut self, max: usize) -> Self {
        self.max_results = max;
        self
    }

    /// Set context lines to show before/after matches.
    pub fn with_context_lines(mut self, lines: usize) -> Self {
        self.context_lines = lines;
        self
    }

    /// Resolve the search directory.
    fn resolve_dir(&self, dir: Option<&str>) -> PathBuf {
        if let Some(d) = dir {
            let path = PathBuf::from(d);
            if let Some(ref base) = self.base_dir {
                if path.is_absolute() {
                    path
                } else {
                    base.join(path)
                }
            } else {
                path
            }
        } else {
            self.base_dir.clone().unwrap_or_else(|| PathBuf::from("."))
        }
    }

    /// Check if a file should be searched.
    fn should_search_file(&self, path: &Path) -> bool {
        // Skip hidden files/directories
        if let Some(name) = path.file_name()
            && name.to_string_lossy().starts_with('.')
        {
            return false;
        }

        // Skip binary files (common extensions)
        if let Some(ext) = path.extension() {
            let ext = ext.to_string_lossy().to_lowercase();
            let binary_exts = [
                "exe", "dll", "so", "dylib", "bin", "o", "a", "lib", "png", "jpg", "jpeg", "gif",
                "bmp", "ico", "webp", "mp3", "mp4", "avi", "mov", "mkv", "wav", "flac", "zip",
                "tar", "gz", "bz2", "xz", "7z", "rar", "pdf", "doc", "docx", "xls", "xlsx", "ppt",
                "pptx", "wasm", "pyc", "class",
            ];
            if binary_exts.contains(&ext.as_str()) {
                return false;
            }
        }

        // Skip files that are too large
        if let Ok(metadata) = path.metadata()
            && metadata.len() > self.max_file_size
        {
            return false;
        }

        true
    }

    /// Search a single file.
    fn search_file(&self, path: &Path, regex: &Regex, base_dir: &Path) -> Vec<GrepMatch> {
        let mut matches = Vec::new();

        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return matches, // Skip files that can't be read as text
        };

        let display_path = path
            .strip_prefix(base_dir)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        for (line_num, line) in content.lines().enumerate() {
            if regex.is_match(line) {
                matches.push(GrepMatch {
                    file: display_path.clone(),
                    line_number: line_num + 1,
                    line: line.to_string(),
                });
            }
        }

        matches
    }
}

impl Default for GrepTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for GrepTool {
    fn name(&self) -> &str {
        "grep"
    }

    fn description(&self) -> &str {
        "Search file contents using regular expressions. Returns matching lines with file paths and line numbers."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "The regex pattern to search for"
                },
                "directory": {
                    "type": "string",
                    "description": "Directory to search in. Defaults to current directory."
                },
                "file_pattern": {
                    "type": "string",
                    "description": "Optional glob pattern to filter files (e.g., '*.rs', '*.py')"
                },
                "case_insensitive": {
                    "type": "boolean",
                    "description": "Whether to ignore case. Defaults to false.",
                    "default": false
                }
            },
            "required": ["pattern"]
        })
    }

    async fn execute(&self, params: Value, ctx: &ToolContext) -> Result<ToolResult> {
        if ctx.is_cancelled() {
            return Ok(ToolResult::error("Operation cancelled"));
        }

        let pattern = params
            .get("pattern")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::AgentError::Tool("Missing 'pattern' parameter".to_string())
            })?;

        let directory = params.get("directory").and_then(|v| v.as_str());
        let file_pattern = params.get("file_pattern").and_then(|v| v.as_str());
        let case_insensitive = params
            .get("case_insensitive")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let search_dir = self.resolve_dir(directory);

        if !search_dir.exists() {
            return Ok(ToolResult::error(format!(
                "Directory not found: {}",
                search_dir.display()
            )));
        }

        // Build regex
        let regex_pattern = if case_insensitive {
            format!("(?i){}", pattern)
        } else {
            pattern.to_string()
        };

        let regex = match Regex::new(&regex_pattern) {
            Ok(r) => r,
            Err(e) => return Ok(ToolResult::error(format!("Invalid regex pattern: {}", e))),
        };

        // Build file pattern matcher if provided
        let file_glob = file_pattern.and_then(|p| glob::Pattern::new(p).ok());

        // Walk directory and search files
        let mut all_matches = Vec::new();
        let mut files_searched = 0;

        let walker = WalkDir::new(&search_dir)
            .max_depth(self.max_depth)
            .follow_links(false);

        for entry in walker.into_iter().filter_map(|e| e.ok()) {
            if ctx.is_cancelled() {
                return Ok(ToolResult::error("Operation cancelled"));
            }

            if all_matches.len() >= self.max_results {
                break;
            }

            let path = entry.path();

            // Skip directories
            if path.is_dir() {
                continue;
            }

            // Check file pattern
            if let Some(ref glob) = file_glob
                && let Some(name) = path.file_name()
                && !glob.matches(&name.to_string_lossy())
            {
                continue;
            }

            // Check if file should be searched
            if !self.should_search_file(path) {
                continue;
            }

            files_searched += 1;
            let matches = self.search_file(path, &regex, &search_dir);
            all_matches.extend(matches);
        }

        let truncated = all_matches.len() >= self.max_results;
        all_matches.truncate(self.max_results);

        let results: Vec<Value> = all_matches
            .into_iter()
            .map(|m| {
                json!({
                    "file": m.file,
                    "line_number": m.line_number,
                    "line": m.line
                })
            })
            .collect();

        Ok(ToolResult::json(json!({
            "pattern": pattern,
            "directory": search_dir.display().to_string(),
            "files_searched": files_searched,
            "match_count": results.len(),
            "truncated": truncated,
            "matches": results
        })))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_glob_tool_metadata() {
        let tool = GlobTool::new();
        assert_eq!(tool.name(), "glob");
        assert!(!tool.description().is_empty());

        let params = tool.parameters();
        assert!(params["properties"].get("pattern").is_some());
    }

    #[test]
    fn test_grep_tool_metadata() {
        let tool = GrepTool::new();
        assert_eq!(tool.name(), "grep");
        assert!(!tool.description().is_empty());

        let params = tool.parameters();
        assert!(params["properties"].get("pattern").is_some());
    }

    #[test]
    fn test_calculate_walk_depth() {
        let tool = GlobTool::new(); // max_depth defaults to 20

        // Simple patterns should only walk depth 1
        assert_eq!(tool.calculate_walk_depth("*.txt"), 1);
        assert_eq!(tool.calculate_walk_depth("*.rs"), 1);
        assert_eq!(tool.calculate_walk_depth("Cargo.toml"), 1);

        // Patterns with one directory level should walk depth 2
        assert_eq!(tool.calculate_walk_depth("src/*.rs"), 2);
        assert_eq!(tool.calculate_walk_depth("tests/*.txt"), 2);

        // Patterns with two directory levels should walk depth 3
        assert_eq!(tool.calculate_walk_depth("src/bin/*.rs"), 3);

        // Patterns with ** should use full max_depth
        assert_eq!(tool.calculate_walk_depth("**/*.rs"), 20);
        assert_eq!(tool.calculate_walk_depth("src/**/*.rs"), 20);
        assert_eq!(tool.calculate_walk_depth("**/test.txt"), 20);
    }

    #[test]
    fn test_calculate_walk_depth_respects_max() {
        let tool = GlobTool::new().with_max_depth(5);

        // Should cap at configured max_depth
        assert_eq!(tool.calculate_walk_depth("**/*.rs"), 5);

        // Very deep explicit paths should also cap
        assert_eq!(tool.calculate_walk_depth("a/b/c/d/e/f/g/*.rs"), 5);
    }

    #[tokio::test]
    async fn test_glob_find_files() {
        let temp_dir = TempDir::new().unwrap();

        // Create test files
        fs::write(temp_dir.path().join("test1.txt"), "content1").unwrap();
        fs::write(temp_dir.path().join("test2.txt"), "content2").unwrap();
        fs::write(temp_dir.path().join("other.rs"), "fn main() {}").unwrap();

        let tool = GlobTool::new().with_base_dir(temp_dir.path());
        let ctx = ToolContext::default();

        let result = tool
            .execute(json!({"pattern": "*.txt"}), &ctx)
            .await
            .unwrap();

        assert!(result.is_success());
        let content = result.to_llm_content();
        assert!(content.contains("test1.txt"));
        assert!(content.contains("test2.txt"));
        assert!(!content.contains("other.rs"));
    }

    #[tokio::test]
    async fn test_glob_recursive() {
        let temp_dir = TempDir::new().unwrap();

        // Create nested structure
        let sub_dir = temp_dir.path().join("subdir");
        fs::create_dir(&sub_dir).unwrap();
        fs::write(temp_dir.path().join("root.rs"), "// root").unwrap();
        fs::write(sub_dir.join("nested.rs"), "// nested").unwrap();

        let tool = GlobTool::new().with_base_dir(temp_dir.path());
        let ctx = ToolContext::default();

        let result = tool
            .execute(json!({"pattern": "**/*.rs"}), &ctx)
            .await
            .unwrap();

        assert!(result.is_success());
        let content = result.to_llm_content();
        assert!(content.contains("root.rs"));
        assert!(content.contains("nested.rs"));
    }

    #[tokio::test]
    async fn test_glob_non_recursive_excludes_nested() {
        let temp_dir = TempDir::new().unwrap();

        // Create nested structure with .rs files at both levels
        let sub_dir = temp_dir.path().join("subdir");
        fs::create_dir(&sub_dir).unwrap();
        fs::write(temp_dir.path().join("root.rs"), "// root").unwrap();
        fs::write(sub_dir.join("nested.rs"), "// nested").unwrap();

        let tool = GlobTool::new().with_base_dir(temp_dir.path());
        let ctx = ToolContext::default();

        // Non-recursive pattern should only find root.rs
        let result = tool
            .execute(json!({"pattern": "*.rs"}), &ctx)
            .await
            .unwrap();

        assert!(result.is_success());
        let content = result.to_llm_content();
        assert!(content.contains("root.rs"), "Should find root.rs");
        assert!(
            !content.contains("nested.rs"),
            "Should NOT find nested.rs with non-recursive pattern"
        );
    }

    #[tokio::test]
    async fn test_glob_invalid_pattern() {
        let tool = GlobTool::new();
        let ctx = ToolContext::default();

        let result = tool
            .execute(json!({"pattern": "[invalid"}), &ctx)
            .await
            .unwrap();

        assert!(result.is_error());
    }

    #[tokio::test]
    async fn test_grep_find_matches() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(
            temp_dir.path().join("test.txt"),
            "line one\nfoo bar\nline three\nfoo baz\n",
        )
        .unwrap();

        let tool = GrepTool::new().with_base_dir(temp_dir.path());
        let ctx = ToolContext::default();

        let result = tool.execute(json!({"pattern": "foo"}), &ctx).await.unwrap();

        assert!(result.is_success());
        let content = result.to_llm_content();
        assert!(content.contains("foo bar"));
        assert!(content.contains("foo baz"));
        assert!(content.contains("\"match_count\":2") || content.contains("\"match_count\": 2"));
    }

    #[tokio::test]
    async fn test_grep_case_insensitive() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(
            temp_dir.path().join("test.txt"),
            "Hello\nHELLO\nhello\nworld\n",
        )
        .unwrap();

        let tool = GrepTool::new().with_base_dir(temp_dir.path());
        let ctx = ToolContext::default();

        let result = tool
            .execute(
                json!({
                    "pattern": "hello",
                    "case_insensitive": true
                }),
                &ctx,
            )
            .await
            .unwrap();

        assert!(result.is_success());
        let content = result.to_llm_content();
        assert!(content.contains("\"match_count\":3") || content.contains("\"match_count\": 3"));
    }

    #[tokio::test]
    async fn test_grep_file_pattern() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(temp_dir.path().join("test.rs"), "fn foo() {}").unwrap();
        fs::write(temp_dir.path().join("test.txt"), "fn bar() {}").unwrap();

        let tool = GrepTool::new().with_base_dir(temp_dir.path());
        let ctx = ToolContext::default();

        let result = tool
            .execute(
                json!({
                    "pattern": "fn",
                    "file_pattern": "*.rs"
                }),
                &ctx,
            )
            .await
            .unwrap();

        assert!(result.is_success());
        let content = result.to_llm_content();
        assert!(content.contains("test.rs"));
        assert!(!content.contains("test.txt"));
    }

    #[tokio::test]
    async fn test_grep_regex() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(temp_dir.path().join("test.txt"), "foo123\nbar456\nbaz789\n").unwrap();

        let tool = GrepTool::new().with_base_dir(temp_dir.path());
        let ctx = ToolContext::default();

        let result = tool
            .execute(json!({"pattern": r"\d{3}"}), &ctx)
            .await
            .unwrap();

        assert!(result.is_success());
        let content = result.to_llm_content();
        assert!(content.contains("\"match_count\":3") || content.contains("\"match_count\": 3"));
    }

    #[tokio::test]
    async fn test_grep_invalid_regex() {
        let tool = GrepTool::new();
        let ctx = ToolContext::default();

        let result = tool
            .execute(json!({"pattern": "[invalid"}), &ctx)
            .await
            .unwrap();

        assert!(result.is_error());
        assert!(result.to_llm_content().contains("Invalid regex"));
    }

    #[test]
    fn test_should_search_file() {
        let tool = GrepTool::new();

        // Should search text files
        assert!(tool.should_search_file(Path::new("test.rs")));
        assert!(tool.should_search_file(Path::new("test.txt")));
        assert!(tool.should_search_file(Path::new("test.py")));

        // Should skip binary files
        assert!(!tool.should_search_file(Path::new("test.png")));
        assert!(!tool.should_search_file(Path::new("test.exe")));
        assert!(!tool.should_search_file(Path::new("test.zip")));

        // Should skip hidden files
        assert!(!tool.should_search_file(Path::new(".hidden")));
        assert!(!tool.should_search_file(Path::new(".gitignore")));
    }
}
