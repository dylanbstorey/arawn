//! Command autocomplete popup component.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
};

/// A command available for execution.
#[derive(Debug, Clone)]
pub struct CommandInfo {
    /// Command name (e.g., "compact").
    pub name: String,
    /// Short description.
    pub description: String,
}

impl CommandInfo {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
        }
    }
}

/// State for the command autocomplete popup.
#[derive(Debug, Default)]
pub struct CommandPopup {
    /// Available commands.
    commands: Vec<CommandInfo>,
    /// Filtered commands based on current input.
    filtered: Vec<usize>,
    /// Currently selected index in filtered list.
    selected: usize,
    /// Whether the popup is visible.
    visible: bool,
}

impl CommandPopup {
    /// Create a new command popup with available commands.
    pub fn new() -> Self {
        Self {
            commands: Self::default_commands(),
            filtered: Vec::new(),
            selected: 0,
            visible: false,
        }
    }

    /// Get the default list of commands.
    /// These will be replaced with commands fetched from the server.
    fn default_commands() -> Vec<CommandInfo> {
        vec![
            CommandInfo::new(
                "compact",
                "Compact session history by summarizing older turns",
            ),
            CommandInfo::new("help", "Show available commands"),
        ]
    }

    /// Set the available commands (fetched from server).
    pub fn set_commands(&mut self, commands: Vec<CommandInfo>) {
        self.commands = commands;
        // Re-filter with current state
        self.filter("");
    }

    /// Show the popup and filter by prefix.
    pub fn show(&mut self, prefix: &str) {
        self.visible = true;
        self.filter(prefix);
    }

    /// Hide the popup.
    pub fn hide(&mut self) {
        self.visible = false;
        self.selected = 0;
    }

    /// Check if the popup is visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Filter commands by prefix.
    pub fn filter(&mut self, prefix: &str) {
        let prefix_lower = prefix.to_lowercase();
        self.filtered = self
            .commands
            .iter()
            .enumerate()
            .filter(|(_, cmd)| cmd.name.to_lowercase().starts_with(&prefix_lower))
            .map(|(i, _)| i)
            .collect();

        // Reset selection if out of bounds
        if self.selected >= self.filtered.len() {
            self.selected = 0;
        }
    }

    /// Select previous item.
    pub fn select_prev(&mut self) {
        if !self.filtered.is_empty() {
            self.selected = self.selected.saturating_sub(1);
        }
    }

    /// Select next item.
    pub fn select_next(&mut self) {
        if !self.filtered.is_empty() {
            self.selected = (self.selected + 1).min(self.filtered.len() - 1);
        }
    }

    /// Get the currently selected command.
    pub fn selected_command(&self) -> Option<&CommandInfo> {
        self.filtered
            .get(self.selected)
            .and_then(|&idx| self.commands.get(idx))
    }

    /// Get the number of filtered commands.
    pub fn filtered_count(&self) -> usize {
        self.filtered.len()
    }

    /// Render the popup.
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if !self.visible || self.filtered.is_empty() {
            return;
        }

        // Calculate popup dimensions
        let max_width = 50;
        let max_height = 8;
        let height = (self.filtered.len() + 2).min(max_height) as u16;
        let width = max_width.min(area.width.saturating_sub(4));

        // Position above the input area
        let popup_area = Rect {
            x: area.x + 1,
            y: area.y.saturating_sub(height),
            width,
            height,
        };

        // Clear the area behind the popup
        frame.render_widget(Clear, popup_area);

        // Build the list items
        let items: Vec<ListItem> = self
            .filtered
            .iter()
            .map(|&idx| {
                let cmd = &self.commands[idx];
                let line = Line::from(vec![
                    Span::styled(
                        format!("/{}", cmd.name),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" - "),
                    Span::styled(&cmd.description, Style::default().fg(Color::Gray)),
                ]);
                ListItem::new(line)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Commands ")
                    .border_style(Style::default().fg(Color::Blue)),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            );

        let mut state = ListState::default();
        state.select(Some(self.selected));

        frame.render_stateful_widget(list, popup_area, &mut state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_popup_filter() {
        let mut popup = CommandPopup::new();

        // Filter by "c" should match "compact"
        popup.filter("c");
        assert_eq!(popup.filtered_count(), 1);
        assert_eq!(popup.selected_command().unwrap().name, "compact");

        // Filter by "h" should match "help"
        popup.filter("h");
        assert_eq!(popup.filtered_count(), 1);
        assert_eq!(popup.selected_command().unwrap().name, "help");

        // Empty filter shows all
        popup.filter("");
        assert_eq!(popup.filtered_count(), 2);

        // No match
        popup.filter("xyz");
        assert_eq!(popup.filtered_count(), 0);
        assert!(popup.selected_command().is_none());
    }

    #[test]
    fn test_command_popup_navigation() {
        let mut popup = CommandPopup::new();
        popup.filter("");

        // Start at first item
        assert_eq!(popup.selected, 0);

        // Move down
        popup.select_next();
        assert_eq!(popup.selected, 1);

        // Can't go past end
        popup.select_next();
        popup.select_next();
        assert_eq!(popup.selected, 1);

        // Move up
        popup.select_prev();
        assert_eq!(popup.selected, 0);

        // Can't go before start
        popup.select_prev();
        assert_eq!(popup.selected, 0);
    }

    #[test]
    fn test_command_popup_visibility() {
        let mut popup = CommandPopup::new();

        assert!(!popup.is_visible());

        popup.show("");
        assert!(popup.is_visible());

        popup.hide();
        assert!(!popup.is_visible());
    }

    #[test]
    fn test_command_popup_set_commands() {
        let mut popup = CommandPopup::new();

        // Set custom commands
        popup.set_commands(vec![
            CommandInfo::new("foo", "Do foo"),
            CommandInfo::new("bar", "Do bar"),
            CommandInfo::new("baz", "Do baz"),
        ]);

        popup.filter("");
        assert_eq!(popup.filtered_count(), 3);

        popup.filter("b");
        assert_eq!(popup.filtered_count(), 2);
    }
}
