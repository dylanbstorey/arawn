//! Config file discovery and layered merging.
//!
//! Resolution order (later overrides earlier):
//! 1. `~/.config/arawn/config.toml` (XDG user config)
//! 2. `./arawn.toml` (project-local)
//! 3. CLI arguments (handled externally)

use std::path::{Path, PathBuf};

use crate::{ArawnConfig, ConfigError, Result};

/// Default config filename for project-local config.
const PROJECT_CONFIG_FILE: &str = "arawn.toml";

/// Default config filename within XDG config directory.
const USER_CONFIG_FILE: &str = "config.toml";

/// Application name for XDG directory resolution.
const APP_NAME: &str = "arawn";

/// Tracks where each config layer was loaded from.
#[derive(Debug, Clone)]
pub struct ConfigSource {
    /// Path to the config file.
    pub path: PathBuf,
    /// Whether the file was found and loaded.
    pub loaded: bool,
}

/// Result of config discovery and loading.
#[derive(Debug, Clone)]
pub struct LoadedConfig {
    /// The merged configuration.
    pub config: ArawnConfig,
    /// Sources that were checked, in order of precedence (lowest first).
    pub sources: Vec<ConfigSource>,
    /// Primary source file (first successfully loaded), for save operations.
    pub source: Option<ConfigSource>,
    /// Warnings generated during loading (e.g., plaintext API keys).
    pub warnings: Vec<String>,
}

impl LoadedConfig {
    /// Get paths of sources that were actually loaded.
    pub fn loaded_from(&self) -> Vec<&Path> {
        self.sources
            .iter()
            .filter(|s| s.loaded)
            .map(|s| s.path.as_path())
            .collect()
    }
}

/// Load configuration by discovering and merging all config layers.
///
/// Searches for config files in order:
/// 1. User config dir (from `config_dir`, `ARAWN_CONFIG_DIR` env, or platform default)
/// 2. Project-local (`./arawn.toml` or specified project dir)
///
/// Later files override earlier ones.
pub fn load_config(project_dir: Option<&Path>) -> Result<LoadedConfig> {
    load_config_with_options(project_dir, None)
}

/// Load configuration with explicit control over the user config directory.
///
/// `config_dir` overrides both `ARAWN_CONFIG_DIR` and the platform default.
/// Pass `Some(path)` to use a specific directory, or `None` to use the default resolution.
pub fn load_config_with_options(
    project_dir: Option<&Path>,
    config_dir: Option<&Path>,
) -> Result<LoadedConfig> {
    let mut config = ArawnConfig::new();
    let mut sources = Vec::new();
    let mut warnings = Vec::new();

    // 1. User config — explicit override, then env var, then platform default
    let user_config_path = match config_dir {
        Some(dir) => Some(dir.join(USER_CONFIG_FILE)),
        None => xdg_config_path(),
    };
    if let Some(path) = user_config_path {
        let source = load_layer(&mut config, &path, &mut warnings)?;
        sources.push(source);
    }

    // 2. Project-local config
    let project_path = project_dir
        .map(|d| d.join(PROJECT_CONFIG_FILE))
        .unwrap_or_else(|| PathBuf::from(PROJECT_CONFIG_FILE));
    let source = load_layer(&mut config, &project_path, &mut warnings)?;
    sources.push(source);

    // Check for plaintext API keys
    check_plaintext_keys(&config, &mut warnings);

    // Find primary source (first successfully loaded file)
    let source = sources.iter().find(|s| s.loaded).cloned();

    Ok(LoadedConfig {
        config,
        sources,
        source,
        warnings,
    })
}

/// Load config from a specific file path (no discovery).
pub fn load_config_file(path: &Path) -> Result<ArawnConfig> {
    let contents = std::fs::read_to_string(path).map_err(|e| ConfigError::ReadFile {
        path: path.display().to_string(),
        source: e,
    })?;
    ArawnConfig::from_toml(&contents)
}

/// Save configuration to a file.
///
/// Creates parent directories if they don't exist.
pub fn save_config(config: &ArawnConfig, path: &Path) -> Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| ConfigError::WriteFile {
            path: parent.display().to_string(),
            source: e,
        })?;
    }

    let contents = config.to_toml()?;
    std::fs::write(path, contents).map_err(|e| ConfigError::WriteFile {
        path: path.display().to_string(),
        source: e,
    })?;

    Ok(())
}

/// Environment variable to override the config directory.
///
/// When set, this takes precedence over the platform default (XDG/Application Support).
/// Useful for testing and running multiple instances with different configs.
const CONFIG_DIR_ENV: &str = "ARAWN_CONFIG_DIR";

/// Get the XDG config directory path for arawn.
///
/// Checks `ARAWN_CONFIG_DIR` env var first, then falls back to platform default
/// (`~/.config/arawn/config.toml` on Linux, `~/Library/Application Support/arawn/config.toml` on macOS).
pub fn xdg_config_path() -> Option<PathBuf> {
    xdg_config_dir().map(|d| d.join(USER_CONFIG_FILE))
}

/// Get the XDG config directory for arawn.
///
/// Checks `ARAWN_CONFIG_DIR` env var first, then falls back to platform default.
pub fn xdg_config_dir() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var(CONFIG_DIR_ENV)
        && !dir.is_empty()
    {
        return Some(PathBuf::from(dir));
    }
    dirs::config_dir().map(|d| d.join(APP_NAME))
}

/// Try to load a config file and merge it into the existing config.
fn load_layer(
    config: &mut ArawnConfig,
    path: &Path,
    warnings: &mut Vec<String>,
) -> Result<ConfigSource> {
    if !path.is_file() {
        return Ok(ConfigSource {
            path: path.to_path_buf(),
            loaded: false,
        });
    }

    match load_config_file(path) {
        Ok(layer) => {
            config.merge(layer);
            Ok(ConfigSource {
                path: path.to_path_buf(),
                loaded: true,
            })
        }
        Err(e) => {
            warnings.push(format!("Failed to load {}: {}", path.display(), e));
            Ok(ConfigSource {
                path: path.to_path_buf(),
                loaded: false,
            })
        }
    }
}

/// Check for plaintext API keys in the config and emit warnings.
fn check_plaintext_keys(config: &ArawnConfig, warnings: &mut Vec<String>) {
    if let Some(ref llm) = config.llm
        && llm.has_plaintext_api_key()
    {
        warnings.push(
            "Default [llm] config contains a plaintext API key. \
                 Consider using the system keyring (arawn config set-secret) \
                 or an environment variable instead."
                .to_string(),
        );
    }

    for (name, llm) in &config.llm_profiles {
        if llm.has_plaintext_api_key() {
            warnings.push(format!(
                "[llm.{}] contains a plaintext API key. \
                 Consider using the system keyring or an environment variable instead.",
                name
            ));
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    use crate::Backend;

    #[test]
    fn test_xdg_config_path_exists() {
        // Just verify it returns something on all platforms
        let path = xdg_config_path();
        // May be None in some CI environments, but should work on macOS/Linux
        if let Some(p) = path {
            assert!(p.ends_with("arawn/config.toml"));
        }
    }

    #[test]
    fn test_load_config_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.toml");
        fs::write(
            &path,
            r#"
[llm]
backend = "groq"
model = "test-model"
"#,
        )
        .unwrap();

        let config = load_config_file(&path).unwrap();
        assert_eq!(config.llm.as_ref().unwrap().backend, Some(Backend::Groq));
    }

    #[test]
    fn test_load_config_file_not_found() {
        let err = load_config_file(Path::new("/nonexistent/config.toml")).unwrap_err();
        assert!(matches!(err, ConfigError::ReadFile { .. }));
    }

    #[test]
    fn test_load_config_invalid_toml() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.toml");
        fs::write(&path, "this is not valid toml {{{{").unwrap();

        let err = load_config_file(&path).unwrap_err();
        assert!(matches!(err, ConfigError::Parse(_)));
    }

    #[test]
    fn test_load_config_project_only() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join("arawn.toml");
        fs::write(
            &config_path,
            r#"
[llm]
backend = "anthropic"
model = "claude-model"

[server]
port = 9090
"#,
        )
        .unwrap();

        let loaded = load_config(Some(dir.path())).unwrap();
        let config = &loaded.config;

        assert_eq!(
            config.llm.as_ref().unwrap().backend,
            Some(Backend::Anthropic)
        );
        assert_eq!(config.server.as_ref().unwrap().port, 9090);

        // At least one source should be loaded
        assert!(loaded.loaded_from().len() >= 1);
    }

    #[test]
    fn test_load_config_no_files() {
        let dir = TempDir::new().unwrap();
        let empty_config_dir = TempDir::new().unwrap();
        // Use explicit empty config dir so we don't pick up the real user config
        let loaded =
            load_config_with_options(Some(dir.path()), Some(empty_config_dir.path())).unwrap();
        assert!(loaded.config.llm.is_none());
        assert!(loaded.loaded_from().is_empty());
    }

    #[test]
    fn test_load_config_layered_merge() {
        let xdg_dir = TempDir::new().unwrap();
        let project_dir = TempDir::new().unwrap();

        // Simulate XDG config
        let xdg_config = xdg_dir.path().join("config.toml");
        fs::write(
            &xdg_config,
            r#"
[llm]
backend = "groq"
model = "base-model"

[llm.claude]
backend = "anthropic"
model = "claude-base"

[server]
port = 8080
"#,
        )
        .unwrap();

        // Simulate project-local config
        let project_config = project_dir.path().join("arawn.toml");
        fs::write(
            &project_config,
            r#"
[llm]
backend = "anthropic"
model = "project-model"

[server]
port = 3000
"#,
        )
        .unwrap();

        // Load XDG manually, then project on top
        let mut config = ArawnConfig::new();
        let mut warnings = Vec::new();
        load_layer(&mut config, &xdg_config, &mut warnings).unwrap();
        load_layer(&mut config, &project_config, &mut warnings).unwrap();

        // Project-local overrides XDG
        let llm = config.llm.as_ref().unwrap();
        assert_eq!(llm.backend, Some(Backend::Anthropic));
        assert_eq!(llm.model.as_deref(), Some("project-model"));
        assert_eq!(config.server.as_ref().unwrap().port, 3000);

        // XDG profiles preserved (project didn't override them)
        assert!(config.llm_profiles.contains_key("claude"));
    }

    #[test]
    fn test_plaintext_key_warning() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("arawn.toml");
        fs::write(
            &path,
            r#"
[llm]
backend = "groq"
model = "model"
api_key = "gsk_secret"

[llm.claude]
backend = "anthropic"
model = "claude"
api_key = "sk-ant-secret"
"#,
        )
        .unwrap();

        let loaded = load_config(Some(dir.path())).unwrap();
        assert_eq!(loaded.warnings.len(), 2);
        assert!(loaded.warnings[0].contains("plaintext"));
        assert!(loaded.warnings[1].contains("[llm.claude]"));
    }

    #[test]
    fn test_no_warnings_without_keys() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("arawn.toml");
        fs::write(
            &path,
            r#"
[llm]
backend = "groq"
model = "model"
"#,
        )
        .unwrap();

        let loaded = load_config(Some(dir.path())).unwrap();
        assert!(loaded.warnings.is_empty());
    }

    #[test]
    fn test_malformed_config_warns_but_continues() {
        let dir = TempDir::new().unwrap();
        let bad_file = dir.path().join("arawn.toml");
        fs::write(&bad_file, "not valid toml {{{{").unwrap();

        let loaded = load_config(Some(dir.path())).unwrap();
        // Should have a warning but not fail
        assert!(!loaded.warnings.is_empty());
        assert!(loaded.warnings[0].contains("Failed to load"));
    }

    #[test]
    fn test_loaded_from_tracks_sources() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("arawn.toml");
        fs::write(
            &path,
            r#"
[llm]
backend = "groq"
model = "model"
"#,
        )
        .unwrap();

        let loaded = load_config(Some(dir.path())).unwrap();
        let loaded_paths = loaded.loaded_from();
        assert!(loaded_paths.iter().any(|p| p.ends_with("arawn.toml")));
    }
}
