//! HTTP runtime — makes HTTP requests based on config fields.
//!
//! Config fields:
//! - `url` (string, required): The URL to request
//! - `method` (string, optional): HTTP method, defaults to "GET"
//! - `headers` (object, optional): Key-value headers
//! - `body` (string, optional): Request body
//!
//! Since WASI preview 1 doesn't have native HTTP, this runtime uses
//! a minimal approach: it constructs the request representation and
//! delegates actual execution to the host via stdout protocol.
//! The host-side executor intercepts and performs the actual HTTP call.
//!
//! In standalone WASI preview 2 environments, this would use wasi-http directly.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
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

#[derive(Deserialize)]
struct HttpConfig {
    url: String,
    #[serde(default = "default_method")]
    method: String,
    #[serde(default)]
    headers: HashMap<String, String>,
    #[serde(default)]
    body: Option<String>,
}

fn default_method() -> String {
    "GET".into()
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

    let http_cfg: HttpConfig = match serde_json::from_value(ri.config) {
        Ok(c) => c,
        Err(e) => {
            return RuntimeOutput {
                status: "error".into(),
                output: None,
                error: Some(format!("Invalid HTTP config: {e}. Expected: url (required), method, headers, body")),
            };
        }
    };

    let request = serde_json::json!({
        "request": {
            "url": http_cfg.url,
            "method": http_cfg.method,
            "headers": http_cfg.headers,
            "body": http_cfg.body,
        },
        "context": ri.context,
    });

    RuntimeOutput {
        status: "ok".into(),
        output: Some(request),
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
    fn test_http_get_request() {
        let input = json!({
            "config": {"url": "https://example.com/api"},
            "context": {}
        }).to_string();
        let result = process(&input);
        assert_eq!(result.status, "ok");
        let out = result.output.unwrap();
        assert_eq!(out["request"]["url"], "https://example.com/api");
        assert_eq!(out["request"]["method"], "GET");
    }

    #[test]
    fn test_http_post_with_body() {
        let input = json!({
            "config": {
                "url": "https://example.com/api",
                "method": "POST",
                "body": "{\"key\": \"value\"}"
            },
            "context": {}
        }).to_string();
        let result = process(&input);
        assert_eq!(result.status, "ok");
        let out = result.output.unwrap();
        assert_eq!(out["request"]["method"], "POST");
        assert_eq!(out["request"]["body"], "{\"key\": \"value\"}");
    }

    #[test]
    fn test_http_with_headers() {
        let input = json!({
            "config": {
                "url": "https://example.com",
                "headers": {"Authorization": "Bearer token123"}
            },
            "context": {}
        }).to_string();
        let result = process(&input);
        assert_eq!(result.status, "ok");
        let out = result.output.unwrap();
        assert_eq!(out["request"]["headers"]["Authorization"], "Bearer token123");
    }

    #[test]
    fn test_http_missing_url() {
        let input = json!({
            "config": {"method": "GET"},
            "context": {}
        }).to_string();
        let result = process(&input);
        assert_eq!(result.status, "error");
        assert!(result.error.unwrap().contains("Invalid HTTP config"));
    }

    #[test]
    fn test_http_empty_config() {
        let input = json!({"config": {}, "context": {}}).to_string();
        let result = process(&input);
        assert_eq!(result.status, "error");
        assert!(result.error.unwrap().contains("url"));
    }

    #[test]
    fn test_http_invalid_json() {
        let result = process("{{garbage");
        assert_eq!(result.status, "error");
        assert!(result.error.unwrap().contains("Invalid input JSON"));
    }

    #[test]
    fn test_http_preserves_context() {
        let input = json!({
            "config": {"url": "https://example.com"},
            "context": {"session_id": "abc123"}
        }).to_string();
        let result = process(&input);
        assert_eq!(result.status, "ok");
        assert_eq!(result.output.unwrap()["context"]["session_id"], "abc123");
    }

    #[test]
    fn test_http_default_method() {
        let input = json!({
            "config": {"url": "https://example.com"},
            "context": {}
        }).to_string();
        let result = process(&input);
        assert_eq!(result.output.unwrap()["request"]["method"], "GET");
    }
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
