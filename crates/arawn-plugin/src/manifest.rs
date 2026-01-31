//! Plugin manifest parsing and validation.
//!
//! A plugin manifest (`.claude-plugin/plugin.json`) describes plugin metadata
//! and paths to component directories. This follows Claude Code's plugin format.

use crate::validation::{self, ManifestValidationError};
use crate::{PluginError, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Summary of declared vs discovered capabilities for a plugin.
#[derive(Debug, Clone, Default)]
pub struct CapabilitySummary {
    /// Whether skills are declared in manifest.
    pub skills_declared: bool,
    /// Number of skills discovered on disk.
    pub skills_found: usize,
    /// Whether agents are declared in manifest.
    pub agents_declared: bool,
    /// Number of agents discovered on disk.
    pub agents_found: usize,
    /// Whether hooks are declared in manifest.
    pub hooks_declared: bool,
    /// Number of hooks config files found.
    pub hooks_found: usize,
    /// Whether commands are declared in manifest.
    pub commands_declared: bool,
    /// Number of commands discovered on disk.
    pub commands_found: usize,
}

impl CapabilitySummary {
    /// Check if there are any capability mismatches.
    ///
    /// A mismatch occurs when:
    /// - Capabilities are declared but none found
    /// - Capabilities are found but not declared (warning, not error)
    pub fn has_errors(&self) -> bool {
        (self.skills_declared && self.skills_found == 0)
            || (self.agents_declared && self.agents_found == 0)
            || (self.hooks_declared && self.hooks_found == 0)
            || (self.commands_declared && self.commands_found == 0)
    }

    /// Get a list of warnings (undeclared but found capabilities).
    pub fn warnings(&self) -> Vec<String> {
        let mut warnings = Vec::new();

        if !self.skills_declared && self.skills_found > 0 {
            warnings.push(format!(
                "found {} skills but 'skills' not declared in manifest",
                self.skills_found
            ));
        }
        if !self.agents_declared && self.agents_found > 0 {
            warnings.push(format!(
                "found {} agents but 'agents' not declared in manifest",
                self.agents_found
            ));
        }

        warnings
    }

    /// Get a list of errors (declared but not found capabilities).
    pub fn errors(&self) -> Vec<ManifestValidationError> {
        let mut errors = Vec::new();

        if self.skills_declared && self.skills_found == 0 {
            errors.push(ManifestValidationError::capability_mismatch(
                "skills",
                "path declared",
                "no skills found",
            ));
        }
        if self.agents_declared && self.agents_found == 0 {
            errors.push(ManifestValidationError::capability_mismatch(
                "agents",
                "path declared",
                "no agents found",
            ));
        }
        if self.hooks_declared && self.hooks_found == 0 {
            errors.push(ManifestValidationError::capability_mismatch(
                "hooks",
                "path declared",
                "no hooks config found",
            ));
        }
        if self.commands_declared && self.commands_found == 0 {
            errors.push(ManifestValidationError::capability_mismatch(
                "commands",
                "path declared",
                "no commands found",
            ));
        }

        errors
    }
}

/// Top-level plugin manifest parsed from `.claude-plugin/plugin.json`.
///
/// This follows Claude Code's plugin.json schema. Component discovery happens
/// by scanning the directories specified in the manifest (skills, agents, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Plugin name (unique identifier, kebab-case).
    pub name: String,

    /// Semantic version (e.g., "1.0.0").
    #[serde(default)]
    pub version: Option<String>,

    /// Human-readable description.
    #[serde(default)]
    pub description: Option<String>,

    /// Author information.
    #[serde(default)]
    pub author: Option<PluginAuthor>,

    /// Documentation URL.
    #[serde(default)]
    pub homepage: Option<String>,

    /// Source repository URL.
    #[serde(default)]
    pub repository: Option<String>,

    /// SPDX license identifier.
    #[serde(default)]
    pub license: Option<String>,

    /// Discovery keywords.
    #[serde(default)]
    pub keywords: Vec<String>,

    /// Path to commands directory or files (relative to plugin root).
    #[serde(default)]
    pub commands: Option<PathOrPaths>,

    /// Path to agents directory or files (relative to plugin root).
    #[serde(default)]
    pub agents: Option<PathOrPaths>,

    /// Path to skills directory (relative to plugin root).
    #[serde(default)]
    pub skills: Option<PathOrPaths>,

    /// Path to hooks config file or directory (relative to plugin root).
    #[serde(default)]
    pub hooks: Option<PathOrPaths>,

    /// Path to MCP servers config file or inline config.
    #[serde(default, rename = "mcpServers")]
    pub mcp_servers: Option<serde_json::Value>,

    /// Path to LSP servers config file or inline config.
    #[serde(default, rename = "lspServers")]
    pub lsp_servers: Option<serde_json::Value>,

    /// Path to output styles directory.
    #[serde(default, rename = "outputStyles")]
    pub output_styles: Option<PathOrPaths>,
}

/// A path or array of paths (Claude supports both).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PathOrPaths {
    /// Single path.
    Single(PathBuf),
    /// Multiple paths.
    Multiple(Vec<PathBuf>),
}

impl PathOrPaths {
    /// Get all paths as a vector.
    pub fn to_vec(&self) -> Vec<PathBuf> {
        match self {
            PathOrPaths::Single(p) => vec![p.clone()],
            PathOrPaths::Multiple(ps) => ps.clone(),
        }
    }

    /// Resolve all paths against a base directory.
    pub fn resolve(&self, base: &Path) -> Vec<PathBuf> {
        self.to_vec()
            .into_iter()
            .map(|p| if p.is_relative() { base.join(p) } else { p })
            .collect()
    }
}

/// Plugin author information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginAuthor {
    /// Author name.
    pub name: String,
    /// Author email (optional).
    #[serde(default)]
    pub email: Option<String>,
    /// Author URL (optional).
    #[serde(default)]
    pub url: Option<String>,
}

/// Legacy plugin metadata (for internal compatibility).
/// This wraps the new manifest format for code that expects the old structure.
#[derive(Debug, Clone)]
pub struct PluginMeta {
    /// Plugin name (unique identifier).
    pub name: String,
    /// Semantic version.
    pub version: String,
    /// Human-readable description.
    pub description: String,
}

impl From<&PluginManifest> for PluginMeta {
    fn from(manifest: &PluginManifest) -> Self {
        Self {
            name: manifest.name.clone(),
            version: manifest
                .version
                .clone()
                .unwrap_or_else(|| "0.0.0".to_string()),
            description: manifest.description.clone().unwrap_or_default(),
        }
    }
}

impl PluginManifest {
    /// Parse a manifest from a JSON string.
    pub fn from_json(json_str: &str) -> Result<Self> {
        let manifest: Self =
            serde_json::from_str(json_str).map_err(|e| PluginError::ManifestParse {
                reason: e.to_string(),
            })?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Parse a manifest from a file on disk.
    ///
    /// The file should be at `.claude-plugin/plugin.json` relative to the plugin root.
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let manifest = Self::from_json(&content)?;
        Ok(manifest)
    }

    /// Validate required fields and constraints.
    ///
    /// This performs comprehensive validation including:
    /// - Name format (kebab-case, starts with letter, etc.)
    /// - Version format (semver if provided)
    ///
    /// For path validation (checking declared paths exist), use
    /// [`validate_paths`] after loading from disk.
    pub fn validate(&self) -> Result<()> {
        // Validate name format
        validation::validate_name(&self.name).map_err(|e| PluginError::Validation {
            field: e.field_name().unwrap_or("unknown").to_string(),
            message: e.to_string(),
        })?;

        // Validate version format if provided
        if let Some(ref version) = self.version {
            validation::validate_version(version).map_err(|e| PluginError::Validation {
                field: "version".to_string(),
                message: e.to_string(),
            })?;
        }

        Ok(())
    }

    /// Validate that declared paths exist on disk.
    ///
    /// This should be called after loading a manifest from a specific plugin directory.
    /// It verifies that all declared component paths (skills, agents, hooks, etc.)
    /// actually exist.
    ///
    /// # Arguments
    ///
    /// * `plugin_dir` - The root directory of the plugin (containing `.claude-plugin/`)
    ///
    /// # Returns
    ///
    /// A list of validation errors for any missing paths.
    pub fn validate_paths(&self, plugin_dir: &Path) -> Vec<ManifestValidationError> {
        let mut errors = Vec::new();

        // Check skills paths
        if let Some(ref skills) = self.skills {
            if let Err(e) = validation::validate_paths_exist("skills", &skills.to_vec(), plugin_dir)
            {
                errors.push(e);
            }
        }

        // Check agents paths
        if let Some(ref agents) = self.agents {
            if let Err(e) = validation::validate_paths_exist("agents", &agents.to_vec(), plugin_dir)
            {
                errors.push(e);
            }
        }

        // Check hooks paths
        if let Some(ref hooks) = self.hooks {
            if let Err(e) = validation::validate_paths_exist("hooks", &hooks.to_vec(), plugin_dir) {
                errors.push(e);
            }
        }

        // Check commands paths
        if let Some(ref commands) = self.commands {
            if let Err(e) =
                validation::validate_paths_exist("commands", &commands.to_vec(), plugin_dir)
            {
                errors.push(e);
            }
        }

        errors
    }

    /// Get a summary of declared vs discovered capabilities.
    ///
    /// This helps identify mismatches between what a manifest declares
    /// and what's actually available in the plugin directory.
    pub fn capability_summary(&self, plugin_dir: &Path) -> CapabilitySummary {
        let skills_paths = self.skills_paths(plugin_dir);
        let agents_paths = self.agents_paths(plugin_dir);
        let hooks_paths = self.hooks_paths(plugin_dir);
        let commands_paths = self.commands_paths(plugin_dir);

        CapabilitySummary {
            skills_declared: self.skills.is_some(),
            skills_found: validation::count_discovered_items(&skills_paths, plugin_dir, "dir"),
            agents_declared: self.agents.is_some(),
            agents_found: validation::count_discovered_items(&agents_paths, plugin_dir, "md"),
            hooks_declared: self.hooks.is_some(),
            hooks_found: if hooks_paths.iter().any(|p| p.exists()) {
                1
            } else {
                0
            },
            commands_declared: self.commands.is_some(),
            commands_found: validation::count_discovered_items(&commands_paths, plugin_dir, "dir"),
        }
    }

    /// Get the skills directory paths resolved against a base directory.
    pub fn skills_paths(&self, plugin_dir: &Path) -> Vec<PathBuf> {
        self.skills
            .as_ref()
            .map(|p| p.resolve(plugin_dir))
            .unwrap_or_default()
    }

    /// Get the agents directory paths resolved against a base directory.
    pub fn agents_paths(&self, plugin_dir: &Path) -> Vec<PathBuf> {
        self.agents
            .as_ref()
            .map(|p| p.resolve(plugin_dir))
            .unwrap_or_default()
    }

    /// Get the hooks config paths resolved against a base directory.
    pub fn hooks_paths(&self, plugin_dir: &Path) -> Vec<PathBuf> {
        self.hooks
            .as_ref()
            .map(|p| p.resolve(plugin_dir))
            .unwrap_or_default()
    }

    /// Get the commands paths resolved against a base directory.
    pub fn commands_paths(&self, plugin_dir: &Path) -> Vec<PathBuf> {
        self.commands
            .as_ref()
            .map(|p| p.resolve(plugin_dir))
            .unwrap_or_default()
    }

    /// Get plugin metadata in the legacy format.
    pub fn plugin_meta(&self) -> PluginMeta {
        PluginMeta::from(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_manifest_json() -> &'static str {
        r#"{
  "name": "journal",
  "version": "0.1.0",
  "description": "Notes and journal plugin",
  "author": {
    "name": "Test Author",
    "email": "test@example.com"
  },
  "homepage": "https://example.com/journal",
  "repository": "https://github.com/example/journal-plugin",
  "license": "MIT",
  "keywords": ["notes", "journaling"],
  "skills": "./skills/",
  "agents": "./agents/",
  "hooks": "./hooks/hooks.json"
}"#
    }

    #[test]
    fn test_parse_full_manifest() {
        let manifest = PluginManifest::from_json(sample_manifest_json()).unwrap();

        assert_eq!(manifest.name, "journal");
        assert_eq!(manifest.version.as_deref(), Some("0.1.0"));
        assert_eq!(
            manifest.description.as_deref(),
            Some("Notes and journal plugin")
        );

        let author = manifest.author.as_ref().unwrap();
        assert_eq!(author.name, "Test Author");
        assert_eq!(author.email.as_deref(), Some("test@example.com"));

        assert_eq!(
            manifest.homepage.as_deref(),
            Some("https://example.com/journal")
        );
        assert_eq!(manifest.license.as_deref(), Some("MIT"));
        assert_eq!(manifest.keywords, vec!["notes", "journaling"]);
    }

    #[test]
    fn test_minimal_manifest() {
        let json = r#"{ "name": "minimal" }"#;
        let manifest = PluginManifest::from_json(json).unwrap();
        assert_eq!(manifest.name, "minimal");
        assert!(manifest.version.is_none());
        assert!(manifest.skills.is_none());
        assert!(manifest.agents.is_none());
        assert!(manifest.hooks.is_none());
    }

    #[test]
    fn test_empty_name_fails_validation() {
        let json = r#"{ "name": "" }"#;
        let err = PluginManifest::from_json(json).unwrap_err();
        assert!(matches!(err, PluginError::Validation { ref field, .. } if field == "name"));
    }

    #[test]
    fn test_non_kebab_name_fails_validation() {
        let json = r#"{ "name": "My Plugin" }"#;
        let err = PluginManifest::from_json(json).unwrap_err();
        assert!(matches!(err, PluginError::Validation { ref field, .. } if field == "name"));
    }

    #[test]
    fn test_uppercase_name_fails_validation() {
        let json = r#"{ "name": "MyPlugin" }"#;
        let err = PluginManifest::from_json(json).unwrap_err();
        assert!(matches!(err, PluginError::Validation { ref field, .. } if field == "name"));
    }

    #[test]
    fn test_path_or_paths_single() {
        let json = r#"{ "name": "test", "skills": "./skills/" }"#;
        let manifest = PluginManifest::from_json(json).unwrap();

        let paths = manifest.skills_paths(Path::new("/plugin"));
        assert_eq!(paths, vec![PathBuf::from("/plugin/skills/")]);
    }

    #[test]
    fn test_path_or_paths_multiple() {
        let json = r#"{ "name": "test", "skills": ["./skills/", "./extra-skills/"] }"#;
        let manifest = PluginManifest::from_json(json).unwrap();

        let paths = manifest.skills_paths(Path::new("/plugin"));
        assert_eq!(
            paths,
            vec![
                PathBuf::from("/plugin/skills/"),
                PathBuf::from("/plugin/extra-skills/")
            ]
        );
    }

    #[test]
    fn test_agents_paths() {
        let json = r#"{ "name": "test", "agents": "./agents/" }"#;
        let manifest = PluginManifest::from_json(json).unwrap();

        let paths = manifest.agents_paths(Path::new("/home/user/plugin"));
        assert_eq!(paths, vec![PathBuf::from("/home/user/plugin/agents/")]);
    }

    #[test]
    fn test_hooks_paths() {
        let json = r#"{ "name": "test", "hooks": "./hooks/hooks.json" }"#;
        let manifest = PluginManifest::from_json(json).unwrap();

        let paths = manifest.hooks_paths(Path::new("/plugin"));
        assert_eq!(paths, vec![PathBuf::from("/plugin/hooks/hooks.json")]);
    }

    #[test]
    fn test_plugin_meta_conversion() {
        let manifest = PluginManifest::from_json(sample_manifest_json()).unwrap();
        let meta = manifest.plugin_meta();

        assert_eq!(meta.name, "journal");
        assert_eq!(meta.version, "0.1.0");
        assert_eq!(meta.description, "Notes and journal plugin");
    }

    #[test]
    fn test_plugin_meta_defaults() {
        let json = r#"{ "name": "minimal" }"#;
        let manifest = PluginManifest::from_json(json).unwrap();
        let meta = manifest.plugin_meta();

        assert_eq!(meta.name, "minimal");
        assert_eq!(meta.version, "0.0.0"); // default
        assert_eq!(meta.description, ""); // default
    }

    #[test]
    fn test_roundtrip_serialize() {
        let manifest = PluginManifest::from_json(sample_manifest_json()).unwrap();
        let serialized = serde_json::to_string(&manifest).unwrap();
        let reparsed = PluginManifest::from_json(&serialized).unwrap();
        assert_eq!(reparsed.name, manifest.name);
        assert_eq!(reparsed.version, manifest.version);
    }

    #[test]
    fn test_from_file() {
        let dir = tempfile::tempdir().unwrap();
        let manifest_path = dir.path().join("plugin.json");
        std::fs::write(&manifest_path, sample_manifest_json()).unwrap();

        let manifest = PluginManifest::from_file(&manifest_path).unwrap();
        assert_eq!(manifest.name, "journal");
    }

    #[test]
    fn test_invalid_json() {
        let err = PluginManifest::from_json("not valid json {{{}}}").unwrap_err();
        assert!(matches!(err, PluginError::ManifestParse { .. }));
    }

    #[test]
    fn test_valid_version() {
        let json = r#"{ "name": "test", "version": "1.0.0" }"#;
        let manifest = PluginManifest::from_json(json).unwrap();
        assert_eq!(manifest.version.as_deref(), Some("1.0.0"));
    }

    #[test]
    fn test_valid_version_with_prerelease() {
        let json = r#"{ "name": "test", "version": "1.0.0-alpha" }"#;
        let manifest = PluginManifest::from_json(json).unwrap();
        assert_eq!(manifest.version.as_deref(), Some("1.0.0-alpha"));
    }

    #[test]
    fn test_valid_version_two_parts() {
        let json = r#"{ "name": "test", "version": "1.0" }"#;
        let manifest = PluginManifest::from_json(json).unwrap();
        assert_eq!(manifest.version.as_deref(), Some("1.0"));
    }

    #[test]
    fn test_invalid_version_single_number() {
        let json = r#"{ "name": "test", "version": "1" }"#;
        let err = PluginManifest::from_json(json).unwrap_err();
        assert!(matches!(err, PluginError::Validation { ref field, .. } if field == "version"));
    }

    #[test]
    fn test_invalid_version_non_numeric() {
        let json = r#"{ "name": "test", "version": "1.x.0" }"#;
        let err = PluginManifest::from_json(json).unwrap_err();
        assert!(matches!(err, PluginError::Validation { ref field, .. } if field == "version"));
    }

    #[test]
    fn test_invalid_version_leading_zero() {
        let json = r#"{ "name": "test", "version": "01.0.0" }"#;
        let err = PluginManifest::from_json(json).unwrap_err();
        assert!(matches!(err, PluginError::Validation { ref field, .. } if field == "version"));
    }

    #[test]
    fn test_name_starts_with_hyphen_fails() {
        let json = r#"{ "name": "-plugin" }"#;
        let err = PluginManifest::from_json(json).unwrap_err();
        assert!(matches!(err, PluginError::Validation { ref field, .. } if field == "name"));
    }

    #[test]
    fn test_name_ends_with_hyphen_fails() {
        let json = r#"{ "name": "plugin-" }"#;
        let err = PluginManifest::from_json(json).unwrap_err();
        assert!(matches!(err, PluginError::Validation { ref field, .. } if field == "name"));
    }

    #[test]
    fn test_name_consecutive_hyphens_fails() {
        let json = r#"{ "name": "my--plugin" }"#;
        let err = PluginManifest::from_json(json).unwrap_err();
        assert!(matches!(err, PluginError::Validation { ref field, .. } if field == "name"));
    }

    #[test]
    fn test_name_starts_with_number_fails() {
        let json = r#"{ "name": "123plugin" }"#;
        let err = PluginManifest::from_json(json).unwrap_err();
        assert!(matches!(err, PluginError::Validation { ref field, .. } if field == "name"));
    }

    #[test]
    fn test_capability_summary_empty() {
        let json = r#"{ "name": "test" }"#;
        let manifest = PluginManifest::from_json(json).unwrap();
        let summary = manifest.capability_summary(Path::new("/nonexistent"));
        assert!(!summary.skills_declared);
        assert!(!summary.agents_declared);
        assert!(!summary.has_errors());
    }

    #[test]
    fn test_capability_summary_declared_but_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let json = r#"{ "name": "test", "skills": "./skills/" }"#;
        let manifest = PluginManifest::from_json(json).unwrap();
        let summary = manifest.capability_summary(dir.path());
        assert!(summary.skills_declared);
        assert_eq!(summary.skills_found, 0);
        assert!(summary.has_errors());
        assert_eq!(summary.errors().len(), 1);
    }

    #[test]
    fn test_validate_paths_missing() {
        let dir = tempfile::tempdir().unwrap();
        let json = r#"{ "name": "test", "skills": "./skills/", "agents": "./agents/" }"#;
        let manifest = PluginManifest::from_json(json).unwrap();
        let errors = manifest.validate_paths(dir.path());
        assert_eq!(errors.len(), 2);
    }

    #[test]
    fn test_validate_paths_exists() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("skills")).unwrap();
        let json = r#"{ "name": "test", "skills": "./skills/" }"#;
        let manifest = PluginManifest::from_json(json).unwrap();
        let errors = manifest.validate_paths(dir.path());
        assert!(errors.is_empty());
    }

    #[test]
    fn test_mcp_servers_inline() {
        let json = r#"{
            "name": "test",
            "mcpServers": {
                "my-server": {
                    "command": "${CLAUDE_PLUGIN_ROOT}/server"
                }
            }
        }"#;
        let manifest = PluginManifest::from_json(json).unwrap();
        assert!(manifest.mcp_servers.is_some());
    }

    #[test]
    fn test_mcp_servers_path() {
        let json = r#"{ "name": "test", "mcpServers": "./.mcp.json" }"#;
        let manifest = PluginManifest::from_json(json).unwrap();
        assert!(manifest.mcp_servers.is_some());
    }
}
