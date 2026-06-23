use std::process::{Command as StdCommand, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crate::command::Command;
use crate::config::Config;

/// Re-run a command and capture its output (stdout + stderr).
/// On Windows, uses CREATE_NO_WINDOW. Timeout enforced via a watcher thread.
pub fn rerun_and_capture(script: &str, config: &Config) -> Option<String> {
    let timeout_secs = if config.is_slow_command(script) {
        config.wait_slow_command
    } else {
        config.wait_command
    };
    if timeout_secs == 0 { return None; }
    let timeout = Duration::from_secs(timeout_secs);

    let parts: Vec<&str> = script.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }

    let mut cmd = StdCommand::new(parts[0]);
    if parts.len() > 1 {
        cmd.args(&parts[1..]);
    }
    cmd.stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null());

    // Windows: prevent console window from appearing
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    for (k, v) in &config.env {
        cmd.env(k, v);
    }

    let mut child = cmd.spawn().ok()?;

    // Watcher thread signals when timeout is reached
    let (tx, rx) = mpsc::channel();
    let t = timeout;
    thread::spawn(move || {
        thread::sleep(t);
        let _ = tx.send(());
    });

    // Poll child until it exits or timeout fires
    loop {
        match child.try_wait() {
            Ok(Some(_status)) => {
                let output = child.wait_with_output().ok()?;
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let combined = format!("{}{}", stdout, stderr);
                return if combined.trim().is_empty() { None } else { Some(combined) };
            }
            Ok(None) => {
                if rx.try_recv().is_ok() {
                    let _ = child.kill();
                    let _ = child.wait();
                    return None;
                }
                thread::sleep(Duration::from_millis(50));
            }
            Err(_) => return None,
        }
    }
}

/// Get the command from shell history via environment variables.
/// Returns the last command as a vector of script parts.
pub fn get_command_from_history() -> Option<Vec<String>> {
    // Read TF_HISTORY first (set by bash/zsh aliases), then OOPS_HISTORY
    let history = std::env::var("TF_HISTORY")
        .or_else(|_| std::env::var(crate::consts::ENV_HISTORY))
        .ok()?;

    let lines: Vec<&str> = history.lines().rev().collect();
    for line in lines {
        let trimmed = line.trim();
        if !trimmed.is_empty()
            && !trimmed.starts_with("oops")
            && !trimmed.starts_with("fuck")
            && !trimmed.contains("command oops")
        {
            return Some(parse_script(trimmed));
        }
    }
    None
}

/// Parse a script string into parts (same as command splitting).
fn parse_script(script: &str) -> Vec<String> {
    crate::command::split_script(script)
}

/// Format raw script parts into a command string.
pub fn format_raw_script(raw_script: &[String]) -> String {
    raw_script.join(" ")
}

/// Create a Command from raw script parts and config.
///
/// In standard mode: re-runs the command to capture error output.
/// In **instant mode** (OOPS_INSTANT_MODE set): reads the error output from the
/// shell logger's ring-buffer log file, avoiding the need to re-run.
pub fn create_command(raw_script: &[String], config: &Config) -> Command {
    let script = format_raw_script(raw_script);

    // Instant mode: read output from the shell logger's log file.
    let output = if is_instant_mode() {
        instant_mode_log_path()
            .and_then(|p| read_output_from_log(&p, &script))
    } else {
        None
    };

    // Fall back to re-running if instant mode produced nothing.
    let output = output.or_else(|| rerun_and_capture(&script, config));
    Command::new(script, output)
}

/// Check whether instant mode is active (OOPS_INSTANT_MODE is set).
pub fn is_instant_mode() -> bool {
    std::env::var(crate::consts::ENV_INSTANT_MODE).is_ok()
}

/// Get the instant mode log file path from the environment.
pub fn instant_mode_log_path() -> Option<std::path::PathBuf> {
    std::env::var(crate::consts::ENV_OUTPUT_LOG)
        .ok()
        .map(std::path::PathBuf::from)
}

/// (Unix) Read command error output from the instant-mode shell logger log file.
///
/// The log is a ring-buffer filled with raw PTY output.  We strip ANSI escape
/// codes, search for the command text, and extract the lines after it (until
/// the next prompt-like line or end of file).
#[cfg(unix)]
pub fn read_output_from_log(log_path: &std::path::Path, script: &str) -> Option<String> {
    use std::io::Read;

    let mut file = std::fs::File::open(log_path).ok()?;
    let mut data = Vec::new();
    file.read_to_end(&mut data).ok()?;

    // Strip null bytes (ring-buffer zero-fill).
    data.retain(|&b| b != 0);
    let raw = String::from_utf8_lossy(&data);

    // Remove ANSI CSI and OSC escape sequences.
    let re = regex::Regex::new("\x1b\\[[0-9;]*[A-Za-z]|\x1b\\].*?\x07").ok()?;
    let clean = re.replace_all(&raw, "");

    // Find the command in the cleaned output.
    let idx = clean.find(script)?;

    // Extract from after the command line until a prompt-like line or EOF.
    let after: &str = &clean[idx + script.len()..];
    let lines: Vec<&str> = after
        .lines()
        .map(|l| l.trim())
        .skip_while(|l| l.is_empty())
        .take_while(|l| !looks_like_prompt(l))
        .collect();

    let output = lines.join("\n").trim().to_string();
    if output.is_empty() { None } else { Some(output) }
}

/// Heuristic: does this line look like a shell prompt?
#[cfg(unix)]
fn looks_like_prompt(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return false;
    }
    trimmed.ends_with("$ ")
        || trimmed.ends_with("# ")
        || trimmed.ends_with("❯ ")
        || trimmed.ends_with("> ")
        || trimmed.ends_with("$")
        || trimmed.ends_with("#")
        || trimmed.contains("\x1b]777;oops") // our instant-mode PS1 marker
}

/// Instant mode is Unix-only.
#[cfg(not(unix))]
pub fn read_output_from_log(_log_path: &std::path::Path, _script: &str) -> Option<String> {
    None
}

/// Get the alias name from environment or default.
pub fn get_alias() -> String {
    std::env::var(crate::consts::ENV_ALIAS).unwrap_or_else(|_| "oops".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_raw_script() {
        let parts = vec!["git".into(), "push".into()];
        assert_eq!(format_raw_script(&parts), "git push");
    }

    #[test]
    fn test_get_alias_default() {
        unsafe { std::env::remove_var(crate::consts::ENV_ALIAS) };
        // Just check it returns something
        let alias = get_alias();
        assert!(!alias.is_empty());
    }
}
