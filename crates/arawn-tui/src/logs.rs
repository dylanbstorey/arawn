//! Log capture and display for TUI.
//!
//! Captures tracing events and stores them in a ring buffer for display.

use std::collections::VecDeque;
use std::sync::Arc;

use parking_lot::Mutex;
use tracing::field::{Field, Visit};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;

/// Maximum number of log entries to keep.
const MAX_LOG_ENTRIES: usize = 500;

/// A single log entry.
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// Log level.
    pub level: Level,
    /// Target (module path).
    pub target: String,
    /// Log message.
    pub message: String,
}

impl LogEntry {
    /// Get a color for this log level.
    pub fn level_color(&self) -> ratatui::style::Color {
        use ratatui::style::Color;
        match self.level {
            Level::ERROR => Color::Red,
            Level::WARN => Color::Yellow,
            Level::INFO => Color::Green,
            Level::DEBUG => Color::Cyan,
            Level::TRACE => Color::DarkGray,
        }
    }

    /// Get a short level prefix.
    pub fn level_prefix(&self) -> &'static str {
        match self.level {
            Level::ERROR => "ERR",
            Level::WARN => "WRN",
            Level::INFO => "INF",
            Level::DEBUG => "DBG",
            Level::TRACE => "TRC",
        }
    }
}

/// Shared log buffer that can be read by the TUI.
#[derive(Debug, Clone, Default)]
pub struct LogBuffer {
    entries: Arc<Mutex<VecDeque<LogEntry>>>,
}

impl LogBuffer {
    /// Create a new log buffer.
    pub fn new() -> Self {
        Self {
            entries: Arc::new(Mutex::new(VecDeque::with_capacity(MAX_LOG_ENTRIES))),
        }
    }

    /// Get all current entries.
    pub fn entries(&self) -> Vec<LogEntry> {
        self.entries.lock().iter().cloned().collect()
    }

    /// Get the number of entries.
    pub fn len(&self) -> usize {
        self.entries.lock().len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.entries.lock().is_empty()
    }

    /// Clear all entries.
    pub fn clear(&self) {
        self.entries.lock().clear();
    }

    /// Add an entry.
    fn push(&self, entry: LogEntry) {
        let mut entries = self.entries.lock();
        if entries.len() >= MAX_LOG_ENTRIES {
            entries.pop_front();
        }
        entries.push_back(entry);
    }
}

/// A tracing layer that captures logs to a buffer.
pub struct TuiLogLayer {
    buffer: LogBuffer,
    /// Minimum level to capture.
    min_level: Level,
}

impl TuiLogLayer {
    /// Create a new TUI log layer.
    pub fn new(buffer: LogBuffer) -> Self {
        Self {
            buffer,
            min_level: Level::DEBUG,
        }
    }

    /// Set minimum log level to capture.
    pub fn with_min_level(mut self, level: Level) -> Self {
        self.min_level = level;
        self
    }
}

/// Visitor to extract the message field from events.
struct MessageVisitor {
    message: String,
}

impl MessageVisitor {
    fn new() -> Self {
        Self {
            message: String::new(),
        }
    }
}

impl Visit for MessageVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{:?}", value);
            // Remove surrounding quotes if present
            if self.message.starts_with('"') && self.message.ends_with('"') {
                self.message = self.message[1..self.message.len() - 1].to_string();
            }
        } else if self.message.is_empty() {
            // Use first field as message if no "message" field
            self.message = format!("{:?}", value);
        }
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" || self.message.is_empty() {
            self.message = value.to_string();
        }
    }
}

impl<S: Subscriber> Layer<S> for TuiLogLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let metadata = event.metadata();

        // Filter by level
        if *metadata.level() > self.min_level {
            return;
        }

        // Extract message
        let mut visitor = MessageVisitor::new();
        event.record(&mut visitor);

        let entry = LogEntry {
            level: *metadata.level(),
            target: metadata.target().to_string(),
            message: visitor.message,
        };

        self.buffer.push(entry);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_buffer() {
        let buffer = LogBuffer::new();
        assert!(buffer.is_empty());

        buffer.push(LogEntry {
            level: Level::INFO,
            target: "test".to_string(),
            message: "Hello".to_string(),
        });

        assert_eq!(buffer.len(), 1);
        let entries = buffer.entries();
        assert_eq!(entries[0].message, "Hello");
    }

    #[test]
    fn test_log_entry_colors() {
        use ratatui::style::Color;

        let error = LogEntry {
            level: Level::ERROR,
            target: "test".to_string(),
            message: "error".to_string(),
        };
        assert_eq!(error.level_color(), Color::Red);
        assert_eq!(error.level_prefix(), "ERR");

        let info = LogEntry {
            level: Level::INFO,
            target: "test".to_string(),
            message: "info".to_string(),
        };
        assert_eq!(info.level_color(), Color::Green);
        assert_eq!(info.level_prefix(), "INF");
    }
}
