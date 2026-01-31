//! Text and string utilities for scripts.

use regex::Regex;

use crate::error::ScriptResult;

/// Check if a string matches a regex pattern.
pub fn matches(text: &str, pattern: &str) -> ScriptResult<bool> {
    let re = Regex::new(pattern)?;
    Ok(re.is_match(text))
}

/// Find all matches of a regex pattern in a string.
pub fn find_all(text: &str, pattern: &str) -> ScriptResult<Vec<String>> {
    let re = Regex::new(pattern)?;
    Ok(re.find_iter(text).map(|m| m.as_str().to_string()).collect())
}

/// Replace all matches of a regex pattern.
pub fn replace_all(text: &str, pattern: &str, replacement: &str) -> ScriptResult<String> {
    let re = Regex::new(pattern)?;
    Ok(re.replace_all(text, replacement).to_string())
}

/// Split a string by a regex pattern.
pub fn split(text: &str, pattern: &str) -> ScriptResult<Vec<String>> {
    let re = Regex::new(pattern)?;
    Ok(re.split(text).map(|s| s.to_string()).collect())
}

/// Extract named capture groups from a regex match.
pub fn extract(
    text: &str,
    pattern: &str,
) -> ScriptResult<Option<std::collections::HashMap<String, String>>> {
    let re = Regex::new(pattern)?;
    let Some(caps) = re.captures(text) else {
        return Ok(None);
    };

    let mut result = std::collections::HashMap::new();
    for name in re.capture_names().flatten() {
        if let Some(m) = caps.name(name) {
            result.insert(name.to_string(), m.as_str().to_string());
        }
    }
    Ok(Some(result))
}

/// Truncate a string to a maximum length, appending `...` if truncated.
pub fn truncate(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else if max_len <= 3 {
        text[..max_len].to_string()
    } else {
        format!("{}...", &text[..max_len - 3])
    }
}

/// Count words in a string (whitespace-separated).
pub fn word_count(text: &str) -> usize {
    text.split_whitespace().count()
}

/// Estimate token count (rough approximation: chars / 4).
pub fn estimate_tokens(text: &str) -> usize {
    text.len().div_ceil(4)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches() {
        assert!(matches("hello world", r"\bworld\b").unwrap());
        assert!(!matches("hello", r"\bworld\b").unwrap());
    }

    #[test]
    fn test_find_all() {
        let results = find_all("foo123bar456", r"\d+").unwrap();
        assert_eq!(results, vec!["123", "456"]);
    }

    #[test]
    fn test_replace_all() {
        let result = replace_all("foo bar baz", r"\s+", "-").unwrap();
        assert_eq!(result, "foo-bar-baz");
    }

    #[test]
    fn test_split() {
        let parts = split("a, b, c", r",\s*").unwrap();
        assert_eq!(parts, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_extract() {
        let result = extract(
            "2024-01-15",
            r"(?P<year>\d{4})-(?P<month>\d{2})-(?P<day>\d{2})",
        )
        .unwrap()
        .unwrap();
        assert_eq!(result["year"], "2024");
        assert_eq!(result["month"], "01");
        assert_eq!(result["day"], "15");
    }

    #[test]
    fn test_extract_no_match() {
        let result = extract("hello", r"(?P<num>\d+)").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello world", 5), "he...");
        assert_eq!(truncate("hi", 5), "hi");
        assert_eq!(truncate("hello world", 50), "hello world");
    }

    #[test]
    fn test_word_count() {
        assert_eq!(word_count("hello world foo"), 3);
        assert_eq!(word_count(""), 0);
    }

    #[test]
    fn test_estimate_tokens() {
        assert_eq!(estimate_tokens("hello world"), 3); // 11 chars / 4 ≈ 3
    }
}
