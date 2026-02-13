//! Main layout rendering.

use crate::app::App;
use crate::focus::FocusTarget;
use crate::client::ConnectionStatus;
use crate::ui::chat::render_chat;
use crate::ui::input::{calculate_input_height, render_input as render_input_area};
use crate::ui::logs::{render_logs_footer, render_logs_panel};
use crate::ui::palette::render_palette_overlay as render_palette;
use crate::ui::sessions::render_sessions_overlay as render_sessions;
use crate::ui::sidebar::{render_sidebar, SIDEBAR_HINT_WIDTH, SIDEBAR_WIDTH};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Render the entire application UI.
pub fn render(app: &App, frame: &mut Frame) {
    let area = frame.area();

    // Determine sidebar width based on open/closed state
    // Always show at least a hint when closed
    let sidebar_width = if app.sidebar.is_open() {
        SIDEBAR_WIDTH
    } else {
        SIDEBAR_HINT_WIDTH
    };

    // Build horizontal layout: [sidebar hint/panel] [main] [logs]
    // Sidebar always has some presence (hint when closed, full panel when open)
    let (sidebar_area, main_area, logs_area) = if app.show_logs {
        let chunks = Layout::horizontal([
            Constraint::Length(sidebar_width),
            Constraint::Min(30),        // Main area (minimum width)
            Constraint::Percentage(35), // Logs panel
        ])
        .split(area);
        (Some(chunks[0]), chunks[1], Some(chunks[2]))
    } else {
        let chunks = Layout::horizontal([
            Constraint::Length(sidebar_width),
            Constraint::Min(30),
        ])
        .split(area);
        (Some(chunks[0]), chunks[1], None)
    };

    // Render sidebar if visible
    if let Some(sidebar_area) = sidebar_area {
        render_sidebar(&app.sidebar, frame, sidebar_area);
    }

    // Calculate dynamic input height based on content
    let available_for_input = main_area.height.saturating_sub(2); // Minus header and status
    let input_height = calculate_input_height(&app.input, available_for_input);

    // Main layout: header, content, input, status
    let chunks = Layout::vertical([
        Constraint::Length(1),            // Header
        Constraint::Min(3),               // Content (chat area)
        Constraint::Length(input_height), // Input (dynamic)
        Constraint::Length(1),            // Status bar
    ])
    .split(main_area);

    render_header(app, frame, chunks[0]);
    render_content(app, frame, chunks[1]);
    render_input(app, frame, chunks[2]);
    render_status_bar(app, frame, chunks[3]);

    // Render logs panel if visible
    if let Some(logs_area) = logs_area {
        let logs_chunks = Layout::vertical([
            Constraint::Min(3),    // Logs content
            Constraint::Length(1), // Footer
        ])
        .split(logs_area);

        render_logs_panel(&app.log_buffer, app.log_scroll, frame, logs_chunks[0]);
        render_logs_footer(frame, logs_chunks[1]);
    }

    // Render overlays on top
    match app.focus.current() {
        FocusTarget::Sessions => render_sessions_overlay(app, frame, area),
        FocusTarget::Workstreams => render_workstreams_overlay(app, frame, area),
        FocusTarget::CommandPalette => render_command_palette(app, frame, area),
        _ => {}
    }
}

/// Render the header bar.
fn render_header(app: &App, frame: &mut Frame, area: Rect) {
    let title = Span::styled(" arawn ", Style::default().add_modifier(Modifier::BOLD));

    // Connection status indicator
    let (status_text, status_color) = match app.connection_status {
        ConnectionStatus::Connected => ("●", Color::Green),
        ConnectionStatus::Connecting => ("◐", Color::Yellow),
        ConnectionStatus::Reconnecting { .. } => ("◐", Color::Yellow),
        ConnectionStatus::Disconnected => ("○", Color::Red),
    };
    let status = Span::styled(format!("{} ", status_text), Style::default().fg(status_color));

    // Context indicator (if set)
    let context_span = if let Some(ref ctx) = app.context_name {
        Span::styled(format!("ctx:{} ", ctx), Style::default().fg(Color::DarkGray))
    } else {
        Span::raw("")
    };

    let workstream = Span::styled(
        format!("ws:{} ", app.workstream),
        Style::default().fg(Color::DarkGray),
    );

    // Create spans that fill the line
    let right_width = status.width() + context_span.width() + workstream.width();
    let spacer_width = area.width.saturating_sub(title.width() as u16 + right_width as u16);
    let spacer = Span::raw("─".repeat(spacer_width as usize));

    let line = Line::from(vec![title, spacer, status, context_span, workstream]);
    let header = Paragraph::new(line).style(Style::default().fg(Color::Cyan));

    frame.render_widget(header, area);
}

/// Render the main content area (chat messages).
fn render_content(app: &App, frame: &mut Frame, area: Rect) {
    render_chat(app, frame, area);
}

/// Render the input area.
fn render_input(app: &App, frame: &mut Frame, area: Rect) {
    render_input_area(&app.input, app.waiting, frame, area);
}

/// Render the status bar.
fn render_status_bar(app: &App, frame: &mut Frame, area: Rect) {
    let left_text = if let Some(ref msg) = app.status_message {
        msg.clone()
    } else if app.waiting {
        "Thinking...".to_string()
    } else if app.focus.is(FocusTarget::Sidebar) {
        use crate::sidebar::SidebarSection;
        let section = match app.sidebar.section {
            SidebarSection::Workstreams => "workstreams",
            SidebarSection::Sessions => "sessions",
        };
        format!("[{}] ↑↓ navigate │ Tab switch │ Enter select │ n new │ → close", section)
    } else {
        "^K palette │ ^W sidebar │ ^L logs │ ^Q quit".to_string()
    };

    // Show connection status on the right if not connected
    let right_text = match app.connection_status {
        ConnectionStatus::Connected => String::new(),
        status => format!(" {} ", status),
    };

    let left = Span::styled(&left_text, Style::default().fg(Color::DarkGray));
    let spacer_width = area
        .width
        .saturating_sub(left_text.len() as u16 + right_text.len() as u16);
    let spacer = Span::raw(" ".repeat(spacer_width as usize));
    let right = Span::styled(
        right_text,
        Style::default().fg(Color::Yellow),
    );

    let line = Line::from(vec![left, spacer, right]);
    let status = Paragraph::new(line);

    frame.render_widget(status, area);
}

/// Render the sessions overlay.
fn render_sessions_overlay(app: &App, frame: &mut Frame, area: Rect) {
    render_sessions(&app.sessions, frame, area);
}

/// Render the workstreams overlay.
fn render_workstreams_overlay(_app: &App, frame: &mut Frame, area: Rect) {
    let overlay_area = centered_rect(60, 50, area);
    frame.render_widget(Clear, overlay_area);

    let block = Block::default()
        .title(" workstreams ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let content = Paragraph::new(vec![
        Line::from(" > search..."),
        Line::from(""),
        Line::from(Span::styled(
            " ★ default",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("   project-alpha                          45 msgs  2d"),
        Line::from("   research-notes                         12 msgs  5d"),
    ])
    .block(block);

    frame.render_widget(content, overlay_area);
}

/// Render the command palette.
fn render_command_palette(app: &App, frame: &mut Frame, area: Rect) {
    render_palette(&app.palette, frame, area);
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
