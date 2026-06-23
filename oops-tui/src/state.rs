use oops_core::corrected_command::CorrectedCommand;

/// Application mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    /// User is browsing/navigating the correction list
    Selecting,
    /// User is in the filter input bar (typing mode)
    Filtering,
    /// User has confirmed a selection
    Confirmed,
    /// User has aborted
    Aborted,
}

/// The main TUI application state.
pub struct TuiState {
    /// All corrections from the corrector
    pub all_corrections: Vec<CorrectedCommand>,

    /// Indices into all_corrections after filtering
    pub filtered_indices: Vec<usize>,

    /// Currently selected index within filtered_indices
    pub selected_index: usize,

    /// Current filter/search text
    pub filter_text: String,

    /// Cursor position in the filter text
    pub filter_cursor: usize,

    /// Current application mode
    pub mode: AppMode,

    /// Whether the preview panel is visible
    pub show_preview: bool,

    /// Scroll offset for the preview panel
    pub preview_scroll: u16,

    /// Scroll offset for the list panel
    pub list_scroll: usize,

    /// The confirmed correction (set when user selects)
    pub confirmed: Option<CorrectedCommand>,

    /// Terminal width and height
    pub term_width: u16,
    pub term_height: u16,

    /// The original failed command (shown in preview)
    pub original_script: Option<String>,

    /// Whether to keep running the event loop
    pub running: bool,
}

impl TuiState {
    pub fn new(corrections: Vec<CorrectedCommand>, original_script: Option<String>) -> Self {
        let len = corrections.len();
        let indices: Vec<usize> = (0..len).collect();
        Self {
            all_corrections: corrections,
            filtered_indices: indices,
            selected_index: 0,
            filter_text: String::new(),
            filter_cursor: 0,
            mode: AppMode::Selecting,
            show_preview: true,
            preview_scroll: 0,
            list_scroll: 0,
            confirmed: None,
            original_script,
            term_width: 80,
            term_height: 24,
            running: true,
        }
    }

    /// Get the currently selected correction.
    pub fn selected_correction(&self) -> Option<&CorrectedCommand> {
        self.filtered_indices
            .get(self.selected_index)
            .and_then(|&idx| self.all_corrections.get(idx))
    }

    /// Apply nucleo fuzzy filter and update filtered_indices.
    pub fn apply_filter(&mut self) {
        if self.filter_text.is_empty() {
            self.filtered_indices = (0..self.all_corrections.len()).collect();
        } else {
            let mut matcher = nucleo::Matcher::new(nucleo::Config::DEFAULT);
            let mut buf1 = Vec::new();
            let mut buf2 = Vec::new();
            let needle = nucleo::Utf32Str::new(&self.filter_text, &mut buf1);
            let mut scored: Vec<(usize, u16)> = self
                .all_corrections
                .iter()
                .enumerate()
                .filter_map(|(i, cmd)| {
                    let haystack = nucleo::Utf32Str::new(&cmd.script, &mut buf2);
                    matcher
                        .fuzzy_match(haystack, needle)
                        .map(|score| (i, score))
                })
                .collect();

            // Sort by score descending, then by priority ascending (as tiebreaker)
            scored.sort_by(|a, b| {
                b.1.cmp(&a.1).then_with(|| {
                    self.all_corrections[a.0]
                        .priority
                        .cmp(&self.all_corrections[b.0].priority)
                })
            });

            self.filtered_indices = scored.into_iter().map(|(i, _)| i).collect();
        }

        // Reset selection if out of bounds
        if !self.filtered_indices.is_empty() {
            if self.selected_index >= self.filtered_indices.len() {
                self.selected_index = 0;
            }
        } else {
            self.selected_index = 0;
        }
    }

    /// Move selection down.
    pub fn select_next(&mut self) {
        if !self.filtered_indices.is_empty() {
            let len = self.filtered_indices.len();
            self.selected_index = (self.selected_index + 1) % len;
            self.scroll_to_selection();
        }
    }

    /// Move selection up.
    pub fn select_previous(&mut self) {
        if !self.filtered_indices.is_empty() {
            let len = self.filtered_indices.len();
            self.selected_index = if self.selected_index == 0 {
                len - 1
            } else {
                self.selected_index - 1
            };
            self.scroll_to_selection();
        }
    }

    /// Ensure the selected item is visible in the list scroll area.
    fn scroll_to_selection(&mut self) {
        let visible_height = (self.term_height as usize).saturating_sub(5); // minus borders, input, status
        if self.selected_index < self.list_scroll {
            self.list_scroll = self.selected_index;
        } else if self.selected_index >= self.list_scroll + visible_height {
            self.list_scroll = self.selected_index.saturating_sub(visible_height - 1);
        }
    }

    /// Confirm the current selection.
    pub fn confirm(&mut self) {
        if let Some(cmd) = self.selected_correction().cloned() {
            self.confirmed = Some(cmd);
            self.mode = AppMode::Confirmed;
            self.running = false;
        }
    }

    /// Abort the selection.
    pub fn abort(&mut self) {
        self.mode = AppMode::Aborted;
        self.running = false;
    }

    /// Toggle preview panel visibility.
    pub fn toggle_preview(&mut self) {
        self.show_preview = !self.show_preview;
    }

    /// Scroll the preview panel.
    pub fn scroll_preview_up(&mut self) {
        self.preview_scroll = self.preview_scroll.saturating_sub(1);
    }

    pub fn scroll_preview_down(&mut self) {
        self.preview_scroll = self.preview_scroll.saturating_add(1);
    }
}
