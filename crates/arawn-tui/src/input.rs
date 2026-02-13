//! Input state management with history support.

use std::collections::VecDeque;

/// Maximum number of history entries to keep.
const MAX_HISTORY: usize = 100;

/// Input state with text editing and history navigation.
#[derive(Debug, Clone)]
pub struct InputState {
    /// Current input content.
    content: String,
    /// Cursor position (byte offset).
    cursor: usize,
    /// Input history (most recent last).
    history: VecDeque<String>,
    /// Current position in history (None = not browsing).
    history_index: Option<usize>,
    /// Draft saved when starting to browse history.
    draft: Option<String>,
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}

impl InputState {
    /// Create a new empty input state.
    pub fn new() -> Self {
        Self {
            content: String::new(),
            cursor: 0,
            history: VecDeque::with_capacity(MAX_HISTORY),
            history_index: None,
            draft: None,
        }
    }

    /// Get the current input content.
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Get the cursor position (byte offset).
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Check if the input is empty.
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    /// Count the number of lines in the input.
    pub fn line_count(&self) -> usize {
        self.content.lines().count().max(1)
    }

    /// Get the cursor's line and column position.
    ///
    /// Returns (line, column) where line is 0-indexed from the top
    /// and column is the byte offset from the start of that line.
    pub fn cursor_position(&self) -> (usize, usize) {
        // Clamp cursor to valid range to prevent panics on edge cases
        let safe_cursor = self.cursor.min(self.content.len());
        let before_cursor = &self.content[..safe_cursor];
        let line = before_cursor.matches('\n').count();
        let last_newline = before_cursor.rfind('\n').map(|i| i + 1).unwrap_or(0);
        let column = safe_cursor - last_newline;
        (line, column)
    }

    /// Insert a character at the cursor position.
    pub fn insert_char(&mut self, c: char) {
        self.content.insert(self.cursor, c);
        self.cursor += c.len_utf8();
        self.exit_history_mode();
    }

    /// Insert a newline at the cursor position.
    pub fn insert_newline(&mut self) {
        self.insert_char('\n');
    }

    /// Delete the character before the cursor (backspace).
    pub fn delete_char_before(&mut self) {
        if self.cursor > 0 {
            // Find the start of the previous character
            let prev_char_start = self.content[..self.cursor]
                .char_indices()
                .last()
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.content.remove(prev_char_start);
            self.cursor = prev_char_start;
            self.exit_history_mode();
        }
    }

    /// Delete the character at the cursor (delete key).
    pub fn delete_char_at(&mut self) {
        if self.cursor < self.content.len() {
            self.content.remove(self.cursor);
            self.exit_history_mode();
        }
    }

    /// Move cursor left by one character.
    pub fn move_left(&mut self) {
        if self.cursor > 0 {
            // Find the start of the previous character
            self.cursor = self.content[..self.cursor]
                .char_indices()
                .last()
                .map(|(i, _)| i)
                .unwrap_or(0);
        }
    }

    /// Move cursor right by one character.
    pub fn move_right(&mut self) {
        if self.cursor < self.content.len() {
            // Find the start of the next character
            self.cursor = self.content[self.cursor..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| self.cursor + i)
                .unwrap_or(self.content.len());
        }
    }

    /// Move cursor to the start of the current line.
    pub fn move_to_line_start(&mut self) {
        let before_cursor = &self.content[..self.cursor];
        self.cursor = before_cursor.rfind('\n').map(|i| i + 1).unwrap_or(0);
    }

    /// Move cursor to the end of the current line.
    pub fn move_to_line_end(&mut self) {
        let after_cursor = &self.content[self.cursor..];
        self.cursor = after_cursor
            .find('\n')
            .map(|i| self.cursor + i)
            .unwrap_or(self.content.len());
    }

    /// Move cursor to the start of input.
    pub fn move_to_start(&mut self) {
        self.cursor = 0;
    }

    /// Move cursor to the end of input.
    pub fn move_to_end(&mut self) {
        self.cursor = self.content.len();
    }

    /// Move cursor up one line.
    pub fn move_up(&mut self) {
        let (line, column) = self.cursor_position();
        if line > 0 {
            // Find the start of the previous line
            let lines: Vec<&str> = self.content.lines().collect();
            let prev_line_len = lines.get(line - 1).map(|s| s.len()).unwrap_or(0);
            let target_column = column.min(prev_line_len);

            // Calculate new cursor position
            let mut new_cursor = 0;
            for (i, l) in lines.iter().enumerate() {
                if i == line - 1 {
                    new_cursor += target_column;
                    break;
                }
                new_cursor += l.len() + 1; // +1 for newline
            }
            self.cursor = new_cursor;
        }
    }

    /// Move cursor down one line.
    pub fn move_down(&mut self) {
        let (line, column) = self.cursor_position();
        let lines: Vec<&str> = self.content.lines().collect();
        let total_lines = lines.len();

        if line < total_lines.saturating_sub(1) {
            // Find the start of the next line
            let next_line_len = lines.get(line + 1).map(|s| s.len()).unwrap_or(0);
            let target_column = column.min(next_line_len);

            // Calculate new cursor position
            let mut new_cursor = 0;
            for (i, l) in lines.iter().enumerate() {
                if i == line + 1 {
                    new_cursor += target_column;
                    break;
                }
                new_cursor += l.len() + 1; // +1 for newline
            }
            self.cursor = new_cursor.min(self.content.len());
        }
    }

    /// Navigate to previous history entry.
    /// Returns true if history was navigated.
    pub fn history_prev(&mut self) -> bool {
        if self.history.is_empty() {
            return false;
        }

        match self.history_index {
            None => {
                // Save current input as draft
                self.draft = Some(self.content.clone());
                self.history_index = Some(self.history.len() - 1);
            }
            Some(0) => {
                // Already at oldest entry
                return false;
            }
            Some(idx) => {
                self.history_index = Some(idx - 1);
            }
        }

        if let Some(idx) = self.history_index {
            if let Some(entry) = self.history.get(idx) {
                self.content = entry.clone();
                self.cursor = self.content.len();
            }
        }

        true
    }

    /// Navigate to next history entry or restore draft.
    /// Returns true if history was navigated.
    pub fn history_next(&mut self) -> bool {
        match self.history_index {
            None => false,
            Some(idx) if idx >= self.history.len() - 1 => {
                // Restore draft
                if let Some(draft) = self.draft.take() {
                    self.content = draft;
                    self.cursor = self.content.len();
                }
                self.history_index = None;
                true
            }
            Some(idx) => {
                self.history_index = Some(idx + 1);
                if let Some(entry) = self.history.get(idx + 1) {
                    self.content = entry.clone();
                    self.cursor = self.content.len();
                }
                true
            }
        }
    }

    /// Check if currently browsing history.
    pub fn is_browsing_history(&self) -> bool {
        self.history_index.is_some()
    }

    /// Exit history browsing mode without restoring draft.
    ///
    /// This clears both the history index and the saved draft. Once the user
    /// starts editing (typing, deleting, etc.), they are committed to the
    /// current content and can no longer navigate back to their original draft.
    /// This is intentional: the draft is only for temporary storage while
    /// browsing history, not for persisting across editing operations.
    fn exit_history_mode(&mut self) {
        self.history_index = None;
        self.draft = None;
    }

    /// Submit the current input and add to history.
    /// Returns the submitted content.
    pub fn submit(&mut self) -> String {
        let content = std::mem::take(&mut self.content);
        self.cursor = 0;
        self.history_index = None;
        self.draft = None;

        // Add to history if non-empty and different from last entry
        if !content.trim().is_empty() && self.history.back() != Some(&content) {
            if self.history.len() >= MAX_HISTORY {
                self.history.pop_front();
            }
            self.history.push_back(content.clone());
        }

        content
    }

    /// Clear the current input.
    pub fn clear(&mut self) {
        self.content.clear();
        self.cursor = 0;
        self.exit_history_mode();
    }

    /// Set the input text and move cursor to the end.
    pub fn set_text(&mut self, text: &str) {
        self.content = text.to_string();
        self.cursor = self.content.len();
        self.exit_history_mode();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_input() {
        let mut input = InputState::new();
        assert!(input.is_empty());

        input.insert_char('h');
        input.insert_char('i');
        assert_eq!(input.content(), "hi");
        assert_eq!(input.cursor(), 2);
    }

    #[test]
    fn test_cursor_movement() {
        let mut input = InputState::new();
        input.insert_char('a');
        input.insert_char('b');
        input.insert_char('c');

        input.move_left();
        assert_eq!(input.cursor(), 2);

        input.move_to_start();
        assert_eq!(input.cursor(), 0);

        input.move_to_end();
        assert_eq!(input.cursor(), 3);
    }

    #[test]
    fn test_backspace() {
        let mut input = InputState::new();
        input.insert_char('a');
        input.insert_char('b');
        input.delete_char_before();
        assert_eq!(input.content(), "a");
    }

    #[test]
    fn test_history() {
        let mut input = InputState::new();

        // Add some history
        input.insert_char('f');
        input.insert_char('o');
        input.insert_char('o');
        input.submit();

        input.insert_char('b');
        input.insert_char('a');
        input.insert_char('r');
        input.submit();

        // Navigate back
        assert!(input.history_prev());
        assert_eq!(input.content(), "bar");

        assert!(input.history_prev());
        assert_eq!(input.content(), "foo");

        // Navigate forward
        assert!(input.history_next());
        assert_eq!(input.content(), "bar");

        assert!(input.history_next());
        assert!(input.is_empty()); // Back to draft (empty)
    }

    #[test]
    fn test_multiline() {
        let mut input = InputState::new();
        input.insert_char('a');
        input.insert_newline();
        input.insert_char('b');

        assert_eq!(input.content(), "a\nb");
        assert_eq!(input.line_count(), 2);

        let (line, col) = input.cursor_position();
        assert_eq!(line, 1);
        assert_eq!(col, 1);
    }

    #[test]
    fn test_history_with_draft() {
        let mut input = InputState::new();

        // Add history
        input.insert_char('o');
        input.insert_char('l');
        input.insert_char('d');
        input.submit();

        // Type new content (draft)
        input.insert_char('n');
        input.insert_char('e');
        input.insert_char('w');

        // Browse history
        assert!(input.history_prev());
        assert_eq!(input.content(), "old");

        // Return to draft
        assert!(input.history_next());
        assert_eq!(input.content(), "new");
    }
}
