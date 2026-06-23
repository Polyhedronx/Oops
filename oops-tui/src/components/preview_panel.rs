use oops_core::corrected_command::CorrectedCommand;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// Render the preview panel showing details of the selected correction.
pub fn render_preview(
    f: &mut Frame,
    correction: Option<&CorrectedCommand>,
    original_script: Option<&str>,
    scroll: u16,
    area: Rect,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Preview ")
        .border_style(Style::default().fg(Color::Cyan));

    let _inner_area = block.inner(area);

    let content = match correction {
        None => Text::from(vec![Line::from(Span::styled(
            "No matching correction",
            Style::default().fg(Color::DarkGray),
        ))]),
        Some(cmd) => build_preview_content(cmd, original_script),
    };

    let paragraph = Paragraph::new(content)
        .block(block)
        .scroll((scroll, 0))
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

fn build_preview_content(
    cmd: &CorrectedCommand,
    original_script: Option<&str>,
) -> Text<'static> {
    let mut lines: Vec<Line<'static>> = Vec::new();

    // Header
    lines.push(Line::from(vec![
        Span::styled("Rule: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            cmd.rule_name.to_string(),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
    ]));

    // Description
    if let Some(desc) = &cmd.description {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            desc.clone(),
            Style::default().fg(Color::Yellow),
        )]));
    }

    // Priority
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("Priority: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            cmd.priority.to_string(),
            Style::default().fg(Color::White),
        ),
    ]));

    // Corrected command
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "Corrected command:",
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from(Span::styled(
        cmd.script.clone(),
        Style::default().fg(Color::White),
    )));

    // Diff with original
    if let Some(original) = original_script {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            "Diff:",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )]));
        let diff_lines = build_diff(original, &cmd.script);
        lines.extend(diff_lines);
    } else {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Original command: (unknown)",
            Style::default().fg(Color::DarkGray),
        )));
    }

    Text::from(lines)
}

/// Build a simple character-level diff between original and corrected scripts.
fn build_diff(original: &str, corrected: &str) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();

    if original == corrected {
        lines.push(Line::from(Span::styled(
            "(no change)",
            Style::default().fg(Color::DarkGray),
        )));
        return lines;
    }

    // Show original in red (removed), corrected in green (added)
    lines.push(Line::from(vec![
        Span::styled(
            format!("- {}", original),
            Style::default().fg(Color::Red),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled(
            format!("+ {}", corrected),
            Style::default().fg(Color::Green),
        ),
    ]));

    // Show character-level diff using similar crate
    let diff_ops = similar::capture_diff_slices(
        similar::Algorithm::Myers,
        original.as_bytes(),
        corrected.as_bytes(),
    );
    let mut diff_spans: Vec<Span<'static>> = Vec::new();
    diff_spans.push(Span::styled("  detail: ", Style::default().fg(Color::DarkGray)));

    for op in &diff_ops {
        match op {
            similar::DiffOp::Equal { old_index, len, .. } => {
                let start = *old_index;
                let end = start + *len;
                if let Some(text) = original.get(start..end) {
                    diff_spans.push(Span::styled(text.to_string(), Style::default().fg(Color::Gray)));
                }
            }
            similar::DiffOp::Delete { old_index, old_len, .. } => {
                let start = *old_index;
                let end = start + *old_len;
                if let Some(text) = original.get(start..end) {
                    diff_spans.push(Span::styled(
                        text.to_string(),
                        Style::default()
                            .fg(Color::Red)
                            .add_modifier(Modifier::CROSSED_OUT),
                    ));
                }
            }
            similar::DiffOp::Insert { new_index, new_len, .. } => {
                let start = *new_index;
                let end = start + *new_len;
                if let Some(text) = corrected.get(start..end) {
                    diff_spans.push(Span::styled(
                        text.to_string(),
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ));
                }
            }
            similar::DiffOp::Replace { old_index, old_len, new_index, new_len } => {
                let old_start = *old_index;
                let old_end = old_start + *old_len;
                let new_start = *new_index;
                let new_end = new_start + *new_len;
                if let Some(text) = original.get(old_start..old_end) {
                    diff_spans.push(Span::styled(
                        text.to_string(),
                        Style::default()
                            .fg(Color::Red)
                            .add_modifier(Modifier::CROSSED_OUT),
                    ));
                }
                if let Some(text) = corrected.get(new_start..new_end) {
                    diff_spans.push(Span::styled(
                        text.to_string(),
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ));
                }
            }
        }
    }

    if diff_spans.len() > 1 {
        lines.push(Line::from(diff_spans));
    }

    lines
}
