//! Chat view rendering with streaming support.

use crate::app::{App, ChatMessage, ToolExecution};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
    Frame,
};

/// Streaming cursor indicator.
const STREAMING_CURSOR: &str = "▌";

/// Render the chat view with all messages.
pub fn render_chat(app: &App, frame: &mut Frame, area: Rect) {
    // If no messages, show welcome screen
    if app.messages.is_empty() {
        render_welcome(frame, area);
        return;
    }

    let mut lines = Vec::new();

    for (i, msg) in app.messages.iter().enumerate() {
        // Add spacing between messages (except before first)
        if i > 0 {
            lines.push(Line::from(""));
        }

        if msg.is_user {
            // User message with > prefix
            render_user_message(&mut lines, msg);
        } else {
            // Assistant message
            render_assistant_message(&mut lines, msg, area.width as usize);

            // Render tool executions after the current assistant message.
            // Tools are stored globally in app.tools and cleared when a new
            // user message is sent (see App::send_message). This means tools
            // always represent the current response cycle.
            //
            // Note: During rapid streaming updates, tools may briefly appear
            // unordered as ToolStart/ToolOutput/ToolEnd events arrive. This is
            // a visual timing artifact, not data corruption. Each tool maintains
            // its own state correctly via tool_id correlation.
            if msg.streaming || (i == app.messages.len() - 1 && !app.tools.is_empty()) {
                render_tools(&mut lines, &app.tools);
            }
        }
    }

    // Calculate scroll position
    let content_height = lines.len();
    let view_height = area.height as usize;

    let scroll_offset = if app.chat_auto_scroll {
        // Auto-scroll: show the bottom of the content
        content_height.saturating_sub(view_height)
    } else {
        // Manual scroll: use the stored offset, clamped to valid range
        app.chat_scroll.min(content_height.saturating_sub(view_height))
    };

    let chat = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((scroll_offset as u16, 0));

    frame.render_widget(chat, area);
}

/// Render user message with > prefix.
fn render_user_message(lines: &mut Vec<Line<'static>>, msg: &ChatMessage) {
    let prefix = Span::styled("> ", Style::default().fg(Color::Cyan));
    let content = Span::styled(
        msg.content.clone(),
        Style::default().fg(Color::White),
    );
    lines.push(Line::from(vec![prefix, content]));
}

/// Render assistant message with word wrapping and streaming cursor.
fn render_assistant_message(lines: &mut Vec<Line<'static>>, msg: &ChatMessage, _width: usize) {
    let content = if msg.streaming {
        format!("{}{}", msg.content, STREAMING_CURSOR)
    } else {
        msg.content.clone()
    };

    // Split content by newlines and add each as a line
    for line_text in content.lines() {
        let style = if msg.streaming {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::Gray)
        };
        lines.push(Line::from(Span::styled(line_text.to_string(), style)));
    }

    // Handle trailing newline or empty content
    if (content.ends_with('\n') || content.is_empty()) && msg.streaming {
        lines.push(Line::from(Span::styled(
            STREAMING_CURSOR.to_string(),
            Style::default().fg(Color::White),
        )));
    }
}

/// Render tool executions between messages.
fn render_tools(lines: &mut Vec<Line<'static>>, tools: &[ToolExecution]) {
    if tools.is_empty() {
        return;
    }

    lines.push(Line::from(""));

    for tool in tools {
        // Tool header line
        let status_indicator = if tool.running {
            Span::styled("◐ ", Style::default().fg(Color::Yellow))
        } else if tool.success == Some(true) {
            Span::styled("✓ ", Style::default().fg(Color::Green))
        } else {
            Span::styled("✗ ", Style::default().fg(Color::Red))
        };

        let tool_name = Span::styled(
            format!("[{}]", tool.name),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );

        lines.push(Line::from(vec![
            Span::raw("  "),
            status_indicator,
            tool_name,
        ]));

        // Show truncated output preview if there's output
        if !tool.output.is_empty() {
            let line_count = tool.output.lines().count();
            let preview = tool.output.lines().next().unwrap_or("");

            // Build the preview string with truncation indicator
            let mut display = if preview.len() > 60 {
                format!("{}...", &preview[..57])
            } else {
                preview.to_string()
            };

            // Add indicator if there are more lines
            if line_count > 1 {
                display.push_str(&format!(" (+{} more lines)", line_count - 1));
            }

            lines.push(Line::from(Span::styled(
                format!("    {}", display),
                Style::default().fg(Color::DarkGray),
            )));
        }
    }
}

/// Render the welcome screen when there are no messages.
fn render_welcome(frame: &mut Frame, area: Rect) {
    let content = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            "  Welcome to Arawn TUI",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("  Type a message and press Enter to send."),
        Line::from(""),
        Line::from(Span::styled(
            "  Keyboard shortcuts:",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("    Ctrl+K  Command palette"),
        Line::from("    Ctrl+S  Sessions"),
        Line::from("    Ctrl+W  Workstreams"),
        Line::from("    Ctrl+E  Tool output pane"),
        Line::from("    Ctrl+Q  Quit"),
        Line::from(""),
        Line::from(Span::styled(
            "  Chat navigation:",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("    ↑/↓     Scroll chat history"),
        Line::from("    PgUp    Scroll up one page"),
        Line::from("    PgDn    Scroll down one page"),
        Line::from("    Home    Scroll to top"),
        Line::from("    End     Scroll to bottom (auto-scroll)"),
    ])
    .wrap(Wrap { trim: false });

    frame.render_widget(content, area);
}
