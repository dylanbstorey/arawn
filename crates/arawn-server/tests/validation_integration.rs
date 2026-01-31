//! Interface validation integration tests.
//!
//! These tests verify that validation works correctly at interface boundaries,
//! testing malformed inputs across plugin, tool, LLM, and memory interfaces
//! to ensure graceful error handling.
//!
//! # Test Categories
//!
//! - Plugin manifest validation (malformed manifests rejected at load time)
//! - Tool parameter validation (invalid parameters rejected before execution)
//! - LLM response validation (malformed responses handled gracefully)
//! - Memory operation validation (invalid data returns proper errors)
//! - Output sanitization (oversized/binary content handled correctly)

mod common;

use anyhow::Result;
use serde_json::json;
use tempfile::TempDir;

// ─────────────────────────────────────────────────────────────────────────────
// Test Fixtures
// ─────────────────────────────────────────────────────────────────────────────

mod fixtures {
    use serde_json::json;

    /// Create an invalid plugin manifest missing required fields.
    pub fn plugin_manifest_missing_name() -> serde_json::Value {
        json!({
            "version": "1.0.0",
            "description": "A plugin without a name"
        })
    }

    /// Create an invalid plugin manifest with non-kebab-case name.
    pub fn plugin_manifest_invalid_name() -> serde_json::Value {
        json!({
            "name": "My Plugin With Spaces",
            "version": "1.0.0"
        })
    }

    /// Create an invalid plugin manifest with bad version format.
    pub fn plugin_manifest_invalid_version() -> serde_json::Value {
        json!({
            "name": "my-plugin",
            "version": "not-a-version"
        })
    }

    /// Create tool parameters with missing required field.
    pub fn shell_params_missing_command() -> serde_json::Value {
        json!({
            "timeout_secs": 30
        })
    }

    /// Create tool parameters with empty command.
    pub fn shell_params_empty_command() -> serde_json::Value {
        json!({
            "command": "",
            "timeout_secs": 30
        })
    }

    /// Create tool parameters with invalid timeout.
    pub fn shell_params_invalid_timeout() -> serde_json::Value {
        json!({
            "command": "echo hello",
            "timeout_secs": 0
        })
    }

    /// Create tool parameters with out of range timeout.
    pub fn shell_params_timeout_too_large() -> serde_json::Value {
        json!({
            "command": "echo hello",
            "timeout_secs": 9999999
        })
    }

    /// Create memory store params with empty content.
    pub fn memory_store_empty_content() -> serde_json::Value {
        json!({
            "content": "",
            "importance": 0.5
        })
    }

    /// Create memory store params with invalid importance.
    pub fn memory_store_invalid_importance() -> serde_json::Value {
        json!({
            "content": "Some content",
            "importance": 1.5
        })
    }

    /// Create web search params with zero max_results.
    pub fn web_search_zero_results() -> serde_json::Value {
        json!({
            "query": "test query",
            "max_results": 0
        })
    }

    /// Create file read params with empty path.
    pub fn file_read_empty_path() -> serde_json::Value {
        json!({
            "path": ""
        })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Plugin Manifest Validation Tests
// ─────────────────────────────────────────────────────────────────────────────

mod plugin_tests {
    use super::*;
    use arawn_plugin::{ManifestValidationError, PluginManifest};

    #[test]
    fn test_manifest_missing_name_rejected() {
        let json = fixtures::plugin_manifest_missing_name().to_string();
        let result = PluginManifest::from_json(&json);

        assert!(result.is_err(), "Manifest without name should be rejected");
        let err = result.unwrap_err();
        let err_str = err.to_string();

        // Error should mention the missing name field
        assert!(
            err_str.contains("name"),
            "Error should mention 'name' field: {}",
            err_str
        );
    }

    #[test]
    fn test_manifest_invalid_name_format_rejected() {
        let json = fixtures::plugin_manifest_invalid_name().to_string();
        let result = PluginManifest::from_json(&json);

        assert!(
            result.is_err(),
            "Manifest with spaces in name should be rejected"
        );
        let err = result.unwrap_err();
        let err_str = err.to_string();

        // Error should be actionable - mention what's wrong with the name
        assert!(
            err_str.contains("name")
                && (err_str.contains("kebab") || err_str.contains("lowercase")),
            "Error should explain name format issue: {}",
            err_str
        );
    }

    #[test]
    fn test_manifest_invalid_version_rejected() {
        let json = fixtures::plugin_manifest_invalid_version().to_string();
        let result = PluginManifest::from_json(&json);

        assert!(
            result.is_err(),
            "Manifest with invalid version should be rejected"
        );
        let err = result.unwrap_err();
        let err_str = err.to_string();

        // Error should mention version
        assert!(
            err_str.contains("version"),
            "Error should mention 'version' field: {}",
            err_str
        );
    }

    #[test]
    fn test_manifest_missing_path_detected() {
        let temp_dir = TempDir::new().unwrap();
        let json = json!({
            "name": "test-plugin",
            "skills": "./nonexistent-skills/"
        });

        let manifest = PluginManifest::from_json(&json.to_string()).unwrap();
        let errors = manifest.validate_paths(temp_dir.path());

        assert!(!errors.is_empty(), "Should detect missing skills path");

        let has_path_error = errors.iter().any(|e| {
            matches!(
                e,
                ManifestValidationError::PathNotFound {
                    field: "skills",
                    ..
                }
            )
        });
        assert!(has_path_error, "Should have PathNotFound error for skills");
    }

    #[test]
    fn test_manifest_capability_mismatch_detected() {
        let temp_dir = TempDir::new().unwrap();
        // Create empty skills directory (declared but no actual skills inside)
        std::fs::create_dir(temp_dir.path().join("skills")).unwrap();

        let json = json!({
            "name": "test-plugin",
            "skills": "./skills/"
        });

        let manifest = PluginManifest::from_json(&json.to_string()).unwrap();
        let summary = manifest.capability_summary(temp_dir.path());

        assert!(summary.skills_declared, "Skills should be declared");
        assert_eq!(summary.skills_found, 0, "No actual skills found");
        assert!(
            summary.has_errors(),
            "Should have capability mismatch error"
        );
    }

    #[test]
    fn test_manifest_name_edge_cases() {
        // Name starting with hyphen
        let json1 = json!({ "name": "-invalid" });
        assert!(
            PluginManifest::from_json(&json1.to_string()).is_err(),
            "Name starting with hyphen should be rejected"
        );

        // Name ending with hyphen
        let json2 = json!({ "name": "invalid-" });
        assert!(
            PluginManifest::from_json(&json2.to_string()).is_err(),
            "Name ending with hyphen should be rejected"
        );

        // Name with consecutive hyphens
        let json3 = json!({ "name": "in--valid" });
        assert!(
            PluginManifest::from_json(&json3.to_string()).is_err(),
            "Name with consecutive hyphens should be rejected"
        );

        // Name starting with number
        let json4 = json!({ "name": "123plugin" });
        assert!(
            PluginManifest::from_json(&json4.to_string()).is_err(),
            "Name starting with number should be rejected"
        );
    }

    #[test]
    fn test_manifest_version_edge_cases() {
        // Single number version
        let json1 = json!({ "name": "test", "version": "1" });
        assert!(
            PluginManifest::from_json(&json1.to_string()).is_err(),
            "Single number version should be rejected"
        );

        // Version with leading zeros
        let json2 = json!({ "name": "test", "version": "01.0.0" });
        assert!(
            PluginManifest::from_json(&json2.to_string()).is_err(),
            "Version with leading zeros should be rejected"
        );

        // Empty component in version
        let json3 = json!({ "name": "test", "version": "1..0" });
        assert!(
            PluginManifest::from_json(&json3.to_string()).is_err(),
            "Version with empty component should be rejected"
        );

        // Valid version with prerelease
        let json4 = json!({ "name": "test", "version": "1.0.0-alpha" });
        assert!(
            PluginManifest::from_json(&json4.to_string()).is_ok(),
            "Version with prerelease should be accepted"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tool Parameter Validation Tests
// ─────────────────────────────────────────────────────────────────────────────

mod tool_tests {
    use super::*;
    use arawn_agent::{
        DelegateParams, FileReadParams, FileWriteParams, MemoryRecallParams, MemoryStoreParams,
        ParameterValidationError, ShellParams, ThinkParams, WebSearchParams,
    };
    use std::convert::TryFrom;

    #[test]
    fn test_shell_params_missing_command() {
        let params = fixtures::shell_params_missing_command();
        let result = ShellParams::try_from(params);

        assert!(result.is_err(), "Missing command should be rejected");
        let err = result.unwrap_err();

        // Check error type and message
        assert!(
            matches!(
                err,
                ParameterValidationError::MissingRequired {
                    name: "command",
                    ..
                }
            ),
            "Should be MissingRequired error for 'command': {:?}",
            err
        );

        // Error message should be actionable
        let err_str = err.to_string();
        assert!(
            err_str.contains("command"),
            "Error should mention 'command': {}",
            err_str
        );
    }

    #[test]
    fn test_shell_params_empty_command() {
        let params = fixtures::shell_params_empty_command();
        let result = ShellParams::try_from(params);

        assert!(result.is_err(), "Empty command should be rejected");
        let err = result.unwrap_err();
        assert!(
            matches!(
                err,
                ParameterValidationError::InvalidValue {
                    name: "command",
                    ..
                }
            ),
            "Should be InvalidValue error: {:?}",
            err
        );
    }

    #[test]
    fn test_shell_params_zero_timeout() {
        let params = fixtures::shell_params_invalid_timeout();
        let result = ShellParams::try_from(params);

        assert!(result.is_err(), "Zero timeout should be rejected");
        let err = result.unwrap_err();
        assert!(
            matches!(
                err,
                ParameterValidationError::OutOfRange {
                    name: "timeout_secs",
                    ..
                }
            ),
            "Should be OutOfRange error: {:?}",
            err
        );
    }

    #[test]
    fn test_shell_params_timeout_too_large() {
        let params = fixtures::shell_params_timeout_too_large();
        let result = ShellParams::try_from(params);

        assert!(result.is_err(), "Very large timeout should be rejected");
        let err = result.unwrap_err();
        assert!(
            matches!(
                err,
                ParameterValidationError::OutOfRange {
                    name: "timeout_secs",
                    ..
                }
            ),
            "Should be OutOfRange error: {:?}",
            err
        );

        // Error should mention the constraint
        let err_str = err.to_string();
        assert!(
            err_str.contains("3600") || err_str.contains("hour"),
            "Error should mention max timeout: {}",
            err_str
        );
    }

    #[test]
    fn test_memory_store_empty_content() {
        let params = fixtures::memory_store_empty_content();
        let result = MemoryStoreParams::try_from(params);

        assert!(result.is_err(), "Empty content should be rejected");
        let err = result.unwrap_err();
        assert!(
            matches!(
                err,
                ParameterValidationError::InvalidValue {
                    name: "content",
                    ..
                }
            ),
            "Should be InvalidValue error: {:?}",
            err
        );
    }

    #[test]
    fn test_memory_store_invalid_importance() {
        let params = fixtures::memory_store_invalid_importance();
        let result = MemoryStoreParams::try_from(params);

        assert!(result.is_err(), "Importance > 1.0 should be rejected");
        let err = result.unwrap_err();
        assert!(
            matches!(
                err,
                ParameterValidationError::OutOfRange {
                    name: "importance",
                    ..
                }
            ),
            "Should be OutOfRange error: {:?}",
            err
        );
    }

    #[test]
    fn test_web_search_zero_results() {
        let params = fixtures::web_search_zero_results();
        let result = WebSearchParams::try_from(params);

        assert!(result.is_err(), "max_results=0 should be rejected");
        let err = result.unwrap_err();
        assert!(
            matches!(
                err,
                ParameterValidationError::OutOfRange {
                    name: "max_results",
                    ..
                }
            ),
            "Should be OutOfRange error: {:?}",
            err
        );
    }

    #[test]
    fn test_web_search_too_many_results() {
        let params = json!({
            "query": "test",
            "max_results": 101
        });
        let result = WebSearchParams::try_from(params);

        assert!(result.is_err(), "max_results > 100 should be rejected");
    }

    #[test]
    fn test_file_read_empty_path() {
        let params = fixtures::file_read_empty_path();
        let result = FileReadParams::try_from(params);

        assert!(result.is_err(), "Empty path should be rejected");
        let err = result.unwrap_err();
        assert!(
            matches!(
                err,
                ParameterValidationError::InvalidValue { name: "path", .. }
            ),
            "Should be InvalidValue error: {:?}",
            err
        );
    }

    #[test]
    fn test_file_write_empty_path() {
        let params = json!({
            "path": "",
            "content": "test content"
        });
        let result = FileWriteParams::try_from(params);

        assert!(result.is_err(), "Empty path should be rejected");
    }

    #[test]
    fn test_file_write_missing_content() {
        let params = json!({
            "path": "/tmp/test.txt"
        });
        let result = FileWriteParams::try_from(params);

        assert!(result.is_err(), "Missing content should be rejected");
        let err = result.unwrap_err();
        assert!(
            matches!(
                err,
                ParameterValidationError::MissingRequired {
                    name: "content",
                    ..
                }
            ),
            "Should be MissingRequired error: {:?}",
            err
        );
    }

    #[test]
    fn test_think_params_empty_thought() {
        let params = json!({ "thought": "" });
        let result = ThinkParams::try_from(params);

        assert!(result.is_err(), "Empty thought should be rejected");
    }

    #[test]
    fn test_delegate_params_empty_task() {
        let params = json!({ "task": "" });
        let result = DelegateParams::try_from(params);

        assert!(result.is_err(), "Empty task should be rejected");
    }

    #[test]
    fn test_memory_recall_empty_query() {
        let params = json!({ "query": "" });
        let result = MemoryRecallParams::try_from(params);

        assert!(result.is_err(), "Empty query should be rejected");
    }

    #[test]
    fn test_memory_recall_zero_limit() {
        let params = json!({
            "query": "test",
            "limit": 0
        });
        let result = MemoryRecallParams::try_from(params);

        assert!(result.is_err(), "limit=0 should be rejected");
    }

    #[test]
    fn test_memory_recall_limit_too_large() {
        let params = json!({
            "query": "test",
            "limit": 101
        });
        let result = MemoryRecallParams::try_from(params);

        assert!(result.is_err(), "limit > 100 should be rejected");
    }

    #[test]
    fn test_valid_params_accepted() {
        // Verify that valid parameters are still accepted
        let shell_params = json!({
            "command": "echo hello",
            "timeout_secs": 60
        });
        assert!(
            ShellParams::try_from(shell_params).is_ok(),
            "Valid shell params should be accepted"
        );

        let web_params = json!({
            "query": "test query",
            "max_results": 10
        });
        assert!(
            WebSearchParams::try_from(web_params).is_ok(),
            "Valid web search params should be accepted"
        );

        let memory_params = json!({
            "content": "Test memory",
            "importance": 0.5
        });
        assert!(
            MemoryStoreParams::try_from(memory_params).is_ok(),
            "Valid memory store params should be accepted"
        );
    }

    #[test]
    fn test_parameter_error_into_agent_error() {
        use arawn_agent::AgentError;

        let param_err = ParameterValidationError::missing("test", "provide test");
        let agent_err: AgentError = param_err.into();

        // Verify the error converts properly
        assert!(
            matches!(agent_err, AgentError::Tool(_)),
            "Should convert to AgentError::Tool"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// LLM Response Validation Tests
// ─────────────────────────────────────────────────────────────────────────────

mod llm_tests {
    use arawn_llm::{LlmError, ResponseValidationError};

    #[test]
    fn test_missing_field_error_is_critical() {
        let err = ResponseValidationError::missing_field("id");
        assert!(err.is_critical(), "Missing field should be critical");
    }

    #[test]
    fn test_invalid_tool_use_error_is_critical() {
        let err = ResponseValidationError::invalid_tool_use("tool_123", "empty name");
        assert!(err.is_critical(), "Invalid tool_use should be critical");
    }

    #[test]
    fn test_invalid_token_count_is_not_critical() {
        let err = ResponseValidationError::invalid_token_count("input_tokens", -5, "must be >= 0");
        assert!(
            !err.is_critical(),
            "Invalid token count should not be critical"
        );
    }

    #[test]
    fn test_malformed_content_is_not_critical() {
        let err = ResponseValidationError::malformed_content(0, "unknown type");
        assert!(
            !err.is_critical(),
            "Malformed content should not be critical"
        );
    }

    #[test]
    fn test_error_messages_are_actionable() {
        let err = ResponseValidationError::missing_field("id");
        let msg = err.to_string();
        assert!(
            msg.contains("id"),
            "Error should mention the field: {}",
            msg
        );

        let err2 = ResponseValidationError::invalid_tool_use("tool_x", "name is empty");
        let msg2 = err2.to_string();
        assert!(
            msg2.contains("tool_x") && msg2.contains("name is empty"),
            "Error should include id and reason: {}",
            msg2
        );

        let err3 = ResponseValidationError::invalid_token_count("input_tokens", -1, "must be >= 0");
        let msg3 = err3.to_string();
        assert!(
            msg3.contains("input_tokens") && msg3.contains("-1") && msg3.contains("must be >= 0"),
            "Error should include field, value, and constraint: {}",
            msg3
        );
    }

    #[test]
    fn test_multiple_errors_aggregated() {
        let errors = vec![
            ResponseValidationError::missing_field("id"),
            ResponseValidationError::invalid_tool_use("x", "bad"),
        ];
        let combined = ResponseValidationError::multiple(errors);
        let msg = combined.to_string();

        assert!(
            msg.contains("id") && msg.contains("bad"),
            "Combined error should include all errors: {}",
            msg
        );
    }

    #[test]
    fn test_validation_error_into_llm_error() {
        let val_err = ResponseValidationError::missing_field("id");
        let llm_err: LlmError = val_err.into();

        assert!(
            matches!(llm_err, LlmError::InvalidRequest(_)),
            "Should convert to LlmError::InvalidRequest"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Memory Validation Tests
// ─────────────────────────────────────────────────────────────────────────────

mod memory_tests {
    use arawn_memory::{
        ContentType, Memory, MemoryConfidence, MemoryError, ValidationError,
        validate_confidence_score, validate_embedding, validate_memory, validate_memory_content,
        validate_session_id,
    };

    #[test]
    fn test_empty_content_rejected() {
        let result = validate_memory_content("");
        assert!(matches!(result, Err(ValidationError::EmptyContent)));
    }

    #[test]
    fn test_null_byte_content_rejected() {
        let result = validate_memory_content("hello\0world");
        assert!(matches!(result, Err(ValidationError::InvalidUtf8)));
    }

    #[test]
    fn test_valid_content_accepted() {
        assert!(validate_memory_content("Hello, world!").is_ok());
        assert!(validate_memory_content("日本語テスト").is_ok());
        assert!(validate_memory_content("a").is_ok());
    }

    #[test]
    fn test_confidence_range_validation() {
        assert!(validate_confidence_score(0.0).is_ok());
        assert!(validate_confidence_score(0.5).is_ok());
        assert!(validate_confidence_score(1.0).is_ok());

        assert!(matches!(
            validate_confidence_score(-0.1),
            Err(ValidationError::InvalidConfidence(_))
        ));
        assert!(matches!(
            validate_confidence_score(1.1),
            Err(ValidationError::InvalidConfidence(_))
        ));
        assert!(matches!(
            validate_confidence_score(f32::NAN),
            Err(ValidationError::InvalidConfidence(_))
        ));
    }

    #[test]
    fn test_embedding_dimension_validation() {
        let embedding = vec![0.1, 0.2, 0.3, 0.4];

        assert!(validate_embedding(&embedding, 4).is_ok());

        let result = validate_embedding(&embedding, 5);
        assert!(matches!(
            result,
            Err(ValidationError::DimensionMismatch {
                expected: 5,
                actual: 4
            })
        ));
    }

    #[test]
    fn test_embedding_nan_rejected() {
        let embedding = vec![0.1, f32::NAN, 0.3, 0.4];
        let result = validate_embedding(&embedding, 4);
        assert!(matches!(
            result,
            Err(ValidationError::InvalidEmbeddingValues { count: 1 })
        ));
    }

    #[test]
    fn test_embedding_infinity_rejected() {
        let embedding = vec![0.1, f32::INFINITY, f32::NEG_INFINITY, 0.4];
        let result = validate_embedding(&embedding, 4);
        assert!(matches!(
            result,
            Err(ValidationError::InvalidEmbeddingValues { count: 2 })
        ));
    }

    #[test]
    fn test_session_id_validation() {
        // Valid UUID
        let valid_uuid = uuid::Uuid::new_v4().to_string();
        assert!(validate_session_id(&valid_uuid).is_ok());

        // Empty session ID
        assert!(matches!(
            validate_session_id(""),
            Err(ValidationError::EmptySessionId)
        ));

        // Invalid format
        assert!(matches!(
            validate_session_id("not-a-uuid"),
            Err(ValidationError::InvalidSessionIdFormat(_))
        ));
    }

    #[test]
    fn test_full_memory_validation() {
        let valid_memory = Memory::new(ContentType::Note, "Test content");
        assert!(validate_memory(&valid_memory).is_ok());

        // Empty content
        let mut empty_memory = Memory::new(ContentType::Note, "x");
        empty_memory.content = String::new();
        assert!(matches!(
            validate_memory(&empty_memory),
            Err(ValidationError::EmptyContent)
        ));

        // Invalid confidence
        let mut bad_confidence = Memory::new(ContentType::Note, "Test");
        bad_confidence.confidence = MemoryConfidence {
            score: 2.0,
            ..Default::default()
        };
        assert!(matches!(
            validate_memory(&bad_confidence),
            Err(ValidationError::InvalidConfidence(_))
        ));
    }

    #[test]
    fn test_validation_error_into_memory_error() {
        let val_err = ValidationError::EmptyContent;
        let mem_err: MemoryError = val_err.into();

        assert!(
            matches!(mem_err, MemoryError::InvalidData(_)),
            "Should convert to MemoryError::InvalidData"
        );
    }

    #[test]
    fn test_error_messages_are_descriptive() {
        let err = ValidationError::DimensionMismatch {
            expected: 1024,
            actual: 512,
        };
        let msg = err.to_string();
        assert!(
            msg.contains("1024") && msg.contains("512"),
            "Error should include both dimensions: {}",
            msg
        );

        let err2 = ValidationError::InvalidConfidence(1.5);
        let msg2 = err2.to_string();
        assert!(
            msg2.contains("1.5") && msg2.contains("0.0") && msg2.contains("1.0"),
            "Error should include value and range: {}",
            msg2
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Output Sanitization Tests
// ─────────────────────────────────────────────────────────────────────────────

mod output_tests {
    use arawn_agent::{
        DEFAULT_MAX_OUTPUT_SIZE, OutputConfig, OutputSanitizationError, sanitize_output,
        validate_json_output,
    };
    use serde_json::json;

    #[test]
    fn test_default_config() {
        let config = OutputConfig::default();
        assert_eq!(config.max_size_bytes, DEFAULT_MAX_OUTPUT_SIZE);
        assert!(config.strip_control_chars);
        assert!(config.strip_null_bytes);
    }

    #[test]
    fn test_tool_specific_configs() {
        let shell = OutputConfig::for_shell();
        assert_eq!(shell.max_size_bytes, 100 * 1024);

        let file_read = OutputConfig::for_file_read();
        assert_eq!(file_read.max_size_bytes, 500 * 1024);

        let web_fetch = OutputConfig::for_web_fetch();
        assert_eq!(web_fetch.max_size_bytes, 200 * 1024);

        let search = OutputConfig::for_search();
        assert_eq!(search.max_size_bytes, 50 * 1024);
    }

    #[test]
    fn test_truncation() {
        let config = OutputConfig::with_max_size(100);
        let long_input = "x".repeat(200);

        let (output, truncated) = sanitize_output(&long_input, &config).unwrap();

        assert!(truncated, "Output should be marked as truncated");
        assert!(
            output.len() <= 100,
            "Output should be truncated to max size"
        );
        assert!(
            output.contains("truncated"),
            "Output should contain truncation message"
        );
    }

    #[test]
    fn test_no_truncation_for_small_output() {
        let config = OutputConfig::with_max_size(1000);
        let small_input = "Hello, world!";

        let (output, truncated) = sanitize_output(small_input, &config).unwrap();

        assert!(!truncated, "Small output should not be truncated");
        assert_eq!(output, small_input);
    }

    #[test]
    fn test_binary_content_detected() {
        let config = OutputConfig::default();
        // Create string with many null bytes (simulating binary)
        let binary_content = "\0".repeat(100);

        let result = sanitize_output(&binary_content, &config);

        assert!(
            matches!(result, Err(OutputSanitizationError::BinaryContent { .. })),
            "Binary content should be rejected: {:?}",
            result
        );
    }

    #[test]
    fn test_control_chars_stripped() {
        let config = OutputConfig::default();
        let input = "Hello\x01\x02\x03World\n\tOK";

        let (output, _) = sanitize_output(input, &config).unwrap();

        // Control chars stripped but newlines and tabs preserved
        assert!(output.contains("HelloWorld"));
        assert!(output.contains('\n'));
        assert!(output.contains('\t'));
        assert!(!output.contains('\x01'));
    }

    #[test]
    fn test_null_bytes_stripped() {
        let config = OutputConfig::default();
        // Few null bytes (not enough to trigger binary detection)
        let input = "Hello\0World";

        let (output, _) = sanitize_output(input, &config).unwrap();

        assert!(!output.contains('\0'));
        assert!(output.contains("HelloWorld"));
    }

    #[test]
    fn test_json_depth_validation() {
        // Valid shallow JSON
        let shallow = json!({"a": {"b": {"c": 1}}});
        assert!(validate_json_output(&shallow).is_ok());

        // Create deeply nested JSON (over 50 levels)
        fn create_deep_json(depth: usize) -> serde_json::Value {
            if depth == 0 {
                json!(1)
            } else {
                json!({"nested": create_deep_json(depth - 1)})
            }
        }

        let deep = create_deep_json(60);
        let result = validate_json_output(&deep);

        assert!(
            matches!(result, Err(OutputSanitizationError::MalformedJson { .. })),
            "Deeply nested JSON should be rejected: {:?}",
            result
        );
    }

    #[test]
    fn test_truncation_preserves_utf8() {
        let config = OutputConfig::with_max_size(10);
        // Multi-byte UTF-8 characters
        let input = "日本語テスト";

        let (output, truncated) = sanitize_output(input, &config).unwrap();

        assert!(truncated);
        // Verify output is valid UTF-8 (the sanitize function should not break UTF-8)
        assert!(String::from_utf8(output.as_bytes().to_vec()).is_ok());
    }

    #[test]
    fn test_custom_truncation_message() {
        let config = OutputConfig::with_max_size(50).with_truncation_message("[...cut...]");
        let long_input = "x".repeat(100);

        let (output, _) = sanitize_output(&long_input, &config).unwrap();

        assert!(output.contains("[...cut...]"));
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Integration Tests - Error Propagation Through Layers
// ─────────────────────────────────────────────────────────────────────────────

mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_server_starts_with_validation() -> Result<()> {
        // Verify server starts and validation infrastructure is working
        let server = common::TestServer::start().await?;
        assert!(server.health().await?);
        Ok(())
    }

    #[test]
    fn test_error_chain_plugin_to_user() {
        // Verify error chain from plugin validation to user-facing error
        use arawn_plugin::{ManifestValidationError, PluginError};

        let val_err = ManifestValidationError::missing_field("name", "add a plugin name");
        let plugin_err = PluginError::Validation {
            field: val_err.field_name().unwrap_or("unknown").to_string(),
            message: val_err.to_string(),
        };

        let err_str = plugin_err.to_string();
        assert!(err_str.contains("name"), "Error should mention field name");
    }

    #[test]
    fn test_error_chain_tool_to_user() {
        // Verify error chain from tool validation to agent error
        use arawn_agent::{AgentError, ParameterValidationError};

        let param_err = ParameterValidationError::out_of_range("timeout", 9999, "must be <= 3600");
        let agent_err: AgentError = param_err.into();

        let err_str = agent_err.to_string();
        assert!(
            err_str.contains("timeout"),
            "Error should propagate field name"
        );
    }

    #[test]
    fn test_error_chain_memory_to_user() {
        // Verify error chain from memory validation to memory error
        use arawn_memory::{MemoryError, ValidationError};

        let val_err = ValidationError::InvalidConfidence(1.5);
        let mem_err: MemoryError = val_err.into();

        let err_str = mem_err.to_string();
        assert!(
            err_str.contains("1.5"),
            "Error should include the invalid value"
        );
    }
}
