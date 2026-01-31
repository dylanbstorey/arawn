//! Plugin discovery, loading, and management.
//!
//! `PluginManager` scans configured directories for plugins (directories
//! containing `.claude-plugin/plugin.json`), loads their manifests, and reads all
//! component files (skills, agent configs) from disk.

use crate::manifest::{PluginManifest, PluginMeta};
use crate::types::{HooksConfig, HooksConfigExt, PluginAgentConfig, PluginAgentDef, SkillDef};
use crate::{PluginError, Result};
use std::path::{Path, PathBuf};

/// The path to the plugin manifest relative to the plugin root.
pub const MANIFEST_PATH: &str = ".claude-plugin/plugin.json";

/// A fully loaded plugin with all component content read from disk.
#[derive(Debug, Clone)]
pub struct LoadedPlugin {
    /// The parsed manifest.
    pub manifest: PluginManifest,
    /// Directory the plugin was loaded from (the root, not .claude-plugin/).
    pub plugin_dir: PathBuf,
    /// Loaded skill content (discovered from skills/<name>/SKILL.md).
    pub skill_contents: Vec<LoadedSkill>,
    /// Loaded agent configs (discovered from agents/<name>.md).
    pub agent_configs: Vec<LoadedAgent>,
    /// Loaded hooks configuration (from hooks/hooks.json or path in manifest).
    pub hooks_config: Option<HooksConfig>,
}

impl LoadedPlugin {
    /// Get the plugin metadata (name, version, description).
    pub fn meta(&self) -> PluginMeta {
        self.manifest.plugin_meta()
    }
}

/// A skill with its markdown content loaded from disk.
#[derive(Debug, Clone)]
pub struct LoadedSkill {
    /// The skill definition from the manifest.
    pub def: SkillDef,
    /// Raw markdown content of the skill file.
    pub content: String,
}

/// An agent with its config loaded from disk.
#[derive(Debug, Clone)]
pub struct LoadedAgent {
    /// The agent definition from the manifest.
    pub def: PluginAgentDef,
    /// Parsed agent configuration.
    pub config: PluginAgentConfig,
}

/// Manages plugin discovery and loading.
#[derive(Debug)]
pub struct PluginManager {
    /// Directories to scan for plugins.
    plugin_dirs: Vec<PathBuf>,
}

impl PluginManager {
    /// Create a new `PluginManager` with the given plugin directories.
    pub fn new(plugin_dirs: Vec<PathBuf>) -> Self {
        Self { plugin_dirs }
    }

    /// Create a `PluginManager` with default directories.
    ///
    /// Scans `~/.config/arawn/plugins/` and `./plugins/`.
    pub fn with_defaults() -> Self {
        let mut dirs = Vec::new();

        // User-level plugin directory
        if let Some(config_dir) = dirs::config_dir() {
            dirs.push(config_dir.join("arawn").join("plugins"));
        }

        // Project-level plugin directory
        dirs.push(PathBuf::from("./plugins"));

        Self::new(dirs)
    }

    /// Get the configured plugin directories.
    pub fn plugin_dirs(&self) -> &[PathBuf] {
        &self.plugin_dirs
    }

    /// Discover and load all plugins from configured directories.
    ///
    /// Scans each directory for subdirectories containing `plugin.toml`.
    /// Invalid plugins are logged as warnings and skipped.
    pub fn load_all(&self) -> Vec<LoadedPlugin> {
        let mut loaded = Vec::new();

        for dir in &self.plugin_dirs {
            if !dir.exists() {
                tracing::debug!(dir = %dir.display(), "plugin directory does not exist, skipping");
                continue;
            }

            match self.scan_directory(dir) {
                Ok(plugins) => loaded.extend(plugins),
                Err(e) => {
                    tracing::warn!(
                        dir = %dir.display(),
                        error = %e,
                        "failed to scan plugin directory"
                    );
                }
            }
        }

        tracing::info!(
            count = loaded.len(),
            plugins = ?loaded.iter().map(|p| &p.manifest.name).collect::<Vec<_>>(),
            "loaded plugins"
        );

        loaded
    }

    /// Scan a single directory for plugin subdirectories.
    ///
    /// A plugin is identified by having a `.claude-plugin/plugin.json` file.
    fn scan_directory(&self, dir: &Path) -> Result<Vec<LoadedPlugin>> {
        let mut plugins = Vec::new();

        let entries = std::fs::read_dir(dir)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            let manifest_path = path.join(MANIFEST_PATH);
            if !manifest_path.exists() {
                continue;
            }

            match self.load_plugin(&path, &manifest_path) {
                Ok(plugin) => {
                    let meta = plugin.meta();
                    tracing::info!(
                        name = %meta.name,
                        version = %meta.version,
                        skills = plugin.skill_contents.len(),
                        agents = plugin.agent_configs.len(),
                        "loaded plugin"
                    );
                    plugins.push(plugin);
                }
                Err(e) => {
                    tracing::warn!(
                        dir = %path.display(),
                        error = %e,
                        "failed to load plugin, skipping"
                    );
                }
            }
        }

        Ok(plugins)
    }

    /// Load a single plugin from its directory.
    ///
    /// Discovers skills from `skills/<name>/SKILL.md`, agents from `agents/<name>.md`,
    /// and hooks from `hooks/hooks.json` (or path specified in manifest).
    fn load_plugin(&self, plugin_dir: &Path, manifest_path: &Path) -> Result<LoadedPlugin> {
        let manifest = PluginManifest::from_file(manifest_path)?;

        // Discover skills from skills directories
        let skill_contents = self.discover_skills(plugin_dir, &manifest);

        // Discover agents from agents directories
        let agent_configs = self.discover_agents(plugin_dir, &manifest);

        // Load hooks configuration
        let hooks_config = self.load_hooks(plugin_dir, &manifest);

        Ok(LoadedPlugin {
            manifest,
            plugin_dir: plugin_dir.to_path_buf(),
            skill_contents,
            agent_configs,
            hooks_config,
        })
    }

    /// Discover skills from the skills directories.
    ///
    /// Claude format: `skills/<name>/SKILL.md`
    fn discover_skills(&self, plugin_dir: &Path, manifest: &PluginManifest) -> Vec<LoadedSkill> {
        let mut skills = Vec::new();

        for skills_dir in manifest.skills_paths(plugin_dir) {
            if !skills_dir.exists() || !skills_dir.is_dir() {
                continue;
            }

            // Each subdirectory in skills/ is a skill
            let Ok(entries) = std::fs::read_dir(&skills_dir) else {
                continue;
            };

            for entry in entries.flatten() {
                let skill_dir = entry.path();
                if !skill_dir.is_dir() {
                    continue;
                }

                let skill_file = skill_dir.join("SKILL.md");
                if !skill_file.exists() {
                    continue;
                }

                let skill_name = skill_dir
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                match std::fs::read_to_string(&skill_file) {
                    Ok(content) => {
                        // Parse frontmatter to get description (basic parsing for now)
                        let description =
                            extract_frontmatter_field(&content, "description").unwrap_or_default();

                        skills.push(LoadedSkill {
                            def: SkillDef {
                                name: skill_name.clone(),
                                description,
                                file: skill_file,
                                uses_tools: vec![],
                            },
                            content,
                        });
                        tracing::debug!(skill = %skill_name, "discovered skill");
                    }
                    Err(e) => {
                        tracing::warn!(
                            skill = %skill_name,
                            file = %skill_file.display(),
                            error = %e,
                            "failed to load skill file, skipping skill"
                        );
                    }
                }
            }
        }

        skills
    }

    /// Discover agents from the agents directories.
    ///
    /// Claude format: `agents/<name>.md`
    fn discover_agents(&self, plugin_dir: &Path, manifest: &PluginManifest) -> Vec<LoadedAgent> {
        let mut agents = Vec::new();

        for agents_dir in manifest.agents_paths(plugin_dir) {
            if !agents_dir.exists() || !agents_dir.is_dir() {
                continue;
            }

            let Ok(entries) = std::fs::read_dir(&agents_dir) else {
                continue;
            };

            for entry in entries.flatten() {
                let agent_file = entry.path();
                if !agent_file.is_file() {
                    continue;
                }

                // Only process .md files
                if agent_file.extension().and_then(|e| e.to_str()) != Some("md") {
                    continue;
                }

                let agent_name = agent_file
                    .file_stem()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                match std::fs::read_to_string(&agent_file) {
                    Ok(content) => {
                        // Parse agent config from markdown frontmatter
                        match parse_agent_markdown(&agent_name, &content) {
                            Ok((def, config)) => {
                                agents.push(LoadedAgent { def, config });
                                tracing::debug!(agent = %agent_name, "discovered agent");
                            }
                            Err(e) => {
                                tracing::warn!(
                                    agent = %agent_name,
                                    error = %e,
                                    "failed to parse agent config, skipping agent"
                                );
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!(
                            agent = %agent_name,
                            file = %agent_file.display(),
                            error = %e,
                            "failed to load agent file, skipping agent"
                        );
                    }
                }
            }
        }

        agents
    }

    /// Load hooks configuration from hooks.json.
    ///
    /// Claude format: `hooks/hooks.json` or path specified in manifest `hooks` field.
    fn load_hooks(&self, plugin_dir: &Path, manifest: &PluginManifest) -> Option<HooksConfig> {
        // Get hooks paths from manifest, or default to hooks/hooks.json
        let hooks_paths = manifest.hooks_paths(plugin_dir);

        // If no hooks paths specified in manifest, try default location
        let paths_to_try = if hooks_paths.is_empty() {
            vec![plugin_dir.join("hooks/hooks.json")]
        } else {
            hooks_paths
        };

        for hooks_path in paths_to_try {
            if !hooks_path.exists() {
                continue;
            }

            match HooksConfig::from_file(&hooks_path) {
                Ok(config) => {
                    if !config.is_empty() {
                        tracing::debug!(
                            path = %hooks_path.display(),
                            "loaded hooks config"
                        );
                        return Some(config);
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        path = %hooks_path.display(),
                        error = %e,
                        "failed to parse hooks.json, skipping"
                    );
                }
            }
        }

        None
    }

    /// Load a single plugin by directory path (for hot-reload).
    pub fn load_single(&self, plugin_dir: &Path) -> Result<LoadedPlugin> {
        let manifest_path = plugin_dir.join(MANIFEST_PATH);
        if !manifest_path.exists() {
            return Err(PluginError::Validation {
                field: "plugin_dir".to_string(),
                message: format!("no {} found in {}", MANIFEST_PATH, plugin_dir.display()),
            });
        }
        self.load_plugin(plugin_dir, &manifest_path)
    }
}

/// Extract a field value from YAML frontmatter in a markdown file.
///
/// Expects frontmatter delimited by `---` at the start of the file.
fn extract_frontmatter_field(content: &str, field: &str) -> Option<String> {
    let content = content.trim_start();
    if !content.starts_with("---") {
        return None;
    }

    let rest = &content[3..];
    let end = rest.find("---")?;
    let frontmatter = &rest[..end];

    // Simple line-by-line parsing (not full YAML)
    for line in frontmatter.lines() {
        let line = line.trim();
        if let Some(value) = line.strip_prefix(&format!("{}:", field)) {
            return Some(
                value
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string(),
            );
        }
    }
    None
}

/// Parse an agent configuration from a Claude-format markdown file.
///
/// Format:
/// ```markdown
/// ---
/// description: Agent description
/// capabilities: ["task1", "task2"]
/// tools: ["shell", "file_read"]
/// ---
///
/// # Agent Name
///
/// System prompt / detailed instructions...
/// ```
fn parse_agent_markdown(name: &str, content: &str) -> Result<(PluginAgentDef, PluginAgentConfig)> {
    use crate::types::{AgentConstraints, AgentSection, AgentSystemPrompt};

    let description = extract_frontmatter_field(content, "description").unwrap_or_default();

    // Extract tools from frontmatter (simplified parsing)
    let tools: Vec<String> = extract_frontmatter_field(content, "tools")
        .map(|s| {
            // Parse JSON-like array: ["tool1", "tool2"]
            s.trim_matches(|c| c == '[' || c == ']')
                .split(',')
                .map(|t| t.trim().trim_matches('"').trim_matches('\'').to_string())
                .filter(|t| !t.is_empty())
                .collect()
        })
        .unwrap_or_default();

    // Extract body as system prompt (everything after frontmatter)
    let system_prompt = content
        .trim_start()
        .strip_prefix("---")
        .and_then(|rest| rest.find("---").map(|end| &rest[end + 3..]))
        .map(|body| body.trim().to_string())
        .filter(|s| !s.is_empty());

    let def = PluginAgentDef {
        name: name.to_string(),
        description: description.clone(),
        file: PathBuf::new(), // Not used in new format
        tools: tools.clone(),
    };

    let config = PluginAgentConfig {
        agent: AgentSection {
            name: name.to_string(),
            description,
            model: None,
            system_prompt: system_prompt.map(|text| AgentSystemPrompt { text }),
            constraints: if tools.is_empty() {
                None
            } else {
                Some(AgentConstraints {
                    tools,
                    max_iterations: None,
                })
            },
        },
    };

    Ok((def, config))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Create a minimal plugin directory structure for testing (Claude format).
    fn create_test_plugin(base_dir: &Path, name: &str) -> PathBuf {
        let plugin_dir = base_dir.join(name);
        fs::create_dir_all(plugin_dir.join(".claude-plugin")).unwrap();
        fs::create_dir_all(plugin_dir.join("skills").join(format!("{}-skill", name))).unwrap();
        fs::create_dir_all(plugin_dir.join("agents")).unwrap();
        fs::create_dir_all(plugin_dir.join("scripts")).unwrap();
        fs::create_dir_all(plugin_dir.join("hooks")).unwrap();

        // Write manifest (JSON format)
        fs::write(
            plugin_dir.join(".claude-plugin/plugin.json"),
            format!(
                r#"{{
  "name": "{name}",
  "version": "0.1.0",
  "description": "Test plugin: {name}",
  "skills": "./skills/",
  "agents": "./agents/",
  "hooks": "./hooks/hooks.json"
}}"#
            ),
        )
        .unwrap();

        // Write skill file (Claude format: skills/<name>/SKILL.md)
        fs::write(
            plugin_dir.join(format!("skills/{}-skill/SKILL.md", name)),
            format!(
                r#"---
name: {name}-skill
description: A test skill
---

# Test Skill

This is the {name} skill content.
"#
            ),
        )
        .unwrap();

        // Write agent config (Claude format: agents/<name>.md)
        fs::write(
            plugin_dir.join(format!("agents/{}-agent.md", name)),
            format!(
                r#"---
description: A test agent for {name}
tools: ["{name}-tool", "shell"]
---

# {name} Agent

You are the {name} agent. Help the user with {name}-related tasks.
"#
            ),
        )
        .unwrap();

        // Write hooks config (JSON format)
        fs::write(
            plugin_dir.join("hooks/hooks.json"),
            r#"{
  "hooks": {
    "SessionEnd": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "${CLAUDE_PLUGIN_ROOT}/scripts/on-end.sh"
          }
        ]
      }
    ]
  }
}"#,
        )
        .unwrap();

        // Write script
        fs::write(
            plugin_dir.join("scripts/on-end.sh"),
            "#!/bin/bash\necho '{}'",
        )
        .unwrap();

        plugin_dir
    }

    #[test]
    fn test_load_single_plugin() {
        let tmp = TempDir::new().unwrap();
        let plugin_dir = create_test_plugin(tmp.path(), "journal");

        let manager = PluginManager::new(vec![tmp.path().to_path_buf()]);
        let plugin = manager.load_single(&plugin_dir).unwrap();

        assert_eq!(plugin.manifest.name, "journal");
        assert_eq!(plugin.manifest.version.as_deref(), Some("0.1.0"));
        assert_eq!(plugin.skill_contents.len(), 1);
        assert!(plugin.skill_contents[0].content.contains("Test Skill"));
        assert_eq!(plugin.skill_contents[0].def.name, "journal-skill");
        assert_eq!(plugin.agent_configs.len(), 1);
        assert_eq!(plugin.agent_configs[0].config.agent.name, "journal-agent");

        // Verify hooks loaded
        assert!(plugin.hooks_config.is_some());
        let hooks = plugin.hooks_config.as_ref().unwrap();
        assert!(hooks.hooks.contains_key(&crate::HookEvent::SessionEnd));
    }

    #[test]
    fn test_load_all_discovers_multiple_plugins() {
        let tmp = TempDir::new().unwrap();
        create_test_plugin(tmp.path(), "alpha");
        create_test_plugin(tmp.path(), "beta");

        let manager = PluginManager::new(vec![tmp.path().to_path_buf()]);
        let plugins = manager.load_all();

        assert_eq!(plugins.len(), 2);
        let names: Vec<&str> = plugins.iter().map(|p| p.manifest.name.as_str()).collect();
        assert!(names.contains(&"alpha"));
        assert!(names.contains(&"beta"));
    }

    #[test]
    fn test_load_all_skips_nonexistent_dirs() {
        let manager = PluginManager::new(vec![PathBuf::from("/nonexistent/path/plugins")]);
        let plugins = manager.load_all();
        assert!(plugins.is_empty());
    }

    #[test]
    fn test_load_all_skips_invalid_plugins() {
        let tmp = TempDir::new().unwrap();

        // Valid plugin
        create_test_plugin(tmp.path(), "good");

        // Invalid plugin (bad JSON)
        let bad_dir = tmp.path().join("bad");
        fs::create_dir_all(bad_dir.join(".claude-plugin")).unwrap();
        fs::write(
            bad_dir.join(".claude-plugin/plugin.json"),
            "not valid json {{{}}}",
        )
        .unwrap();

        let manager = PluginManager::new(vec![tmp.path().to_path_buf()]);
        let plugins = manager.load_all();

        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].manifest.name, "good");
    }

    #[test]
    fn test_load_skips_missing_skill_dirs() {
        let tmp = TempDir::new().unwrap();
        let plugin_dir = tmp.path().join("test");
        fs::create_dir_all(plugin_dir.join(".claude-plugin")).unwrap();

        fs::write(
            plugin_dir.join(".claude-plugin/plugin.json"),
            r#"{ "name": "test", "skills": "./nonexistent-skills/" }"#,
        )
        .unwrap();

        let manager = PluginManager::new(vec![tmp.path().to_path_buf()]);
        let plugins = manager.load_all();

        assert_eq!(plugins.len(), 1);
        // No skills since directory doesn't exist
        assert_eq!(plugins[0].skill_contents.len(), 0);
    }

    #[test]
    fn test_load_skips_missing_agent_dirs() {
        let tmp = TempDir::new().unwrap();
        let plugin_dir = tmp.path().join("test");
        fs::create_dir_all(plugin_dir.join(".claude-plugin")).unwrap();

        fs::write(
            plugin_dir.join(".claude-plugin/plugin.json"),
            r#"{ "name": "test", "agents": "./nonexistent-agents/" }"#,
        )
        .unwrap();

        let manager = PluginManager::new(vec![tmp.path().to_path_buf()]);
        let plugins = manager.load_all();

        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].agent_configs.len(), 0);
    }

    #[test]
    fn test_load_single_missing_manifest() {
        let tmp = TempDir::new().unwrap();
        let result = PluginManager::new(vec![]).load_single(tmp.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_plugin_dir_stored() {
        let tmp = TempDir::new().unwrap();
        let plugin_dir = create_test_plugin(tmp.path(), "dirtest");

        let manager = PluginManager::new(vec![tmp.path().to_path_buf()]);
        let plugin = manager.load_single(&plugin_dir).unwrap();

        assert_eq!(plugin.plugin_dir, plugin_dir);
    }

    #[test]
    fn test_with_defaults() {
        let manager = PluginManager::with_defaults();
        // Should have at least the ./plugins dir
        assert!(!manager.plugin_dirs().is_empty());
    }

    #[test]
    fn test_ignores_files_in_plugin_dir() {
        let tmp = TempDir::new().unwrap();
        // Put a regular file in the scan dir (not a plugin subdirectory)
        fs::write(tmp.path().join("README.md"), "not a plugin").unwrap();

        let manager = PluginManager::new(vec![tmp.path().to_path_buf()]);
        let plugins = manager.load_all();
        assert!(plugins.is_empty());
    }

    #[test]
    fn test_plugin_meta() {
        let tmp = TempDir::new().unwrap();
        let plugin_dir = create_test_plugin(tmp.path(), "metatest");

        let manager = PluginManager::new(vec![tmp.path().to_path_buf()]);
        let plugin = manager.load_single(&plugin_dir).unwrap();

        let meta = plugin.meta();
        assert_eq!(meta.name, "metatest");
        assert_eq!(meta.version, "0.1.0");
        assert_eq!(meta.description, "Test plugin: metatest");
    }

    #[test]
    fn test_extract_frontmatter_field() {
        let content = r#"---
name: my-skill
description: A great skill
---

# Skill Content
"#;
        assert_eq!(
            extract_frontmatter_field(content, "name"),
            Some("my-skill".to_string())
        );
        assert_eq!(
            extract_frontmatter_field(content, "description"),
            Some("A great skill".to_string())
        );
        assert_eq!(extract_frontmatter_field(content, "nonexistent"), None);
    }

    #[test]
    fn test_extract_frontmatter_field_no_frontmatter() {
        let content = "# Just a heading\n\nNo frontmatter here.";
        assert_eq!(extract_frontmatter_field(content, "name"), None);
    }

    #[test]
    fn test_parse_agent_markdown() {
        let content = r#"---
description: Code review agent
tools: ["shell", "file_read"]
---

# Code Reviewer

You are a code review expert.
"#;
        let (def, config) = parse_agent_markdown("reviewer", content).unwrap();

        assert_eq!(def.name, "reviewer");
        assert_eq!(def.description, "Code review agent");
        assert_eq!(def.tools, vec!["shell", "file_read"]);

        assert_eq!(config.agent.name, "reviewer");
        assert!(config.agent.system_prompt.is_some());
        assert!(
            config
                .agent
                .system_prompt
                .as_ref()
                .unwrap()
                .text
                .contains("code review expert")
        );

        let constraints = config.agent.constraints.unwrap();
        assert_eq!(constraints.tools, vec!["shell", "file_read"]);
    }

    #[test]
    fn test_manifest_path_constant() {
        assert_eq!(MANIFEST_PATH, ".claude-plugin/plugin.json");
    }

    #[test]
    fn test_load_hooks_from_default_path() {
        let tmp = TempDir::new().unwrap();
        let plugin_dir = tmp.path().join("test");
        fs::create_dir_all(plugin_dir.join(".claude-plugin")).unwrap();
        fs::create_dir_all(plugin_dir.join("hooks")).unwrap();

        // Manifest without hooks path - should use default hooks/hooks.json
        fs::write(
            plugin_dir.join(".claude-plugin/plugin.json"),
            r#"{ "name": "test" }"#,
        )
        .unwrap();

        // Write hooks config at default location
        fs::write(
            plugin_dir.join("hooks/hooks.json"),
            r#"{
                "hooks": {
                    "PreToolUse": [
                        {
                            "matcher": "Bash",
                            "hooks": [{ "type": "command", "command": "./validate.sh" }]
                        }
                    ]
                }
            }"#,
        )
        .unwrap();

        let manager = PluginManager::new(vec![tmp.path().to_path_buf()]);
        let plugin = manager.load_single(&plugin_dir).unwrap();

        assert!(plugin.hooks_config.is_some());
        let hooks = plugin.hooks_config.as_ref().unwrap();
        assert!(hooks.hooks.contains_key(&crate::HookEvent::PreToolUse));
    }

    #[test]
    fn test_load_hooks_missing_file() {
        let tmp = TempDir::new().unwrap();
        let plugin_dir = tmp.path().join("test");
        fs::create_dir_all(plugin_dir.join(".claude-plugin")).unwrap();

        // Manifest with hooks path that doesn't exist
        fs::write(
            plugin_dir.join(".claude-plugin/plugin.json"),
            r#"{ "name": "test", "hooks": "./nonexistent/hooks.json" }"#,
        )
        .unwrap();

        let manager = PluginManager::new(vec![tmp.path().to_path_buf()]);
        let plugin = manager.load_single(&plugin_dir).unwrap();

        // Should not fail, just have no hooks
        assert!(plugin.hooks_config.is_none());
    }

    #[test]
    fn test_load_hooks_invalid_json() {
        let tmp = TempDir::new().unwrap();
        let plugin_dir = tmp.path().join("test");
        fs::create_dir_all(plugin_dir.join(".claude-plugin")).unwrap();
        fs::create_dir_all(plugin_dir.join("hooks")).unwrap();

        fs::write(
            plugin_dir.join(".claude-plugin/plugin.json"),
            r#"{ "name": "test", "hooks": "./hooks/hooks.json" }"#,
        )
        .unwrap();

        // Write invalid JSON for hooks
        fs::write(plugin_dir.join("hooks/hooks.json"), "not valid json {{{").unwrap();

        let manager = PluginManager::new(vec![tmp.path().to_path_buf()]);
        let plugin = manager.load_single(&plugin_dir).unwrap();

        // Should not fail, just have no hooks
        assert!(plugin.hooks_config.is_none());
    }
}
