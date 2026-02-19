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
use crate::ui::tools::{render_tool_pane, render_tool_pane_footer};
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

    // Check for active disk warnings to show banner
    let has_disk_warning = !app.disk_warnings.is_empty();
    let warning_height = if has_disk_warning { 1 } else { 0 };

    // Calculate dynamic input height based on content
    let available_for_input = main_area.height.saturating_sub(2 + warning_height); // Minus header, status, warning
    let input_height = calculate_input_height(&app.input, available_for_input);

    // Main layout: header, [warning], content, input, status
    let chunks = if has_disk_warning {
        Layout::vertical([
            Constraint::Length(1),            // Header
            Constraint::Length(1),            // Warning banner
            Constraint::Min(3),               // Content (chat area)
            Constraint::Length(input_height), // Input (dynamic)
            Constraint::Length(1),            // Status bar
        ])
        .split(main_area)
    } else {
        Layout::vertical([
            Constraint::Length(1),            // Header
            Constraint::Min(3),               // Content (chat area)
            Constraint::Length(input_height), // Input (dynamic)
            Constraint::Length(1),            // Status bar
        ])
        .split(main_area)
    };

    if has_disk_warning {
        render_header(app, frame, chunks[0]);
        render_warning_banner(app, frame, chunks[1]);
        render_content(app, frame, chunks[2]);
        render_input(app, frame, chunks[3]);
        render_status_bar(app, frame, chunks[4]);
    } else {
        render_header(app, frame, chunks[0]);
        render_content(app, frame, chunks[1]);
        render_input(app, frame, chunks[2]);
        render_status_bar(app, frame, chunks[3]);
    }

    // Render command popup above input if visible
    if app.command_popup.is_visible() {
        app.command_popup.render(frame, chunks[2]);
    }

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

    // Render usage popup if visible (Ctrl+U)
    if app.show_usage_popup {
        render_usage_popup(app, frame, area);
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

    // Workstream with usage indicator
    let (workstream_text, usage_span) = if let Some(ref usage) = app.workstream_usage {
        let ws_prefix = if usage.is_scratch { "⚡" } else { "" };
        let ws_text = format!("{}ws:{} ", ws_prefix, app.workstream);

        // Format usage: [~120MB/1GB] or [120MB] if no limit
        let usage_text = if usage.limit_bytes > 0 {
            format!("[~{}/{}] ", usage.total_size(), usage.limit_size())
        } else {
            format!("[~{}] ", usage.total_size())
        };

        // Color based on usage percentage
        let usage_color = if usage.percent >= 90 {
            Color::Red
        } else if usage.percent >= 70 {
            Color::Yellow
        } else {
            Color::DarkGray
        };

        (ws_text, Span::styled(usage_text, Style::default().fg(usage_color)))
    } else {
        (format!("ws:{} ", app.workstream), Span::raw(""))
    };

    let workstream = Span::styled(workstream_text, Style::default().fg(Color::DarkGray));

    // Create spans that fill the line
    let right_width = status.width() + context_span.width() + workstream.width() + usage_span.width();
    let spacer_width = area.width.saturating_sub(title.width() as u16 + right_width as u16);
    let spacer = Span::raw("─".repeat(spacer_width as usize));

    let line = Line::from(vec![title, spacer, status, context_span, workstream, usage_span]);
    let header = Paragraph::new(line).style(Style::default().fg(Color::Cyan));

    frame.render_widget(header, area);
}

/// Render the main content area (chat messages + optional tool pane).
fn render_content(app: &App, frame: &mut Frame, area: Rect) {
    if app.show_tool_pane {
        // Split vertically: chat (top 70%), tool pane (bottom 30%)
        let chunks = Layout::vertical([
            Constraint::Percentage(70),
            Constraint::Percentage(30),
        ])
        .split(area);

        render_chat(app, frame, chunks[0]);

        // Tool pane with footer
        let tool_chunks = Layout::vertical([
            Constraint::Min(3),
            Constraint::Length(1),
        ])
        .split(chunks[1]);

        render_tool_pane(app, frame, tool_chunks[0]);
        render_tool_pane_footer(frame, tool_chunks[1]);
    } else {
        render_chat(app, frame, area);
    }
}

/// Render the input area.
fn render_input(app: &App, frame: &mut Frame, area: Rect) {
    let read_only = !app.is_session_owner;
    render_input_area(&app.input, app.waiting, read_only, frame, area);
}

/// Render the status bar.
fn render_status_bar(app: &App, frame: &mut Frame, area: Rect) {
    let left_text = if let Some(ref msg) = app.status_message {
        msg.clone()
    } else if app.waiting {
        "Thinking...".to_string()
    } else if app.focus.is(FocusTarget::ToolPane) {
        "←→ tools │ ↑↓ scroll │ ^O editor │ Esc close".to_string()
    } else if app.focus.is(FocusTarget::Sidebar) {
        use crate::sidebar::SidebarSection;
        let section = match app.sidebar.section {
            SidebarSection::Workstreams => "workstreams",
            SidebarSection::Sessions => "sessions",
        };
        format!("[{}] ↑↓ navigate │ Tab switch │ Enter select │ n new │ → close", section)
    } else {
        "^K palette │ ^W sidebar │ ^E tools │ ^L logs │ ^Q quit".to_string()
    };

    // Build right side with context info and connection status
    let mut right_spans: Vec<Span> = Vec::new();

    // Add context indicator if available
    if let Some(ref ctx) = app.context_info {
        let (ctx_text, ctx_color) = format_context_indicator(ctx);
        right_spans.push(Span::styled(ctx_text, Style::default().fg(ctx_color)));
        right_spans.push(Span::raw(" "));
    }

    // Show connection status if not connected
    match app.connection_status {
        ConnectionStatus::Connected => {}
        status => {
            right_spans.push(Span::styled(
                format!(" {} ", status),
                Style::default().fg(Color::Yellow),
            ));
        }
    }

    // Calculate widths for spacing
    let right_text_len: usize = right_spans.iter().map(|s| s.width()).sum();
    let left = Span::styled(&left_text, Style::default().fg(Color::DarkGray));
    let spacer_width = area
        .width
        .saturating_sub(left_text.len() as u16 + right_text_len as u16);
    let spacer = Span::raw(" ".repeat(spacer_width as usize));

    let mut spans = vec![left, spacer];
    spans.extend(right_spans);

    let line = Line::from(spans);
    let status = Paragraph::new(line);

    frame.render_widget(status, area);
}

/// Format the context indicator with appropriate color.
fn format_context_indicator(ctx: &crate::app::ContextState) -> (String, Color) {
    let color = match ctx.status.as_str() {
        "ok" => Color::Green,
        "warning" => Color::Yellow,
        "critical" => Color::Red,
        _ => Color::Gray,
    };

    // Format as "[Context: XX%]" or with token details
    let text = if ctx.max_tokens > 0 {
        let current_k = ctx.current_tokens / 1000;
        let max_k = ctx.max_tokens / 1000;
        format!("[~{}k/{}k {}%]", current_k, max_k, ctx.percent)
    } else {
        format!("[Context: {}%]", ctx.percent)
    };

    (text, color)
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

/// Render the disk warning banner.
fn render_warning_banner(app: &App, frame: &mut Frame, area: Rect) {
    if let Some(warning) = app.disk_warnings.first() {
        let (icon, color) = if warning.level == "critical" {
            ("⛔", Color::Red)
        } else {
            ("⚠", Color::Yellow)
        };

        let usage_str = crate::app::UsageStats::format_size(warning.usage_bytes);
        let limit_str = crate::app::UsageStats::format_size(warning.limit_bytes);

        let text = format!(
            " {} Disk {}: {} at {}% ({}/{})",
            icon, warning.level, warning.workstream, warning.percent, usage_str, limit_str
        );

        let banner = Paragraph::new(Line::from(vec![
            Span::styled(text, Style::default().fg(color).add_modifier(Modifier::BOLD)),
        ]))
        .style(Style::default().bg(Color::DarkGray));

        frame.render_widget(banner, area);
    }
}

/// Render the usage stats popup (Ctrl+U).
fn render_usage_popup(app: &App, frame: &mut Frame, area: Rect) {
    let popup_area = centered_rect(50, 40, area);
    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(" Disk Usage (^U to close) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let mut lines = Vec::new();

    // Current workstream usage
    if let Some(ref usage) = app.workstream_usage {
        let ws_type = if usage.is_scratch { "⚡ scratch" } else { "workstream" };
        lines.push(Line::from(vec![
            Span::styled(
                format!(" {} {} ", ws_type, usage.workstream_name),
                Style::default().add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::raw("   production/  "),
            Span::styled(
                format!("{:>10}", usage.production_size()),
                Style::default().fg(Color::Green),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::raw("   work/        "),
            Span::styled(
                format!("{:>10}", usage.work_size()),
                Style::default().fg(Color::Yellow),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::raw("   ─────────────────────"),
        ]));
        lines.push(Line::from(vec![
            Span::raw("   total        "),
            Span::styled(
                format!("{:>10}", usage.total_size()),
                Style::default().add_modifier(Modifier::BOLD),
            ),
        ]));

        if usage.limit_bytes > 0 {
            let color = if usage.percent >= 90 {
                Color::Red
            } else if usage.percent >= 70 {
                Color::Yellow
            } else {
                Color::Green
            };

            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::raw("   limit        "),
                Span::styled(format!("{:>10}", usage.limit_size()), Style::default().fg(Color::DarkGray)),
            ]));
            lines.push(Line::from(vec![
                Span::raw("   usage        "),
                Span::styled(format!("{:>9}%", usage.percent), Style::default().fg(color)),
            ]));

            // Progress bar
            let bar_width = 20;
            let filled = (usage.percent as usize * bar_width / 100).min(bar_width);
            let empty = bar_width - filled;
            lines.push(Line::from(vec![
                Span::raw("   ["),
                Span::styled("█".repeat(filled), Style::default().fg(color)),
                Span::styled("░".repeat(empty), Style::default().fg(Color::DarkGray)),
                Span::raw("]"),
            ]));
        }
    } else {
        lines.push(Line::from(vec![
            Span::styled(
                format!(" workstream: {} ", app.workstream),
                Style::default().add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "   No usage data available",
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(Span::styled(
            "   (requires server support)",
            Style::default().fg(Color::DarkGray),
        )));
    }

    // Show warnings section if any
    if !app.disk_warnings.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(" Active Warnings ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]));

        for warning in &app.disk_warnings {
            let (icon, color) = if warning.level == "critical" {
                ("⛔", Color::Red)
            } else {
                ("⚠", Color::Yellow)
            };
            lines.push(Line::from(vec![
                Span::styled(format!("   {} ", icon), Style::default().fg(color)),
                Span::raw(format!("{}: {}%", warning.workstream, warning.percent)),
            ]));
        }
    }

    let content = Paragraph::new(lines);
    frame.render_widget(content, inner);
}
