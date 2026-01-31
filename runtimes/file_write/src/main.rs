//! File write runtime — writes content to a file path.
//!
//! Config fields:
//! - `path` (string, required): File path to write
//! - `content` (string, required): Content to write
//! - `append` (bool, optional): Append instead of overwrite (default: false)
//!
//! Under WASI preview 1, only preopened directories are accessible.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::Read;

#[derive(Deserialize)]
struct RuntimeInput {
    #[serde(default)]
    config: Value,
    #[serde(default)]
    context: Value,
}

#[derive(Serialize)]
struct RuntimeOutput {
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    output: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

fn emit_error(msg: &str) {
    let out = RuntimeOutput {
        status: "error".into(),
        output: None,
        error: Some(msg.to_string()),
    };
    match serde_json::to_string(&out) {
        Ok(json) => print!("{json}"),
        Err(_) => print!(r#"{{"status":"error","error":"serialization failed"}}"#),
    }
    std::process::exit(1);
}

fn emit_output(out: &RuntimeOutput) {
    match serde_json::to_string(out) {
        Ok(json) => print!("{json}"),
        Err(e) => emit_error(&format!("Failed to serialize output: {e}")),
    }
}

/// Core processing logic, separated for testability.
fn process(input: &str) -> RuntimeOutput {
    let ri: RuntimeInput = match serde_json::from_str(input) {
        Ok(ri) => ri,
        Err(e) => {
            return RuntimeOutput {
                status: "error".into(),
                output: None,
                error: Some(format!("Invalid input JSON: {e}")),
            };
        }
    };

    let path = match ri.config.get("path").and_then(|v| v.as_str()) {
        Some(p) => p.to_string(),
        None => {
            return RuntimeOutput {
                status: "error".into(),
                output: None,
                error: Some("Missing required config field 'path'".into()),
            };
        }
    };

    let content = match ri.config.get("content").and_then(|v| v.as_str()) {
        Some(c) => c.to_string(),
        None => {
            return RuntimeOutput {
                status: "error".into(),
                output: None,
                error: Some("Missing required config field 'content'".into()),
            };
        }
    };

    let append = ri.config.get("append").and_then(|v| v.as_bool()).unwrap_or(false);

    let result = if append {
        use std::io::Write;
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .and_then(|mut f| f.write_all(content.as_bytes()))
    } else {
        std::fs::write(&path, &content).map(|_| ())
    };

    match result {
        Ok(()) => RuntimeOutput {
            status: "ok".into(),
            output: Some(serde_json::json!({
                "path": path,
                "bytes_written": content.len(),
                "append": append,
            })),
            error: None,
        },
        Err(e) => RuntimeOutput {
            status: "error".into(),
            output: None,
            error: Some(format!("Failed to write file '{}': {}", path, e)),
        },
    }
}

fn main() {
    let mut input = String::new();
    if let Err(e) = std::io::stdin().read_to_string(&mut input) {
        emit_error(&format!("Failed to read stdin: {e}"));
        return;
    }

    let result = process(&input);
    if result.status == "error" {
        emit_error(result.error.as_deref().unwrap_or("unknown error"));
    } else {
        emit_output(&result);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_write_new_file() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("out.txt");

        let input = json!({
            "config": {"path": file.display().to_string(), "content": "hello"},
            "context": {}
        }).to_string();
        let result = process(&input);
        assert_eq!(result.status, "ok");
        let out = result.output.unwrap();
        assert_eq!(out["bytes_written"], 5);
        assert_eq!(out["append"], false);
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "hello");
    }

    #[test]
    fn test_write_overwrites_existing() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("out.txt");
        std::fs::write(&file, "old content").unwrap();

        let input = json!({
            "config": {"path": file.display().to_string(), "content": "new"},
            "context": {}
        }).to_string();
        let result = process(&input);
        assert_eq!(result.status, "ok");
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "new");
    }

    #[test]
    fn test_write_append_mode() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("out.txt");
        std::fs::write(&file, "first").unwrap();

        let input = json!({
            "config": {"path": file.display().to_string(), "content": "second", "append": true},
            "context": {}
        }).to_string();
        let result = process(&input);
        assert_eq!(result.status, "ok");
        assert_eq!(result.output.unwrap()["append"], true);
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "firstsecond");
    }

    #[test]
    fn test_write_missing_path() {
        let input = json!({"config": {"content": "data"}, "context": {}}).to_string();
        let result = process(&input);
        assert_eq!(result.status, "error");
        assert!(result.error.unwrap().contains("Missing required config field 'path'"));
    }

    #[test]
    fn test_write_missing_content() {
        let input = json!({"config": {"path": "/tmp/test.txt"}, "context": {}}).to_string();
        let result = process(&input);
        assert_eq!(result.status, "error");
        assert!(result.error.unwrap().contains("Missing required config field 'content'"));
    }

    #[test]
    fn test_write_invalid_path() {
        let input = json!({
            "config": {"path": "/nonexistent_dir_xyz/sub/file.txt", "content": "data"},
            "context": {}
        }).to_string();
        let result = process(&input);
        assert_eq!(result.status, "error");
        assert!(result.error.unwrap().contains("Failed to write file"));
    }

    #[test]
    fn test_write_invalid_json() {
        let result = process("broken");
        assert_eq!(result.status, "error");
        assert!(result.error.unwrap().contains("Invalid input JSON"));
    }

    #[test]
    fn test_write_empty_content() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("empty.txt");

        let input = json!({
            "config": {"path": file.display().to_string(), "content": ""},
            "context": {}
        }).to_string();
        let result = process(&input);
        assert_eq!(result.status, "ok");
        assert_eq!(result.output.unwrap()["bytes_written"], 0);
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "");
    }
}
