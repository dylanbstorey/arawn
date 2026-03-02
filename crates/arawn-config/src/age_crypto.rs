//! Age encryption primitives for the secret store.
//!
//! Provides identity (keypair) management and encrypt/decrypt operations
//! using the `age` crate. The identity file is stored at
//! `~/.config/arawn/identity.age`.

use std::path::{Path, PathBuf};
use std::str::FromStr;

use age::secrecy::ExposeSecret;

/// Get the default path for the age identity file.
pub fn default_identity_path() -> Option<PathBuf> {
    crate::xdg_config_dir().map(|d| d.join("identity.age"))
}

/// Generate a new age identity and save it to a file.
///
/// The file is created with restrictive permissions (owner-only read/write).
/// Returns the public recipient string for reference.
pub fn generate_identity(path: &Path) -> Result<String, AgeError> {
    let identity = age::x25519::Identity::generate();
    let recipient = identity.to_public().to_string();
    let secret_key = identity.to_string();

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| AgeError::Io(format!("creating identity directory: {}", e)))?;
    }

    // Write identity file
    std::fs::write(path, secret_key.expose_secret())
        .map_err(|e| AgeError::Io(format!("writing identity file: {}", e)))?;

    // Set restrictive permissions (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(path, perms)
            .map_err(|e| AgeError::Io(format!("setting identity permissions: {}", e)))?;
    }

    Ok(recipient)
}

/// Load an age identity from a file, generating one if it doesn't exist.
///
/// This is the main entry point — call this at startup to ensure
/// an identity is always available.
pub fn load_or_generate_identity(path: &Path) -> Result<age::x25519::Identity, AgeError> {
    if path.exists() {
        load_identity(path)
    } else {
        generate_identity(path)?;
        load_identity(path)
    }
}

/// Load an existing age identity from a file.
pub fn load_identity(path: &Path) -> Result<age::x25519::Identity, AgeError> {
    let contents = std::fs::read_to_string(path)
        .map_err(|e| AgeError::Io(format!("reading identity: {}", e)))?;

    age::x25519::Identity::from_str(contents.trim())
        .map_err(|e| AgeError::Identity(format!("parsing identity: {}", e)))
}

/// Encrypt data to a recipient (public key).
///
/// Uses the `age` crate's simple API for single-recipient encryption.
pub fn encrypt(data: &[u8], recipient: &age::x25519::Recipient) -> Result<Vec<u8>, AgeError> {
    age::encrypt(recipient, data).map_err(|e| AgeError::Encrypt(e.to_string()))
}

/// Decrypt data with an identity (private key).
///
/// Uses the `age` crate's simple API which handles armored and binary formats.
pub fn decrypt(encrypted: &[u8], identity: &age::x25519::Identity) -> Result<Vec<u8>, AgeError> {
    age::decrypt(identity, encrypted).map_err(|e| AgeError::Decrypt(e.to_string()))
}

/// Errors from age crypto operations.
#[derive(Debug, thiserror::Error)]
pub enum AgeError {
    #[error("IO error: {0}")]
    Io(String),

    #[error("Identity error: {0}")]
    Identity(String),

    #[error("Encryption error: {0}")]
    Encrypt(String),

    #[error("Decryption error: {0}")]
    Decrypt(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let identity = age::x25519::Identity::generate();
        let recipient = identity.to_public();
        let plaintext = b"super secret api key";

        let encrypted = encrypt(plaintext, &recipient).unwrap();
        assert_ne!(encrypted, plaintext);

        let decrypted = decrypt(&encrypted, &identity).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_decrypt_empty() {
        let identity = age::x25519::Identity::generate();
        let recipient = identity.to_public();

        let encrypted = encrypt(b"", &recipient).unwrap();
        let decrypted = decrypt(&encrypted, &identity).unwrap();
        assert_eq!(decrypted, b"");
    }

    #[test]
    fn test_encrypt_decrypt_large() {
        let identity = age::x25519::Identity::generate();
        let recipient = identity.to_public();
        let plaintext = "x".repeat(100_000);

        let encrypted = encrypt(plaintext.as_bytes(), &recipient).unwrap();
        let decrypted = decrypt(&encrypted, &identity).unwrap();
        assert_eq!(decrypted, plaintext.as_bytes());
    }

    #[test]
    fn test_wrong_identity_fails() {
        let identity1 = age::x25519::Identity::generate();
        let identity2 = age::x25519::Identity::generate();
        let recipient1 = identity1.to_public();

        let encrypted = encrypt(b"secret", &recipient1).unwrap();
        let result = decrypt(&encrypted, &identity2);
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_and_load_identity() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test-identity.age");

        let recipient = generate_identity(&path).unwrap();
        assert!(recipient.starts_with("age1"));
        assert!(path.exists());

        let loaded = load_identity(&path).unwrap();
        assert_eq!(loaded.to_public().to_string(), recipient);
    }

    #[test]
    fn test_load_or_generate_creates_when_missing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("new-identity.age");

        assert!(!path.exists());
        let identity = load_or_generate_identity(&path).unwrap();
        assert!(path.exists());

        // Loading again returns the same identity
        let identity2 = load_or_generate_identity(&path).unwrap();
        assert_eq!(
            identity.to_public().to_string(),
            identity2.to_public().to_string()
        );
    }
}
