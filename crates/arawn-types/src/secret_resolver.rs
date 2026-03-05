//! Secret resolver trait for opaque handle resolution.
//!
//! Defines the [`SecretResolver`] trait that resolves `${{secrets.<name>}}`
//! handles in tool parameters without exposing raw secret values to the agent.
//! The trait is defined here in `arawn-types` so both `arawn-agent` (consumer)
//! and `arawn-config` (implementor) can reference it without circular dependencies.

use std::sync::Arc;

/// Resolver that looks up secrets by name.
///
/// Tool parameters may contain opaque handles like `${{secrets.github_token}}`.
/// The `ToolRegistry` resolves these handles at execution time by calling
/// `resolve()`, passing the real value to the tool while logging only the handle.
pub trait SecretResolver: Send + Sync {
    /// Resolve a secret by name.
    ///
    /// Returns `Some(value)` if the secret exists, `None` otherwise.
    fn resolve(&self, name: &str) -> Option<String>;

    /// List all available secret names (never values).
    fn names(&self) -> Vec<String>;
}

/// Type alias for a shared secret resolver.
pub type SharedSecretResolver = Arc<dyn SecretResolver>;

/// The handle pattern prefix and suffix for secret references in tool params.
pub const SECRET_HANDLE_PREFIX: &str = "${{secrets.";
pub const SECRET_HANDLE_SUFFIX: &str = "}}";

/// Extract a secret name from a handle string, if it matches the pattern.
///
/// Returns `Some("github_token")` for `${{secrets.github_token}}`,
/// `None` for non-matching strings.
///
/// # Examples
///
/// ```rust,ignore
/// use arawn_types::extract_secret_name;
///
/// assert_eq!(extract_secret_name("${{secrets.api_key}}"), Some("api_key"));
/// assert_eq!(extract_secret_name("plain text"), None);
/// ```
pub fn extract_secret_name(s: &str) -> Option<&str> {
    let rest = s.strip_prefix(SECRET_HANDLE_PREFIX)?;
    let name = rest.strip_suffix(SECRET_HANDLE_SUFFIX)?;
    if name.is_empty() {
        return None;
    }
    Some(name)
}

/// Check if a string contains any secret handle references.
pub fn contains_secret_handle(s: &str) -> bool {
    s.contains(SECRET_HANDLE_PREFIX)
}

/// Resolve all `${{secrets.*}}` handles in a string using the given resolver.
///
/// Returns the string with all handles replaced by their values.
/// Unresolved handles are left as-is (the tool will see them and can report an error).
///
/// # Examples
///
/// ```rust,ignore
/// use arawn_types::resolve_handles_in_string;
///
/// let resolved = resolve_handles_in_string(
///     "Bearer ${{secrets.token}}",
///     &my_resolver,
/// );
/// // If "token" resolves to "abc123": "Bearer abc123"
/// ```
pub fn resolve_handles_in_string(s: &str, resolver: &dyn SecretResolver) -> String {
    if !contains_secret_handle(s) {
        return s.to_string();
    }

    let mut result = s.to_string();
    // Find all ${{secrets.*}} patterns and replace them
    while let Some(start) = result.find(SECRET_HANDLE_PREFIX) {
        let after_prefix = start + SECRET_HANDLE_PREFIX.len();
        if let Some(end_offset) = result[after_prefix..].find(SECRET_HANDLE_SUFFIX) {
            let end = after_prefix + end_offset + SECRET_HANDLE_SUFFIX.len();
            let handle = &result[start..end];
            if let Some(name) = extract_secret_name(handle)
                && let Some(value) = resolver.resolve(name)
            {
                result.replace_range(start..end, &value);
                continue; // Re-scan from same position (value might be shorter)
            }
            // Unresolved — skip past this handle to avoid infinite loop
            break;
        } else {
            // No closing `}}` — malformed, stop scanning
            break;
        }
    }
    result
}

/// Recursively resolve all secret handles in a JSON value.
///
/// Returns a new `Value` with all string values containing `${{secrets.*}}`
/// replaced by their resolved values. Non-string values are unchanged.
pub fn resolve_handles_in_json(
    value: &serde_json::Value,
    resolver: &dyn SecretResolver,
) -> serde_json::Value {
    match value {
        serde_json::Value::String(s) => {
            if contains_secret_handle(s) {
                serde_json::Value::String(resolve_handles_in_string(s, resolver))
            } else {
                value.clone()
            }
        }
        serde_json::Value::Object(map) => {
            let resolved: serde_json::Map<String, serde_json::Value> = map
                .iter()
                .map(|(k, v)| (k.clone(), resolve_handles_in_json(v, resolver)))
                .collect();
            serde_json::Value::Object(resolved)
        }
        serde_json::Value::Array(arr) => {
            let resolved: Vec<serde_json::Value> = arr
                .iter()
                .map(|v| resolve_handles_in_json(v, resolver))
                .collect();
            serde_json::Value::Array(resolved)
        }
        // Numbers, bools, null — unchanged
        _ => value.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestResolver {
        secrets: std::collections::HashMap<String, String>,
    }

    impl TestResolver {
        fn new(pairs: &[(&str, &str)]) -> Self {
            Self {
                secrets: pairs
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
            }
        }
    }

    impl SecretResolver for TestResolver {
        fn resolve(&self, name: &str) -> Option<String> {
            self.secrets.get(name).cloned()
        }
        fn names(&self) -> Vec<String> {
            self.secrets.keys().cloned().collect()
        }
    }

    #[test]
    fn test_extract_secret_name() {
        assert_eq!(
            extract_secret_name("${{secrets.github_token}}"),
            Some("github_token")
        );
        assert_eq!(extract_secret_name("${{secrets.api_key}}"), Some("api_key"));
        assert_eq!(extract_secret_name("${{secrets.}}"), None);
        assert_eq!(extract_secret_name("not a handle"), None);
        assert_eq!(extract_secret_name("${{secrets.incomplete"), None);
    }

    #[test]
    fn test_contains_secret_handle() {
        assert!(contains_secret_handle("Bearer ${{secrets.github_token}}"));
        assert!(!contains_secret_handle("no handles here"));
        assert!(contains_secret_handle("${{secrets.a}}${{secrets.b}}"));
    }

    #[test]
    fn test_resolve_handles_in_string() {
        let resolver = TestResolver::new(&[("token", "abc123")]);

        assert_eq!(
            resolve_handles_in_string("Bearer ${{secrets.token}}", &resolver),
            "Bearer abc123"
        );

        // No handle — returned as-is
        assert_eq!(
            resolve_handles_in_string("no handles", &resolver),
            "no handles"
        );

        // Unknown secret — left as-is
        assert_eq!(
            resolve_handles_in_string("${{secrets.unknown}}", &resolver),
            "${{secrets.unknown}}"
        );
    }

    #[test]
    fn test_resolve_handles_in_json_deep() {
        let resolver = TestResolver::new(&[("token", "real_value"), ("key", "secret_key")]);

        let input = serde_json::json!({
            "headers": {
                "Authorization": "Bearer ${{secrets.token}}",
                "X-Api-Key": "${{secrets.key}}"
            },
            "body": "plain text",
            "count": 42,
            "items": ["${{secrets.token}}", "literal"]
        });

        let resolved = resolve_handles_in_json(&input, &resolver);

        assert_eq!(resolved["headers"]["Authorization"], "Bearer real_value");
        assert_eq!(resolved["headers"]["X-Api-Key"], "secret_key");
        assert_eq!(resolved["body"], "plain text");
        assert_eq!(resolved["count"], 42);
        assert_eq!(resolved["items"][0], "real_value");
        assert_eq!(resolved["items"][1], "literal");
    }

    #[test]
    fn test_resolve_handles_in_json_no_handles() {
        let resolver = TestResolver::new(&[]);
        let input = serde_json::json!({"path": "/tmp/file.txt"});
        let resolved = resolve_handles_in_json(&input, &resolver);
        assert_eq!(resolved, input);
    }
}
