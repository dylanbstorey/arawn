//! JSON context wrapper with typed field access helpers.

use serde_json::Value;

/// Wrapper around the JSON context passed to a script via stdin.
#[derive(Debug, Clone)]
pub struct Context {
    data: Value,
}

impl Context {
    /// Parse a `Context` from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let data: Value = serde_json::from_str(json)?;
        Ok(Self { data })
    }

    /// Create a context from an existing JSON value.
    pub fn from_value(data: Value) -> Self {
        Self { data }
    }

    /// Get the raw JSON value.
    pub fn raw(&self) -> &Value {
        &self.data
    }

    /// Get a nested value by dot-separated path (e.g. `"input.text"`).
    pub fn get(&self, path: &str) -> Option<&Value> {
        let mut current = &self.data;
        for segment in path.split('.') {
            // Try array index: segment could be a number
            if let Ok(idx) = segment.parse::<usize>() {
                current = current.get(idx)?;
            } else {
                current = current.get(segment)?;
            }
        }
        Some(current)
    }

    /// Get a string value at the given path.
    pub fn get_str(&self, path: &str) -> Option<&str> {
        self.get(path)?.as_str()
    }

    /// Get an i64 value at the given path.
    pub fn get_i64(&self, path: &str) -> Option<i64> {
        self.get(path)?.as_i64()
    }

    /// Get an f64 value at the given path.
    pub fn get_f64(&self, path: &str) -> Option<f64> {
        self.get(path)?.as_f64()
    }

    /// Get a bool value at the given path.
    pub fn get_bool(&self, path: &str) -> Option<bool> {
        self.get(path)?.as_bool()
    }

    /// Get an array value at the given path.
    pub fn get_array(&self, path: &str) -> Option<&Vec<Value>> {
        self.get(path)?.as_array()
    }

    /// Get an object value at the given path.
    pub fn get_object(&self, path: &str) -> Option<&serde_json::Map<String, Value>> {
        self.get(path)?.as_object()
    }

    /// Deserialize a value at the given path into a typed struct.
    pub fn get_as<T: serde::de::DeserializeOwned>(&self, path: &str) -> Option<T> {
        let val = self.get(path)?;
        serde_json::from_value(val.clone()).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_from_json() {
        let ctx = Context::from_json(r#"{"name": "test"}"#).unwrap();
        assert_eq!(ctx.get_str("name"), Some("test"));
    }

    #[test]
    fn test_nested_path() {
        let ctx = Context::from_value(json!({
            "input": { "text": "hello", "count": 42 }
        }));
        assert_eq!(ctx.get_str("input.text"), Some("hello"));
        assert_eq!(ctx.get_i64("input.count"), Some(42));
    }

    #[test]
    fn test_array_index() {
        let ctx = Context::from_value(json!({
            "items": ["a", "b", "c"]
        }));
        assert_eq!(ctx.get_str("items.1"), Some("b"));
    }

    #[test]
    fn test_missing_path() {
        let ctx = Context::from_value(json!({"a": 1}));
        assert!(ctx.get("b").is_none());
        assert!(ctx.get("a.b").is_none());
    }

    #[test]
    fn test_get_bool() {
        let ctx = Context::from_value(json!({"flag": true}));
        assert_eq!(ctx.get_bool("flag"), Some(true));
    }

    #[test]
    fn test_get_f64() {
        let ctx = Context::from_value(json!({"pi": 3.14}));
        assert!((ctx.get_f64("pi").unwrap() - 3.14).abs() < f64::EPSILON);
    }

    #[test]
    fn test_get_array() {
        let ctx = Context::from_value(json!({"list": [1, 2, 3]}));
        let arr = ctx.get_array("list").unwrap();
        assert_eq!(arr.len(), 3);
    }

    #[test]
    fn test_get_object() {
        let ctx = Context::from_value(json!({"nested": {"key": "val"}}));
        let obj = ctx.get_object("nested").unwrap();
        assert_eq!(obj.get("key").unwrap(), "val");
    }

    #[test]
    fn test_get_as() {
        #[derive(serde::Deserialize, PartialEq, Debug)]
        struct Item {
            name: String,
            count: u32,
        }

        let ctx = Context::from_value(json!({
            "item": { "name": "widget", "count": 5 }
        }));
        let item: Item = ctx.get_as("item").unwrap();
        assert_eq!(item.name, "widget");
        assert_eq!(item.count, 5);
    }
}
