//! Validation utilities for memory interface data.
//!
//! This module provides validation for:
//! - Embedding dimensions and values
//! - Memory content
//! - Confidence scores
//! - Session ID formats

use crate::error::{MemoryError, Result};
use crate::types::Memory;

// ─────────────────────────────────────────────────────────────────────────────
// Validation Error
// ─────────────────────────────────────────────────────────────────────────────

/// Specific validation error types for memory data.
#[derive(Debug, Clone, thiserror::Error)]
pub enum ValidationError {
    /// Memory content is empty.
    #[error("memory content is empty")]
    EmptyContent,

    /// Memory content is not valid UTF-8.
    #[error("memory content contains invalid UTF-8")]
    InvalidUtf8,

    /// Confidence score is out of valid range (0.0-1.0).
    #[error("confidence score {0} is out of range [0.0, 1.0]")]
    InvalidConfidence(f32),

    /// Embedding dimension mismatch.
    #[error("embedding dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch {
        /// Expected dimension.
        expected: usize,
        /// Actual dimension.
        actual: usize,
    },

    /// Embedding contains invalid values (NaN or Inf).
    #[error("embedding contains {count} invalid values (NaN or Inf)")]
    InvalidEmbeddingValues {
        /// Number of invalid values found.
        count: usize,
    },

    /// Session ID is empty.
    #[error("session ID is empty")]
    EmptySessionId,

    /// Session ID format is invalid (not a valid UUID).
    #[error("session ID is not a valid UUID: {0}")]
    InvalidSessionIdFormat(String),
}

impl From<ValidationError> for MemoryError {
    fn from(err: ValidationError) -> Self {
        MemoryError::InvalidData(err.to_string())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Embedding Validation
// ─────────────────────────────────────────────────────────────────────────────

/// Validate an embedding vector.
///
/// Checks:
/// 1. Dimension matches expected size
/// 2. No NaN or Inf values
///
/// # Arguments
/// * `embedding` - The embedding vector to validate
/// * `expected_dim` - The expected dimension size
///
/// # Returns
/// Ok(()) if valid, Err(ValidationError) otherwise
pub fn validate_embedding(
    embedding: &[f32],
    expected_dim: usize,
) -> std::result::Result<(), ValidationError> {
    // Check dimension
    if embedding.len() != expected_dim {
        return Err(ValidationError::DimensionMismatch {
            expected: expected_dim,
            actual: embedding.len(),
        });
    }

    // Check for NaN or Inf values
    let invalid_count = embedding
        .iter()
        .filter(|v| v.is_nan() || v.is_infinite())
        .count();

    if invalid_count > 0 {
        return Err(ValidationError::InvalidEmbeddingValues {
            count: invalid_count,
        });
    }

    Ok(())
}

/// Validate an embedding vector, returning a Result<(), MemoryError>.
///
/// Convenience wrapper for `validate_embedding` that converts to MemoryError.
pub fn validate_embedding_result(embedding: &[f32], expected_dim: usize) -> Result<()> {
    validate_embedding(embedding, expected_dim).map_err(MemoryError::from)
}

// ─────────────────────────────────────────────────────────────────────────────
// Memory Validation
// ─────────────────────────────────────────────────────────────────────────────

/// Validate a memory's content.
///
/// Checks:
/// 1. Content is not empty
/// 2. Content is valid UTF-8 (already guaranteed by String type, but checked explicitly)
///
/// # Returns
/// Ok(()) if valid, Err(ValidationError) otherwise
pub fn validate_memory_content(content: &str) -> std::result::Result<(), ValidationError> {
    if content.is_empty() {
        return Err(ValidationError::EmptyContent);
    }

    // Content is already a valid String, so UTF-8 is guaranteed.
    // But we can check for null bytes which might indicate binary data
    if content.contains('\0') {
        return Err(ValidationError::InvalidUtf8);
    }

    Ok(())
}

/// Validate a complete memory structure.
///
/// Checks:
/// 1. Content is not empty
/// 2. Confidence score is in valid range
///
/// # Returns
/// Ok(()) if valid, Err(ValidationError) otherwise
pub fn validate_memory(memory: &Memory) -> std::result::Result<(), ValidationError> {
    validate_memory_content(&memory.content)?;
    validate_confidence_score(memory.confidence.score)?;
    Ok(())
}

/// Validate a memory, returning a Result<(), MemoryError>.
///
/// Convenience wrapper for `validate_memory` that converts to MemoryError.
pub fn validate_memory_result(memory: &Memory) -> Result<()> {
    validate_memory(memory).map_err(MemoryError::from)
}

// ─────────────────────────────────────────────────────────────────────────────
// Confidence Validation
// ─────────────────────────────────────────────────────────────────────────────

/// Validate a confidence score is in the valid range [0.0, 1.0].
///
/// # Returns
/// Ok(()) if valid, Err(ValidationError) otherwise
pub fn validate_confidence_score(score: f32) -> std::result::Result<(), ValidationError> {
    if !(0.0..=1.0).contains(&score) || score.is_nan() {
        return Err(ValidationError::InvalidConfidence(score));
    }
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Session ID Validation
// ─────────────────────────────────────────────────────────────────────────────

/// Validate a session ID string.
///
/// Checks:
/// 1. Session ID is not empty
/// 2. Session ID is a valid UUID format
///
/// # Returns
/// Ok(()) if valid, Err(ValidationError) otherwise
pub fn validate_session_id(session_id: &str) -> std::result::Result<(), ValidationError> {
    if session_id.is_empty() {
        return Err(ValidationError::EmptySessionId);
    }

    // Validate UUID format
    if uuid::Uuid::parse_str(session_id).is_err() {
        return Err(ValidationError::InvalidSessionIdFormat(
            session_id.to_string(),
        ));
    }

    Ok(())
}

/// Validate a session ID, returning a Result<(), MemoryError>.
///
/// Convenience wrapper for `validate_session_id` that converts to MemoryError.
pub fn validate_session_id_result(session_id: &str) -> Result<()> {
    validate_session_id(session_id).map_err(MemoryError::from)
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ContentType, MemoryConfidence};

    #[test]
    fn test_validate_embedding_valid() {
        let embedding = vec![0.1, 0.2, 0.3, 0.4];
        assert!(validate_embedding(&embedding, 4).is_ok());
    }

    #[test]
    fn test_validate_embedding_wrong_dimension() {
        let embedding = vec![0.1, 0.2, 0.3];
        let result = validate_embedding(&embedding, 4);
        assert!(matches!(
            result,
            Err(ValidationError::DimensionMismatch {
                expected: 4,
                actual: 3
            })
        ));
    }

    #[test]
    fn test_validate_embedding_nan() {
        let embedding = vec![0.1, f32::NAN, 0.3, 0.4];
        let result = validate_embedding(&embedding, 4);
        assert!(matches!(
            result,
            Err(ValidationError::InvalidEmbeddingValues { count: 1 })
        ));
    }

    #[test]
    fn test_validate_embedding_infinity() {
        let embedding = vec![0.1, f32::INFINITY, f32::NEG_INFINITY, 0.4];
        let result = validate_embedding(&embedding, 4);
        assert!(matches!(
            result,
            Err(ValidationError::InvalidEmbeddingValues { count: 2 })
        ));
    }

    #[test]
    fn test_validate_embedding_empty() {
        let embedding: Vec<f32> = vec![];
        assert!(validate_embedding(&embedding, 0).is_ok());

        let result = validate_embedding(&embedding, 4);
        assert!(matches!(
            result,
            Err(ValidationError::DimensionMismatch {
                expected: 4,
                actual: 0
            })
        ));
    }

    #[test]
    fn test_validate_memory_content_valid() {
        assert!(validate_memory_content("Hello, world!").is_ok());
        assert!(validate_memory_content("日本語テスト").is_ok());
        assert!(validate_memory_content("a").is_ok());
    }

    #[test]
    fn test_validate_memory_content_empty() {
        let result = validate_memory_content("");
        assert!(matches!(result, Err(ValidationError::EmptyContent)));
    }

    #[test]
    fn test_validate_memory_content_null_byte() {
        let result = validate_memory_content("Hello\0World");
        assert!(matches!(result, Err(ValidationError::InvalidUtf8)));
    }

    #[test]
    fn test_validate_confidence_score_valid() {
        assert!(validate_confidence_score(0.0).is_ok());
        assert!(validate_confidence_score(0.5).is_ok());
        assert!(validate_confidence_score(1.0).is_ok());
    }

    #[test]
    fn test_validate_confidence_score_invalid() {
        let result = validate_confidence_score(-0.1);
        assert!(matches!(result, Err(ValidationError::InvalidConfidence(_))));

        let result = validate_confidence_score(1.1);
        assert!(matches!(result, Err(ValidationError::InvalidConfidence(_))));

        let result = validate_confidence_score(f32::NAN);
        assert!(matches!(result, Err(ValidationError::InvalidConfidence(_))));
    }

    #[test]
    fn test_validate_memory_valid() {
        let memory = Memory::new(ContentType::Note, "Test content");
        assert!(validate_memory(&memory).is_ok());
    }

    #[test]
    fn test_validate_memory_empty_content() {
        let mut memory = Memory::new(ContentType::Note, "x");
        memory.content = String::new();
        let result = validate_memory(&memory);
        assert!(matches!(result, Err(ValidationError::EmptyContent)));
    }

    #[test]
    fn test_validate_memory_invalid_confidence() {
        let mut memory = Memory::new(ContentType::Note, "Test");
        memory.confidence = MemoryConfidence {
            score: 2.0,
            ..Default::default()
        };
        let result = validate_memory(&memory);
        assert!(matches!(result, Err(ValidationError::InvalidConfidence(_))));
    }

    #[test]
    fn test_validate_session_id_valid() {
        let uuid = uuid::Uuid::new_v4().to_string();
        assert!(validate_session_id(&uuid).is_ok());
    }

    #[test]
    fn test_validate_session_id_empty() {
        let result = validate_session_id("");
        assert!(matches!(result, Err(ValidationError::EmptySessionId)));
    }

    #[test]
    fn test_validate_session_id_invalid_format() {
        let result = validate_session_id("not-a-uuid");
        assert!(matches!(
            result,
            Err(ValidationError::InvalidSessionIdFormat(_))
        ));

        let result = validate_session_id("12345");
        assert!(matches!(
            result,
            Err(ValidationError::InvalidSessionIdFormat(_))
        ));
    }

    #[test]
    fn test_validation_error_to_memory_error() {
        let val_err = ValidationError::EmptyContent;
        let mem_err: MemoryError = val_err.into();
        assert!(matches!(mem_err, MemoryError::InvalidData(_)));
    }

    #[test]
    fn test_validate_embedding_result() {
        let embedding = vec![0.1, 0.2, 0.3, 0.4];
        assert!(validate_embedding_result(&embedding, 4).is_ok());

        let result = validate_embedding_result(&embedding, 5);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_memory_result() {
        let memory = Memory::new(ContentType::Note, "Test");
        assert!(validate_memory_result(&memory).is_ok());
    }

    #[test]
    fn test_validate_session_id_result() {
        let uuid = uuid::Uuid::new_v4().to_string();
        assert!(validate_session_id_result(&uuid).is_ok());

        let result = validate_session_id_result("");
        assert!(result.is_err());
    }
}
