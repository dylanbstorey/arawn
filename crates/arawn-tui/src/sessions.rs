//! Session list state and management.

use chrono::{DateTime, Utc};

/// Summary information about a session.
#[derive(Debug, Clone)]
pub struct SessionSummary {
    /// Unique session identifier.
    pub id: String,
    /// Session title (derived from first message or generated).
    pub title: String,
    /// Last activity timestamp.
    pub last_active: DateTime<Utc>,
    /// Number of messages in the session.
    pub message_count: usize,
    /// Whether this is the currently active session.
    pub is_current: bool,
}

/// State for the session list overlay.
#[derive(Debug, Clone)]
pub struct SessionList {
    /// All available sessions.
    items: Vec<SessionSummary>,
    /// Currently selected index (in filtered list).
    selected: usize,
    /// Current filter text.
    filter: String,
    /// Indices of items matching the filter.
    filtered_indices: Vec<usize>,
    /// Whether the list is currently loading.
    loading: bool,
}

impl Default for SessionList {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionList {
    /// Create a new empty session list.
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selected: 0,
            filter: String::new(),
            filtered_indices: Vec::new(),
            loading: false,
        }
    }

    /// Get the filter text.
    pub fn filter(&self) -> &str {
        &self.filter
    }

    /// Check if the list is loading.
    pub fn is_loading(&self) -> bool {
        self.loading
    }

    /// Set loading state.
    pub fn set_loading(&mut self, loading: bool) {
        self.loading = loading;
    }

    /// Update the session list with new items.
    pub fn set_items(&mut self, items: Vec<SessionSummary>) {
        self.items = items;
        self.loading = false;
        self.update_filtered();
    }

    /// Get the currently selected session (if any).
    pub fn selected_session(&self) -> Option<&SessionSummary> {
        self.filtered_indices
            .get(self.selected)
            .and_then(|&idx| self.items.get(idx))
    }

    /// Get the selected index in the filtered list.
    pub fn selected_index(&self) -> usize {
        self.selected
    }

    /// Get an iterator over visible sessions with their selected state.
    pub fn visible_sessions(&self) -> impl Iterator<Item = (bool, &SessionSummary)> {
        self.filtered_indices.iter().enumerate().map(|(i, &idx)| {
            let is_selected = i == self.selected;
            (is_selected, &self.items[idx])
        })
    }

    /// Get the count of visible sessions.
    pub fn visible_count(&self) -> usize {
        self.filtered_indices.len()
    }

    /// Move selection up.
    pub fn select_prev(&mut self) {
        if !self.filtered_indices.is_empty() && self.selected > 0 {
            self.selected -= 1;
        }
    }

    /// Move selection down.
    pub fn select_next(&mut self) {
        if !self.filtered_indices.is_empty() && self.selected < self.filtered_indices.len() - 1 {
            self.selected += 1;
        }
    }

    /// Move selection to first item.
    pub fn select_first(&mut self) {
        self.selected = 0;
    }

    /// Move selection to last item.
    pub fn select_last(&mut self) {
        if !self.filtered_indices.is_empty() {
            self.selected = self.filtered_indices.len() - 1;
        }
    }

    /// Add a character to the filter.
    pub fn filter_push(&mut self, c: char) {
        self.filter.push(c);
        self.update_filtered();
    }

    /// Remove last character from filter.
    pub fn filter_pop(&mut self) {
        self.filter.pop();
        self.update_filtered();
    }

    /// Clear the filter.
    pub fn filter_clear(&mut self) {
        self.filter.clear();
        self.update_filtered();
    }

    /// Update the filtered indices based on current filter.
    fn update_filtered(&mut self) {
        self.filtered_indices = if self.filter.is_empty() {
            (0..self.items.len()).collect()
        } else {
            let filter_lower = self.filter.to_lowercase();
            self.items
                .iter()
                .enumerate()
                .filter(|(_, session)| fuzzy_match(&session.title, &filter_lower))
                .map(|(i, _)| i)
                .collect()
        };

        // Reset selection to valid range
        if self.filtered_indices.is_empty() {
            self.selected = 0;
        } else if self.selected >= self.filtered_indices.len() {
            self.selected = self.filtered_indices.len() - 1;
        }
    }

    /// Reset the list state (e.g., when closing the overlay).
    pub fn reset(&mut self) {
        self.filter.clear();
        self.selected = 0;
        self.update_filtered();
    }

    /// Mark a session as current by ID.
    pub fn set_current(&mut self, session_id: &str) {
        for session in &mut self.items {
            session.is_current = session.id == session_id;
        }
    }
}

/// Simple fuzzy matching - checks if all filter characters appear in order.
fn fuzzy_match(text: &str, filter: &str) -> bool {
    let text_lower = text.to_lowercase();
    let mut text_chars = text_lower.chars().peekable();

    for filter_char in filter.chars() {
        loop {
            match text_chars.next() {
                Some(c) if c == filter_char => break,
                Some(_) => continue,
                None => return false,
            }
        }
    }
    true
}

/// Format a timestamp as a relative time string.
pub fn format_relative_time(time: DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(time);

    if duration.num_seconds() < 60 {
        "just now".to_string()
    } else if duration.num_minutes() < 60 {
        let mins = duration.num_minutes();
        format!("{} min ago", mins)
    } else if duration.num_hours() < 24 {
        let hours = duration.num_hours();
        if hours == 1 {
            "1 hour ago".to_string()
        } else {
            format!("{} hours ago", hours)
        }
    } else if duration.num_days() < 2 {
        "yesterday".to_string()
    } else if duration.num_days() < 7 {
        format!("{} days ago", duration.num_days())
    } else if duration.num_weeks() < 4 {
        let weeks = duration.num_weeks();
        if weeks == 1 {
            "1 week ago".to_string()
        } else {
            format!("{} weeks ago", weeks)
        }
    } else {
        time.format("%b %d").to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_match() {
        assert!(fuzzy_match("async/await explanation", "async"));
        assert!(fuzzy_match("async/await explanation", "aw"));
        assert!(fuzzy_match("async/await explanation", "awe"));
        assert!(fuzzy_match("Debug auth middleware", "dam"));
        assert!(fuzzy_match("Debug auth middleware", "dma")); // d-m-a appear in order
        assert!(!fuzzy_match("Debug auth middleware", "xyz")); // No x, y, z
        assert!(!fuzzy_match("abc", "acb")); // c before b
        assert!(!fuzzy_match("hello", "olleh")); // Reversed
    }

    #[test]
    fn test_session_list_filtering() {
        let mut list = SessionList::new();
        list.set_items(vec![
            SessionSummary {
                id: "1".to_string(),
                title: "async/await explanation".to_string(),
                last_active: Utc::now(),
                message_count: 5,
                is_current: true,
            },
            SessionSummary {
                id: "2".to_string(),
                title: "Debug auth middleware".to_string(),
                last_active: Utc::now(),
                message_count: 3,
                is_current: false,
            },
        ]);

        assert_eq!(list.visible_count(), 2);

        list.filter_push('a');
        list.filter_push('s');
        assert_eq!(list.visible_count(), 1);

        list.filter_clear();
        assert_eq!(list.visible_count(), 2);
    }

    #[test]
    fn test_session_list_navigation() {
        let mut list = SessionList::new();
        list.set_items(vec![
            SessionSummary {
                id: "1".to_string(),
                title: "First".to_string(),
                last_active: Utc::now(),
                message_count: 1,
                is_current: false,
            },
            SessionSummary {
                id: "2".to_string(),
                title: "Second".to_string(),
                last_active: Utc::now(),
                message_count: 2,
                is_current: false,
            },
            SessionSummary {
                id: "3".to_string(),
                title: "Third".to_string(),
                last_active: Utc::now(),
                message_count: 3,
                is_current: false,
            },
        ]);

        assert_eq!(list.selected_index(), 0);

        list.select_next();
        assert_eq!(list.selected_index(), 1);

        list.select_next();
        assert_eq!(list.selected_index(), 2);

        list.select_next(); // Should stay at last
        assert_eq!(list.selected_index(), 2);

        list.select_prev();
        assert_eq!(list.selected_index(), 1);

        list.select_first();
        assert_eq!(list.selected_index(), 0);

        list.select_last();
        assert_eq!(list.selected_index(), 2);
    }
}
