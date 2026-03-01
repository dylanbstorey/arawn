//! Sidebar panel rendering for workstreams and sessions.

// Context usage thresholds (canonical source: arawn_types::config::defaults)
const CONTEXT_WARNING_PERCENT: u8 = 70;
const CONTEXT_CRITICAL_PERCENT: u8 = 90;

use crate::sessions::format_relative_time;
use crate::sidebar::{Sidebar, SidebarSection};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

/// Width of the expanded sidebar (when open).
pub const SIDEBAR_WIDTH: u16 = 28;
/// Width of the closed sidebar hint.
pub const SIDEBAR_HINT_WIDTH: u16 = 2;

/// Render the sidebar panel based on open/closed state.
pub fn render_sidebar(sidebar: &Sidebar, frame: &mut Frame, area: Rect) {
    if sidebar.is_open() {
        render_open_sidebar(sidebar, frame, area);
    } else {
        render_closed_hint(frame, area);
    }
}

/// Render the closed sidebar hint (minimal indicator).
fn render_closed_hint(frame: &mut Frame, area: Rect) {
    // Just a thin vertical line with a hint character
    let hint = Paragraph::new(vec![Line::from(Span::styled(
        "│",
        Style::default().fg(Color::DarkGray),
    ))]);
    frame.render_widget(hint, area);
}

/// Render the open sidebar with full content (has focus).
fn render_open_sidebar(sidebar: &Sidebar, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::RIGHT)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Split into workstreams section and sessions section
    let chunks = Layout::vertical([
        Constraint::Length(1), // Workstreams header
        Constraint::Length(6), // Workstreams list (show ~5 items)
        Constraint::Length(1), // Sessions header
        Constraint::Min(3),    // Sessions list
        Constraint::Length(1), // Footer
    ])
    .split(inner);

    render_workstreams_header(sidebar, frame, chunks[0]);
    render_workstreams_list(sidebar, frame, chunks[1]);
    render_sessions_header(sidebar, frame, chunks[2]);
    render_sessions_list(sidebar, frame, chunks[3]);
    render_sidebar_footer(frame, chunks[4]);
}

/// Render the workstreams section header.
fn render_workstreams_header(sidebar: &Sidebar, frame: &mut Frame, area: Rect) {
    let style = if sidebar.section == SidebarSection::Workstreams {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let header = Paragraph::new(Line::from(Span::styled("WORKSTREAMS", style)));
    frame.render_widget(header, area);
}

/// Render the workstreams list.
fn render_workstreams_list(sidebar: &Sidebar, frame: &mut Frame, area: Rect) {
    let mut lines = Vec::new();

    // Render active workstreams
    for (is_selected, ws) in sidebar.visible_workstreams() {
        lines.push(render_workstream_line(
            sidebar,
            ws,
            is_selected,
            area.width,
            false,
        ));
    }

    // Render archived section if there are archived workstreams
    if sidebar.has_archived_workstreams() {
        // Add separator
        let separator_width = area.width.saturating_sub(2) as usize;
        let label = " archived ";
        let dash_count = separator_width.saturating_sub(label.len()) / 2;
        let separator = format!(
            "{}{}{}",
            "─".repeat(dash_count),
            label,
            "─".repeat(separator_width.saturating_sub(dash_count + label.len()))
        );
        lines.push(Line::from(Span::styled(
            separator,
            Style::default().fg(Color::DarkGray),
        )));

        // Render archived workstreams
        for (is_selected, ws) in sidebar.visible_archived_workstreams() {
            lines.push(render_workstream_line(
                sidebar,
                ws,
                is_selected,
                area.width,
                true,
            ));
        }
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "  Loading...",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let list = Paragraph::new(lines);
    frame.render_widget(list, area);
}

/// Render a single workstream line.
fn render_workstream_line(
    sidebar: &Sidebar,
    ws: &crate::sidebar::WorkstreamEntry,
    is_selected: bool,
    width: u16,
    is_archived: bool,
) -> Line<'static> {
    let prefix = if ws.is_current { "● " } else { "  " };

    // Visual distinction for scratch workstreams
    let name_display = if ws.is_scratch {
        format!("⚡{}", ws.name) // Lightning bolt for scratch
    } else {
        ws.name.clone()
    };

    // Format usage if available
    let usage_str = if let Some(usage) = ws.usage_bytes {
        format_size(usage)
    } else {
        String::new()
    };

    // Calculate available width for name
    let usage_width = if usage_str.is_empty() {
        0
    } else {
        usage_str.len() + 1
    };
    let name_width = (width as usize).saturating_sub(3 + usage_width); // 3 = prefix + space
    let name = truncate_str(&name_display, name_width);

    // Style based on state
    let style = if is_archived {
        // Archived workstreams: dimmed and italic
        if is_selected && sidebar.section == SidebarSection::Workstreams {
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC | Modifier::BOLD)
        } else {
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC)
        }
    } else if is_selected && sidebar.section == SidebarSection::Workstreams {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else if ws.is_current {
        Style::default().add_modifier(Modifier::BOLD)
    } else if ws.is_scratch {
        Style::default().fg(Color::Yellow) // Scratch workstreams in yellow
    } else {
        Style::default()
    };

    // Active workstream gets a dark dot indicator
    let prefix_style = Style::default().fg(Color::DarkGray);

    // Usage color based on percentage
    let usage_color = if let (Some(usage), Some(limit)) = (ws.usage_bytes, ws.limit_bytes) {
        let percent = (usage as f64 / limit as f64 * 100.0) as u8;
        if percent >= CONTEXT_CRITICAL_PERCENT {
            Color::Red
        } else if percent >= CONTEXT_WARNING_PERCENT {
            Color::Yellow
        } else {
            Color::DarkGray
        }
    } else {
        Color::DarkGray
    };

    let mut spans = vec![
        Span::styled(prefix.to_string(), prefix_style),
        Span::styled(name, style),
    ];

    if !usage_str.is_empty() {
        spans.push(Span::styled(
            format!(" {}", usage_str),
            Style::default().fg(usage_color),
        ));
    }

    Line::from(spans)
}

/// Format byte size as human-readable string.
fn format_size(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 {
        format!("{:.1}G", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 {
        format!("{}M", bytes / (1024 * 1024))
    } else if bytes >= 1024 {
        format!("{}K", bytes / 1024)
    } else {
        format!("{}B", bytes)
    }
}

/// Render the sessions section header.
fn render_sessions_header(sidebar: &Sidebar, frame: &mut Frame, area: Rect) {
    let style = if sidebar.section == SidebarSection::Sessions {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    // Show workstream name in header
    let ws_name = sidebar
        .selected_workstream()
        .map(|ws| ws.name.as_str())
        .unwrap_or("SESSIONS");

    let header_text = format!("SESSIONS ({})", ws_name);
    let header = Paragraph::new(Line::from(Span::styled(
        truncate_str(&header_text, area.width as usize),
        style,
    )));
    frame.render_widget(header, area);
}

/// Render the sessions list.
fn render_sessions_list(sidebar: &Sidebar, frame: &mut Frame, area: Rect) {
    let mut lines = Vec::new();

    // Always show "+ New Session" at the top (index 0)
    let new_session_selected =
        sidebar.is_new_session_selected() && sidebar.section == SidebarSection::Sessions;
    let new_session_style = if new_session_selected {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    lines.push(Line::from(vec![
        Span::styled("+ ", new_session_style),
        Span::styled("New Session", new_session_style),
    ]));

    // Render actual sessions (indices 1+)
    for (is_selected, session) in sidebar.visible_sessions() {
        let prefix = if session.is_current { "● " } else { "  " };
        let time_str = format_relative_time(session.last_active);
        let title_width = (area.width as usize).saturating_sub(prefix.len() + time_str.len() + 1);
        let title = truncate_str(&session.title, title_width);

        let style = if is_selected && sidebar.section == SidebarSection::Sessions {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else if session.is_current {
            Style::default().add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        // Active session gets a dark dot indicator (consistent with workstreams)
        let prefix_style = Style::default().fg(Color::DarkGray);

        // Calculate spacing
        let spacer_width = title_width.saturating_sub(title.len());
        let spacer = " ".repeat(spacer_width);

        lines.push(Line::from(vec![
            Span::styled(prefix, prefix_style),
            Span::styled(title, style),
            Span::raw(spacer),
            Span::styled(
                format!(" {}", time_str),
                Style::default().fg(Color::DarkGray),
            ),
        ]));
    }

    let list = Paragraph::new(lines);
    frame.render_widget(list, area);
}

/// Render the sidebar footer with keybinding hints.
fn render_sidebar_footer(frame: &mut Frame, area: Rect) {
    let footer = Paragraph::new(Line::from(vec![
        Span::styled("Tab", Style::default().fg(Color::DarkGray)),
        Span::styled(" switch", Style::default().fg(Color::DarkGray)),
    ]));
    frame.render_widget(footer, area);
}

/// Truncate a string to fit within the given width.
fn truncate_str(s: &str, max_width: usize) -> String {
    if s.len() <= max_width {
        s.to_string()
    } else if max_width <= 3 {
        s.chars().take(max_width).collect()
    } else {
        format!("{}...", &s[..max_width.saturating_sub(3)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_str() {
        assert_eq!(truncate_str("hello", 10), "hello");
        assert_eq!(truncate_str("hello world", 8), "hello...");
        assert_eq!(truncate_str("hi", 2), "hi");
        assert_eq!(truncate_str("hello", 3), "hel");
    }
}
