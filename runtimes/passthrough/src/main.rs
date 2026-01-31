//! Passthrough runtime — reads RuntimeInput, writes context back unchanged.
//!
//! Useful for testing pipelines and debugging context propagation.

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

    let output = serde_json::json!({
        "config": ri.config,
        "context": ri.context,
    });

    RuntimeOutput {
        status: "ok".into(),
        output: Some(output),
        error: None,
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
    fn test_passthrough_preserves_context() {
        let input = json!({"config": {"key": "val"}, "context": {"a": 1}}).to_string();
        let result = process(&input);
        assert_eq!(result.status, "ok");
        let out = result.output.unwrap();
        assert_eq!(out["context"]["a"], 1);
        assert_eq!(out["config"]["key"], "val");
    }

    #[test]
    fn test_passthrough_empty_input() {
        let input = json!({"config": {}, "context": {}}).to_string();
        let result = process(&input);
        assert_eq!(result.status, "ok");
        assert!(result.output.is_some());
    }

    #[test]
    fn test_passthrough_defaults_on_missing_fields() {
        let input = json!({}).to_string();
        let result = process(&input);
        assert_eq!(result.status, "ok");
        let out = result.output.unwrap();
        assert_eq!(out["config"], json!(null));
        assert_eq!(out["context"], json!(null));
    }

    #[test]
    fn test_passthrough_invalid_json() {
        let result = process("not json at all");
        assert_eq!(result.status, "error");
        assert!(result.error.unwrap().contains("Invalid input JSON"));
    }

    #[test]
    fn test_passthrough_empty_string() {
        let result = process("");
        assert_eq!(result.status, "error");
    }

    #[test]
    fn test_passthrough_nested_context() {
        let input = json!({
            "config": {},
            "context": {"deep": {"nested": {"value": 42}}}
        }).to_string();
        let result = process(&input);
        assert_eq!(result.status, "ok");
        assert_eq!(result.output.unwrap()["context"]["deep"]["nested"]["value"], 42);
    }
}
