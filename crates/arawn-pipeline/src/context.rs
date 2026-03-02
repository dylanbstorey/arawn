//! Context template resolver for workflow data flow.
//!
//! Resolves `{{expression}}` templates in task parameters against the workflow
//! context, enabling upstream task outputs to feed into downstream inputs.
//!
//! # Template Syntax
//!
//! - `{{input.field}}` — access workflow-level input context
//! - `{{task_id.output}}` — full output of a completed task
//! - `{{task_id.output.field}}` — nested field access via dot notation
//! - `{{task_id.output.items[0].name}}` — array index access
//!
//! Templates can appear anywhere in string values within action params,
//! LLM prompts, or tool parameters. Multiple templates per string are supported.

use std::collections::HashMap;

use serde_json::Value;

use crate::error::PipelineError;

/// Resolves `{{expression}}` templates against a context data map.
pub struct ContextResolver<'a> {
    data: &'a HashMap<String, Value>,
}

impl<'a> ContextResolver<'a> {
    /// Create a resolver backed by a context data map.
    ///
    /// The map keys are top-level identifiers (task IDs, "input", etc.)
    /// and values are the JSON data associated with each.
    pub fn new(data: &'a HashMap<String, Value>) -> Self {
        Self { data }
    }

    /// Resolve all `{{...}}` templates in a JSON value tree.
    ///
    /// - Strings: template expressions are replaced inline
    /// - Objects/Arrays: recursively resolved
    /// - Other types: returned unchanged
    pub fn resolve_value(&self, value: &Value) -> Result<Value, PipelineError> {
        match value {
            Value::String(s) => self.resolve_string(s),
            Value::Object(map) => {
                let mut resolved = serde_json::Map::new();
                for (k, v) in map {
                    resolved.insert(k.clone(), self.resolve_value(v)?);
                }
                Ok(Value::Object(resolved))
            }
            Value::Array(arr) => {
                let resolved: Result<Vec<Value>, _> =
                    arr.iter().map(|v| self.resolve_value(v)).collect();
                Ok(Value::Array(resolved?))
            }
            other => Ok(other.clone()),
        }
    }

    /// Resolve all `{{...}}` templates in a string.
    ///
    /// If the entire string is a single template expression (e.g., `"{{task.output}}"`)
    /// the result preserves the original JSON type (object, array, number, etc.).
    ///
    /// If the string contains mixed text and templates (e.g., `"Hello {{name}}"`),
    /// all expressions are stringified and concatenated.
    fn resolve_string(&self, s: &str) -> Result<Value, PipelineError> {
        let expressions = parse_template_expressions(s);

        if expressions.is_empty() {
            // No templates — return as-is
            return Ok(Value::String(s.to_string()));
        }

        // If the entire string is exactly one expression, preserve the JSON type
        if expressions.len() == 1 && expressions[0].full_match == s {
            return self.resolve_expression(&expressions[0].path);
        }

        // Mixed text + expressions: stringify everything
        let mut result = s.to_string();
        for expr in &expressions {
            let resolved = self.resolve_expression(&expr.path)?;
            let replacement = value_to_string(&resolved);
            result = result.replace(&expr.full_match, &replacement);
        }

        Ok(Value::String(result))
    }

    /// Resolve a single dot-separated path expression against the context.
    fn resolve_expression(&self, path: &str) -> Result<Value, PipelineError> {
        let segments = parse_path_segments(path);

        if segments.is_empty() {
            return Err(PipelineError::Runtime("Empty template expression".into()));
        }

        // First segment is the top-level key (task ID or "input")
        let root_key = &segments[0].name;
        let root_value = self.data.get(root_key.as_str()).ok_or_else(|| {
            PipelineError::Runtime(format!(
                "Template '{{{{{}}}}}': unknown context key '{}'",
                path, root_key
            ))
        })?;

        // Walk remaining segments
        let mut current = root_value;
        for segment in &segments[1..] {
            current = navigate_segment(current, segment).ok_or_else(|| {
                PipelineError::Runtime(format!(
                    "Template '{{{{{}}}}}': cannot resolve segment '{}' in path",
                    path, segment
                ))
            })?;
        }

        Ok(current.clone())
    }
}

// ---------------------------------------------------------------------------
// Template expression parsing
// ---------------------------------------------------------------------------

/// A parsed `{{expression}}` occurrence in a string.
#[derive(Debug)]
struct TemplateExpression {
    /// The full match including braces, e.g. `"{{task.output.field}}"`.
    full_match: String,
    /// The inner path, e.g. `"task.output.field"`.
    path: String,
}

/// Find all `{{...}}` expressions in a string.
fn parse_template_expressions(s: &str) -> Vec<TemplateExpression> {
    let mut results = Vec::new();
    let mut remaining = s;

    while let Some(start) = remaining.find("{{") {
        if let Some(end) = remaining[start..].find("}}") {
            let full_end = start + end + 2;
            let full_match = &remaining[start..full_end];
            let inner = remaining[start + 2..start + end].trim();

            if !inner.is_empty() {
                results.push(TemplateExpression {
                    full_match: full_match.to_string(),
                    path: inner.to_string(),
                });
            }

            remaining = &remaining[full_end..];
        } else {
            break; // Unclosed `{{` — stop parsing
        }
    }

    results
}

// ---------------------------------------------------------------------------
// Path navigation
// ---------------------------------------------------------------------------

/// A segment of a dot-separated path, optionally with an array index.
#[derive(Debug)]
struct PathSegment {
    name: String,
    index: Option<usize>,
}

impl std::fmt::Display for PathSegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.index {
            Some(i) => write!(f, "{}[{}]", self.name, i),
            None => write!(f, "{}", self.name),
        }
    }
}

/// Parse a dot-separated path into segments, handling array indices.
///
/// `"task.output.items[0].name"` → `[("task", None), ("output", None), ("items", Some(0)), ("name", None)]`
fn parse_path_segments(path: &str) -> Vec<PathSegment> {
    path.split('.')
        .map(|part| {
            if let Some(bracket_start) = part.find('[')
                && let Some(bracket_end) = part.find(']')
            {
                let name = part[..bracket_start].to_string();
                let idx_str = &part[bracket_start + 1..bracket_end];
                let index = idx_str.parse::<usize>().ok();
                return PathSegment { name, index };
            }
            PathSegment {
                name: part.to_string(),
                index: None,
            }
        })
        .collect()
}

/// Navigate one segment of a path through a JSON value.
fn navigate_segment<'a>(value: &'a Value, segment: &PathSegment) -> Option<&'a Value> {
    // First navigate by field name
    let field = if segment.name.is_empty() {
        value
    } else {
        value.get(&segment.name)?
    };

    // Then index into array if needed
    match segment.index {
        Some(i) => field.get(i),
        None => Some(field),
    }
}

/// Convert a JSON value to its string representation for template interpolation.
fn value_to_string(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        // Objects and arrays get JSON serialized
        other => serde_json::to_string(other).unwrap_or_else(|_| "null".to_string()),
    }
}

// ---------------------------------------------------------------------------
// Convenience: resolve params for a task definition
// ---------------------------------------------------------------------------

/// Resolve all template expressions in a set of action parameters.
pub fn resolve_params(
    params: &HashMap<String, Value>,
    context_data: &HashMap<String, Value>,
) -> Result<HashMap<String, Value>, PipelineError> {
    let resolver = ContextResolver::new(context_data);
    let mut resolved = HashMap::new();
    for (key, value) in params {
        resolved.insert(key.clone(), resolver.resolve_value(value)?);
    }
    Ok(resolved)
}

/// Resolve template expressions in a single string (e.g., LLM prompt).
pub fn resolve_template_string(
    template: &str,
    context_data: &HashMap<String, Value>,
) -> Result<String, PipelineError> {
    let resolver = ContextResolver::new(context_data);
    match resolver.resolve_string(template)? {
        Value::String(s) => Ok(s),
        other => Ok(value_to_string(&other)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn test_context() -> HashMap<String, Value> {
        let mut ctx = HashMap::new();
        ctx.insert(
            "input".to_string(),
            json!({
                "text": "Hello world",
                "count": 42,
                "tags": ["rust", "wasm"]
            }),
        );
        ctx.insert(
            "extract".to_string(),
            json!({
                "output": {
                    "entities": [
                        {"name": "Alice", "type": "person"},
                        {"name": "Acme", "type": "org"}
                    ],
                    "summary": "A greeting"
                }
            }),
        );
        ctx.insert("simple".to_string(), json!("plain_value"));
        ctx
    }

    #[test]
    fn test_simple_field_access() {
        let ctx = test_context();
        let resolver = ContextResolver::new(&ctx);
        let result = resolver.resolve_expression("input.text").unwrap();
        assert_eq!(result, json!("Hello world"));
    }

    #[test]
    fn test_numeric_field() {
        let ctx = test_context();
        let resolver = ContextResolver::new(&ctx);
        let result = resolver.resolve_expression("input.count").unwrap();
        assert_eq!(result, json!(42));
    }

    #[test]
    fn test_nested_object_access() {
        let ctx = test_context();
        let resolver = ContextResolver::new(&ctx);
        let result = resolver
            .resolve_expression("extract.output.summary")
            .unwrap();
        assert_eq!(result, json!("A greeting"));
    }

    #[test]
    fn test_array_index_access() {
        let ctx = test_context();
        let resolver = ContextResolver::new(&ctx);
        let result = resolver
            .resolve_expression("extract.output.entities[0].name")
            .unwrap();
        assert_eq!(result, json!("Alice"));
    }

    #[test]
    fn test_array_index_second_element() {
        let ctx = test_context();
        let resolver = ContextResolver::new(&ctx);
        let result = resolver
            .resolve_expression("extract.output.entities[1].type")
            .unwrap();
        assert_eq!(result, json!("org"));
    }

    #[test]
    fn test_full_output_object() {
        let ctx = test_context();
        let resolver = ContextResolver::new(&ctx);
        let result = resolver.resolve_expression("extract.output").unwrap();
        assert!(result.is_object());
        assert_eq!(result["summary"], json!("A greeting"));
    }

    #[test]
    fn test_full_array_access() {
        let ctx = test_context();
        let resolver = ContextResolver::new(&ctx);
        let result = resolver.resolve_expression("input.tags").unwrap();
        assert_eq!(result, json!(["rust", "wasm"]));
    }

    #[test]
    fn test_string_template_preserves_type() {
        let ctx = test_context();
        let resolver = ContextResolver::new(&ctx);
        // Entire string is one expression — preserves JSON type
        let result = resolver.resolve_string("{{input.count}}").unwrap();
        assert_eq!(result, json!(42));
    }

    #[test]
    fn test_mixed_text_and_template() {
        let ctx = test_context();
        let resolver = ContextResolver::new(&ctx);
        let result = resolver
            .resolve_string("Hello {{input.text}}, count={{input.count}}")
            .unwrap();
        assert_eq!(result, json!("Hello Hello world, count=42"));
    }

    #[test]
    fn test_multiple_templates_in_string() {
        let ctx = test_context();
        let resolver = ContextResolver::new(&ctx);
        let result = resolver
            .resolve_string(
                "{{extract.output.entities[0].name}} works at {{extract.output.entities[1].name}}",
            )
            .unwrap();
        assert_eq!(result, json!("Alice works at Acme"));
    }

    #[test]
    fn test_no_templates() {
        let ctx = test_context();
        let resolver = ContextResolver::new(&ctx);
        let result = resolver.resolve_string("no templates here").unwrap();
        assert_eq!(result, json!("no templates here"));
    }

    #[test]
    fn test_missing_root_key_error() {
        let ctx = test_context();
        let resolver = ContextResolver::new(&ctx);
        let err = resolver
            .resolve_expression("nonexistent.field")
            .unwrap_err();
        assert!(
            err.to_string()
                .contains("unknown context key 'nonexistent'")
        );
    }

    #[test]
    fn test_missing_nested_field_error() {
        let ctx = test_context();
        let resolver = ContextResolver::new(&ctx);
        let err = resolver
            .resolve_expression("input.nonexistent")
            .unwrap_err();
        assert!(err.to_string().contains("cannot resolve segment"));
    }

    #[test]
    fn test_array_index_out_of_bounds() {
        let ctx = test_context();
        let resolver = ContextResolver::new(&ctx);
        let err = resolver
            .resolve_expression("extract.output.entities[99].name")
            .unwrap_err();
        assert!(err.to_string().contains("cannot resolve segment"));
    }

    #[test]
    fn test_resolve_value_object() {
        let ctx = test_context();
        let resolver = ContextResolver::new(&ctx);
        let params = json!({
            "prompt": "Summarize: {{input.text}}",
            "count": "{{input.count}}",
            "static": "no templates"
        });
        let result = resolver.resolve_value(&params).unwrap();
        assert_eq!(result["prompt"], json!("Summarize: Hello world"));
        assert_eq!(result["count"], json!(42)); // preserves type when sole expression
        assert_eq!(result["static"], json!("no templates"));
    }

    #[test]
    fn test_resolve_value_array() {
        let ctx = test_context();
        let resolver = ContextResolver::new(&ctx);
        let arr = json!(["{{input.text}}", "literal", "{{input.count}}"]);
        let result = resolver.resolve_value(&arr).unwrap();
        assert_eq!(result[0], json!("Hello world"));
        assert_eq!(result[1], json!("literal"));
        assert_eq!(result[2], json!(42));
    }

    #[test]
    fn test_resolve_value_nested_objects() {
        let ctx = test_context();
        let resolver = ContextResolver::new(&ctx);
        let nested = json!({
            "outer": {
                "inner": "{{extract.output.summary}}"
            }
        });
        let result = resolver.resolve_value(&nested).unwrap();
        assert_eq!(result["outer"]["inner"], json!("A greeting"));
    }

    #[test]
    fn test_resolve_value_primitives_unchanged() {
        let ctx = test_context();
        let resolver = ContextResolver::new(&ctx);
        assert_eq!(resolver.resolve_value(&json!(42)).unwrap(), json!(42));
        assert_eq!(resolver.resolve_value(&json!(true)).unwrap(), json!(true));
        assert_eq!(resolver.resolve_value(&json!(null)).unwrap(), json!(null));
    }

    #[test]
    fn test_resolve_params_convenience() {
        let ctx = test_context();
        let mut params = HashMap::new();
        params.insert("url".to_string(), json!("https://example.com"));
        params.insert("query".to_string(), json!("{{input.text}}"));

        let resolved = resolve_params(&params, &ctx).unwrap();
        assert_eq!(resolved["url"], json!("https://example.com"));
        assert_eq!(resolved["query"], json!("Hello world"));
    }

    #[test]
    fn test_resolve_template_string_convenience() {
        let ctx = test_context();
        let result =
            resolve_template_string("Extract entities from: {{input.text}}", &ctx).unwrap();
        assert_eq!(result, "Extract entities from: Hello world");
    }

    #[test]
    fn test_object_in_mixed_string_serialized() {
        let ctx = test_context();
        let resolver = ContextResolver::new(&ctx);
        let result = resolver.resolve_string("Data: {{extract.output}}").unwrap();
        // Object gets JSON-serialized when embedded in a mixed string
        let s = result.as_str().unwrap();
        assert!(s.starts_with("Data: "));
        assert!(s.contains("summary"));
        assert!(s.contains("entities"));
    }

    #[test]
    fn test_boolean_in_mixed_string() {
        let mut ctx = HashMap::new();
        ctx.insert("flags".to_string(), json!({"enabled": true}));
        let resolver = ContextResolver::new(&ctx);
        let result = resolver
            .resolve_string("Enabled: {{flags.enabled}}")
            .unwrap();
        assert_eq!(result, json!("Enabled: true"));
    }

    #[test]
    fn test_null_in_mixed_string() {
        let mut ctx = HashMap::new();
        ctx.insert("data".to_string(), json!({"missing": null}));
        let resolver = ContextResolver::new(&ctx);
        let result = resolver.resolve_string("Value: {{data.missing}}").unwrap();
        assert_eq!(result, json!("Value: null"));
    }

    #[test]
    fn test_whitespace_in_expression() {
        let ctx = test_context();
        let resolver = ContextResolver::new(&ctx);
        // Whitespace inside braces should be trimmed
        let result = resolver.resolve_string("{{ input.text }}").unwrap();
        assert_eq!(result, json!("Hello world"));
    }

    #[test]
    fn test_unclosed_brace_ignored() {
        let ctx = test_context();
        let resolver = ContextResolver::new(&ctx);
        let result = resolver.resolve_string("open {{ but no close").unwrap();
        assert_eq!(result, json!("open {{ but no close"));
    }
}
