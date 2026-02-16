//! Input area rendering with multi-line support.

use crate::input::InputState;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// Minimum height for the input area (in lines).
pub const MIN_INPUT_HEIGHT: u16 = 3;

/// Maximum height for the input area as fraction of screen (30%).
pub const MAX_INPUT_FRACTION: f32 = 0.30;

/// Calculate the desired height for the input area based on content.
pub fn calculate_input_height(input: &InputState, available_height: u16) -> u16 {
    let content_lines = input.line_count() as u16;
    // +2 for border and some padding
    let desired = content_lines + 2;
    let max_height = ((available_height as f32) * MAX_INPUT_FRACTION) as u16;

    desired.clamp(MIN_INPUT_HEIGHT, max_height.max(MIN_INPUT_HEIGHT))
}

/// Render the input area with multi-line support.
pub fn render_input(
    input: &InputState,
    waiting: bool,
    frame: &mut Frame,
    area: Rect,
) {
    let input_block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = input_block.inner(area);
    frame.render_widget(input_block, area);

    // Build lines with prompt on first line only
    let content = input.content();
    let mut lines: Vec<Line> = Vec::new();

    if content.is_empty() {
        // Empty input - just show prompt
        let prompt_style = if waiting {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default().fg(Color::Cyan)
        };
        lines.push(Line::from(Span::styled("> ", prompt_style)));
    } else {
        // Multi-line content
        for (i, line_text) in content.split('\n').enumerate() {
            if i == 0 {
                // First line gets prompt
                lines.push(Line::from(vec![
                    Span::styled("> ", Style::default().fg(Color::Cyan)),
                    Span::raw(line_text.to_string()),
                ]));
            } else {
                // Continuation lines get indent
                lines.push(Line::from(vec![
                    Span::styled("  ", Style::default().fg(Color::DarkGray)),
                    Span::raw(line_text.to_string()),
                ]));
            }
        }
    }

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(paragraph, inner);

    // Position cursor (only if not waiting)
    if !waiting {
        let (cursor_line, cursor_col) = input.cursor_position();
        // +2 for prompt "> "
        let cursor_x = inner.x + 2 + cursor_col as u16;
        let cursor_y = inner.y + cursor_line as u16;

        // Only set cursor if within visible area
        if cursor_y < inner.y + inner.height {
            frame.set_cursor_position((cursor_x, cursor_y));
        }
    }
}
