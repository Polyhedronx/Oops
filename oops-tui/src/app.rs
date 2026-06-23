use std::io;
use std::time::Duration;

use crossterm::{
    event::{self, Event as CEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, layout::Rect, Terminal};

use crate::components::{input_bar, list_panel, preview_panel, status_bar};
use crate::event::{key_to_action, Action};
use crate::state::{AppMode, TuiState};

/// Run the interactive TUI and return the user's selection.
pub fn run(
    corrections: Vec<oops_core::corrected_command::CorrectedCommand>,
    original_script: Option<String>,
) -> io::Result<Option<oops_core::corrected_command::CorrectedCommand>> {
    // Setup terminal — use stderr so shell alias doesn't capture TUI output
    enable_raw_mode()?;
    let mut stderr = io::stderr();
    execute!(stderr, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal, corrections, original_script);

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stderr>>,
    corrections: Vec<oops_core::corrected_command::CorrectedCommand>,
    original_script: Option<String>,
) -> io::Result<Option<oops_core::corrected_command::CorrectedCommand>> {
    let mut state = TuiState::new(corrections, original_script);

    // Drain buffered input before starting event loop
    while event::poll(Duration::from_millis(0)).unwrap_or(false) {
        let _ = event::read();
    }

    // Main event loop
    while state.running {
        // Draw
        if terminal.draw(|f| {
            let term_size = f.area();
            state.term_width = term_size.width;
            state.term_height = term_size.height;

            // Layout: main (list+preview), input (3 rows for bordered), status (1 row)
            let input_h = 3u16;
            let status_h = 1u16;
            let bottom_h = input_h + status_h;
            let main_h = term_size.height.saturating_sub(bottom_h);

            let main_area = Rect { x: 0, y: 0, width: term_size.width, height: main_h };
            let input_area = Rect { x: 0, y: main_h, width: term_size.width, height: input_h };
            let status_area = Rect { x: 0, y: main_h + input_h, width: term_size.width, height: status_h };

            if term_size.height < 12 || term_size.width < 40 {
                let msg = ratatui::widgets::Paragraph::new(
                    "Terminal too small. Please resize to at least 40x10.",
                );
                f.render_widget(msg, main_area);
                return;
            }

            let (list_area, preview_area) = if state.show_preview {
                let list_width = (main_area.width as f32 * 0.55) as u16;
                (Rect { x: main_area.x, y: main_area.y, width: list_width, height: main_area.height },
                 Some(Rect { x: main_area.x + list_width, y: main_area.y, width: main_area.width.saturating_sub(list_width), height: main_area.height }))
            } else {
                (main_area, None)
            };

            list_panel::render_list(f, &state, list_area);
            if let Some(prev_area) = preview_area {
                preview_panel::render_preview(f, state.selected_correction(), state.original_script.as_deref(), state.preview_scroll, prev_area);
            }
            input_bar::render_input(f, &state, input_area);
            status_bar::render_status(f, &state, status_area);
        }).is_err() {
            break; // draw failed — exit gracefully
        }

        // Handle input — don't exit on transient errors
        match event::poll(Duration::from_millis(100)) {
            Ok(true) => {
                if let Ok(CEvent::Key(key)) = event::read() {
                    // Only process Press events; ignore Release to avoid double-input
                    if key.kind == crossterm::event::KeyEventKind::Release {
                        continue;
                    }
                    let action = key_to_action(key, state.mode == AppMode::Filtering);
                    match action {
                        Action::SelectNext => state.select_next(),
                        Action::SelectPrevious => state.select_previous(),
                        Action::ScrollPreviewDown => state.scroll_preview_down(),
                        Action::ScrollPreviewUp => state.scroll_preview_up(),
                        Action::Confirm => state.confirm(),
                        Action::Abort => state.abort(),
                        Action::TogglePreview => state.toggle_preview(),
                        Action::EnterFilterMode => state.mode = AppMode::Filtering,
                        Action::ClearFilter => {
                            state.filter_text.clear();
                            state.filter_cursor = 0;
                            state.mode = AppMode::Selecting;
                            state.apply_filter();
                        }
                        Action::InsertChar(ch) => {
                            if state.mode != AppMode::Filtering {
                                state.mode = AppMode::Filtering;
                            }
                            state.filter_text.insert(state.filter_cursor, ch);
                            state.filter_cursor += 1;
                            state.apply_filter();
                        }
                        Action::DeleteChar => {
                            if state.filter_cursor > 0 {
                                state.filter_cursor -= 1;
                                state.filter_text.remove(state.filter_cursor);
                                state.apply_filter();
                            }
                        }
                        Action::DeleteForward => {
                            if state.filter_cursor < state.filter_text.len() {
                                state.filter_text.remove(state.filter_cursor);
                                state.apply_filter();
                            }
                        }
                        Action::CursorLeft => {
                            if state.filter_cursor > 0 { state.filter_cursor -= 1; }
                        }
                        Action::CursorRight => {
                            if state.filter_cursor < state.filter_text.len() { state.filter_cursor += 1; }
                        }
                        Action::Noop => {}
                        Action::Resize(w, h) => {
                            state.term_width = w;
                            state.term_height = h;
                        }
                    }
                }
            }
            Ok(false) => {}, // no event, continue loop
            Err(_) => {}     // transient error, continue loop
        }
    }

    Ok(state.confirmed)
}
