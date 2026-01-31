//! Transform runtime — dot-path traversal and template interpolation on context data.
//!
//! Config fields:
//! - `expression` (string, optional): Dot-path expression to extract from context
//!   (e.g., "task1.body", "step2.result.items")
//! - `template` (string, optional): Template string with `{{expression}}` placeholders
//!   that are resolved against the context
//! - `mappings` (object, optional): Key-value pairs where values are dot-path
//!   expressions to extract, building a new output object
//!
//! At least one of `expression`, `template`, or `mappings` must be provided.

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

/// Resolve a dot-path expression against a JSON value.
/// E.g., "task1.body.items" traverses context["task1"]["body"]["items"].
fn resolve_path<'a>(root: &'a Value, path: &str) -> Option<&'a Value> {
    let mut current = root;
    for segment in path.split('.') {
        // Try object key first
        if let Some(val) = current.get(segment) {
            current = val;
        } else if let Some(idx) = segment.parse::<usize>().ok() {
            // Try array index
            if let Some(val) = current.get(idx) {
                current = val;
            } else {
                return None;
            }
        } else {
            return None;
        }
    }
    Some(current)
}

/// Interpolate `{{expression}}` placeholders in a template string.
fn interpolate(template: &str, context: &Value) -> String {
    let mut result = String::new();
    let mut rest = template;

    while let Some(start) = rest.find("{{") {
        result.push_str(&rest[..start]);
        let after_open = &rest[start + 2..];
        if let Some(end) = after_open.find("}}") {
            let expr = after_open[..end].trim();
            let resolved = resolve_path(context, expr);
            match resolved {
                Some(Value::String(s)) => result.push_str(s),
                Some(v) => result.push_str(&v.to_string()),
                None => {
                    result.push_str("{{");
                    result.push_str(expr);
                    result.push_str("}}");
                }
            }
            rest = &after_open[end + 2..];
        } else {
            // No closing }}, output literally
            result.push_str(&rest[start..]);
            rest = "";
        }
    }
    result.push_str(rest);
    result
}

fn main() {
    let mut input = String::new();
    if let Err(e) = std::io::stdin().read_to_string(&mut input) {
        emit_error(&format!("Failed to read stdin: {e}"));
        return;
    }

    let ri: RuntimeInput = match serde_json::from_str(&input) {
        Ok(ri) => ri,
        Err(e) => {
            emit_error(&format!("Invalid input JSON: {e}"));
            return;
        }
    };

    let expression = ri.config.get("expression").and_then(|v| v.as_str());
    let template = ri.config.get("template").and_then(|v| v.as_str());
    let mappings = ri.config.get("mappings").and_then(|v| v.as_object());

    if expression.is_none() && template.is_none() && mappings.is_none() {
        emit_error("At least one of 'expression', 'template', or 'mappings' must be provided");
        return;
    }

    let output = if let Some(expr) = expression {
        match resolve_path(&ri.context, expr) {
            Some(val) => val.clone(),
            None => Value::Null,
        }
    } else if let Some(tmpl) = template {
        Value::String(interpolate(tmpl, &ri.context))
    } else if let Some(maps) = mappings {
        let mut obj = serde_json::Map::new();
        for (key, val) in maps {
            if let Some(path) = val.as_str() {
                let resolved = resolve_path(&ri.context, path)
                    .cloned()
                    .unwrap_or(Value::Null);
                obj.insert(key.clone(), resolved);
            }
        }
        Value::Object(obj)
    } else {
        Value::Null
    };

    let out = RuntimeOutput {
        status: "ok".into(),
        output: Some(output),
        error: None,
    };
    emit_output(&out);
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_resolve_simple_path() {
        let ctx = json!({"task1": {"body": "hello"}});
        assert_eq!(resolve_path(&ctx, "task1.body"), Some(&json!("hello")));
    }

    #[test]
    fn test_resolve_nested_path() {
        let ctx = json!({"a": {"b": {"c": 42}}});
        assert_eq!(resolve_path(&ctx, "a.b.c"), Some(&json!(42)));
    }

    #[test]
    fn test_resolve_missing_path() {
        let ctx = json!({"a": 1});
        assert_eq!(resolve_path(&ctx, "a.b.c"), None);
    }

    #[test]
    fn test_resolve_array_index() {
        let ctx = json!({"items": [10, 20, 30]});
        assert_eq!(resolve_path(&ctx, "items.1"), Some(&json!(20)));
    }

    #[test]
    fn test_resolve_root_key() {
        let ctx = json!({"key": "value"});
        assert_eq!(resolve_path(&ctx, "key"), Some(&json!("value")));
    }

    #[test]
    fn test_interpolate_basic() {
        let ctx = json!({"name": "world"});
        assert_eq!(interpolate("Hello {{name}}!", &ctx), "Hello world!");
    }

    #[test]
    fn test_interpolate_nested() {
        let ctx = json!({"task1": {"result": "ok"}});
        assert_eq!(
            interpolate("Status: {{task1.result}}", &ctx),
            "Status: ok"
        );
    }

    #[test]
    fn test_interpolate_missing() {
        let ctx = json!({});
        assert_eq!(
            interpolate("{{missing}} value", &ctx),
            "{{missing}} value"
        );
    }

    #[test]
    fn test_interpolate_number() {
        let ctx = json!({"count": 42});
        assert_eq!(interpolate("Count: {{count}}", &ctx), "Count: 42");
    }

    #[test]
    fn test_interpolate_multiple() {
        let ctx = json!({"a": "X", "b": "Y"});
        assert_eq!(interpolate("{{a}}-{{b}}", &ctx), "X-Y");
    }

    #[test]
    fn test_resolve_empty_path() {
        let ctx = json!({"a": 1});
        // Empty string should return the root since split('.') yields [""]
        assert_eq!(resolve_path(&ctx, ""), None);
    }

    #[test]
    fn test_resolve_array_out_of_bounds() {
        let ctx = json!({"items": [10, 20]});
        assert_eq!(resolve_path(&ctx, "items.5"), None);
    }

    #[test]
    fn test_resolve_null_value() {
        let ctx = json!({"key": null});
        assert_eq!(resolve_path(&ctx, "key"), Some(&json!(null)));
    }

    #[test]
    fn test_resolve_boolean_value() {
        let ctx = json!({"flag": true});
        assert_eq!(resolve_path(&ctx, "flag"), Some(&json!(true)));
    }

    #[test]
    fn test_interpolate_unclosed_braces() {
        let ctx = json!({"a": 1});
        assert_eq!(interpolate("start {{a end", &ctx), "start {{a end");
    }

    #[test]
    fn test_interpolate_empty_expression() {
        let ctx = json!({});
        // {{}} should try to resolve "" which yields None, so literal
        assert_eq!(interpolate("{{}}", &ctx), "{{}}");
    }

    #[test]
    fn test_interpolate_no_placeholders() {
        let ctx = json!({"a": 1});
        assert_eq!(interpolate("plain text", &ctx), "plain text");
    }

    #[test]
    fn test_interpolate_adjacent_placeholders() {
        let ctx = json!({"a": "X", "b": "Y"});
        assert_eq!(interpolate("{{a}}{{b}}", &ctx), "XY");
    }
}
