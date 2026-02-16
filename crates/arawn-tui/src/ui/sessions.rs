//! Session list overlay rendering.

use crate::sessions::{format_relative_time, SessionList};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Render the sessions overlay.
pub fn render_sessions_overlay(
    sessions: &SessionList,
    frame: &mut Frame,
    area: Rect,
) {
    // Create centered overlay (60% width, 60% height)
    let overlay_area = centered_rect(60, 60, area);
    frame.render_widget(Clear, overlay_area);

    let block = Block::default()
        .title(" sessions ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(overlay_area);
    frame.render_widget(block, overlay_area);

    // Split into search box, list, and footer
    let chunks = Layout::vertical([
        Constraint::Length(1), // Search
        Constraint::Length(1), // Separator
        Constraint::Min(3),    // List
        Constraint::Length(1), // Footer
    ])
    .split(inner);

    render_search_box(sessions, frame, chunks[0]);
    render_separator(frame, chunks[1]);
    render_session_list(sessions, frame, chunks[2]);
    render_footer(frame, chunks[3]);
}

/// Render the search/filter box.
fn render_search_box(sessions: &SessionList, frame: &mut Frame, area: Rect) {
    let filter = sessions.filter();
    let prompt = if filter.is_empty() {
        Span::styled(" > search...", Style::default().fg(Color::DarkGray))
    } else {
        Span::styled(format!(" > {}", filter), Style::default().fg(Color::White))
    };
    let search = Paragraph::new(Line::from(prompt));
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

/// Render the session list.
fn render_session_list(sessions: &SessionList, frame: &mut Frame, area: Rect) {
    let mut lines = Vec::new();

    if sessions.is_loading() {
        lines.push(Line::from(Span::styled(
            "  Loading...",
            Style::default().fg(Color::DarkGray),
        )));
    } else if sessions.visible_count() == 0 {
        if sessions.filter().is_empty() {
            lines.push(Line::from(Span::styled(
                "  No sessions yet",
                Style::default().fg(Color::DarkGray),
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "  Press Ctrl+N to create a new session",
                Style::default().fg(Color::DarkGray),
            )));
        } else {
            lines.push(Line::from(Span::styled(
                "  No matching sessions",
                Style::default().fg(Color::DarkGray),
            )));
        }
    } else {
        for (is_selected, session) in sessions.visible_sessions() {
            let line = format_session_line(session, is_selected, area.width as usize);
            lines.push(line);
        }
    }

    let list = Paragraph::new(lines);
    frame.render_widget(list, area);
}

/// Format a single session line.
fn format_session_line(
    session: &crate::sessions::SessionSummary,
    is_selected: bool,
    width: usize,
) -> Line<'static> {
    // Calculate available width for title
    let time_str = format_relative_time(session.last_active);
    let prefix_width = 4; // " • " or "   "
    let time_width = time_str.len() + 2; // padding
    let title_width = width.saturating_sub(prefix_width + time_width);

    // Truncate title if needed
    let title = if session.title.len() > title_width {
        format!("{}...", &session.title[..title_width.saturating_sub(3)])
    } else {
        session.title.clone()
    };

    // Build the line
    let prefix = if session.is_current {
        Span::styled(" • ", Style::default().fg(Color::Cyan))
    } else {
        Span::raw("   ")
    };

    let title_style = if is_selected {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else if session.is_current {
        Style::default().add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let title_span = Span::styled(title.clone(), title_style);

    // Spacer between title and time
    let spacer_width = title_width.saturating_sub(title.len());
    let spacer = Span::raw(" ".repeat(spacer_width));

    let time_span = Span::styled(
        format!("  {}", time_str),
        Style::default().fg(Color::DarkGray),
    );

    Line::from(vec![prefix, title_span, spacer, time_span])
}

/// Render the footer with keyboard hints.
fn render_footer(frame: &mut Frame, area: Rect) {
    let footer = Paragraph::new(Line::from(vec![
        Span::styled("  ↑↓", Style::default().fg(Color::DarkGray)),
        Span::styled(" navigate", Style::default().fg(Color::DarkGray)),
        Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
        Span::styled("enter", Style::default().fg(Color::DarkGray)),
        Span::styled(" select", Style::default().fg(Color::DarkGray)),
        Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
        Span::styled("^N", Style::default().fg(Color::DarkGray)),
        Span::styled(" new", Style::default().fg(Color::DarkGray)),
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
