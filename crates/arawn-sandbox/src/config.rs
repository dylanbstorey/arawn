//! Sandbox configuration.

use std::path::PathBuf;
use std::time::Duration;

/// Configuration for sandbox execution.
///
/// # Security Model
///
/// - **Write paths**: Explicitly allowed directories for write access.
///   Everything else is read-only.
/// - **Deny read paths**: Sensitive paths that should be blocked even for reading.
/// - **Network**: Domain-based allowlist for network access.
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    /// Paths allowed for writing (allow-only model).
    pub write_paths: Vec<PathBuf>,

    /// Paths denied for reading (deny-only model).
    pub deny_read_paths: Vec<PathBuf>,

    /// Allowed network domains (empty = no network).
    pub allowed_domains: Vec<String>,

    /// Working directory for command execution.
    pub working_dir: Option<PathBuf>,

    /// Command timeout.
    pub timeout: Duration,

    /// Environment variables to pass to the command.
    pub env_vars: Vec<(String, String)>,

    /// Whether to allow access to .git directories.
    pub allow_git: bool,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            write_paths: Vec::new(),
            deny_read_paths: Self::default_deny_read_paths(),
            allowed_domains: Vec::new(),
            working_dir: None,
            timeout: Duration::from_secs(30),
            env_vars: Vec::new(),
            allow_git: false,
        }
    }
}

impl SandboxConfig {
    /// Create a new sandbox configuration with defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set paths allowed for writing.
    pub fn with_write_paths(mut self, paths: Vec<PathBuf>) -> Self {
        self.write_paths = paths;
        self
    }

    /// Add a single write path.
    pub fn add_write_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.write_paths.push(path.into());
        self
    }

    /// Set paths denied for reading.
    pub fn with_deny_read_paths(mut self, paths: Vec<PathBuf>) -> Self {
        self.deny_read_paths = paths;
        self
    }

    /// Add a path to deny for reading.
    pub fn add_deny_read_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.deny_read_paths.push(path.into());
        self
    }

    /// Set allowed network domains.
    pub fn with_allowed_domains(mut self, domains: Vec<String>) -> Self {
        self.allowed_domains = domains;
        self
    }

    /// Add an allowed network domain.
    pub fn add_allowed_domain(mut self, domain: impl Into<String>) -> Self {
        self.allowed_domains.push(domain.into());
        self
    }

    /// Set the working directory.
    pub fn with_working_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }

    /// Set the command timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Add an environment variable.
    pub fn add_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env_vars.push((key.into(), value.into()));
        self
    }

    /// Allow access to .git directories.
    pub fn with_git_access(mut self, allow: bool) -> Self {
        self.allow_git = allow;
        self
    }

    /// Get the default paths to deny for reading.
    ///
    /// These are sensitive system and user paths that should not be
    /// accessible to sandboxed commands.
    pub fn default_deny_read_paths() -> Vec<PathBuf> {
        let mut paths = vec![
            PathBuf::from("/etc/shadow"),
            PathBuf::from("/etc/sudoers"),
            PathBuf::from("/etc/sudoers.d"),
        ];

        // Add home directory sensitive paths
        if let Some(home) = dirs::home_dir() {
            paths.push(home.join(".ssh"));
            paths.push(home.join(".gnupg"));
            paths.push(home.join(".aws"));
            paths.push(home.join(".config/gcloud"));
            paths.push(home.join(".kube"));
            paths.push(home.join(".docker/config.json"));
            paths.push(home.join(".npmrc"));
            paths.push(home.join(".netrc"));
            paths.push(home.join(".gitconfig"));
            paths.push(home.join(".bash_history"));
            paths.push(home.join(".zsh_history"));
            // Arawn config directory (credentials, keys)
            paths.push(home.join(".arawn/config"));
        }

        paths
    }

    /// Create a config for a workstream session.
    ///
    /// Sets up write access to the workstream's production and work directories.
    pub fn for_workstream(
        workstream_production: PathBuf,
        workstream_work: PathBuf,
    ) -> Self {
        Self::default()
            .with_write_paths(vec![workstream_production, workstream_work])
    }

    /// Create a config for a scratch session.
    ///
    /// Sets up write access to the session's isolated work directory.
    pub fn for_scratch_session(session_work: PathBuf) -> Self {
        Self::default()
            .with_write_paths(vec![session_work])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SandboxConfig::default();
        assert!(config.write_paths.is_empty());
        assert!(!config.deny_read_paths.is_empty());
        assert!(config.allowed_domains.is_empty());
        assert!(config.working_dir.is_none());
        assert_eq!(config.timeout, Duration::from_secs(30));
    }

    #[test]
    fn test_builder_pattern() {
        let config = SandboxConfig::new()
            .with_write_paths(vec![PathBuf::from("/tmp")])
            .with_timeout(Duration::from_secs(60))
            .add_allowed_domain("github.com")
            .with_working_dir("/home/user/project");

        assert_eq!(config.write_paths.len(), 1);
        assert_eq!(config.timeout, Duration::from_secs(60));
        assert_eq!(config.allowed_domains, vec!["github.com"]);
        assert!(config.working_dir.is_some());
    }

    #[test]
    fn test_default_deny_paths() {
        let paths = SandboxConfig::default_deny_read_paths();

        // Should include shadow file
        assert!(paths.iter().any(|p| p.ends_with("shadow")));

        // Should include SSH directory if home exists
        if dirs::home_dir().is_some() {
            assert!(paths.iter().any(|p| p.to_string_lossy().contains(".ssh")));
            assert!(paths.iter().any(|p| p.to_string_lossy().contains(".aws")));
        }
    }

    #[test]
    fn test_workstream_config() {
        let config = SandboxConfig::for_workstream(
            PathBuf::from("/data/ws/production"),
            PathBuf::from("/data/ws/work"),
        );

        assert_eq!(config.write_paths.len(), 2);
        assert!(config.write_paths.contains(&PathBuf::from("/data/ws/production")));
        assert!(config.write_paths.contains(&PathBuf::from("/data/ws/work")));
    }

    #[test]
    fn test_scratch_config() {
        let config = SandboxConfig::for_scratch_session(
            PathBuf::from("/data/scratch/session-123/work"),
        );

        assert_eq!(config.write_paths.len(), 1);
        assert!(config.write_paths.contains(&PathBuf::from("/data/scratch/session-123/work")));
    }
}
