//! Shared CLI output formatting helpers.
//!
//! Provides consistent styling for all CLI commands:
//! - Headers with bold titles and dim separators
//! - Success/error markers with consistent colors
//! - Key-value pair formatting
//! - String truncation
//!
//! The `console` crate automatically disables colors when stdout is not a TTY.

use std::fmt::Display;

use console::{Style, style};

/// Print a section header: bold title + dim separator line.
pub fn header(title: &str) {
    println!("{}", style(title).bold());
    println!("{}", Style::new().dim().apply_to("─".repeat(50)));
    println!();
}

/// Print a success message with a green checkmark.
pub fn success(msg: impl Display) {
    println!("{} {}", Style::new().green().apply_to("✓"), msg);
}

/// Print an error message to stderr with red "Error:" prefix.
pub fn error(msg: impl Display) {
    eprintln!("{} {}", Style::new().red().apply_to("Error:"), msg);
}

/// Print a dim-labeled key-value pair, indented.
///
/// ```text
///   Label:  value
/// ```
pub fn kv(label: &str, value: impl Display) {
    println!(
        "  {:<12} {}",
        Style::new().dim().apply_to(format!("{}:", label)),
        value,
    );
}

/// Print a dim hint/note line.
pub fn hint(msg: impl Display) {
    println!("{}", Style::new().dim().apply_to(msg));
}

/// Truncate a string to a maximum length, collapsing newlines to spaces.
pub fn truncate(s: &str, max_len: usize) -> String {
    let s = s.replace('\n', " ");
    if s.len() <= max_len {
        s
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

/// Truncate a multiline string, preserving indentation on continuation.
pub fn truncate_multiline(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.replace('\n', "\n  ")
    } else {
        format!(
            "{}... ({} chars total)",
            s[..max_len].replace('\n', "\n  "),
            s.len()
        )
    }
}
