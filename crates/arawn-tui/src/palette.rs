//! Command palette state and action registry.

/// An action that can be executed from the command palette.
#[derive(Debug, Clone)]
pub struct Action {
    /// Unique action identifier.
    pub id: ActionId,
    /// Display label.
    pub label: &'static str,
    /// Category for grouping.
    pub category: &'static str,
    /// Optional keyboard shortcut hint.
    pub shortcut: Option<&'static str>,
}

/// Identifiers for all palette actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionId {
    // Sessions
    SessionsSwitch,
    SessionsNew,
    SessionsDelete,
    SessionsMoveToWorkstream,
    // Workstreams
    WorkstreamsSwitch,
    WorkstreamsCreate,
    // View
    ViewToggleToolPane,
    // App
    AppQuit,
}

impl Action {
    /// Create a new action.
    const fn new(
        id: ActionId,
        label: &'static str,
        category: &'static str,
        shortcut: Option<&'static str>,
    ) -> Self {
        Self {
            id,
            label,
            category,
            shortcut,
        }
    }
}

/// Default set of actions available in the palette.
pub const DEFAULT_ACTIONS: &[Action] = &[
    Action::new(
        ActionId::SessionsSwitch,
        "Sessions: Switch...",
        "Sessions",
        Some("Ctrl+S"),
    ),
    Action::new(
        ActionId::SessionsNew,
        "Sessions: New",
        "Sessions",
        Some("Ctrl+N"),
    ),
    Action::new(
        ActionId::SessionsDelete,
        "Sessions: Delete current",
        "Sessions",
        None,
    ),
    Action::new(
        ActionId::SessionsMoveToWorkstream,
        "Sessions: Move to workstream...",
        "Sessions",
        None,
    ),
    Action::new(
        ActionId::WorkstreamsSwitch,
        "Workstreams: Switch...",
        "Workstreams",
        Some("Ctrl+W"),
    ),
    Action::new(
        ActionId::WorkstreamsCreate,
        "Workstreams: Create",
        "Workstreams",
        None,
    ),
    Action::new(
        ActionId::ViewToggleToolPane,
        "View: Toggle tool pane",
        "View",
        Some("Ctrl+E"),
    ),
    Action::new(ActionId::AppQuit, "App: Quit", "App", Some("Ctrl+Q")),
];

/// State for the command palette.
#[derive(Debug, Clone)]
pub struct CommandPalette {
    /// All available actions.
    actions: Vec<Action>,
    /// Current filter text.
    filter: String,
    /// Indices of actions matching the filter.
    filtered_indices: Vec<usize>,
    /// Currently selected index (in filtered list).
    selected: usize,
}

impl Default for CommandPalette {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandPalette {
    /// Create a new command palette with default actions.
    pub fn new() -> Self {
        let actions = DEFAULT_ACTIONS.to_vec();
        let filtered_indices: Vec<usize> = (0..actions.len()).collect();

        Self {
            actions,
            filter: String::new(),
            filtered_indices,
            selected: 0,
        }
    }

    /// Get the current filter text.
    pub fn filter(&self) -> &str {
        &self.filter
    }

    /// Get the selected action (if any).
    pub fn selected_action(&self) -> Option<&Action> {
        self.filtered_indices
            .get(self.selected)
            .and_then(|&idx| self.actions.get(idx))
    }

    /// Get the selected index in the filtered list.
    pub fn selected_index(&self) -> usize {
        self.selected
    }

    /// Get an iterator over visible actions with metadata.
    /// Returns (is_selected, is_first_in_category, action).
    pub fn visible_actions(&self) -> impl Iterator<Item = (bool, bool, &Action)> {
        let mut last_category: Option<&str> = None;

        self.filtered_indices
            .iter()
            .enumerate()
            .map(move |(i, &idx)| {
                let action = &self.actions[idx];
                let is_selected = i == self.selected;
                let is_first_in_category = last_category != Some(action.category);
                last_category = Some(action.category);
                (is_selected, is_first_in_category, action)
            })
    }

    /// Get the count of visible actions.
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

    /// Update filtered indices based on current filter.
    fn update_filtered(&mut self) {
        self.filtered_indices = if self.filter.is_empty() {
            (0..self.actions.len()).collect()
        } else {
            let filter_lower = self.filter.to_lowercase();
            self.actions
                .iter()
                .enumerate()
                .filter(|(_, action)| fuzzy_match(action.label, &filter_lower))
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

    /// Reset the palette state.
    pub fn reset(&mut self) {
        self.filter.clear();
        self.selected = 0;
        self.update_filtered();
    }

    /// Register a new action.
    pub fn register_action(&mut self, action: Action) {
        self.actions.push(action);
        self.update_filtered();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_palette_filtering() {
        let mut palette = CommandPalette::new();

        // Should have all actions initially
        let initial_count = palette.visible_count();
        assert!(initial_count > 0);

        // Filter to "ses" should match session-related items
        palette.filter_push('s');
        palette.filter_push('e');
        palette.filter_push('s');

        let filtered_count = palette.visible_count();
        assert!(filtered_count < initial_count);
        assert!(filtered_count >= 3); // At least the 3 session actions

        // Clear filter
        palette.filter_clear();
        assert_eq!(palette.visible_count(), initial_count);
    }

    #[test]
    fn test_palette_navigation() {
        let mut palette = CommandPalette::new();
        let count = palette.visible_count();

        assert_eq!(palette.selected_index(), 0);

        palette.select_next();
        assert_eq!(palette.selected_index(), 1);

        palette.select_last();
        assert_eq!(palette.selected_index(), count - 1);

        palette.select_first();
        assert_eq!(palette.selected_index(), 0);

        palette.select_prev(); // Should stay at 0
        assert_eq!(palette.selected_index(), 0);
    }

    #[test]
    fn test_palette_action_selection() {
        let mut palette = CommandPalette::new();

        let action = palette.selected_action();
        assert!(action.is_some());
        assert_eq!(action.unwrap().id, ActionId::SessionsSwitch);

        palette.select_next();
        let action = palette.selected_action();
        assert!(action.is_some());
        assert_eq!(action.unwrap().id, ActionId::SessionsNew);
    }

    #[test]
    fn test_category_grouping() {
        let palette = CommandPalette::new();
        let mut categories_seen = Vec::new();
        let mut last_category = "";

        for (_, is_first, action) in palette.visible_actions() {
            if is_first {
                assert_ne!(action.category, last_category);
                categories_seen.push(action.category);
                last_category = action.category;
            } else {
                assert_eq!(action.category, last_category);
            }
        }

        // Should have at least Sessions, Workstreams, View, App categories
        assert!(categories_seen.len() >= 4);
    }
}
