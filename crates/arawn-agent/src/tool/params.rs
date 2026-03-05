//! Typed parameter structs for built-in tools.

use super::validation::{ParamExt, ParameterValidationError};

// ─────────────────────────────────────────────────────────────────────────────
// Typed Parameter Structs
// ─────────────────────────────────────────────────────────────────────────────

/// Validated parameters for the shell tool.
#[derive(Debug, Clone)]
pub struct ShellParams {
    /// The command to execute.
    pub command: String,
    /// Whether to run in PTY mode.
    pub pty: bool,
    /// Whether to stream output.
    pub stream: bool,
    /// Working directory override.
    pub cwd: Option<String>,
    /// Timeout in seconds.
    pub timeout_secs: Option<u64>,
}

impl TryFrom<serde_json::Value> for ShellParams {
    type Error = ParameterValidationError;

    fn try_from(params: serde_json::Value) -> std::result::Result<Self, Self::Error> {
        let command = params.required_str("command", "provide the shell command to execute")?;

        // Validate command is not empty
        if command.trim().is_empty() {
            return Err(ParameterValidationError::invalid_value(
                "command",
                command,
                "command cannot be empty",
            ));
        }

        let timeout_secs = params.get("timeout_secs").and_then(|v| v.as_u64());

        // Validate timeout if provided
        if let Some(timeout) = timeout_secs {
            if timeout == 0 {
                return Err(ParameterValidationError::out_of_range(
                    "timeout_secs",
                    timeout,
                    "must be greater than 0",
                ));
            }
            if timeout > 3600 {
                return Err(ParameterValidationError::out_of_range(
                    "timeout_secs",
                    timeout,
                    "must be at most 3600 (1 hour)",
                ));
            }
        }

        Ok(Self {
            command: command.to_string(),
            pty: params.optional_bool("pty", false),
            stream: params.optional_bool("stream", false),
            cwd: params.optional_str("cwd").map(String::from),
            timeout_secs,
        })
    }
}

/// Validated parameters for file read tool.
#[derive(Debug, Clone)]
pub struct FileReadParams {
    /// Path to the file to read.
    pub path: String,
}

impl TryFrom<serde_json::Value> for FileReadParams {
    type Error = ParameterValidationError;

    fn try_from(params: serde_json::Value) -> std::result::Result<Self, Self::Error> {
        let path = params.required_str("path", "provide the file path to read")?;

        // Validate path is not empty
        if path.trim().is_empty() {
            return Err(ParameterValidationError::invalid_value(
                "path",
                path,
                "path cannot be empty",
            ));
        }

        Ok(Self {
            path: path.to_string(),
        })
    }
}

/// Validated parameters for file write tool.
#[derive(Debug, Clone)]
pub struct FileWriteParams {
    /// Path to the file to write.
    pub path: String,
    /// Content to write.
    pub content: String,
    /// Whether to append instead of overwrite.
    pub append: bool,
}

impl TryFrom<serde_json::Value> for FileWriteParams {
    type Error = ParameterValidationError;

    fn try_from(params: serde_json::Value) -> std::result::Result<Self, Self::Error> {
        let path = params.required_str("path", "provide the file path to write")?;
        let content = params.required_str("content", "provide the content to write")?;

        // Validate path is not empty
        if path.trim().is_empty() {
            return Err(ParameterValidationError::invalid_value(
                "path",
                path,
                "path cannot be empty",
            ));
        }

        Ok(Self {
            path: path.to_string(),
            content: content.to_string(),
            append: params.optional_bool("append", false),
        })
    }
}

/// Validated parameters for web search tool.
#[derive(Debug, Clone)]
pub struct WebSearchParams {
    /// The search query.
    pub query: String,
    /// Maximum number of results.
    pub max_results: u64,
}

impl TryFrom<serde_json::Value> for WebSearchParams {
    type Error = ParameterValidationError;

    fn try_from(params: serde_json::Value) -> std::result::Result<Self, Self::Error> {
        let query = params.required_str("query", "provide a search query")?;

        // Validate query is not empty
        if query.trim().is_empty() {
            return Err(ParameterValidationError::invalid_value(
                "query",
                query,
                "query cannot be empty",
            ));
        }

        let max_results = params.optional_u64("max_results", 10);

        // Validate max_results range
        if max_results == 0 {
            return Err(ParameterValidationError::out_of_range(
                "max_results",
                max_results,
                "must be at least 1",
            ));
        }
        if max_results > 100 {
            return Err(ParameterValidationError::out_of_range(
                "max_results",
                max_results,
                "must be at most 100",
            ));
        }

        Ok(Self {
            query: query.to_string(),
            max_results,
        })
    }
}

/// Validated parameters for think tool.
#[derive(Debug, Clone)]
pub struct ThinkParams {
    /// The thought content.
    pub thought: String,
}

impl TryFrom<serde_json::Value> for ThinkParams {
    type Error = ParameterValidationError;

    fn try_from(params: serde_json::Value) -> std::result::Result<Self, Self::Error> {
        let thought = params.required_str("thought", "provide the thought content")?;

        // Validate thought is not empty
        if thought.trim().is_empty() {
            return Err(ParameterValidationError::invalid_value(
                "thought",
                thought,
                "thought cannot be empty",
            ));
        }

        Ok(Self {
            thought: thought.to_string(),
        })
    }
}

/// Validated parameters for memory store tool.
#[derive(Debug, Clone)]
pub struct MemoryStoreParams {
    /// Content to store.
    pub content: String,
    /// Optional memory type tag.
    pub memory_type: Option<String>,
    /// Optional importance score (0.0-1.0).
    pub importance: Option<f64>,
}

impl TryFrom<serde_json::Value> for MemoryStoreParams {
    type Error = ParameterValidationError;

    fn try_from(params: serde_json::Value) -> std::result::Result<Self, Self::Error> {
        let content = params.required_str("content", "provide the memory content to store")?;

        // Validate content is not empty
        if content.trim().is_empty() {
            return Err(ParameterValidationError::invalid_value(
                "content",
                content,
                "content cannot be empty",
            ));
        }

        // Validate importance if provided
        let importance = params.get("importance").and_then(|v| v.as_f64());
        if let Some(imp) = importance
            && !(0.0..=1.0).contains(&imp)
        {
            return Err(ParameterValidationError::out_of_range(
                "importance",
                imp,
                "must be between 0.0 and 1.0",
            ));
        }

        Ok(Self {
            content: content.to_string(),
            memory_type: params.optional_str("memory_type").map(String::from),
            importance,
        })
    }
}

/// Validated parameters for memory recall tool.
#[derive(Debug, Clone)]
pub struct MemoryRecallParams {
    /// Query to search memories.
    pub query: String,
    /// Maximum number of results.
    pub limit: u64,
    /// Optional memory type filter.
    pub memory_type: Option<String>,
}

impl TryFrom<serde_json::Value> for MemoryRecallParams {
    type Error = ParameterValidationError;

    fn try_from(params: serde_json::Value) -> std::result::Result<Self, Self::Error> {
        let query = params.required_str("query", "provide a query to search memories")?;

        // Validate query is not empty
        if query.trim().is_empty() {
            return Err(ParameterValidationError::invalid_value(
                "query",
                query,
                "query cannot be empty",
            ));
        }

        let limit = params.optional_u64("limit", 10);

        // Validate limit range
        if limit == 0 {
            return Err(ParameterValidationError::out_of_range(
                "limit",
                limit,
                "must be at least 1",
            ));
        }
        if limit > 100 {
            return Err(ParameterValidationError::out_of_range(
                "limit",
                limit,
                "must be at most 100",
            ));
        }

        Ok(Self {
            query: query.to_string(),
            limit,
            memory_type: params.optional_str("memory_type").map(String::from),
        })
    }
}

/// Validated parameters for delegate tool.
#[derive(Debug, Clone)]
pub struct DelegateParams {
    /// The task to delegate.
    pub task: String,
    /// Optional agent type to delegate to.
    pub agent_type: Option<String>,
}

impl TryFrom<serde_json::Value> for DelegateParams {
    type Error = ParameterValidationError;

    fn try_from(params: serde_json::Value) -> std::result::Result<Self, Self::Error> {
        let task = params.required_str("task", "provide the task to delegate")?;

        // Validate task is not empty
        if task.trim().is_empty() {
            return Err(ParameterValidationError::invalid_value(
                "task",
                task,
                "task cannot be empty",
            ));
        }

        Ok(Self {
            task: task.to_string(),
            agent_type: params.optional_str("agent_type").map(String::from),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─────────────────────────────────────────────────────────────────────────
    // Typed Parameter Struct Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_shell_params_valid() {
        let params = serde_json::json!({
            "command": "ls -la",
            "pty": true,
            "stream": false,
            "cwd": "/tmp",
            "timeout_secs": 60
        });
        let shell = ShellParams::try_from(params).unwrap();
        assert_eq!(shell.command, "ls -la");
        assert!(shell.pty);
        assert!(!shell.stream);
        assert_eq!(shell.cwd, Some("/tmp".to_string()));
        assert_eq!(shell.timeout_secs, Some(60));
    }

    #[test]
    fn test_shell_params_minimal() {
        let params = serde_json::json!({"command": "echo hello"});
        let shell = ShellParams::try_from(params).unwrap();
        assert_eq!(shell.command, "echo hello");
        assert!(!shell.pty);
        assert!(!shell.stream);
        assert!(shell.cwd.is_none());
        assert!(shell.timeout_secs.is_none());
    }

    #[test]
    fn test_shell_params_missing_command() {
        let params = serde_json::json!({});
        let err = ShellParams::try_from(params).unwrap_err();
        assert!(matches!(
            err,
            ParameterValidationError::MissingRequired {
                name: "command",
                ..
            }
        ));
    }

    #[test]
    fn test_shell_params_empty_command() {
        let params = serde_json::json!({"command": "   "});
        let err = ShellParams::try_from(params).unwrap_err();
        assert!(matches!(
            err,
            ParameterValidationError::InvalidValue {
                name: "command",
                ..
            }
        ));
    }

    #[test]
    fn test_shell_params_timeout_zero() {
        let params = serde_json::json!({"command": "ls", "timeout_secs": 0});
        let err = ShellParams::try_from(params).unwrap_err();
        assert!(matches!(
            err,
            ParameterValidationError::OutOfRange {
                name: "timeout_secs",
                ..
            }
        ));
    }

    #[test]
    fn test_shell_params_timeout_too_large() {
        let params = serde_json::json!({"command": "ls", "timeout_secs": 7200});
        let err = ShellParams::try_from(params).unwrap_err();
        assert!(matches!(
            err,
            ParameterValidationError::OutOfRange {
                name: "timeout_secs",
                ..
            }
        ));
    }

    #[test]
    fn test_file_read_params_valid() {
        let params = serde_json::json!({"path": "/tmp/file.txt"});
        let file = FileReadParams::try_from(params).unwrap();
        assert_eq!(file.path, "/tmp/file.txt");
    }

    #[test]
    fn test_file_read_params_missing_path() {
        let params = serde_json::json!({});
        let err = FileReadParams::try_from(params).unwrap_err();
        assert!(matches!(
            err,
            ParameterValidationError::MissingRequired { name: "path", .. }
        ));
    }

    #[test]
    fn test_file_read_params_empty_path() {
        let params = serde_json::json!({"path": ""});
        let err = FileReadParams::try_from(params).unwrap_err();
        assert!(matches!(
            err,
            ParameterValidationError::InvalidValue { name: "path", .. }
        ));
    }

    #[test]
    fn test_file_write_params_valid() {
        let params = serde_json::json!({
            "path": "/tmp/out.txt",
            "content": "hello world",
            "append": true
        });
        let file = FileWriteParams::try_from(params).unwrap();
        assert_eq!(file.path, "/tmp/out.txt");
        assert_eq!(file.content, "hello world");
        assert!(file.append);
    }

    #[test]
    fn test_file_write_params_missing_content() {
        let params = serde_json::json!({"path": "/tmp/file.txt"});
        let err = FileWriteParams::try_from(params).unwrap_err();
        assert!(matches!(
            err,
            ParameterValidationError::MissingRequired {
                name: "content",
                ..
            }
        ));
    }

    #[test]
    fn test_web_search_params_valid() {
        let params = serde_json::json!({"query": "rust programming", "max_results": 20});
        let search = WebSearchParams::try_from(params).unwrap();
        assert_eq!(search.query, "rust programming");
        assert_eq!(search.max_results, 20);
    }

    #[test]
    fn test_web_search_params_default_max() {
        let params = serde_json::json!({"query": "test"});
        let search = WebSearchParams::try_from(params).unwrap();
        assert_eq!(search.max_results, 10);
    }

    #[test]
    fn test_web_search_params_max_zero() {
        let params = serde_json::json!({"query": "test", "max_results": 0});
        let err = WebSearchParams::try_from(params).unwrap_err();
        assert!(matches!(
            err,
            ParameterValidationError::OutOfRange {
                name: "max_results",
                ..
            }
        ));
    }

    #[test]
    fn test_web_search_params_max_too_large() {
        let params = serde_json::json!({"query": "test", "max_results": 200});
        let err = WebSearchParams::try_from(params).unwrap_err();
        assert!(matches!(
            err,
            ParameterValidationError::OutOfRange {
                name: "max_results",
                ..
            }
        ));
    }

    #[test]
    fn test_think_params_valid() {
        let params = serde_json::json!({"thought": "I should analyze this"});
        let think = ThinkParams::try_from(params).unwrap();
        assert_eq!(think.thought, "I should analyze this");
    }

    #[test]
    fn test_think_params_empty() {
        let params = serde_json::json!({"thought": "  "});
        let err = ThinkParams::try_from(params).unwrap_err();
        assert!(matches!(
            err,
            ParameterValidationError::InvalidValue {
                name: "thought",
                ..
            }
        ));
    }

    #[test]
    fn test_memory_store_params_valid() {
        let params = serde_json::json!({
            "content": "user prefers dark mode",
            "memory_type": "preference",
            "importance": 0.8
        });
        let mem = MemoryStoreParams::try_from(params).unwrap();
        assert_eq!(mem.content, "user prefers dark mode");
        assert_eq!(mem.memory_type, Some("preference".to_string()));
        assert_eq!(mem.importance, Some(0.8));
    }

    #[test]
    fn test_memory_store_params_importance_invalid() {
        let params = serde_json::json!({"content": "test", "importance": 1.5});
        let err = MemoryStoreParams::try_from(params).unwrap_err();
        assert!(matches!(
            err,
            ParameterValidationError::OutOfRange {
                name: "importance",
                ..
            }
        ));
    }

    #[test]
    fn test_memory_store_params_importance_negative() {
        let params = serde_json::json!({"content": "test", "importance": -0.1});
        let err = MemoryStoreParams::try_from(params).unwrap_err();
        assert!(matches!(
            err,
            ParameterValidationError::OutOfRange {
                name: "importance",
                ..
            }
        ));
    }

    #[test]
    fn test_memory_recall_params_valid() {
        let params = serde_json::json!({
            "query": "user preferences",
            "limit": 5,
            "memory_type": "preference"
        });
        let recall = MemoryRecallParams::try_from(params).unwrap();
        assert_eq!(recall.query, "user preferences");
        assert_eq!(recall.limit, 5);
        assert_eq!(recall.memory_type, Some("preference".to_string()));
    }

    #[test]
    fn test_memory_recall_params_limit_zero() {
        let params = serde_json::json!({"query": "test", "limit": 0});
        let err = MemoryRecallParams::try_from(params).unwrap_err();
        assert!(matches!(
            err,
            ParameterValidationError::OutOfRange { name: "limit", .. }
        ));
    }

    #[test]
    fn test_delegate_params_valid() {
        let params = serde_json::json!({
            "task": "search for documentation",
            "agent_type": "researcher"
        });
        let delegate = DelegateParams::try_from(params).unwrap();
        assert_eq!(delegate.task, "search for documentation");
        assert_eq!(delegate.agent_type, Some("researcher".to_string()));
    }

    #[test]
    fn test_delegate_params_empty_task() {
        let params = serde_json::json!({"task": ""});
        let err = DelegateParams::try_from(params).unwrap_err();
        assert!(matches!(
            err,
            ParameterValidationError::InvalidValue { name: "task", .. }
        ));
    }
}
