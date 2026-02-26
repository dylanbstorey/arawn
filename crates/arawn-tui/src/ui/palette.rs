//! Command palette overlay rendering.

use crate::palette::CommandPalette;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

/// Render the command palette overlay.
pub fn render_palette_overlay(palette: &CommandPalette, frame: &mut Frame, area: Rect) {
    // Create centered overlay (60% width, 50% height)
    let overlay_area = centered_rect(60, 50, area);
    frame.render_widget(Clear, overlay_area);

    let block = Block::default()
        .title(" command ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(overlay_area);
    frame.render_widget(block, overlay_area);

    // Split into search box, separator, list, and footer
    let chunks = Layout::vertical([
        Constraint::Length(1), // Search
        Constraint::Length(1), // Separator
        Constraint::Min(3),    // List
        Constraint::Length(1), // Footer
    ])
    .split(inner);

    render_search_box(palette, frame, chunks[0]);
    render_separator(frame, chunks[1]);
    render_action_list(palette, frame, chunks[2]);
    render_footer(frame, chunks[3]);
}

/// Render the search/filter box.
fn render_search_box(palette: &CommandPalette, frame: &mut Frame, area: Rect) {
    let filter = palette.filter();
    let line = if filter.is_empty() {
        Line::from(Span::styled(
            " > type to search...",
            Style::default().fg(Color::DarkGray),
        ))
    } else {
        Line::from(vec![
            Span::styled(" > ", Style::default().fg(Color::Cyan)),
            Span::styled(filter.to_string(), Style::default().fg(Color::White)),
        ])
    };
    let search = Paragraph::new(line);
    frame.render_widget(search, area);
}

/// Render a separator line.
fn render_separator(frame: &mut Frame, area: Rect) {
    let sep = Paragraph::new(Line::from(Span::styled(
        "─".repeat(area.width as usize),
        Style::default().fg(Color::DarkGray),
    )));
    frame.render_widget(sep, area);
}

/// Render the action list.
fn render_action_list(palette: &CommandPalette, frame: &mut Frame, area: Rect) {
    let mut lines = Vec::new();

    if palette.visible_count() == 0 {
        lines.push(Line::from(Span::styled(
            "  No matching commands",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        let mut prev_category: Option<&str> = None;

        for (is_selected, is_first_in_category, action) in palette.visible_actions() {
            // Add separator between categories (but not before first)
            if is_first_in_category && prev_category.is_some() {
                lines.push(Line::from(Span::styled(
                    format!("  {}", "─".repeat(area.width.saturating_sub(4) as usize)),
                    Style::default().fg(Color::DarkGray),
                )));
            }
            prev_category = Some(action.category);

            // Format the action line
            let line = format_action_line(action, is_selected, area.width as usize);
            lines.push(line);
        }
    }

    let list = Paragraph::new(lines);
    frame.render_widget(list, area);
}

/// Format a single action line.
fn format_action_line(
    action: &crate::palette::Action,
    is_selected: bool,
    width: usize,
) -> Line<'static> {
    let shortcut = action.shortcut.unwrap_or("");
    let label = action.label;

    // Calculate spacing
    let prefix_width = 3; // "   " or " > "
    let shortcut_width = shortcut.len() + 2; // padding
    let label_width = width.saturating_sub(prefix_width + shortcut_width);

    // Truncate label if needed
    let display_label = if label.len() > label_width {
        format!("{}...", &label[..label_width.saturating_sub(3)])
    } else {
        label.to_string()
    };

    // Build the line
    let prefix = if is_selected {
        Span::styled(" > ", Style::default().fg(Color::Cyan))
    } else {
        Span::raw("   ")
    };

    let label_style = if is_selected {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let label_span = Span::styled(display_label.clone(), label_style);

    // Spacer between label and shortcut
    let spacer_width = label_width.saturating_sub(display_label.len());
    let spacer = Span::raw(" ".repeat(spacer_width));

    let shortcut_span = Span::styled(
        format!("  {}", shortcut),
        Style::default().fg(Color::DarkGray),
    );

    Line::from(vec![prefix, label_span, spacer, shortcut_span])
}

/// Render the footer with keyboard hints.
fn render_footer(frame: &mut Frame, area: Rect) {
    let footer = Paragraph::new(Line::from(vec![
        Span::styled("  ↑↓", Style::default().fg(Color::DarkGray)),
        Span::styled(" navigate", Style::default().fg(Color::DarkGray)),
        Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
        Span::styled("enter", Style::default().fg(Color::DarkGray)),
        Span::styled(" execute", Style::default().fg(Color::DarkGray)),
        Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
        Span::styled("esc", Style::default().fg(Color::DarkGray)),
        Span::styled(" close", Style::default().fg(Color::DarkGray)),
    ]));
    frame.render_widget(footer, area);
}

/// Create a centered rectangle within the given area.
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(area);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}
