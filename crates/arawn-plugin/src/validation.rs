//! Plugin manifest validation.
//!
//! This module provides rich validation for plugin manifests, ensuring
//! required fields are present, formats are correct, and declared
//! capabilities match actual exports.

use std::path::Path;

/// Error type for manifest validation failures.
///
/// Each variant provides specific context about what failed and how to fix it.
#[derive(Debug, Clone, thiserror::Error)]
pub enum ManifestValidationError {
    /// A required field is missing from the manifest.
    #[error("missing required field '{field}': {hint}")]
    MissingField {
        /// The name of the missing field.
        field: &'static str,
        /// Hint for how to fix the issue.
        hint: &'static str,
    },

    /// A field has an invalid value.
    #[error("invalid value for '{field}': {message}")]
    InvalidField {
        /// The name of the invalid field.
        field: &'static str,
        /// Description of what's wrong.
        message: String,
    },

    /// Version string doesn't follow semver format.
    #[error(
        "invalid version '{version}': {reason}. Expected format: MAJOR.MINOR.PATCH (e.g., 1.0.0)"
    )]
    InvalidVersion {
        /// The invalid version string.
        version: String,
        /// Why it's invalid.
        reason: String,
    },

    /// Declared capabilities don't match actual exports.
    #[error("capability mismatch for '{capability}': declared {declared} but found {actual}")]
    CapabilityMismatch {
        /// The capability type (skills, agents, hooks, etc.).
        capability: &'static str,
        /// What was declared in manifest.
        declared: String,
        /// What was actually found.
        actual: String,
    },

    /// A declared path doesn't exist.
    #[error("path not found: '{path}' declared in '{field}' does not exist")]
    PathNotFound {
        /// The field that declared the path.
        field: &'static str,
        /// The path that wasn't found.
        path: String,
    },
}

impl ManifestValidationError {
    /// Create a missing field error.
    pub fn missing_field(field: &'static str, hint: &'static str) -> Self {
        Self::MissingField { field, hint }
    }

    /// Create an invalid field error.
    pub fn invalid_field(field: &'static str, message: impl Into<String>) -> Self {
        Self::InvalidField {
            field,
            message: message.into(),
        }
    }

    /// Create an invalid version error.
    pub fn invalid_version(version: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidVersion {
            version: version.into(),
            reason: reason.into(),
        }
    }

    /// Create a capability mismatch error.
    pub fn capability_mismatch(
        capability: &'static str,
        declared: impl Into<String>,
        actual: impl Into<String>,
    ) -> Self {
        Self::CapabilityMismatch {
            capability,
            declared: declared.into(),
            actual: actual.into(),
        }
    }

    /// Create a path not found error.
    pub fn path_not_found(field: &'static str, path: impl Into<String>) -> Self {
        Self::PathNotFound {
            field,
            path: path.into(),
        }
    }

    /// Get the field name associated with this error (if any).
    pub fn field_name(&self) -> Option<&str> {
        match self {
            Self::MissingField { field, .. } => Some(field),
            Self::InvalidField { field, .. } => Some(field),
            Self::InvalidVersion { .. } => Some("version"),
            Self::CapabilityMismatch { capability, .. } => Some(capability),
            Self::PathNotFound { field, .. } => Some(field),
        }
    }
}

/// Result type for validation operations.
pub type ValidationResult<T> = std::result::Result<T, ManifestValidationError>;

/// Validate a plugin name.
///
/// Plugin names must be:
/// - Non-empty
/// - Kebab-case (lowercase letters, numbers, hyphens only)
/// - Start with a letter
/// - Not start or end with a hyphen
pub fn validate_name(name: &str) -> ValidationResult<()> {
    if name.is_empty() {
        return Err(ManifestValidationError::missing_field(
            "name",
            "add a unique plugin name in kebab-case (e.g., \"my-plugin\")",
        ));
    }

    // Must start with a letter
    if !name
        .chars()
        .next()
        .map(|c| c.is_ascii_lowercase())
        .unwrap_or(false)
    {
        return Err(ManifestValidationError::invalid_field(
            "name",
            "must start with a lowercase letter",
        ));
    }

    // Must be kebab-case
    if !name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(ManifestValidationError::invalid_field(
            "name",
            "must be kebab-case (lowercase letters, numbers, hyphens only)",
        ));
    }

    // Must not start or end with hyphen
    if name.starts_with('-') || name.ends_with('-') {
        return Err(ManifestValidationError::invalid_field(
            "name",
            "must not start or end with a hyphen",
        ));
    }

    // Must not have consecutive hyphens
    if name.contains("--") {
        return Err(ManifestValidationError::invalid_field(
            "name",
            "must not contain consecutive hyphens",
        ));
    }

    Ok(())
}

/// Validate a semantic version string.
///
/// Accepts formats like:
/// - "1.0.0"
/// - "0.1.0"
/// - "1.2.3-alpha"
/// - "1.2.3-beta.1"
/// - "1.2.3+build.123"
pub fn validate_version(version: &str) -> ValidationResult<()> {
    if version.is_empty() {
        return Err(ManifestValidationError::invalid_version(
            version,
            "version cannot be empty",
        ));
    }

    // Split off pre-release and build metadata
    let version_core = version.split(['-', '+']).next().unwrap_or(version);

    // Check MAJOR.MINOR.PATCH format
    let parts: Vec<&str> = version_core.split('.').collect();

    if parts.len() < 2 || parts.len() > 3 {
        return Err(ManifestValidationError::invalid_version(
            version,
            "must have 2 or 3 numeric components (e.g., 1.0 or 1.0.0)",
        ));
    }

    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            return Err(ManifestValidationError::invalid_version(
                version,
                format!("component {} is empty", i + 1),
            ));
        }

        if !part.chars().all(|c| c.is_ascii_digit()) {
            return Err(ManifestValidationError::invalid_version(
                version,
                format!("component '{}' must be a number", part),
            ));
        }

        // Check for leading zeros (except for "0" itself)
        if part.len() > 1 && part.starts_with('0') {
            return Err(ManifestValidationError::invalid_version(
                version,
                format!("component '{}' has a leading zero", part),
            ));
        }
    }

    Ok(())
}

/// Validate that declared paths exist relative to a plugin directory.
///
/// Returns errors for any paths that don't exist on disk.
pub fn validate_paths_exist(
    field: &'static str,
    paths: &[std::path::PathBuf],
    plugin_dir: &Path,
) -> ValidationResult<()> {
    for path in paths {
        let full_path = if path.is_relative() {
            plugin_dir.join(path)
        } else {
            path.clone()
        };

        if !full_path.exists() {
            return Err(ManifestValidationError::path_not_found(
                field,
                path.display().to_string(),
            ));
        }
    }

    Ok(())
}

/// Count items discovered at the given paths.
///
/// For directories, counts subdirectories (skills) or .md files (agents).
/// For files, counts the file itself.
pub fn count_discovered_items(
    paths: &[std::path::PathBuf],
    plugin_dir: &Path,
    pattern: &str,
) -> usize {
    let mut count = 0;

    for path in paths {
        let full_path = if path.is_relative() {
            plugin_dir.join(path)
        } else {
            path.clone()
        };

        if full_path.is_file() {
            count += 1;
        } else if full_path.is_dir()
            && let Ok(entries) = std::fs::read_dir(&full_path)
        {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                let matches = match pattern {
                    "dir" => entry_path.is_dir(),
                    ext => entry_path.extension().map(|e| e == ext).unwrap_or(false),
                };
                if matches {
                    count += 1;
                }
            }
        }
    }

    count
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─────────────────────────────────────────────────────────────────────────
    // Name Validation Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_valid_names() {
        assert!(validate_name("my-plugin").is_ok());
        assert!(validate_name("plugin123").is_ok());
        assert!(validate_name("a").is_ok());
        assert!(validate_name("my-cool-plugin").is_ok());
        assert!(validate_name("plugin-v2").is_ok());
    }

    #[test]
    fn test_empty_name() {
        let err = validate_name("").unwrap_err();
        assert!(matches!(
            err,
            ManifestValidationError::MissingField { field: "name", .. }
        ));
    }

    #[test]
    fn test_name_starts_with_number() {
        let err = validate_name("123plugin").unwrap_err();
        assert!(matches!(
            err,
            ManifestValidationError::InvalidField { field: "name", .. }
        ));
    }

    #[test]
    fn test_name_starts_with_hyphen() {
        let err = validate_name("-plugin").unwrap_err();
        assert!(matches!(
            err,
            ManifestValidationError::InvalidField { field: "name", .. }
        ));
    }

    #[test]
    fn test_name_ends_with_hyphen() {
        let err = validate_name("plugin-").unwrap_err();
        assert!(matches!(
            err,
            ManifestValidationError::InvalidField { field: "name", .. }
        ));
    }

    #[test]
    fn test_name_consecutive_hyphens() {
        let err = validate_name("my--plugin").unwrap_err();
        assert!(matches!(
            err,
            ManifestValidationError::InvalidField { field: "name", .. }
        ));
    }

    #[test]
    fn test_name_uppercase() {
        let err = validate_name("MyPlugin").unwrap_err();
        assert!(matches!(
            err,
            ManifestValidationError::InvalidField { field: "name", .. }
        ));
    }

    #[test]
    fn test_name_spaces() {
        let err = validate_name("my plugin").unwrap_err();
        assert!(matches!(
            err,
            ManifestValidationError::InvalidField { field: "name", .. }
        ));
    }

    #[test]
    fn test_name_underscores() {
        let err = validate_name("my_plugin").unwrap_err();
        assert!(matches!(
            err,
            ManifestValidationError::InvalidField { field: "name", .. }
        ));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Version Validation Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_valid_versions() {
        assert!(validate_version("1.0.0").is_ok());
        assert!(validate_version("0.1.0").is_ok());
        assert!(validate_version("10.20.30").is_ok());
        assert!(validate_version("1.0").is_ok());
        assert!(validate_version("1.0.0-alpha").is_ok());
        assert!(validate_version("1.0.0-beta.1").is_ok());
        assert!(validate_version("1.0.0+build.123").is_ok());
        assert!(validate_version("1.0.0-rc.1+build.456").is_ok());
    }

    #[test]
    fn test_empty_version() {
        let err = validate_version("").unwrap_err();
        assert!(matches!(
            err,
            ManifestValidationError::InvalidVersion { .. }
        ));
    }

    #[test]
    fn test_version_single_number() {
        let err = validate_version("1").unwrap_err();
        assert!(matches!(
            err,
            ManifestValidationError::InvalidVersion { .. }
        ));
    }

    #[test]
    fn test_version_four_parts() {
        let err = validate_version("1.2.3.4").unwrap_err();
        assert!(matches!(
            err,
            ManifestValidationError::InvalidVersion { .. }
        ));
    }

    #[test]
    fn test_version_non_numeric() {
        let err = validate_version("1.x.0").unwrap_err();
        assert!(matches!(
            err,
            ManifestValidationError::InvalidVersion { .. }
        ));
    }

    #[test]
    fn test_version_leading_zero() {
        let err = validate_version("01.0.0").unwrap_err();
        assert!(matches!(
            err,
            ManifestValidationError::InvalidVersion { .. }
        ));
    }

    #[test]
    fn test_version_empty_component() {
        let err = validate_version("1..0").unwrap_err();
        assert!(matches!(
            err,
            ManifestValidationError::InvalidVersion { .. }
        ));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Error Type Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_error_display() {
        let err = ManifestValidationError::missing_field("name", "add a name");
        assert!(err.to_string().contains("name"));
        assert!(err.to_string().contains("add a name"));
    }

    #[test]
    fn test_error_field_name() {
        let missing = ManifestValidationError::missing_field("name", "hint");
        assert_eq!(missing.field_name(), Some("name"));

        let invalid = ManifestValidationError::invalid_field("version", "bad");
        assert_eq!(invalid.field_name(), Some("version"));

        let version = ManifestValidationError::invalid_version("x", "y");
        assert_eq!(version.field_name(), Some("version"));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Path Validation Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_paths_exist_empty() {
        let paths: Vec<std::path::PathBuf> = vec![];
        assert!(validate_paths_exist("skills", &paths, Path::new("/tmp")).is_ok());
    }

    #[test]
    fn test_paths_exist_missing() {
        let paths = vec![std::path::PathBuf::from("nonexistent-path-12345")];
        let err = validate_paths_exist("skills", &paths, Path::new("/tmp")).unwrap_err();
        assert!(matches!(
            err,
            ManifestValidationError::PathNotFound {
                field: "skills",
                ..
            }
        ));
    }
}
