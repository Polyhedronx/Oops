use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// High-level actions that the TUI can perform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Move selection down one item
    SelectNext,
    /// Move selection up one item
    SelectPrevious,
    /// Scroll preview panel down
    ScrollPreviewDown,
    /// Scroll preview panel up
    ScrollPreviewUp,
    /// Go to filter mode (start typing)
    EnterFilterMode,
    /// Insert a character into the filter text
    InsertChar(char),
    /// Delete character at cursor (backspace)
    DeleteChar,
    /// Delete character forward (delete)
    DeleteForward,
    /// Move cursor left in filter
    CursorLeft,
    /// Move cursor right in filter
    CursorRight,
    /// Clear the entire filter
    ClearFilter,
    /// Confirm the current selection (execute)
    Confirm,
    /// Abort (quit without selecting)
    Abort,
    /// Toggle the preview panel
    TogglePreview,
    /// Resize event
    Resize(u16, u16),
    /// No action (tick, unknown key)
    Noop,
}

/// Map a key event to an action based on the current mode.
pub fn key_to_action(key: KeyEvent, is_filtering: bool) -> Action {
    if is_filtering {
        return filter_mode_key(key);
    }

    match key.code {
        KeyCode::Up | KeyCode::Char('k') => Action::SelectPrevious,
        KeyCode::Down | KeyCode::Char('j') => Action::SelectNext,
        KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::SelectNext,
        KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::SelectPrevious,
        KeyCode::Enter => Action::Confirm,
        KeyCode::Char('q') => Action::Abort,
        KeyCode::Esc => Action::Abort,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::Abort,
        KeyCode::Tab => Action::TogglePreview,
        KeyCode::Char('/') => Action::EnterFilterMode,
        KeyCode::PageDown => Action::ScrollPreviewDown,
        KeyCode::PageUp => Action::ScrollPreviewUp,
        // Printable characters enter filter mode directly
        KeyCode::Char(ch) if ch.is_ascii_alphanumeric() => {
            Action::InsertChar(ch)
        }
        _ => Action::Noop,
    }
}

fn filter_mode_key(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc => Action::ClearFilter,
        KeyCode::Enter => Action::Confirm,
        KeyCode::Backspace => Action::DeleteChar,
        KeyCode::Delete => Action::DeleteForward,
        KeyCode::Left => Action::CursorLeft,
        KeyCode::Right => Action::CursorRight,
        KeyCode::Down => Action::SelectNext,
        KeyCode::Up => Action::SelectPrevious,
        KeyCode::Char(ch) if !ch.is_ascii_control() => Action::InsertChar(ch),
        _ => Action::Noop,
    }
}
