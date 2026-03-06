//! Age-encrypted secret store.
//!
//! Stores secrets as a JSON map encrypted with an age identity.
//! The encrypted file lives at `~/.config/arawn/secrets.age`.
//!
//! The store implements [`SecretResolver`] from `arawn-types` so it can
//! be injected into the agent's `ToolContext` for handle resolution.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use arawn_types::SecretResolver;

use crate::age_crypto::{self, AgeError};

/// Path for the encrypted secrets file.
pub fn default_secrets_path() -> Option<PathBuf> {
    crate::xdg_config_dir().map(|d| d.join("secrets.age"))
}

/// An age-encrypted secret store.
///
/// Secrets are stored as a JSON map `{ "name": "value", ... }` encrypted
/// with the user's age identity. The decrypted map is cached in memory
/// and written back on mutation.
pub struct AgeSecretStore {
    identity: age::x25519::Identity,
    secrets_path: PathBuf,
    /// In-memory cache of decrypted secrets.
    cache: RwLock<BTreeMap<String, String>>,
}

impl AgeSecretStore {
    /// Open or create a secret store.
    ///
    /// Loads the age identity from `identity_path` (generating one if needed),
    /// then decrypts the secrets file at `secrets_path` (creating empty if needed).
    pub fn open(identity_path: &Path, secrets_path: &Path) -> Result<Self, SecretStoreError> {
        let identity =
            age_crypto::load_or_generate_identity(identity_path).map_err(SecretStoreError::Age)?;

        let cache = if secrets_path.exists() {
            let encrypted = std::fs::read(secrets_path)
                .map_err(|e| SecretStoreError::Io(format!("reading secrets: {}", e)))?;

            if encrypted.is_empty() {
                BTreeMap::new()
            } else {
                let decrypted =
                    age_crypto::decrypt(&encrypted, &identity).map_err(SecretStoreError::Age)?;

                serde_json::from_slice(&decrypted)
                    .map_err(|e| SecretStoreError::Format(format!("parsing secrets JSON: {}", e)))?
            }
        } else {
            BTreeMap::new()
        };

        Ok(Self {
            identity,
            secrets_path: secrets_path.to_path_buf(),
            cache: RwLock::new(cache),
        })
    }

    /// Open using default paths (`~/.config/arawn/identity.age` and `secrets.age`).
    pub fn open_default() -> Result<Self, SecretStoreError> {
        let identity_path = age_crypto::default_identity_path()
            .ok_or_else(|| SecretStoreError::Io("cannot determine config directory".to_string()))?;
        let secrets_path = default_secrets_path()
            .ok_or_else(|| SecretStoreError::Io("cannot determine config directory".to_string()))?;

        Self::open(&identity_path, &secrets_path)
    }

    /// Store a secret.
    pub fn set(&self, name: &str, value: &str) -> Result<(), SecretStoreError> {
        {
            let mut cache = self
                .cache
                .write()
                .map_err(|_| SecretStoreError::Io("lock poisoned".to_string()))?;
            cache.insert(name.to_string(), value.to_string());
        }
        self.flush()
    }

    /// Delete a secret.
    ///
    /// Returns `true` if the secret existed.
    pub fn delete(&self, name: &str) -> Result<bool, SecretStoreError> {
        let existed = {
            let mut cache = self
                .cache
                .write()
                .map_err(|_| SecretStoreError::Io("lock poisoned".to_string()))?;
            cache.remove(name).is_some()
        };
        if existed {
            self.flush()?;
        }
        Ok(existed)
    }

    /// Get a secret value by name.
    pub fn get(&self, name: &str) -> Option<String> {
        let cache = self.cache.read().ok()?;
        cache.get(name).cloned()
    }

    /// List all secret names (never values).
    pub fn list(&self) -> Vec<String> {
        self.cache
            .read()
            .map(|c| c.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Check if a secret exists.
    pub fn contains(&self, name: &str) -> bool {
        self.cache
            .read()
            .map(|c| c.contains_key(name))
            .unwrap_or(false)
    }

    /// Flush the in-memory cache to the encrypted file.
    fn flush(&self) -> Result<(), SecretStoreError> {
        let cache = self
            .cache
            .read()
            .map_err(|_| SecretStoreError::Io("lock poisoned".to_string()))?;

        let json = serde_json::to_vec(&*cache)
            .map_err(|e| SecretStoreError::Format(format!("serializing secrets: {}", e)))?;

        let recipient = self.identity.to_public();
        let encrypted = age_crypto::encrypt(&json, &recipient).map_err(SecretStoreError::Age)?;

        // Ensure parent directory exists
        if let Some(parent) = self.secrets_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| SecretStoreError::Io(format!("creating secrets directory: {}", e)))?;
        }

        std::fs::write(&self.secrets_path, &encrypted)
            .map_err(|e| SecretStoreError::Io(format!("writing secrets file: {}", e)))?;

        // Restrictive permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            let _ = std::fs::set_permissions(&self.secrets_path, perms);
        }

        Ok(())
    }
}

impl SecretResolver for AgeSecretStore {
    fn resolve(&self, name: &str) -> Option<String> {
        self.get(name)
    }

    fn names(&self) -> Vec<String> {
        self.list()
    }
}

impl std::fmt::Debug for AgeSecretStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let count = self.cache.read().map(|c| c.len()).unwrap_or(0);
        f.debug_struct("AgeSecretStore")
            .field("secrets_path", &self.secrets_path)
            .field("secret_count", &count)
            .finish()
    }
}

/// Errors from the secret store.
#[derive(Debug, thiserror::Error)]
pub enum SecretStoreError {
    #[error("IO error: {0}")]
    Io(String),

    #[error("Age crypto error: {0}")]
    Age(#[from] AgeError),

    #[error("Format error: {0}")]
    Format(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (tempfile::TempDir, AgeSecretStore) {
        let dir = tempfile::tempdir().unwrap();
        let identity_path = dir.path().join("identity.age");
        let secrets_path = dir.path().join("secrets.age");
        let store = AgeSecretStore::open(&identity_path, &secrets_path).unwrap();
        (dir, store)
    }

    #[test]
    fn test_empty_store() {
        let (_dir, store) = setup();
        assert!(store.list().is_empty());
        assert_eq!(store.get("anything"), None);
        assert!(!store.contains("anything"));
    }

    #[test]
    fn test_set_and_get() {
        let (_dir, store) = setup();
        store.set("api_key", "sk-12345").unwrap();

        assert_eq!(store.get("api_key"), Some("sk-12345".to_string()));
        assert!(store.contains("api_key"));
        assert_eq!(store.list(), vec!["api_key".to_string()]);
    }

    #[test]
    fn test_set_overwrite() {
        let (_dir, store) = setup();
        store.set("key", "v1").unwrap();
        store.set("key", "v2").unwrap();

        assert_eq!(store.get("key"), Some("v2".to_string()));
        assert_eq!(store.list().len(), 1);
    }

    #[test]
    fn test_delete() {
        let (_dir, store) = setup();
        store.set("key", "value").unwrap();
        assert!(store.contains("key"));

        let existed = store.delete("key").unwrap();
        assert!(existed);
        assert!(!store.contains("key"));

        let existed = store.delete("key").unwrap();
        assert!(!existed);
    }

    #[test]
    fn test_multiple_secrets() {
        let (_dir, store) = setup();
        store.set("github_token", "ghp_abc").unwrap();
        store.set("anthropic_key", "sk-ant-xyz").unwrap();
        store.set("groq_key", "gsk-123").unwrap();

        assert_eq!(store.list().len(), 3);
        assert_eq!(store.get("github_token"), Some("ghp_abc".to_string()));
        assert_eq!(store.get("anthropic_key"), Some("sk-ant-xyz".to_string()));
        assert_eq!(store.get("groq_key"), Some("gsk-123".to_string()));
    }

    #[test]
    fn test_persistence_across_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let identity_path = dir.path().join("identity.age");
        let secrets_path = dir.path().join("secrets.age");

        // Store a secret
        {
            let store = AgeSecretStore::open(&identity_path, &secrets_path).unwrap();
            store.set("persistent_key", "persistent_value").unwrap();
        }

        // Reopen and verify
        {
            let store = AgeSecretStore::open(&identity_path, &secrets_path).unwrap();
            assert_eq!(
                store.get("persistent_key"),
                Some("persistent_value".to_string())
            );
        }
    }

    #[test]
    fn test_secret_resolver_trait() {
        let (_dir, store) = setup();
        store.set("token", "abc123").unwrap();

        // Use through the trait
        let resolver: &dyn SecretResolver = &store;
        assert_eq!(resolver.resolve("token"), Some("abc123".to_string()));
        assert_eq!(resolver.resolve("missing"), None);
        assert_eq!(resolver.names(), vec!["token".to_string()]);
    }

    #[test]
    fn test_special_characters_in_values() {
        let (_dir, store) = setup();
        let value = "sk-ant-api03-abc123+/=\n\ttabs & special chars: ${{secrets.nested}}";
        store.set("complex", value).unwrap();
        assert_eq!(store.get("complex"), Some(value.to_string()));
    }

    #[test]
    fn test_groq_key_roundtrip_exact() {
        // Simulate the exact path the server uses:
        // store_named_secret("groq_api_key", key) → reopen → get("groq_api_key")
        let dir = tempfile::tempdir().unwrap();
        let identity_path = dir.path().join("identity.age");
        let secrets_path = dir.path().join("secrets.age");

        let key = "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";

        // Store
        {
            let store = AgeSecretStore::open(&identity_path, &secrets_path).unwrap();
            store.set("groq_api_key", key).unwrap();
        }

        // Reopen and retrieve — simulates server startup reading the store
        {
            let store = AgeSecretStore::open(&identity_path, &secrets_path).unwrap();
            let retrieved = store.get("groq_api_key").unwrap();
            assert_eq!(
                retrieved, key,
                "Key must survive age encrypt/decrypt round-trip exactly"
            );
            assert_eq!(retrieved.len(), 56);
            assert!(retrieved.starts_with("gsk_"));
        }
    }

    #[test]
    fn test_all_backend_key_names_roundtrip() {
        // Verify every backend's store name is just env_var lowercased,
        // consistent everywhere: store, resolve, env var lookup.
        use crate::Backend;

        let dir = tempfile::tempdir().unwrap();
        let identity_path = dir.path().join("identity.age");
        let secrets_path = dir.path().join("secrets.age");

        let store = AgeSecretStore::open(&identity_path, &secrets_path).unwrap();

        let cases = vec![
            (Backend::Groq, "groq_api_key", "gsk_test123"),
            (Backend::Anthropic, "anthropic_api_key", "sk-ant-test456"),
            (Backend::Openai, "openai_api_key", "sk-test789"),
        ];

        for (backend, expected_name, key) in &cases {
            // The canonical name is env_var().to_lowercase()
            let name = backend.env_var().to_lowercase();
            assert_eq!(&name, expected_name, "Name derivation for {:?}", backend);
            store.set(&name, key).unwrap();
        }

        // Verify all retrievable by the same name
        for (_backend, name, key) in &cases {
            let retrieved = store.get(name).unwrap();
            assert_eq!(&retrieved, key);
        }
    }

    #[test]
    fn test_key_no_trailing_newline() {
        // Ensure age store doesn't append newlines to values
        let dir = tempfile::tempdir().unwrap();
        let identity_path = dir.path().join("identity.age");
        let secrets_path = dir.path().join("secrets.age");

        let key = "gsk_test1234567890";
        {
            let store = AgeSecretStore::open(&identity_path, &secrets_path).unwrap();
            store.set("test_key", key).unwrap();
        }
        {
            let store = AgeSecretStore::open(&identity_path, &secrets_path).unwrap();
            let retrieved = store.get("test_key").unwrap();
            assert!(
                !retrieved.ends_with('\n'),
                "Value must not have trailing newline"
            );
            assert!(
                !retrieved.ends_with('\r'),
                "Value must not have trailing CR"
            );
            assert_eq!(retrieved, key);
        }
    }

    #[test]
    fn test_debug_hides_values() {
        let (_dir, store) = setup();
        store.set("secret_key", "very_secret_value").unwrap();

        let debug = format!("{:?}", store);
        assert!(!debug.contains("very_secret_value"));
        assert!(debug.contains("secret_count"));
    }
}
