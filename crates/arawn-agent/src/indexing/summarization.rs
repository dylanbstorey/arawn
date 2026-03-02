//! Session summarization prompt and parser.

/// Builds the summarization prompt for an LLM to generate a concise session summary.
pub struct SummarizationPrompt;

impl SummarizationPrompt {
    /// Format a conversation history into a summarization prompt.
    ///
    /// `messages` is a slice of `(role, content)` pairs representing the conversation.
    /// Returns `None` if the conversation is empty (no summary needed).
    pub fn build(messages: &[(&str, &str)]) -> Option<String> {
        if messages.is_empty() {
            return None;
        }

        let mut prompt = String::with_capacity(4096);

        prompt.push_str(SYSTEM_INSTRUCTION);
        prompt.push_str("\n\n---\n\nConversation:\n\n");

        for (role, content) in messages {
            prompt.push_str(&format!("[{}]: {}\n", role, content));
        }

        prompt.push_str(
            "\nRespond with ONLY the summary text. No markdown headers, no labels, no preamble.\n",
        );

        Some(prompt)
    }
}

const SYSTEM_INSTRUCTION: &str = r#"You are a conversation summarizer. Given a conversation between a user and an assistant, produce a concise 2-3 sentence summary.

Focus on:
1. **What was accomplished**: tasks completed, code written, problems solved.
2. **Key decisions made**: technology choices, design decisions, configuration changes.
3. **Open questions or next steps**: anything left unresolved or explicitly planned for later.

Rules:
- Be specific: mention file names, tool names, and concrete outcomes rather than vague descriptions.
- Be concise: 2-3 sentences maximum.
- Use past tense for completed work, future tense for planned next steps.
- If the conversation was trivial (e.g., just a greeting), respond with a single short sentence.
- Do not include the word "Summary" or any labels — just the summary text itself."#;

/// Clean up LLM summary output by stripping common wrapper patterns.
///
/// Handles:
/// - Leading "Summary:" or "## Summary" labels
/// - Markdown code fences
/// - Excessive whitespace
pub fn clean_summary(raw: &str) -> String {
    let mut s = raw.trim();

    // Strip markdown headers like "## Summary" or "### Summary"
    if let Some(rest) = s.strip_prefix('#') {
        let rest = rest.trim_start_matches('#').trim();
        if let Some(rest) = rest.strip_prefix("Summary") {
            s = rest.trim_start_matches(':').trim();
        }
    }

    // Strip "Summary:" prefix
    if let Some(rest) = s.strip_prefix("Summary:") {
        s = rest.trim();
    }
    if let Some(rest) = s.strip_prefix("Summary") {
        // Only strip if followed by colon or newline (not mid-word)
        let rest = rest.trim_start();
        if rest.starts_with(':') || rest.starts_with('\n') || rest.is_empty() {
            s = rest.trim_start_matches(':').trim();
        }
    }

    // Strip code fences
    if let Some(rest) = s.strip_prefix("```")
        && let Some(inner) = rest.strip_suffix("```")
    {
        s = inner.trim();
    }

    s.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_prompt_basic() {
        let messages = vec![
            ("user", "Help me set up SQLite in Rust"),
            (
                "assistant",
                "I recommend rusqlite. Let me add it to Cargo.toml.",
            ),
            ("user", "Use WAL mode please"),
        ];
        let prompt = SummarizationPrompt::build(&messages).unwrap();
        assert!(prompt.contains("[user]: Help me set up SQLite in Rust"));
        assert!(prompt.contains("[assistant]: I recommend rusqlite."));
        assert!(prompt.contains("[user]: Use WAL mode please"));
        assert!(prompt.contains("conversation summarizer"));
        assert!(prompt.contains("2-3 sentence"));
    }

    #[test]
    fn test_build_prompt_empty_returns_none() {
        assert!(SummarizationPrompt::build(&[]).is_none());
    }

    #[test]
    fn test_build_prompt_single_message() {
        let messages = vec![("user", "Hello")];
        let prompt = SummarizationPrompt::build(&messages).unwrap();
        assert!(prompt.contains("[user]: Hello"));
    }

    #[test]
    fn test_build_prompt_contains_instructions() {
        let messages = vec![("user", "test")];
        let prompt = SummarizationPrompt::build(&messages).unwrap();
        assert!(prompt.contains("What was accomplished"));
        assert!(prompt.contains("Key decisions made"));
        assert!(prompt.contains("Open questions or next steps"));
        assert!(prompt.contains("ONLY the summary text"));
    }

    #[test]
    fn test_clean_summary_plain() {
        assert_eq!(
            clean_summary("Set up SQLite with WAL mode in the Arawn project."),
            "Set up SQLite with WAL mode in the Arawn project."
        );
    }

    #[test]
    fn test_clean_summary_strips_summary_prefix() {
        assert_eq!(clean_summary("Summary: Set up SQLite."), "Set up SQLite.");
    }

    #[test]
    fn test_clean_summary_strips_markdown_header() {
        assert_eq!(
            clean_summary("## Summary\nSet up SQLite."),
            "Set up SQLite."
        );
    }

    #[test]
    fn test_clean_summary_strips_code_fences() {
        assert_eq!(clean_summary("```\nSet up SQLite.\n```"), "Set up SQLite.");
    }

    #[test]
    fn test_clean_summary_trims_whitespace() {
        assert_eq!(
            clean_summary("  \n  Set up SQLite.  \n  "),
            "Set up SQLite."
        );
    }

    #[test]
    fn test_clean_summary_preserves_word_containing_summary() {
        // "Summary" as part of a normal sentence should not be stripped
        assert_eq!(
            clean_summary("The executive summary was approved."),
            "The executive summary was approved."
        );
    }

    #[test]
    fn test_clean_summary_strips_hash_summary_colon() {
        assert_eq!(
            clean_summary("### Summary:\nDid some work."),
            "Did some work."
        );
    }
}
