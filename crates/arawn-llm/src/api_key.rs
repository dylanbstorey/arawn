//! API key provider for LLM backends.
//!
//! Supports static keys (tests, CLI) and dynamic resolvers that re-evaluate
//! on each request, enabling hot-loading of secrets without server restart.

use std::sync::Arc;

/// Provides API keys for LLM backends.
///
/// `Static` keys are baked in at construction time (tests, CLI).
/// `Dynamic` providers re-resolve on every call, so secrets stored
/// after server startup are picked up automatically.
#[derive(Clone)]
pub enum ApiKeyProvider {
    /// No API key (e.g., Ollama, custom endpoints without auth).
    None,
    /// Static API key value.
    Static(String),
    /// Dynamic resolver that re-evaluates on each call.
    Dynamic(Arc<dyn Fn() -> Option<String> + Send + Sync>),
}

impl ApiKeyProvider {
    /// Resolve the current API key value.
    pub fn resolve(&self) -> Option<String> {
        match self {
            Self::None => Option::None,
            Self::Static(key) => Some(key.clone()),
            Self::Dynamic(resolver) => resolver(),
        }
    }

    /// Create a static provider from a string.
    pub fn from_static(key: impl Into<String>) -> Self {
        Self::Static(key.into())
    }

    /// Create a dynamic provider from a closure.
    pub fn dynamic(resolver: impl Fn() -> Option<String> + Send + Sync + 'static) -> Self {
        Self::Dynamic(Arc::new(resolver))
    }
}

impl std::fmt::Debug for ApiKeyProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "ApiKeyProvider::None"),
            Self::Static(_) => write!(f, "ApiKeyProvider::Static([REDACTED])"),
            Self::Dynamic(_) => write!(f, "ApiKeyProvider::Dynamic(...)"),
        }
    }
}

impl From<String> for ApiKeyProvider {
    fn from(s: String) -> Self {
        Self::Static(s)
    }
}

impl From<Option<String>> for ApiKeyProvider {
    fn from(opt: Option<String>) -> Self {
        match opt {
            Some(key) => Self::Static(key),
            None => Self::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_static_provider() {
        let provider = ApiKeyProvider::from_static("test-key");
        assert_eq!(provider.resolve(), Some("test-key".to_string()));
    }

    #[test]
    fn test_none_provider() {
        let provider = ApiKeyProvider::None;
        assert_eq!(provider.resolve(), Option::None);
    }

    #[test]
    fn test_dynamic_provider() {
        let counter = Arc::new(std::sync::atomic::AtomicU32::new(0));
        let c = counter.clone();
        let provider = ApiKeyProvider::dynamic(move || {
            let n = c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Some(format!("key-{}", n))
        });
        assert_eq!(provider.resolve(), Some("key-0".to_string()));
        assert_eq!(provider.resolve(), Some("key-1".to_string()));
    }

    #[test]
    fn test_from_string() {
        let provider: ApiKeyProvider = "my-key".to_string().into();
        assert_eq!(provider.resolve(), Some("my-key".to_string()));
    }

    #[test]
    fn test_from_option_some() {
        let provider: ApiKeyProvider = Some("my-key".to_string()).into();
        assert_eq!(provider.resolve(), Some("my-key".to_string()));
    }

    #[test]
    fn test_from_option_none() {
        let provider: ApiKeyProvider = Option::<String>::None.into();
        assert_eq!(provider.resolve(), Option::None);
    }

    #[test]
    fn test_debug_redacts() {
        let provider = ApiKeyProvider::from_static("super-secret");
        let debug = format!("{:?}", provider);
        assert!(!debug.contains("super-secret"));
        assert!(debug.contains("REDACTED"));
    }

    #[test]
    fn test_clone() {
        let provider = ApiKeyProvider::from_static("key");
        let cloned = provider.clone();
        assert_eq!(cloned.resolve(), Some("key".to_string()));
    }

    #[test]
    fn test_dynamic_preserves_exact_value() {
        // Simulate a real Groq key format — must round-trip exactly
        let key = "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";
        let owned = key.to_string();
        let provider = ApiKeyProvider::dynamic(move || Some(owned.clone()));
        assert_eq!(provider.resolve().unwrap(), key);
        assert_eq!(provider.resolve().unwrap().len(), 56);
    }

    #[test]
    fn test_no_whitespace_trimming() {
        // ApiKeyProvider must NOT trim — consumers decide about trimming
        let key_with_spaces = "  sk-ant-abc123  ";
        let provider = ApiKeyProvider::from_static(key_with_spaces);
        assert_eq!(provider.resolve().unwrap(), key_with_spaces);
    }

    #[test]
    fn test_special_chars_preserved() {
        let key = "sk+test/key=with+special/chars==";
        let provider = ApiKeyProvider::from_static(key);
        assert_eq!(provider.resolve().unwrap(), key);
    }
}
