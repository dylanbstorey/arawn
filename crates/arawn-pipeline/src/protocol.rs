//! Runtime protocol types for WASM task execution.
//!
//! Every workflow task is a WASM module that receives a `RuntimeInput` on stdin
//! and writes a `RuntimeOutput` to stdout, both as JSON.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Input envelope sent to a WASM runtime on stdin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeInput {
    /// Runtime-specific configuration from the workflow definition.
    pub config: Value,
    /// Accumulated context from upstream task outputs.
    pub context: Value,
}

/// Output envelope expected from a WASM runtime on stdout.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeOutput {
    /// Execution status: "ok" or "error".
    pub status: String,
    /// Structured output data (stored under `context[task_id]` for downstream tasks).
    #[serde(default)]
    pub output: Option<Value>,
    /// Error message if status is "error".
    #[serde(default)]
    pub error: Option<String>,
}

impl RuntimeOutput {
    /// Returns true if the runtime reported success.
    pub fn is_ok(&self) -> bool {
        self.status == "ok"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_runtime_input_roundtrip() {
        let input = RuntimeInput {
            config: json!({"url": "https://example.com"}),
            context: json!({"prev_task": {"result": 42}}),
        };
        let serialized = serde_json::to_string(&input).unwrap();
        let deserialized: RuntimeInput = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.config["url"], "https://example.com");
        assert_eq!(deserialized.context["prev_task"]["result"], 42);
    }

    #[test]
    fn test_runtime_output_ok() {
        let output: RuntimeOutput = serde_json::from_value(json!({
            "status": "ok",
            "output": {"data": [1, 2, 3]}
        }))
        .unwrap();
        assert!(output.is_ok());
        assert!(output.error.is_none());
        assert_eq!(output.output.unwrap()["data"][0], 1);
    }

    #[test]
    fn test_runtime_output_error() {
        let output: RuntimeOutput = serde_json::from_value(json!({
            "status": "error",
            "error": "connection refused"
        }))
        .unwrap();
        assert!(!output.is_ok());
        assert_eq!(output.error.unwrap(), "connection refused");
        assert!(output.output.is_none());
    }

    #[test]
    fn test_runtime_output_minimal() {
        let output: RuntimeOutput = serde_json::from_value(json!({
            "status": "ok"
        }))
        .unwrap();
        assert!(output.is_ok());
        assert!(output.output.is_none());
        assert!(output.error.is_none());
    }
}
