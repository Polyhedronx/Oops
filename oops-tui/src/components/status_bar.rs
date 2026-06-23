use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

/// Render the bottom status bar with keybinding hints.
pub fn render_status(f: &mut Frame, state: &crate::state::TuiState, area: Rect) {
    let total = state.filtered_indices.len();
    let current = if total > 0 {
        state.selected_index + 1
    } else {
        0
    };

    let status_style = Style::default()
        .fg(Color::Black)
        .bg(Color::DarkGray);

    let mut spans = vec![
        Span::styled(
            format!(" [{}/{}] ", current, total),
            Style::default().fg(Color::Black).bg(Color::Cyan),
        ),
        Span::styled(" ↑↓/jk ", status_style),
        Span::styled(" Enter:ok ", Style::default().fg(Color::Black).bg(Color::Green)),
        Span::styled(" Esc/q:quit ", Style::default().fg(Color::Black).bg(Color::Red)),
        Span::styled(" /:filter ", status_style),
        Span::styled(" Tab:preview ", Style::default().fg(Color::Black).bg(Color::Magenta)),
    ];
    // Only show scroll hints when preview is visible and has content
    if total > 0 {
        spans.push(Span::styled(" PgUp/PgDn:scroll ", status_style));
    }
    let line = Line::from(spans);

    let paragraph = Paragraph::new(line);
    f.render_widget(paragraph, area);
}
