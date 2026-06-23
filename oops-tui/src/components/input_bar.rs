use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::state::{AppMode, TuiState};

/// Render the filter/search input bar.
pub fn render_input(f: &mut Frame, state: &TuiState, area: Rect) {
    let is_active = state.mode == AppMode::Filtering;

    let border_style = if is_active {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(if is_active { " Filter (type to search) " } else { " Filter (/ to search) " })
        .border_style(border_style);

    let _inner_area = block.inner(area);

    // Build the filter text display with cursor
    let filter_spans: Vec<Span> = if state.filter_text.is_empty() && !is_active {
        vec![Span::styled(
            "Type / to start filtering...",
            Style::default().fg(Color::DarkGray),
        )]
    } else if state.filter_text.is_empty() && is_active {
        vec![Span::styled(
            " ",
            Style::default()
                .bg(Color::Yellow)
                .fg(Color::Black),
        )]
    } else {
        let mut spans = Vec::new();
        for (i, ch) in state.filter_text.chars().enumerate() {
            if is_active && i == state.filter_cursor {
                spans.push(Span::styled(
                    ch.to_string(),
                    Style::default()
                        .bg(Color::Yellow)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD),
                ));
            } else {
                spans.push(Span::styled(
                    ch.to_string(),
                    Style::default().fg(Color::White),
                ));
            }
        }
        // If cursor is at end
        if is_active && state.filter_cursor >= state.filter_text.len() {
            spans.push(Span::styled(
                " ",
                Style::default()
                    .bg(Color::Yellow)
                    .fg(Color::Black),
            ));
        }
        spans
    };

    let paragraph = Paragraph::new(Line::from(filter_spans)).block(block);
    f.render_widget(paragraph, area);
}
