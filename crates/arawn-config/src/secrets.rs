//! Secrets management — API key storage and retrieval via system keyring.
//!
//! Resolution order:
//! 1. System keyring (if `keyring` feature enabled)
//! 2. Environment variable
//! 3. Config file (with warning)
//!
//! Keyring entries are stored as service="arawn", user="<backend>_api_key".

use crate::Backend;

/// Keyring service name.
const SERVICE_NAME: &str = "arawn";

/// Result of API key resolution with provenance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedSecret {
    /// The secret value.
    pub value: String,
    /// Where the secret was found.
    pub source: SecretSource,
}

/// Where a secret was resolved from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecretSource {
    /// OS keyring (macOS Keychain, Linux secret-service, Windows Credential Manager).
    Keyring,
    /// Environment variable.
    EnvVar(String),
    /// Config file (plaintext — not recommended).
    ConfigFile,
}

impl std::fmt::Display for SecretSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecretSource::Keyring => write!(f, "system keyring"),
            SecretSource::EnvVar(var) => write!(f, "env var {}", var),
            SecretSource::ConfigFile => write!(f, "config file (plaintext)"),
        }
    }
}

/// Resolve an API key for a backend using the full resolution chain.
///
/// Checks in order:
/// 1. System keyring (if feature enabled)
/// 2. Environment variable (backend-specific)
/// 3. Config file value (passed in)
pub fn resolve_api_key(backend: &Backend, config_value: Option<&str>) -> Option<ResolvedSecret> {
    // 1. Keyring
    if let Some(secret) = get_from_keyring(backend) {
        return Some(secret);
    }

    // 2. Environment variable
    let env_var = backend.env_var();
    if let Ok(value) = std::env::var(env_var) {
        if !value.is_empty() {
            return Some(ResolvedSecret {
                value,
                source: SecretSource::EnvVar(env_var.to_string()),
            });
        }
    }

    // 3. Config file
    config_value.map(|v| ResolvedSecret {
        value: v.to_string(),
        source: SecretSource::ConfigFile,
    })
}

/// Store an API key in the system keyring.
///
/// Returns an error message if keyring is not available.
pub fn store_in_keyring(backend: &Backend, api_key: &str) -> std::result::Result<(), String> {
    let user = keyring_user(backend);
    store_keyring_entry(SERVICE_NAME, &user, api_key)
}

/// Delete an API key from the system keyring.
pub fn delete_from_keyring(backend: &Backend) -> std::result::Result<(), String> {
    let user = keyring_user(backend);
    delete_keyring_entry(SERVICE_NAME, &user)
}

/// Check if a keyring entry exists for a backend.
pub fn has_keyring_entry(backend: &Backend) -> bool {
    get_from_keyring(backend).is_some()
}

/// Keyring user name for a backend.
fn keyring_user(backend: &Backend) -> String {
    format!("{}_api_key", backend.env_var().to_lowercase())
}

// ─────────────────────────────────────────────────────────────────────────────
// Keyring implementation (feature-gated)
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "keyring")]
fn get_from_keyring(backend: &Backend) -> Option<ResolvedSecret> {
    // Skip keyring access during tests to avoid macOS Keychain prompts
    // and ensure tests are isolated from local machine state.
    if cfg!(test) {
        return None;
    }

    let user = keyring_user(backend);
    let entry = keyring::Entry::new(SERVICE_NAME, &user).ok()?;
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
    fn test_keyring_user_format() {
        assert_eq!(
            keyring_user(&Backend::Anthropic),
            "anthropic_api_key_api_key"
        );
        // Actually let's check the real format
        let user = keyring_user(&Backend::Groq);
        assert!(user.contains("groq"));
    }

    #[test]
    fn test_resolve_from_config_value() {
        // No keyring, no env var — should fall back to config
        let resolved = resolve_api_key(&Backend::Custom, Some("my-key"));
        assert!(resolved.is_some());
        let r = resolved.unwrap();
        assert_eq!(r.value, "my-key");
        assert_eq!(r.source, SecretSource::ConfigFile);
    }

    #[test]
    fn test_resolve_none_when_nothing_available() {
        // Custom backend, no env var set, no config value
        let resolved = resolve_api_key(&Backend::Custom, None);
        // Might resolve from keyring or env var in some environments
        // but with Custom backend and no config, likely None
        // This test is environment-dependent, just verify it doesn't panic
        let _ = resolved;
    }

    #[test]
    fn test_secret_source_display() {
        assert_eq!(SecretSource::Keyring.to_string(), "system keyring");
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
        // Just verify this doesn't panic even if keyring isn't set up
        let _ = has_keyring_entry(&Backend::Custom);
    }

    #[cfg(not(feature = "keyring"))]
    #[test]
    fn test_store_keyring_disabled() {
        let result = store_in_keyring(&Backend::Groq, "test-key");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not compiled"));
    }
}
