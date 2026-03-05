//! Secrets management — API key storage and retrieval.
//!
//! Resolution order:
//! 1. Age-encrypted store (`~/.config/arawn/secrets.age`)
//! 2. System keyring (if `keyring` feature enabled — legacy)
//! 3. Environment variable
//! 4. Config file (with warning)
//!
//! The age store is the primary storage. Keyring support is retained
//! as a legacy fallback but disabled by default.

use crate::Backend;

/// Result of API key resolution with provenance.
///
/// The `Debug` impl intentionally redacts `value` to prevent secret leakage
/// in log output. Use `.value` directly when the actual secret is needed.
#[derive(Clone, PartialEq, Eq)]
pub struct ResolvedSecret {
    /// The secret value.
    pub value: String,
    /// Where the secret was found.
    pub source: SecretSource,
}

impl std::fmt::Debug for ResolvedSecret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResolvedSecret")
            .field("value", &"[REDACTED]")
            .field("source", &self.source)
            .finish()
    }
}

/// Where a secret was resolved from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecretSource {
    /// Age-encrypted secret store.
    AgeStore,
    /// OS keyring (legacy — macOS Keychain, Linux secret-service, Windows Credential Manager).
    Keyring,
    /// Environment variable.
    EnvVar(String),
    /// Config file (plaintext — not recommended).
    ConfigFile,
}

impl std::fmt::Display for SecretSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecretSource::AgeStore => write!(f, "secret store"),
            SecretSource::Keyring => write!(f, "system keyring (legacy)"),
            SecretSource::EnvVar(var) => write!(f, "env var {}", var),
            SecretSource::ConfigFile => write!(f, "config file (plaintext)"),
        }
    }
}

/// Resolve an API key for a backend using the full resolution chain.
///
/// Checks in order:
/// 1. Age-encrypted secret store
/// 2. System keyring (if feature enabled — legacy)
/// 3. Environment variable (backend-specific)
/// 4. Config file value (passed in)
pub fn resolve_api_key(backend: &Backend, config_value: Option<&str>) -> Option<ResolvedSecret> {
    // 1. Age secret store
    if let Some(secret) = get_from_age_store(backend) {
        return Some(secret);
    }

    // 2. Keyring (legacy)
    if let Some(secret) = get_from_keyring(backend) {
        return Some(secret);
    }

    // 3. Environment variable
    let env_var = backend.env_var();
    if let Ok(value) = std::env::var(env_var)
        && !value.is_empty()
    {
        return Some(ResolvedSecret {
            value,
            source: SecretSource::EnvVar(env_var.to_string()),
        });
    }

    // 4. Config file
    config_value.map(|v| ResolvedSecret {
        value: v.to_string(),
        source: SecretSource::ConfigFile,
    })
}

/// Check if the age store has a key for this backend.
pub fn has_age_store_entry(backend: &Backend) -> bool {
    get_from_age_store(backend).is_some()
}

/// Store an API key in the age-encrypted secret store.
///
/// # Examples
///
/// ```rust,ignore
/// use arawn_config::{Backend, secrets};
/// secrets::store_secret(&Backend::Anthropic, "sk-ant-...")?;
/// ```
pub fn store_secret(backend: &Backend, api_key: &str) -> std::result::Result<(), String> {
    let name = age_store_name(backend);
    store_named_secret(&name, api_key)
}

/// Store a named secret in the age-encrypted secret store.
///
/// # Examples
///
/// ```rust,ignore
/// use arawn_config::secrets;
/// secrets::store_named_secret("my_custom_token", "tok-abc123")?;
/// ```
pub fn store_named_secret(name: &str, value: &str) -> std::result::Result<(), String> {
    let store = crate::AgeSecretStore::open_default()
        .map_err(|e| format!("opening secret store: {}", e))?;
    store
        .set(name, value)
        .map_err(|e| format!("storing secret: {}", e))
}

/// Delete an API key from the age-encrypted secret store.
pub fn delete_secret(backend: &Backend) -> std::result::Result<(), String> {
    let name = age_store_name(backend);
    delete_named_secret(&name)
}

/// Delete a named secret from the age-encrypted secret store.
pub fn delete_named_secret(name: &str) -> std::result::Result<(), String> {
    let store = crate::AgeSecretStore::open_default()
        .map_err(|e| format!("opening secret store: {}", e))?;
    store
        .delete(name)
        .map_err(|e| format!("deleting secret: {}", e))?;
    Ok(())
}

/// List all secret names in the age store.
///
/// # Examples
///
/// ```rust,ignore
/// use arawn_config::secrets;
/// let names = secrets::list_secrets()?;
/// for name in &names {
///     println!("stored secret: {}", name);
/// }
/// ```
pub fn list_secrets() -> std::result::Result<Vec<String>, String> {
    let store = crate::AgeSecretStore::open_default()
        .map_err(|e| format!("opening secret store: {}", e))?;
    Ok(store.list())
}

/// Check if an entry exists (age store or keyring).
pub fn has_keyring_entry(backend: &Backend) -> bool {
    has_age_store_entry(backend) || get_from_keyring(backend).is_some()
}

/// Store an API key in the system keyring (legacy).
pub fn store_in_keyring(backend: &Backend, api_key: &str) -> std::result::Result<(), String> {
    let user = keyring_user(backend);
    store_keyring_entry(KEYRING_SERVICE, &user, api_key)
}

/// Delete an API key from the system keyring (legacy).
pub fn delete_from_keyring(backend: &Backend) -> std::result::Result<(), String> {
    let user = keyring_user(backend);
    delete_keyring_entry(KEYRING_SERVICE, &user)
}

// ─────────────────────────────────────────────────────────────────────────────
// Age store implementation
// ─────────────────────────────────────────────────────────────────────────────

/// The secret name used for backend API keys in the age store.
fn age_store_name(backend: &Backend) -> String {
    format!("{}_api_key", backend.env_var().to_lowercase())
}

fn get_from_age_store(backend: &Backend) -> Option<ResolvedSecret> {
    // Skip during tests to avoid touching real config files
    if cfg!(test) {
        return None;
    }

    let store = crate::AgeSecretStore::open_default().ok()?;
    let name = age_store_name(backend);
    let value = store.get(&name)?;

    if value.is_empty() {
        return None;
    }

    Some(ResolvedSecret {
        value,
        source: SecretSource::AgeStore,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Keyring implementation (feature-gated, legacy)
// ─────────────────────────────────────────────────────────────────────────────

/// Keyring service name (legacy).
const KEYRING_SERVICE: &str = "arawn";

/// Keyring user name for a backend (legacy).
fn keyring_user(backend: &Backend) -> String {
    format!("{}_api_key", backend.env_var().to_lowercase())
}

#[cfg(feature = "keyring")]
fn get_from_keyring(backend: &Backend) -> Option<ResolvedSecret> {
    if cfg!(test) {
        return None;
    }

    let user = keyring_user(backend);
    let entry = keyring::Entry::new(KEYRING_SERVICE, &user).ok()?;
    let value = entry.get_password().ok()?;
    if value.is_empty() {
        return None;
    }
    Some(ResolvedSecret {
        value,
        source: SecretSource::Keyring,
    })
}

#[cfg(feature = "keyring")]
fn store_keyring_entry(service: &str, user: &str, secret: &str) -> std::result::Result<(), String> {
    if cfg!(test) {
        return Err("keyring access disabled in tests".to_string());
    }
    let entry = keyring::Entry::new(service, user).map_err(|e| format!("keyring error: {}", e))?;
    entry
        .set_password(secret)
        .map_err(|e| format!("failed to store in keyring: {}", e))
}

#[cfg(feature = "keyring")]
fn delete_keyring_entry(service: &str, user: &str) -> std::result::Result<(), String> {
    if cfg!(test) {
        return Err("keyring access disabled in tests".to_string());
    }
    let entry = keyring::Entry::new(service, user).map_err(|e| format!("keyring error: {}", e))?;
    entry
        .delete_credential()
        .map_err(|e| format!("failed to delete from keyring: {}", e))
}

// ─────────────────────────────────────────────────────────────────────────────
// No-op stubs when keyring feature is disabled
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(not(feature = "keyring"))]
fn get_from_keyring(_backend: &Backend) -> Option<ResolvedSecret> {
    None
}

#[cfg(not(feature = "keyring"))]
fn store_keyring_entry(
    _service: &str,
    _user: &str,
    _secret: &str,
) -> std::result::Result<(), String> {
    Err("keyring support not compiled in (enable the 'keyring' feature)".to_string())
}

#[cfg(not(feature = "keyring"))]
fn delete_keyring_entry(_service: &str, _user: &str) -> std::result::Result<(), String> {
    Err("keyring support not compiled in (enable the 'keyring' feature)".to_string())
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_age_store_name_format() {
        let name = age_store_name(&Backend::Groq);
        assert!(name.contains("groq"));
        assert!(name.ends_with("_api_key"));
    }

    #[test]
    fn test_resolve_from_config_value() {
        // No age store, no keyring, no env var — should fall back to config
        let resolved = resolve_api_key(&Backend::Custom, Some("my-key"));
        assert!(resolved.is_some());
        let r = resolved.unwrap();
        assert_eq!(r.value, "my-key");
        assert_eq!(r.source, SecretSource::ConfigFile);
    }

    #[test]
    fn test_resolve_none_when_nothing_available() {
        let resolved = resolve_api_key(&Backend::Custom, None);
        // Just verify it doesn't panic
        let _ = resolved;
    }

    #[test]
    fn test_secret_source_display() {
        assert_eq!(SecretSource::AgeStore.to_string(), "secret store");
        assert_eq!(SecretSource::Keyring.to_string(), "system keyring (legacy)");
        assert_eq!(
            SecretSource::EnvVar("GROQ_API_KEY".to_string()).to_string(),
            "env var GROQ_API_KEY"
        );
        assert_eq!(
            SecretSource::ConfigFile.to_string(),
            "config file (plaintext)"
        );
    }

    #[test]
    fn test_has_keyring_entry_no_panic() {
        let _ = has_keyring_entry(&Backend::Custom);
    }

    #[cfg(not(feature = "keyring"))]
    #[test]
    fn test_store_keyring_disabled() {
        let result = store_in_keyring(&Backend::Groq, "test-key");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not compiled"));
    }

    #[test]
    fn test_resolved_secret_debug_redacts_value() {
        let secret = ResolvedSecret {
            value: "super-secret-api-key-12345".to_string(),
            source: SecretSource::AgeStore,
        };
        let debug = format!("{:?}", secret);
        assert!(
            !debug.contains("super-secret-api-key-12345"),
            "Debug output must not contain the secret value"
        );
        assert!(debug.contains("[REDACTED]"));
        assert!(debug.contains("AgeStore"));
    }
}
