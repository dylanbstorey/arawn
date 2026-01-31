//! Shared interaction logging infrastructure.
//!
//! Captures LLM request/response exchanges as structured [`InteractionRecord`]s
//! written to daily-rotating JSONL files. Consumed by the router, agent turn loop,
//! session indexer, and future training pipelines.

use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::{CompletionRequest, CompletionResponse, ContentBlock, StopReason, Usage};

// ─────────────────────────────────────────────────────────────────────────────
// Record types
// ─────────────────────────────────────────────────────────────────────────────

/// A single LLM interaction (request + response pair).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionRecord {
    /// Unique identifier for this interaction.
    pub id: String,
    /// ISO-8601 timestamp when the request was sent.
    pub timestamp: String,
    /// Wall-clock duration in milliseconds.
    pub duration_ms: u64,

    // ── Request context ──────────────────────────────────────────────────
    /// Model requested.
    pub model: String,
    /// Number of messages in the request.
    pub message_count: usize,
    /// Whether a system prompt was provided.
    pub has_system_prompt: bool,
    /// Names of tools available in this request.
    pub tools_available: Vec<String>,
    /// Whether streaming was requested.
    pub stream: bool,

    // ── Response data ────────────────────────────────────────────────────
    /// Model that actually served the response (may differ from requested).
    pub response_model: String,
    /// Stop reason returned by the backend.
    pub stop_reason: Option<StopReason>,
    /// Token usage.
    pub usage: Usage,
    /// Tool calls made in the response.
    pub tool_calls: Vec<ToolCallRecord>,
    /// Length of text content in the response (characters).
    pub response_text_len: usize,

    // ── Routing metadata (populated by router, empty otherwise) ──────────
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub routing: Option<RoutingMetadata>,

    // ── Extensible tags ──────────────────────────────────────────────────
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

/// A tool call captured from a response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRecord {
    pub tool_name: String,
    pub call_id: String,
}

/// Routing decision metadata (filled in by the routing layer).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingMetadata {
    /// Profile name selected (e.g. "fast", "quality").
    pub profile: String,
    /// Why this profile was selected.
    pub reason: String,
    /// Confidence score 0.0–1.0 (for learned classifiers).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
}

impl InteractionRecord {
    /// Build a record from a completed request/response exchange.
    pub fn from_exchange(
        request: &CompletionRequest,
        response: &CompletionResponse,
        duration_ms: u64,
    ) -> Self {
        let tool_calls = response
            .content
            .iter()
            .filter_map(|block| match block {
                ContentBlock::ToolUse { id, name, .. } => Some(ToolCallRecord {
                    tool_name: name.clone(),
                    call_id: id.clone(),
                }),
                _ => None,
            })
            .collect();

        let response_text_len = response
            .content
            .iter()
            .filter_map(|block| match block {
                ContentBlock::Text { text, .. } => Some(text.len()),
                _ => None,
            })
            .sum();

        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now().to_rfc3339(),
            duration_ms,
            model: request.model.clone(),
            message_count: request.messages.len(),
            has_system_prompt: request.system.is_some(),
            tools_available: request.tools.iter().map(|t| t.name.clone()).collect(),
            stream: request.stream,
            response_model: response.model.clone(),
            stop_reason: response.stop_reason,
            usage: response.usage.clone(),
            tool_calls,
            response_text_len,
            routing: None,
            tags: Vec::new(),
        }
    }

    /// Attach routing metadata after construction.
    pub fn with_routing(mut self, routing: RoutingMetadata) -> Self {
        self.routing = Some(routing);
        self
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Logger
// ─────────────────────────────────────────────────────────────────────────────

/// Configuration for interaction logging.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct InteractionLogConfig {
    /// Whether logging is enabled.
    pub enabled: bool,
    /// Directory for JSONL files. Defaults to `~/.config/arawn/interactions/`.
    pub path: Option<PathBuf>,
    /// Days to retain log files.
    pub retention_days: u32,
}

impl Default for InteractionLogConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            path: None,
            retention_days: 90,
        }
    }
}

impl InteractionLogConfig {
    /// Resolve the log directory, falling back to the XDG default.
    pub fn resolved_path(&self) -> PathBuf {
        self.path.clone().unwrap_or_else(|| {
            dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("arawn")
                .join("interactions")
        })
    }
}

/// Thread-safe JSONL writer with daily file rotation.
pub struct InteractionLogger {
    config: InteractionLogConfig,
    state: Mutex<WriterState>,
}

struct WriterState {
    current_date: Option<NaiveDate>,
    writer: Option<BufWriter<File>>,
}

impl InteractionLogger {
    /// Create a new logger. Runs retention cleanup on init.
    pub fn new(config: InteractionLogConfig) -> std::io::Result<Self> {
        if config.enabled {
            let dir = config.resolved_path();
            fs::create_dir_all(&dir)?;
            cleanup_old_files(&dir, config.retention_days)?;
        }

        Ok(Self {
            config,
            state: Mutex::new(WriterState {
                current_date: None,
                writer: None,
            }),
        })
    }

    /// Log an interaction record. No-op if disabled.
    pub fn log(&self, record: &InteractionRecord) -> std::io::Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let line = serde_json::to_string(record)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let today = Utc::now().date_naive();
        let mut state = self.state.lock().unwrap();

        // Rotate if date changed or no writer yet.
        if state.current_date != Some(today) {
            let dir = self.config.resolved_path();
            let path = dir.join(format!("interactions-{}.jsonl", today));
            let file = OpenOptions::new().create(true).append(true).open(path)?;
            state.writer = Some(BufWriter::new(file));
            state.current_date = Some(today);
        }

        if let Some(ref mut w) = state.writer {
            writeln!(w, "{}", line)?;
            w.flush()?;
        }

        tracing::debug!(
            interaction_id = %record.id,
            model = %record.model,
            duration_ms = record.duration_ms,
            "interaction logged"
        );

        Ok(())
    }
}

/// Delete JSONL files older than `retention_days`.
fn cleanup_old_files(dir: &Path, retention_days: u32) -> std::io::Result<()> {
    let cutoff = Utc::now().date_naive() - chrono::Duration::days(retention_days as i64);

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_string_lossy();

        // Parse date from filename: interactions-YYYY-MM-DD.jsonl
        if let Some(date_str) = name
            .strip_prefix("interactions-")
            .and_then(|s| s.strip_suffix(".jsonl"))
        {
            if let Ok(file_date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                if file_date < cutoff {
                    fs::remove_file(entry.path())?;
                    tracing::info!(file = %name, "removed expired interaction log");
                }
            }
        }
    }

    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{CompletionRequest, CompletionResponse, ContentBlock, StopReason, Usage};

    fn sample_request() -> CompletionRequest {
        CompletionRequest::new("test-model", vec![], 1024)
    }

    fn sample_response() -> CompletionResponse {
        CompletionResponse {
            id: "resp-1".into(),
            response_type: "message".into(),
            role: crate::types::Role::Assistant,
            content: vec![
                ContentBlock::Text {
                    text: "Hello world".into(),
                    cache_control: None,
                },
                ContentBlock::ToolUse {
                    id: "call-1".into(),
                    name: "read_file".into(),
                    input: serde_json::json!({"path": "/tmp/test"}),
                    cache_control: None,
                },
            ],
            model: "test-model".into(),
            stop_reason: Some(StopReason::ToolUse),
            usage: Usage {
                input_tokens: 100,
                output_tokens: 50,
                cache_creation_input_tokens: 0,
                cache_read_input_tokens: 0,
            },
        }
    }

    #[test]
    fn test_record_from_exchange() {
        let req = sample_request();
        let resp = sample_response();
        let record = InteractionRecord::from_exchange(&req, &resp, 150);

        assert_eq!(record.model, "test-model");
        assert_eq!(record.duration_ms, 150);
        assert_eq!(record.response_text_len, 11); // "Hello world"
        assert_eq!(record.tool_calls.len(), 1);
        assert_eq!(record.tool_calls[0].tool_name, "read_file");
        assert_eq!(record.stop_reason, Some(StopReason::ToolUse));
        assert_eq!(record.usage.input_tokens, 100);
        assert!(record.routing.is_none());
    }

    #[test]
    fn test_record_serialization_roundtrip() {
        let req = sample_request();
        let resp = sample_response();
        let record =
            InteractionRecord::from_exchange(&req, &resp, 200).with_routing(RoutingMetadata {
                profile: "fast".into(),
                reason: "short message, no tools".into(),
                confidence: Some(0.95),
            });

        let json = serde_json::to_string(&record).unwrap();
        let deserialized: InteractionRecord = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, record.id);
        assert_eq!(deserialized.model, "test-model");
        assert_eq!(deserialized.routing.as_ref().unwrap().profile, "fast");
        assert_eq!(
            deserialized.routing.as_ref().unwrap().confidence,
            Some(0.95)
        );
    }

    #[test]
    fn test_jsonl_format() {
        let req = sample_request();
        let resp = sample_response();
        let record = InteractionRecord::from_exchange(&req, &resp, 100);

        let json = serde_json::to_string(&record).unwrap();
        // JSONL: single line, no embedded newlines
        assert!(!json.contains('\n'));
        // Valid JSON
        let _: serde_json::Value = serde_json::from_str(&json).unwrap();
    }

    #[test]
    fn test_logger_disabled_is_noop() {
        let config = InteractionLogConfig {
            enabled: false,
            path: None,
            retention_days: 90,
        };
        let logger = InteractionLogger::new(config).unwrap();

        let req = sample_request();
        let resp = sample_response();
        let record = InteractionRecord::from_exchange(&req, &resp, 50);
        // Should not error even with no valid directory
        logger.log(&record).unwrap();
    }

    #[test]
    fn test_logger_writes_jsonl() {
        let dir = tempfile::tempdir().unwrap();
        let config = InteractionLogConfig {
            enabled: true,
            path: Some(dir.path().to_path_buf()),
            retention_days: 90,
        };
        let logger = InteractionLogger::new(config).unwrap();

        let req = sample_request();
        let resp = sample_response();
        let record = InteractionRecord::from_exchange(&req, &resp, 75);
        logger.log(&record).unwrap();

        // Find the written file
        let files: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert_eq!(files.len(), 1);

        let content = fs::read_to_string(files[0].path()).unwrap();
        let lines: Vec<&str> = content.trim().lines().collect();
        assert_eq!(lines.len(), 1);

        let parsed: InteractionRecord = serde_json::from_str(lines[0]).unwrap();
        assert_eq!(parsed.duration_ms, 75);
    }
}
