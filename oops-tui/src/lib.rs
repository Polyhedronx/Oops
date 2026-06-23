pub mod app;
pub mod components;
pub mod event;
pub mod state;

use oops_core::corrected_command::CorrectedCommand;

/// Run the interactive TUI for selecting a correction.
pub fn run_tui(
    corrections: &[CorrectedCommand],
    original_script: Option<&str>,
) -> Result<Option<CorrectedCommand>, Box<dyn std::error::Error>> {
    if corrections.is_empty() {
        return Ok(None);
    }

    // Both stdout and stdin must be terminals for TUI to work
    use std::io::IsTerminal;
    if !std::io::stdin().is_terminal() {
        return Err("stdin is not a terminal (called from shell alias with piped stdin)".into());
    }
    if !std::io::stderr().is_terminal() {
        return Err("stderr is not a terminal".into());
    }

    // Check terminal compatibility before attempting raw mode
    if !is_tui_capable() {
        return Err("terminal does not support raw mode (try Windows Terminal, WezTerm, or Alacritty)".into());
    }

    let owned: Vec<CorrectedCommand> = corrections.to_vec();
    let orig = original_script.map(|s| s.to_string());
    match app::run(owned, orig) {
        Ok(result) => Ok(result),
        Err(e) => Err(Box::new(e)),
    }
}

/// Detect whether the current terminal is likely to support ratatui's raw mode.
/// Returns true for known-good terminals, false for dubious ones.
fn is_tui_capable() -> bool {
    // Known-good terminal indicators
    if std::env::var("WT_SESSION").is_ok() {
        return true; // Windows Terminal
    }
    if let Ok(term) = std::env::var("TERM_PROGRAM") {
        if term.contains("WezTerm") || term.contains("alacritty") {
            return true;
        }
    }
    if let Ok(term) = std::env::var("TERM") {
        // Unix: xterm-256color, screen-256color, tmux-256color etc. support raw mode
        if term.contains("256color") || term.contains("alacritty") || term.contains("kitty") {
            return true;
        }
    }
    // Check if TERMINAL_EMULATOR is set (JetBrains, VS Code)
    if std::env::var("TERMINAL_EMULATOR").is_ok() {
        return false; // JetBrains IDE terminal — raw mode unreliable
    }
    if std::env::var("VSCODE_INJECTION").is_ok() {
        return false; // VS Code built-in terminal
    }

    // On Unix, assume capable if TERM is set
    #[cfg(unix)]
    {
        return std::env::var("TERM").is_ok();
    }

    // On Windows, only WT_SESSION terminals are known-good
    #[cfg(windows)]
    {
        return false;
    }

    #[allow(unreachable_code)]
    { false }
}
