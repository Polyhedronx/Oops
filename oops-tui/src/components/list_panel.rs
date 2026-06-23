use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::state::TuiState;

/// Render the scrollable, filterable list of corrections.
pub fn render_list(f: &mut Frame, state: &TuiState, area: Rect) {
    let selected_style = Style::default()
        .fg(Color::Black)
        .bg(Color::Cyan)
        .add_modifier(Modifier::BOLD);

    let normal_style = Style::default().fg(Color::White);

    let items: Vec<ListItem> = state
        .filtered_indices
        .iter()
        .skip(state.list_scroll)
        .take(area.height.saturating_sub(2).max(1) as usize) // account for borders
        .enumerate()
        .map(|(visible_idx, &cmd_idx)| {
            let correction = &state.all_corrections[cmd_idx];
            let is_selected = state
                .filtered_indices
                .get(state.selected_index)
                .is_some_and(|&s| s == cmd_idx);

            // Build the line: index + command + rule tag
            let index_str = format!("{:>2}. ", visible_idx + state.list_scroll + 1);
            let rule_str = format!("  [{}]", correction.rule_name);
            let script_str = correction.script.clone();

            let line = if is_selected {
                Line::from(vec![
                    Span::styled(index_str, selected_style),
                    Span::styled(script_str, selected_style),
                    Span::styled(rule_str, Style::default().fg(Color::DarkGray).bg(Color::Cyan)),
                ])
            } else {
                Line::from(vec![
                    Span::styled(index_str, Style::default().fg(Color::DarkGray)),
                    Span::styled(script_str, normal_style),
                    Span::styled(rule_str, Style::default().fg(Color::DarkGray)),
                ])
            };

            ListItem::new(line)
        })
        .collect();

    let title = format!(
        " Corrections [{}/{}] ",
        state.filtered_indices.len(),
        state.all_corrections.len()
    );

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .highlight_style(selected_style);

    f.render_widget(list, area);

    // Show "no matches" if empty
    if state.filtered_indices.is_empty() {
        let no_match = Paragraph::new("No matching corrections.")
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL));
        // Overlay in the center
        let center = ratatui::layout::Rect {
            x: area.x + area.width / 4,
            y: area.y + area.height / 2,
            width: area.width / 2,
            height: 3,
        };
        f.render_widget(no_match, center);
    }
}
