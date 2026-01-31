//! Skill loading, parsing, and invocation.
//!
//! Skills are markdown files with YAML frontmatter that teach the agent
//! how to perform specific tasks. They are invoked via `/skill-name args`
//! or `/plugin-name:skill-name args` in user messages.
//!
//! ## Skill Format (Claude Code Compatible)
//!
//! Skills live in `skills/<skill-name>/SKILL.md`:
//!
//! ```markdown
//! ---
//! name: review-pr
//! description: Review a pull request
//! uses_tools:
//!   - github
//! args:
//!   - name: pr_number
//!     description: PR number to review
//!     required: true
//! ---
//!
//! # PR Review
//!
//! Fetch the diff for PR {pr_number} and review it.
//! ```

use crate::types::SkillArg;
use crate::{PluginError, Result};
use std::collections::HashMap;

/// A parsed skill ready for invocation.
#[derive(Debug, Clone)]
pub struct Skill {
    /// Skill name (used for `/name` invocation).
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Tools this skill uses.
    pub uses_tools: Vec<String>,
    /// Declared arguments.
    pub args: Vec<SkillArg>,
    /// The markdown body template (with `{arg}` placeholders).
    pub body: String,
    /// Which plugin this skill came from.
    pub plugin_name: String,
}

/// Result of parsing a `/skill-name args` or `/plugin:skill args` invocation from a user message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillInvocation {
    /// The skill name (without plugin prefix).
    pub name: String,
    /// Optional plugin name for namespaced invocation (e.g., "journal" in `/journal:entry`).
    pub plugin: Option<String>,
    /// Raw argument string after the skill name.
    pub raw_args: String,
}

/// Frontmatter parsed from a skill markdown file.
#[derive(Debug, Clone, serde::Deserialize)]
struct SkillFrontmatter {
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    uses_tools: Vec<String>,
    #[serde(default)]
    args: Vec<SkillArg>,
}

/// Parse a skill from its markdown content.
///
/// Extracts YAML frontmatter between `---` delimiters and the remaining
/// markdown body. This follows Claude Code's skill format.
pub fn parse_skill(content: &str, plugin_name: &str) -> Result<Skill> {
    let (frontmatter_str, body) = split_frontmatter(content)?;

    let frontmatter: SkillFrontmatter =
        serde_yaml::from_str(&frontmatter_str).map_err(|e| PluginError::ManifestParse {
            reason: format!("skill frontmatter: {}", e),
        })?;

    if frontmatter.name.is_empty() {
        return Err(PluginError::Validation {
            field: "name".to_string(),
            message: "skill name must not be empty".to_string(),
        });
    }

    Ok(Skill {
        name: frontmatter.name,
        description: frontmatter.description,
        uses_tools: frontmatter.uses_tools,
        args: frontmatter.args,
        body,
        plugin_name: plugin_name.to_string(),
    })
}

/// Split markdown content into frontmatter and body.
fn split_frontmatter(content: &str) -> Result<(String, String)> {
    let trimmed = content.trim_start();

    if !trimmed.starts_with("---") {
        return Err(PluginError::Validation {
            field: "frontmatter".to_string(),
            message: "skill file must start with --- frontmatter delimiter".to_string(),
        });
    }

    // Find closing ---
    let after_first = &trimmed[3..];
    let close_pos = after_first
        .find("\n---")
        .ok_or_else(|| PluginError::Validation {
            field: "frontmatter".to_string(),
            message: "no closing --- delimiter found".to_string(),
        })?;

    let frontmatter = after_first[..close_pos].trim().to_string();
    let body = after_first[close_pos + 4..].trim().to_string();

    Ok((frontmatter, body))
}

/// Detect a skill invocation in a user message.
///
/// Matches messages starting with:
/// - `/skill-name` - simple skill invocation
/// - `/plugin-name:skill-name` - namespaced skill invocation
///
/// Names can contain lowercase letters, digits, and hyphens.
/// Returns `None` if the message doesn't match.
pub fn detect_invocation(message: &str) -> Option<SkillInvocation> {
    let trimmed = message.trim();
    if !trimmed.starts_with('/') {
        return None;
    }

    let after_slash = &trimmed[1..];

    // Find the full identifier (including potential colon for namespacing)
    // Valid chars: lowercase letters, digits, hyphens, and one colon for namespacing
    let mut name_end = 0;
    let mut colon_pos = None;
    for (i, c) in after_slash.char_indices() {
        if c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' {
            name_end = i + 1;
        } else if c == ':' && colon_pos.is_none() {
            // Allow one colon for namespacing
            colon_pos = Some(i);
            name_end = i + 1;
        } else {
            break;
        }
    }

    if name_end == 0 {
        return None;
    }

    let full_name = &after_slash[..name_end];
    let raw_args = after_slash[name_end..].trim().to_string();

    // Check for namespaced invocation (plugin:skill)
    if let Some(colon) = colon_pos {
        let plugin = full_name[..colon].to_string();
        let name = full_name[colon + 1..].to_string();
        if plugin.is_empty() || name.is_empty() {
            return None;
        }
        Some(SkillInvocation {
            name,
            plugin: Some(plugin),
            raw_args,
        })
    } else {
        Some(SkillInvocation {
            name: full_name.to_string(),
            plugin: None,
            raw_args,
        })
    }
}

/// Substitute arguments into a skill body template.
///
/// Replaces `{arg_name}` placeholders with provided values.
/// Positional args (space-separated in raw_args) map to declared arg names in order.
pub fn substitute_args(skill: &Skill, raw_args: &str) -> Result<String> {
    let mut values: HashMap<&str, &str> = HashMap::new();

    // Parse positional args
    let positional: Vec<&str> = if raw_args.is_empty() {
        Vec::new()
    } else {
        raw_args.split_whitespace().collect()
    };

    // Map positional args to declared arg names
    for (i, arg_def) in skill.args.iter().enumerate() {
        if let Some(val) = positional.get(i) {
            values.insert(&arg_def.name, val);
        } else if arg_def.required {
            return Err(PluginError::Validation {
                field: arg_def.name.clone(),
                message: format!("required argument '{}' not provided", arg_def.name),
            });
        }
    }

    // Substitute placeholders
    let mut result = skill.body.clone();
    for (name, value) in &values {
        let placeholder = format!("{{{}}}", name);
        result = result.replace(&placeholder, value);
    }

    Ok(result)
}

/// Registry of loaded skills, queryable by name or qualified name.
///
/// Skills can be looked up by:
/// - Simple name: `skill-name` (may be ambiguous if multiple plugins have same skill)
/// - Qualified name: `plugin-name:skill-name` (unambiguous)
#[derive(Debug, Default)]
pub struct SkillRegistry {
    /// Skills keyed by qualified name (plugin:skill).
    skills: HashMap<String, Skill>,
    /// Index from simple name to qualified names (for non-namespaced lookups).
    by_simple_name: HashMap<String, Vec<String>>,
}

impl SkillRegistry {
    /// Create an empty skill registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a skill.
    ///
    /// The skill is stored by its qualified name (`plugin:skill`) and indexed
    /// by its simple name for non-namespaced lookups.
    pub fn register(&mut self, skill: Skill) {
        let qualified = format!("{}:{}", skill.plugin_name, skill.name);
        let simple = skill.name.clone();

        self.skills.insert(qualified.clone(), skill);

        self.by_simple_name
            .entry(simple)
            .or_default()
            .push(qualified);
    }

    /// Look up a skill by name (simple) or qualified name (plugin:skill).
    ///
    /// For simple names, returns the skill if unambiguous (only one plugin has it).
    /// For qualified names, returns the exact match.
    pub fn get(&self, name: &str) -> Option<&Skill> {
        // Try qualified name first
        if let Some(skill) = self.skills.get(name) {
            return Some(skill);
        }

        // Try simple name lookup
        if let Some(qualified_names) = self.by_simple_name.get(name) {
            if qualified_names.len() == 1 {
                return self.skills.get(&qualified_names[0]);
            }
            // Ambiguous - multiple plugins have this skill
        }

        None
    }

    /// Look up a skill by invocation (handles namespacing).
    pub fn get_by_invocation(&self, invocation: &SkillInvocation) -> Option<&Skill> {
        if let Some(ref plugin) = invocation.plugin {
            // Qualified lookup
            let qualified = format!("{}:{}", plugin, invocation.name);
            self.skills.get(&qualified)
        } else {
            // Simple lookup
            self.get(&invocation.name)
        }
    }

    /// Get all registered skill names (qualified names).
    pub fn names(&self) -> Vec<&str> {
        self.skills.keys().map(|s| s.as_str()).collect()
    }

    /// Get the number of registered skills.
    pub fn len(&self) -> usize {
        self.skills.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.skills.is_empty()
    }

    /// Invoke a skill by invocation with raw arguments.
    ///
    /// Returns the rendered skill body with arguments substituted,
    /// or `None` if the skill is not found.
    pub fn invoke(&self, invocation: &SkillInvocation) -> Result<Option<String>> {
        match self.get_by_invocation(invocation) {
            Some(skill) => {
                let rendered = substitute_args(skill, &invocation.raw_args)?;
                Ok(Some(rendered))
            }
            None => Ok(None),
        }
    }

    /// Invoke a skill by simple name with raw arguments (convenience method).
    ///
    /// Returns the rendered skill body with arguments substituted,
    /// or `None` if the skill is not found.
    pub fn invoke_simple(&self, name: &str, raw_args: &str) -> Result<Option<String>> {
        match self.get(name) {
            Some(skill) => {
                let rendered = substitute_args(skill, raw_args)?;
                Ok(Some(rendered))
            }
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_SKILL: &str = r#"---
name: journal-entry
description: Create a guided journal entry
uses_tools:
  - journal
args:
  - name: date
    description: Date for the entry
    required: false
  - name: mood
    description: Current mood
    required: true
---

# Journal Entry for {date}

Mood: {mood}

## Steps

1. Use the `journal` tool to create an entry for {date}.
2. Record the mood as {mood}.
3. Ask about accomplishments and goals.
"#;

    #[test]
    fn test_parse_skill() {
        let skill = parse_skill(SAMPLE_SKILL, "journal").unwrap();
        assert_eq!(skill.name, "journal-entry");
        assert_eq!(skill.description, "Create a guided journal entry");
        assert_eq!(skill.uses_tools, vec!["journal"]);
        assert_eq!(skill.args.len(), 2);
        assert_eq!(skill.args[0].name, "date");
        assert!(!skill.args[0].required);
        assert_eq!(skill.args[1].name, "mood");
        assert!(skill.args[1].required);
        assert!(skill.body.contains("# Journal Entry for {date}"));
        assert_eq!(skill.plugin_name, "journal");
    }

    #[test]
    fn test_parse_skill_no_frontmatter() {
        let err = parse_skill("# Just markdown\nNo frontmatter here", "test").unwrap_err();
        assert!(matches!(err, PluginError::Validation { .. }));
    }

    #[test]
    fn test_parse_skill_no_closing_delimiter() {
        let err = parse_skill("---\nname: test\n# No closing", "test").unwrap_err();
        assert!(matches!(err, PluginError::Validation { .. }));
    }

    #[test]
    fn test_parse_skill_empty_name() {
        let content = "---\nname: \"\"\n---\nbody";
        let err = parse_skill(content, "test").unwrap_err();
        assert!(matches!(err, PluginError::Validation { .. }));
    }

    #[test]
    fn test_parse_skill_minimal() {
        let content = "---\nname: simple\n---\nJust a simple skill.";
        let skill = parse_skill(content, "test").unwrap();
        assert_eq!(skill.name, "simple");
        assert!(skill.args.is_empty());
        assert!(skill.uses_tools.is_empty());
        assert_eq!(skill.body, "Just a simple skill.");
    }

    #[test]
    fn test_detect_invocation_basic() {
        let inv = detect_invocation("/journal-entry 2024-01-01 happy").unwrap();
        assert_eq!(inv.name, "journal-entry");
        assert!(inv.plugin.is_none());
        assert_eq!(inv.raw_args, "2024-01-01 happy");
    }

    #[test]
    fn test_detect_invocation_no_args() {
        let inv = detect_invocation("/help").unwrap();
        assert_eq!(inv.name, "help");
        assert!(inv.plugin.is_none());
        assert_eq!(inv.raw_args, "");
    }

    #[test]
    fn test_detect_invocation_with_whitespace() {
        let inv = detect_invocation("  /review-pr 123  ").unwrap();
        assert_eq!(inv.name, "review-pr");
        assert!(inv.plugin.is_none());
        assert_eq!(inv.raw_args, "123");
    }

    #[test]
    fn test_detect_invocation_not_a_skill() {
        assert!(detect_invocation("just a normal message").is_none());
        assert!(detect_invocation("").is_none());
        assert!(detect_invocation("/ no name").is_none());
    }

    #[test]
    fn test_detect_invocation_uppercase_stops() {
        // Uppercase letters end the skill name
        let inv = detect_invocation("/mySkill arg").unwrap();
        assert_eq!(inv.name, "my");
        assert!(inv.plugin.is_none());
        assert_eq!(inv.raw_args, "Skill arg");
    }

    #[test]
    fn test_detect_invocation_namespaced() {
        let inv = detect_invocation("/journal:entry 2024-01-01").unwrap();
        assert_eq!(inv.name, "entry");
        assert_eq!(inv.plugin, Some("journal".to_string()));
        assert_eq!(inv.raw_args, "2024-01-01");
    }

    #[test]
    fn test_detect_invocation_namespaced_no_args() {
        let inv = detect_invocation("/my-plugin:my-skill").unwrap();
        assert_eq!(inv.name, "my-skill");
        assert_eq!(inv.plugin, Some("my-plugin".to_string()));
        assert_eq!(inv.raw_args, "");
    }

    #[test]
    fn test_detect_invocation_invalid_namespace() {
        // Empty plugin or skill name
        assert!(detect_invocation("/:skill").is_none());
        assert!(detect_invocation("/plugin:").is_none());
    }

    #[test]
    fn test_substitute_args_basic() {
        let skill = parse_skill(SAMPLE_SKILL, "test").unwrap();
        let result = substitute_args(&skill, "2024-01-01 happy").unwrap();
        assert!(result.contains("# Journal Entry for 2024-01-01"));
        assert!(result.contains("Mood: happy"));
        assert!(!result.contains("{date}"));
        assert!(!result.contains("{mood}"));
    }

    #[test]
    fn test_substitute_args_missing_required() {
        let skill = parse_skill(SAMPLE_SKILL, "test").unwrap();
        // Only one arg provided, but "mood" (index 1) is required
        let err = substitute_args(&skill, "2024-01-01").unwrap_err();
        assert!(matches!(err, PluginError::Validation { ref field, .. } if field == "mood"));
    }

    #[test]
    fn test_substitute_args_optional_missing() {
        // date is optional (index 0), mood is required (index 1)
        // With only one positional arg, it maps to date (index 0), mood is missing
        let skill = parse_skill(SAMPLE_SKILL, "test").unwrap();
        let err = substitute_args(&skill, "happy").unwrap_err();
        // "happy" maps to date (index 0), mood (index 1) is missing
        assert!(matches!(err, PluginError::Validation { ref field, .. } if field == "mood"));
    }

    #[test]
    fn test_substitute_args_no_args_needed() {
        let content = "---\nname: simple\n---\nNo args needed here.";
        let skill = parse_skill(content, "test").unwrap();
        let result = substitute_args(&skill, "").unwrap();
        assert_eq!(result, "No args needed here.");
    }

    #[test]
    fn test_skill_registry() {
        let mut registry = SkillRegistry::new();
        assert!(registry.is_empty());

        let skill = parse_skill(
            "---\nname: test-skill\ndescription: A test\n---\nBody here.",
            "plugin",
        )
        .unwrap();
        registry.register(skill);

        assert_eq!(registry.len(), 1);
        assert!(!registry.is_empty());
        // Simple name lookup
        assert!(registry.get("test-skill").is_some());
        // Qualified name lookup
        assert!(registry.get("plugin:test-skill").is_some());
        assert!(registry.get("nonexistent").is_none());
        assert!(registry.names().contains(&"plugin:test-skill"));
    }

    #[test]
    fn test_skill_registry_invoke() {
        let mut registry = SkillRegistry::new();
        let content =
            "---\nname: greet\nargs:\n  - name: who\n    required: true\n---\nHello {who}!";
        let skill = parse_skill(content, "test").unwrap();
        registry.register(skill);

        let invocation = SkillInvocation {
            name: "greet".to_string(),
            plugin: None,
            raw_args: "world".to_string(),
        };
        let result = registry.invoke(&invocation).unwrap().unwrap();
        assert_eq!(result, "Hello world!");

        let missing = SkillInvocation {
            name: "nonexistent".to_string(),
            plugin: None,
            raw_args: "".to_string(),
        };
        assert!(registry.invoke(&missing).unwrap().is_none());
    }

    #[test]
    fn test_skill_registry_invoke_missing_arg() {
        let mut registry = SkillRegistry::new();
        let content =
            "---\nname: greet\nargs:\n  - name: who\n    required: true\n---\nHello {who}!";
        let skill = parse_skill(content, "test").unwrap();
        registry.register(skill);

        let invocation = SkillInvocation {
            name: "greet".to_string(),
            plugin: None,
            raw_args: "".to_string(),
        };
        let err = registry.invoke(&invocation).unwrap_err();
        assert!(matches!(err, PluginError::Validation { .. }));
    }

    #[test]
    fn test_skill_registry_namespaced_lookup() {
        let mut registry = SkillRegistry::new();

        // Register same skill name from two plugins
        let skill1 = parse_skill("---\nname: greet\n---\nHello from alpha!", "alpha").unwrap();
        let skill2 = parse_skill("---\nname: greet\n---\nHello from beta!", "beta").unwrap();
        registry.register(skill1);
        registry.register(skill2);

        // Simple lookup should fail (ambiguous)
        assert!(registry.get("greet").is_none());

        // Qualified lookup should work
        let s1 = registry.get("alpha:greet").unwrap();
        assert!(s1.body.contains("alpha"));

        let s2 = registry.get("beta:greet").unwrap();
        assert!(s2.body.contains("beta"));
    }

    #[test]
    fn test_skill_registry_invoke_namespaced() {
        let mut registry = SkillRegistry::new();
        let skill = parse_skill("---\nname: greet\n---\nHello!", "my-plugin").unwrap();
        registry.register(skill);

        // Namespaced invocation
        let invocation = SkillInvocation {
            name: "greet".to_string(),
            plugin: Some("my-plugin".to_string()),
            raw_args: "".to_string(),
        };
        let result = registry.invoke(&invocation).unwrap().unwrap();
        assert_eq!(result, "Hello!");
    }

    #[test]
    fn test_skill_registry_invoke_simple() {
        let mut registry = SkillRegistry::new();
        let skill = parse_skill("---\nname: greet\n---\nHello!", "plugin").unwrap();
        registry.register(skill);

        let result = registry.invoke_simple("greet", "").unwrap().unwrap();
        assert_eq!(result, "Hello!");
    }
}
