//! Output configuration and sanitization for tool results.

// ─────────────────────────────────────────────────────────────────────────────
// Output Sanitization
// ─────────────────────────────────────────────────────────────────────────────

/// Default maximum output size in bytes (100KB).
pub const DEFAULT_MAX_OUTPUT_SIZE: usize = 100 * 1024;

/// Configuration for sanitizing tool output.
///
/// Controls size limits, truncation behavior, and content sanitization
/// to prevent context overflow and malformed responses.
#[derive(Debug, Clone)]
pub struct OutputConfig {
    /// Maximum size in bytes before truncation.
    pub max_size_bytes: usize,
    /// Message to append when output is truncated.
    pub truncation_message: String,
    /// Whether to strip control characters (except newlines, tabs).
    pub strip_control_chars: bool,
    /// Whether to strip null bytes.
    pub strip_null_bytes: bool,
    /// Whether to validate JSON structure for JSON outputs.
    pub validate_json: bool,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            max_size_bytes: DEFAULT_MAX_OUTPUT_SIZE,
            truncation_message: "\n\n[Output truncated - exceeded size limit]".to_string(),
            strip_control_chars: true,
            strip_null_bytes: true,
            validate_json: true,
        }
    }
}

impl OutputConfig {
    /// Create a new output config with the given size limit.
    pub fn with_max_size(max_size_bytes: usize) -> Self {
        Self {
            max_size_bytes,
            ..Default::default()
        }
    }

    /// Configuration for shell output (100KB default).
    pub fn for_shell() -> Self {
        Self::with_max_size(100 * 1024)
    }

    /// Configuration for file read output (500KB default).
    pub fn for_file_read() -> Self {
        Self::with_max_size(500 * 1024)
    }

    /// Configuration for web fetch output (200KB default).
    pub fn for_web_fetch() -> Self {
        Self::with_max_size(200 * 1024)
    }

    /// Configuration for search output (50KB default).
    pub fn for_search() -> Self {
        Self::with_max_size(50 * 1024)
    }

    /// Set a custom truncation message.
    pub fn with_truncation_message(mut self, message: impl Into<String>) -> Self {
        self.truncation_message = message.into();
        self
    }

    /// Disable control character stripping.
    pub fn without_control_char_stripping(mut self) -> Self {
        self.strip_control_chars = false;
        self
    }
}

/// Error type for output sanitization failures.
#[derive(Debug, Clone, thiserror::Error)]
pub enum OutputSanitizationError {
    /// Output appears to be binary data.
    #[error(
        "output appears to be binary data (detected {null_bytes} null bytes in first {checked_bytes} bytes)"
    )]
    BinaryContent {
        /// Number of null bytes detected.
        null_bytes: usize,
        /// Number of bytes checked.
        checked_bytes: usize,
    },

    /// JSON output is malformed.
    #[error("JSON output is malformed: {reason}")]
    MalformedJson {
        /// Why the JSON is invalid.
        reason: String,
    },
}

/// Sanitize a string according to the output configuration.
///
/// This function:
/// 1. Detects and rejects binary content
/// 2. Strips null bytes if configured
/// 3. Strips control characters (except newlines, tabs) if configured
/// 4. Truncates to max size if needed
///
/// Returns the sanitized string and whether it was truncated.
pub fn sanitize_output(
    input: &str,
    config: &OutputConfig,
) -> std::result::Result<(String, bool), OutputSanitizationError> {
    // Check for binary content by looking for null bytes in the first 8KB
    let check_len = std::cmp::min(input.len(), 8 * 1024);
    let check_bytes = &input.as_bytes()[..check_len];
    let null_count = check_bytes.iter().filter(|&&b| b == 0).count();

    // If more than 1% null bytes, treat as binary
    if null_count > check_len / 100 && null_count > 10 {
        return Err(OutputSanitizationError::BinaryContent {
            null_bytes: null_count,
            checked_bytes: check_len,
        });
    }

    let mut output = input.to_string();

    // Strip null bytes if configured
    if config.strip_null_bytes {
        output = output.replace('\0', "");
    }

    // Strip control characters if configured (keep newlines, tabs, carriage returns)
    if config.strip_control_chars {
        output = output
            .chars()
            .filter(|c| !c.is_control() || *c == '\n' || *c == '\t' || *c == '\r')
            .collect();
    }

    // Truncate if needed
    let truncated = if output.len() > config.max_size_bytes {
        // Find a safe truncation point (don't break UTF-8)
        let mut truncate_at = config.max_size_bytes;
        while truncate_at > 0 && !output.is_char_boundary(truncate_at) {
            truncate_at -= 1;
        }

        // Reserve space for truncation message
        let msg_len = config.truncation_message.len();
        if truncate_at > msg_len {
            truncate_at -= msg_len;
            while truncate_at > 0 && !output.is_char_boundary(truncate_at) {
                truncate_at -= 1;
            }
        }

        output.truncate(truncate_at);
        output.push_str(&config.truncation_message);
        true
    } else {
        false
    };

    Ok((output, truncated))
}

/// Validate that a JSON value has the expected structure.
///
/// Returns an error if the JSON is malformed or has unexpected structure.
pub fn validate_json_output(
    value: &serde_json::Value,
) -> std::result::Result<(), OutputSanitizationError> {
    // Basic validation - ensure it's a valid JSON value
    // The value is already parsed, so it's syntactically valid
    // We check for some edge cases

    // Check for excessively nested structures (could cause stack overflow during processing)
    fn check_depth(value: &serde_json::Value, depth: usize, max_depth: usize) -> bool {
        if depth > max_depth {
            return false;
        }
        match value {
            serde_json::Value::Array(arr) => {
                arr.iter().all(|v| check_depth(v, depth + 1, max_depth))
            }
            serde_json::Value::Object(obj) => {
                obj.values().all(|v| check_depth(v, depth + 1, max_depth))
            }
            _ => true,
        }
    }

    const MAX_JSON_DEPTH: usize = 50;
    if !check_depth(value, 0, MAX_JSON_DEPTH) {
        return Err(OutputSanitizationError::MalformedJson {
            reason: format!("JSON nesting exceeds maximum depth of {}", MAX_JSON_DEPTH),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─────────────────────────────────────────────────────────────────────────
    // Output Sanitization Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_output_config_defaults() {
        let config = OutputConfig::default();
        assert_eq!(config.max_size_bytes, DEFAULT_MAX_OUTPUT_SIZE);
        assert!(config.strip_control_chars);
        assert!(config.strip_null_bytes);
        assert!(config.validate_json);
    }

    #[test]
    fn test_output_config_per_tool() {
        let shell = OutputConfig::for_shell();
        assert_eq!(shell.max_size_bytes, 100 * 1024);

        let file = OutputConfig::for_file_read();
        assert_eq!(file.max_size_bytes, 500 * 1024);

        let web = OutputConfig::for_web_fetch();
        assert_eq!(web.max_size_bytes, 200 * 1024);

        let search = OutputConfig::for_search();
        assert_eq!(search.max_size_bytes, 50 * 1024);
    }

    #[test]
    fn test_sanitize_output_normal() {
        let config = OutputConfig::default();
        let (result, truncated) = sanitize_output("Hello, world!", &config).unwrap();
        assert_eq!(result, "Hello, world!");
        assert!(!truncated);
    }

    #[test]
    fn test_sanitize_output_strips_null_bytes() {
        let config = OutputConfig::default();
        let input = "Hello\0World\0!";
        let (result, _) = sanitize_output(input, &config).unwrap();
        assert_eq!(result, "HelloWorld!");
    }

    #[test]
    fn test_sanitize_output_strips_control_chars() {
        let config = OutputConfig::default();
        // ASCII control chars (except newline, tab, CR)
        let input = "Hello\x07World\x1B!";
        let (result, _) = sanitize_output(input, &config).unwrap();
        assert_eq!(result, "HelloWorld!");
    }

    #[test]
    fn test_sanitize_output_preserves_newlines_tabs() {
        let config = OutputConfig::default();
        let input = "Hello\nWorld\tTest\r\nEnd";
        let (result, _) = sanitize_output(input, &config).unwrap();
        assert_eq!(result, "Hello\nWorld\tTest\r\nEnd");
    }

    #[test]
    fn test_sanitize_output_truncates() {
        let config = OutputConfig::with_max_size(50);
        let input = "A".repeat(200);
        let (result, truncated) = sanitize_output(&input, &config).unwrap();
        assert!(truncated);
        assert!(result.len() <= 50);
        assert!(result.contains("[Output truncated"));
    }

    #[test]
    fn test_sanitize_output_truncates_utf8_safe() {
        let config = OutputConfig::with_max_size(50);
        // Multi-byte UTF-8 characters
        let input = "日本語".repeat(20); // Each char is 3 bytes
        let (result, truncated) = sanitize_output(&input, &config).unwrap();
        assert!(truncated);
        // Should not panic or produce invalid UTF-8
        assert!(result.is_ascii() || !result.is_empty());
    }

    #[test]
    fn test_sanitize_output_detects_binary() {
        let config = OutputConfig::default();
        // Lots of null bytes = binary content
        let input = "\0".repeat(1000);
        let result = sanitize_output(&input, &config);
        assert!(matches!(
            result,
            Err(OutputSanitizationError::BinaryContent { .. })
        ));
    }

    #[test]
    fn test_sanitize_output_few_nulls_ok() {
        let config = OutputConfig::default();
        // Just a few null bytes is fine
        let input = format!("Hello{}World", "\0".repeat(5));
        let result = sanitize_output(&input, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_json_output_valid() {
        let value = serde_json::json!({"key": "value", "nested": {"a": 1}});
        let result = validate_json_output(&value);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_json_output_deep_nesting() {
        // Create deeply nested JSON
        let mut value = serde_json::json!("leaf");
        for _ in 0..60 {
            value = serde_json::json!({"nested": value});
        }
        let result = validate_json_output(&value);
        assert!(matches!(
            result,
            Err(OutputSanitizationError::MalformedJson { .. })
        ));
    }
}
