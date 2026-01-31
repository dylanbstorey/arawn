//! File read runtime — reads a file path from config, returns its contents.
//!
//! Config fields:
//! - `path` (string, required): File path to read
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

    match std::fs::read_to_string(&path) {
        Ok(contents) => RuntimeOutput {
            status: "ok".into(),
            output: Some(serde_json::json!({
                "path": path,
                "contents": contents,
                "bytes": contents.len(),
            })),
            error: None,
        },
        Err(e) => RuntimeOutput {
            status: "error".into(),
            output: None,
            error: Some(format!("Failed to read file '{}': {}", path, e)),
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
    fn test_read_existing_file() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("test.txt");
        std::fs::write(&file, "hello world").unwrap();

        let input = json!({"config": {"path": file.display().to_string()}, "context": {}}).to_string();
        let result = process(&input);
        assert_eq!(result.status, "ok");
        let out = result.output.unwrap();
        assert_eq!(out["contents"], "hello world");
        assert_eq!(out["bytes"], 11);
    }

    #[test]
    fn test_read_nonexistent_file() {
        let input = json!({"config": {"path": "/tmp/nonexistent_arawn_test_xyz.txt"}, "context": {}}).to_string();
        let result = process(&input);
        assert_eq!(result.status, "error");
        assert!(result.error.unwrap().contains("Failed to read file"));
    }

    #[test]
    fn test_read_missing_path_config() {
        let input = json!({"config": {}, "context": {}}).to_string();
        let result = process(&input);
        assert_eq!(result.status, "error");
        assert!(result.error.unwrap().contains("Missing required config field 'path'"));
    }

    #[test]
    fn test_read_empty_file() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("empty.txt");
        std::fs::write(&file, "").unwrap();

        let input = json!({"config": {"path": file.display().to_string()}, "context": {}}).to_string();
        let result = process(&input);
        assert_eq!(result.status, "ok");
        let out = result.output.unwrap();
        assert_eq!(out["contents"], "");
        assert_eq!(out["bytes"], 0);
    }

    #[test]
    fn test_read_invalid_json_input() {
        let result = process("not json");
        assert_eq!(result.status, "error");
        assert!(result.error.unwrap().contains("Invalid input JSON"));
    }

    #[test]
    fn test_read_path_is_number() {
        let input = json!({"config": {"path": 42}, "context": {}}).to_string();
        let result = process(&input);
        assert_eq!(result.status, "error");
        assert!(result.error.unwrap().contains("Missing required config field 'path'"));
    }
}
