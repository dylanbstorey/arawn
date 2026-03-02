//! Bootstrap context file loading.
//!
//! Loads workspace context files (BEHAVIOR.md, BOOTSTRAP.md, etc.) for inclusion
//! in system prompts. Supports smart truncation for large files.

use std::fs;
use std::io;
use std::path::Path;

/// Default maximum characters per bootstrap file before truncation.
pub const DEFAULT_MAX_CHARS: usize = 20_000;

/// Ratio of content to keep from the head when truncating.
const HEAD_RATIO: f64 = 0.7;

/// Ratio of content to keep from the tail when truncating.
const TAIL_RATIO: f64 = 0.2;

/// Standard bootstrap file names to look for.
pub const BOOTSTRAP_FILES: &[&str] = &["BEHAVIOR.md", "BOOTSTRAP.md", "MEMORY.md", "IDENTITY.md"];

/// A single loaded bootstrap file.
#[derive(Debug, Clone)]
pub struct BootstrapFile {
    /// The filename (e.g., "BEHAVIOR.md").
    pub filename: String,
    /// The file content (possibly truncated).
    pub content: String,
    /// Whether the content was truncated.
    pub truncated: bool,
}

/// Collection of loaded bootstrap context files.
#[derive(Debug, Clone, Default)]
pub struct BootstrapContext {
    files: Vec<BootstrapFile>,
}

impl BootstrapContext {
    /// Create an empty bootstrap context.
    pub fn new() -> Self {
        Self { files: Vec::new() }
    }

    /// Load bootstrap files from a workspace directory.
    ///
    /// Looks for standard bootstrap files (BEHAVIOR.md, BOOTSTRAP.md, etc.)
    /// in the given directory and loads them with default options.
    ///
    /// # Arguments
    /// * `workspace` - Path to the workspace directory
    ///
    /// # Returns
    /// A BootstrapContext containing any files found. Returns an empty
    /// context if the directory doesn't exist or contains no bootstrap files.
    pub fn load(workspace: impl AsRef<Path>) -> io::Result<Self> {
        Self::load_with_options(workspace, DEFAULT_MAX_CHARS, None::<fn(&str)>)
    }

    /// Load bootstrap files with custom options.
    ///
    /// # Arguments
    /// * `workspace` - Path to the workspace directory
    /// * `max_chars` - Maximum characters per file before truncation
    /// * `warn_fn` - Optional callback for truncation warnings
    pub fn load_with_options<F>(
        workspace: impl AsRef<Path>,
        max_chars: usize,
        mut warn_fn: Option<F>,
    ) -> io::Result<Self>
    where
        F: FnMut(&str),
    {
        let workspace = workspace.as_ref();

        // If workspace doesn't exist, return empty context
        if !workspace.is_dir() {
            return Ok(Self::new());
        }

        let mut files = Vec::new();

        for filename in BOOTSTRAP_FILES {
            let file_path = workspace.join(filename);
            if file_path.exists() && file_path.is_file() {
                match fs::read_to_string(&file_path) {
                    Ok(content) => {
                        let (content, truncated) = truncate_content(&content, max_chars);

                        if truncated && let Some(ref mut warn) = warn_fn {
                            warn(&format!(
                                "Bootstrap file '{}' exceeded {} chars and was truncated",
                                filename, max_chars
                            ));
                        }

                        files.push(BootstrapFile {
                            filename: filename.to_string(),
                            content,
                            truncated,
                        });
                    }
                    Err(e) => {
                        // Log but continue - missing bootstrap files aren't fatal
                        if let Some(ref mut warn) = warn_fn {
                            warn(&format!(
                                "Failed to read bootstrap file '{}': {}",
                                filename, e
                            ));
                        }
                    }
                }
            }
        }

        Ok(Self { files })
    }

    /// Get the loaded files.
    pub fn files(&self) -> &[BootstrapFile] {
        &self.files
    }

    /// Check if any files were loaded.
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// Get the number of loaded files.
    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// Format the bootstrap context for inclusion in a system prompt.
    ///
    /// Returns an empty string if no files are loaded.
    pub fn to_prompt_section(&self) -> String {
        if self.files.is_empty() {
            return String::new();
        }

        let mut sections = vec!["# Context Files".to_string()];

        for file in &self.files {
            sections.push(format!(
                "## {}{}\n\n{}",
                file.filename,
                if file.truncated { " (truncated)" } else { "" },
                file.content.trim()
            ));
        }

        sections.join("\n\n")
    }

    /// Add a file manually (for testing or custom files).
    pub fn add_file(&mut self, filename: impl Into<String>, content: impl Into<String>) {
        let content = content.into();
        let (content, truncated) = truncate_content(&content, DEFAULT_MAX_CHARS);
        self.files.push(BootstrapFile {
            filename: filename.into(),
            content,
            truncated,
        });
    }
}

/// Truncate content if it exceeds max_chars.
///
/// Uses 70% head + 20% tail strategy, keeping the beginning and end
/// of the content while removing the middle.
fn truncate_content(content: &str, max_chars: usize) -> (String, bool) {
    if content.len() <= max_chars {
        return (content.to_string(), false);
    }

    let head_len = (max_chars as f64 * HEAD_RATIO) as usize;
    let tail_len = (max_chars as f64 * TAIL_RATIO) as usize;

    // Find safe char boundaries
    let head_end = find_char_boundary(content, head_len, true);
    let tail_start = find_char_boundary(content, content.len() - tail_len, false);

    let head = &content[..head_end];
    let tail = &content[tail_start..];

    let truncated = format!(
        "{}\n\n...[content truncated]...\n\n{}",
        head.trim_end(),
        tail.trim_start()
    );

    (truncated, true)
}

/// Find a safe UTF-8 char boundary near the target position.
///
/// # Arguments
/// * `s` - The string to search in
/// * `target` - Target byte position
/// * `search_backward` - If true, search backward from target; if false, search forward
fn find_char_boundary(s: &str, target: usize, search_backward: bool) -> usize {
    if target >= s.len() {
        return s.len();
    }

    if s.is_char_boundary(target) {
        return target;
    }

    if search_backward {
        // Search backward for a valid boundary
        (0..target)
            .rev()
            .find(|&i| s.is_char_boundary(i))
            .unwrap_or(0)
    } else {
        // Search forward for a valid boundary
        (target..s.len())
            .find(|&i| s.is_char_boundary(i))
            .unwrap_or(s.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_empty_context() {
        let ctx = BootstrapContext::new();
        assert!(ctx.is_empty());
        assert_eq!(ctx.len(), 0);
        assert!(ctx.to_prompt_section().is_empty());
    }

    #[test]
    fn test_load_nonexistent_dir() {
        let ctx = BootstrapContext::load("/nonexistent/path/that/does/not/exist").unwrap();
        assert!(ctx.is_empty());
    }

    #[test]
    fn test_load_empty_dir() {
        let temp = TempDir::new().unwrap();
        let ctx = BootstrapContext::load(temp.path()).unwrap();
        assert!(ctx.is_empty());
    }

    #[test]
    fn test_load_soul_md() {
        let temp = TempDir::new().unwrap();
        let soul_content = "# Soul\n\nYou are helpful and kind.";
        fs::write(temp.path().join("BEHAVIOR.md"), soul_content).unwrap();

        let ctx = BootstrapContext::load(temp.path()).unwrap();
        assert_eq!(ctx.len(), 1);
        assert_eq!(ctx.files()[0].filename, "BEHAVIOR.md");
        assert_eq!(ctx.files()[0].content, soul_content);
        assert!(!ctx.files()[0].truncated);
    }

    #[test]
    fn test_load_multiple_files() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("BEHAVIOR.md"), "Soul content").unwrap();
        fs::write(temp.path().join("BOOTSTRAP.md"), "Bootstrap content").unwrap();
        fs::write(temp.path().join("MEMORY.md"), "Memory content").unwrap();

        let ctx = BootstrapContext::load(temp.path()).unwrap();
        assert_eq!(ctx.len(), 3);
    }

    #[test]
    fn test_truncation_under_limit() {
        let content = "Short content";
        let (result, truncated) = truncate_content(content, 1000);
        assert_eq!(result, content);
        assert!(!truncated);
    }

    #[test]
    fn test_truncation_over_limit() {
        let content = "A".repeat(1000);
        let (result, truncated) = truncate_content(&content, 100);

        assert!(truncated);
        assert!(result.len() < content.len());
        assert!(result.contains("...[content truncated]..."));
        // Should have head (70 chars) + marker + tail (20 chars)
        assert!(result.starts_with("AAAA"));
        assert!(result.ends_with("AAAA"));
    }

    #[test]
    fn test_truncation_unicode_boundary() {
        // Create content with multi-byte unicode characters
        let content = "🎉".repeat(100); // Each emoji is 4 bytes
        let (result, truncated) = truncate_content(&content, 50);

        assert!(truncated);
        // Should not panic and should produce valid UTF-8
        assert!(result.is_ascii() || result.chars().count() > 0);
    }

    #[test]
    fn test_to_prompt_section_format() {
        let mut ctx = BootstrapContext::new();
        ctx.add_file("BEHAVIOR.md", "Be helpful.");
        ctx.add_file("BOOTSTRAP.md", "Start here.");

        let section = ctx.to_prompt_section();
        assert!(section.contains("# Context Files"));
        assert!(section.contains("## BEHAVIOR.md"));
        assert!(section.contains("Be helpful."));
        assert!(section.contains("## BOOTSTRAP.md"));
        assert!(section.contains("Start here."));
    }

    #[test]
    fn test_to_prompt_section_shows_truncated() {
        let mut ctx = BootstrapContext::new();
        // Add a file that will be truncated
        let long_content = "A".repeat(DEFAULT_MAX_CHARS + 1000);
        ctx.add_file("LARGE.md", long_content);

        let section = ctx.to_prompt_section();
        assert!(section.contains("(truncated)"));
    }

    #[test]
    fn test_warn_callback() {
        let temp = TempDir::new().unwrap();
        let long_content = "A".repeat(1000);
        fs::write(temp.path().join("BEHAVIOR.md"), long_content).unwrap();

        let mut warnings = Vec::new();
        let _ctx = BootstrapContext::load_with_options(
            temp.path(),
            100, // Low limit to trigger truncation
            Some(|msg: &str| warnings.push(msg.to_string())),
        )
        .unwrap();

        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("truncated"));
    }

    #[test]
    fn test_char_boundary_ascii() {
        let s = "hello world";
        assert_eq!(find_char_boundary(s, 5, true), 5);
        assert_eq!(find_char_boundary(s, 5, false), 5);
    }

    #[test]
    fn test_char_boundary_unicode() {
        let s = "héllo"; // é is 2 bytes
        // Position 2 is in the middle of é
        let boundary = find_char_boundary(s, 2, true);
        assert!(s.is_char_boundary(boundary));
    }
}
