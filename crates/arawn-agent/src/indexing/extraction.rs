//! LLM extraction prompt and JSON parser.

use tracing::warn;

use super::types::ExtractionResult;

/// Builds the extraction prompt for an LLM to extract entities, facts, and
/// relationships from a conversation history.
pub struct ExtractionPrompt;

impl ExtractionPrompt {
    /// Format a conversation history into an extraction prompt.
    ///
    /// `messages` is a slice of `(role, content)` pairs representing the conversation.
    pub fn build(messages: &[(&str, &str)]) -> String {
        let mut prompt = String::with_capacity(4096);

        prompt.push_str(SYSTEM_INSTRUCTION);
        prompt.push_str("\n\n");
        prompt.push_str(FEW_SHOT_EXAMPLE);
        prompt.push_str("\n\n---\n\nNow extract from this conversation:\n\n");

        for (role, content) in messages {
            prompt.push_str(&format!("[{}]: {}\n", role, content));
        }

        prompt.push_str("\nRespond with ONLY the JSON object. No markdown, no explanation.\n");

        prompt
    }
}

const SYSTEM_INSTRUCTION: &str = r#"You are an information extraction system. Given a conversation, extract:

1. **Entities**: Named things mentioned (people, tools, languages, projects, concepts).
2. **Facts**: Concrete factual claims, especially user preferences, configurations, and stated truths.
3. **Relationships**: How entities relate to each other.

Return a JSON object with this structure:
```json
{
  "entities": [
    {"name": "...", "entity_type": "...", "context": "..."}
  ],
  "facts": [
    {"subject": "...", "predicate": "...", "object": "...", "confidence": "stated|observed|inferred"}
  ],
  "relationships": [
    {"from": "...", "relation": "...", "to": "..."}
  ]
}
```

Rules:
- `entity_type` should be one of: person, tool, language, project, concept, organization, file, config
- `confidence` for facts: "stated" if the user explicitly said it, "observed" if derived from behavior, "inferred" if you deduced it
- `subject` for facts should use dot-notation where appropriate (e.g., "user.preferred_model", "project.language")
- Only extract facts that are likely to be useful in future conversations
- Omit trivially obvious or ephemeral information
- If nothing meaningful can be extracted, return `{"entities": [], "facts": [], "relationships": []}`"#;

const FEW_SHOT_EXAMPLE: &str = r#"Example:

Conversation:
[user]: I'm working on my Rust project called Arawn. Can you help me set up SQLite?
[assistant]: Sure! For Rust, I'd recommend using the `rusqlite` crate. Would you like me to add it to your Cargo.toml?
[user]: Yes, and I prefer using WAL mode for better performance.

Extraction:
```json
{
  "entities": [
    {"name": "Arawn", "entity_type": "project", "context": "User's Rust project"},
    {"name": "Rust", "entity_type": "language", "context": "Language used for Arawn"},
    {"name": "SQLite", "entity_type": "tool", "context": "Database being set up"},
    {"name": "rusqlite", "entity_type": "tool", "context": "Rust crate for SQLite"}
  ],
  "facts": [
    {"subject": "project.arawn.language", "predicate": "is", "object": "Rust", "confidence": "stated"},
    {"subject": "user.preference.sqlite_mode", "predicate": "is", "object": "WAL", "confidence": "stated"},
    {"subject": "project.arawn.database", "predicate": "uses", "object": "SQLite", "confidence": "observed"}
  ],
  "relationships": [
    {"from": "Arawn", "relation": "written_in", "to": "Rust"},
    {"from": "Arawn", "relation": "uses", "to": "SQLite"},
    {"from": "Arawn", "relation": "depends_on", "to": "rusqlite"}
  ]
}
```"#;

/// Builds a facts-only extraction prompt for hybrid mode.
///
/// When a local NER engine has already extracted entities and relationships,
/// the LLM only needs to extract facts. The NER entities are provided as
/// context so the LLM can generate higher-quality fact subjects.
pub struct FactsOnlyPrompt;

impl FactsOnlyPrompt {
    /// Build a facts-only extraction prompt with NER entity context.
    ///
    /// `messages` is the conversation history.
    /// `entity_names` are the entities already extracted by NER (provided as context).
    pub fn build(messages: &[(&str, &str)], entity_names: &[&str]) -> String {
        let mut prompt = String::with_capacity(4096);

        prompt.push_str(FACTS_ONLY_INSTRUCTION);
        prompt.push_str("\n\n");

        if !entity_names.is_empty() {
            prompt.push_str("Known entities from this conversation: ");
            prompt.push_str(&entity_names.join(", "));
            prompt.push_str("\n\n");
        }

        prompt.push_str("Conversation:\n\n");
        for (role, content) in messages {
            prompt.push_str(&format!("[{}]: {}\n", role, content));
        }

        prompt.push_str("\nRespond with ONLY the JSON object. No markdown, no explanation.\n");

        prompt
    }
}

const FACTS_ONLY_INSTRUCTION: &str = r#"You are an information extraction system. Entities and relationships have already been extracted. Your job is to extract FACTS only.

Extract concrete factual claims — especially user preferences, configurations, decisions, and stated truths.

Return a JSON object with this structure:
```json
{
  "entities": [],
  "facts": [
    {"subject": "...", "predicate": "...", "object": "...", "confidence": "stated|observed|inferred"}
  ],
  "relationships": []
}
```

Rules:
- `confidence`: "stated" if the user explicitly said it, "observed" if derived from behavior, "inferred" if deduced
- `subject` should use dot-notation where appropriate (e.g., "user.preferred_model", "project.language")
- Reference the known entities in your fact subjects/objects when relevant
- Only extract facts likely to be useful in future conversations
- Omit trivially obvious or ephemeral information
- Keep `entities` and `relationships` as empty arrays"#;

/// Parse LLM output into an ExtractionResult.
///
/// Handles common failure modes:
/// - JSON wrapped in markdown code fences
/// - Trailing commas or minor syntax issues
/// - Partial/malformed output (returns what we can parse)
pub fn parse_extraction(raw: &str) -> ExtractionResult {
    let cleaned = strip_code_fences(raw);

    // Try direct parse first
    if let Ok(result) = serde_json::from_str::<ExtractionResult>(cleaned) {
        return result;
    }

    // Try to find and parse a JSON object within the text
    if let Some(json_str) = extract_json_object(cleaned) {
        if let Ok(result) = serde_json::from_str::<ExtractionResult>(json_str) {
            return result;
        }
    }

    warn!("Failed to parse extraction result, returning empty");
    ExtractionResult::default()
}

/// Strip markdown code fences from LLM output.
fn strip_code_fences(s: &str) -> &str {
    let s = s.trim();

    // Strip ```json ... ``` or ``` ... ```
    if let Some(rest) = s.strip_prefix("```json") {
        if let Some(inner) = rest.strip_suffix("```") {
            return inner.trim();
        }
    }
    if let Some(rest) = s.strip_prefix("```") {
        if let Some(inner) = rest.strip_suffix("```") {
            return inner.trim();
        }
    }

    s
}

/// Try to find a top-level JSON object `{...}` in the text.
fn extract_json_object(s: &str) -> Option<&str> {
    let start = s.find('{')?;
    let end = s.rfind('}')?;
    if end > start {
        Some(&s[start..=end])
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_prompt() {
        let messages = vec![
            ("user", "Hello, I use Rust"),
            ("assistant", "Great choice!"),
        ];
        let prompt = ExtractionPrompt::build(&messages);
        assert!(prompt.contains("[user]: Hello, I use Rust"));
        assert!(prompt.contains("[assistant]: Great choice!"));
        assert!(prompt.contains("information extraction system"));
        assert!(prompt.contains("Example:"));
    }

    #[test]
    fn test_parse_valid_json() {
        let json = r#"{
            "entities": [{"name": "Rust", "entity_type": "language"}],
            "facts": [{"subject": "user.lang", "predicate": "is", "object": "Rust"}],
            "relationships": [{"from": "user", "relation": "uses", "to": "Rust"}]
        }"#;

        let result = parse_extraction(json);
        assert_eq!(result.entities.len(), 1);
        assert_eq!(result.facts.len(), 1);
        assert_eq!(result.relationships.len(), 1);
    }

    #[test]
    fn test_parse_with_code_fences() {
        let raw = r#"```json
{
    "entities": [{"name": "Python", "entity_type": "language"}],
    "facts": [],
    "relationships": []
}
```"#;

        let result = parse_extraction(raw);
        assert_eq!(result.entities.len(), 1);
        assert_eq!(result.entities[0].name, "Python");
    }

    #[test]
    fn test_parse_with_surrounding_text() {
        let raw = r#"Here is the extraction:

{
    "entities": [{"name": "Go", "entity_type": "language"}],
    "facts": [],
    "relationships": []
}

Hope that helps!"#;

        let result = parse_extraction(raw);
        assert_eq!(result.entities.len(), 1);
        assert_eq!(result.entities[0].name, "Go");
    }

    #[test]
    fn test_parse_malformed_returns_empty() {
        let result = parse_extraction("this is not json at all");
        assert!(result.entities.is_empty());
        assert!(result.facts.is_empty());
        assert!(result.relationships.is_empty());
    }

    #[test]
    fn test_parse_partial_json_missing_sections() {
        let json = r#"{"entities": [{"name": "Vim", "entity_type": "tool"}]}"#;
        let result = parse_extraction(json);
        assert_eq!(result.entities.len(), 1);
        assert!(result.facts.is_empty());
    }

    #[test]
    fn test_parse_empty_object() {
        let result = parse_extraction("{}");
        assert!(result.entities.is_empty());
        assert!(result.facts.is_empty());
        assert!(result.relationships.is_empty());
    }

    #[test]
    fn test_strip_code_fences_plain() {
        assert_eq!(strip_code_fences("  hello  "), "hello");
    }

    #[test]
    fn test_strip_code_fences_json() {
        assert_eq!(strip_code_fences("```json\n{}\n```"), "{}");
    }

    #[test]
    fn test_strip_code_fences_bare() {
        assert_eq!(strip_code_fences("```\n{}\n```"), "{}");
    }

    #[test]
    fn test_facts_only_prompt_build() {
        let messages = vec![("user", "I use Rust"), ("assistant", "Nice!")];
        let entities = vec!["Rust"];
        let prompt = FactsOnlyPrompt::build(&messages, &entities);
        assert!(prompt.contains("FACTS only"));
        assert!(prompt.contains("Known entities from this conversation: Rust"));
        assert!(prompt.contains("[user]: I use Rust"));
    }

    #[test]
    fn test_facts_only_prompt_no_entities() {
        let messages = vec![("user", "Hello")];
        let prompt = FactsOnlyPrompt::build(&messages, &[]);
        assert!(prompt.contains("FACTS only"));
        assert!(!prompt.contains("Known entities"));
    }

    #[test]
    fn test_extract_json_object() {
        assert_eq!(
            extract_json_object("prefix {\"a\": 1} suffix"),
            Some("{\"a\": 1}")
        );
        assert!(extract_json_object("no json here").is_none());
    }
}
