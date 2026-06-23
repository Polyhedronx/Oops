use std::sync::OnceLock;

/// Represents a failed command with its error output.
#[derive(Debug, Clone)]
pub struct Command {
    /// The original command string (e.g., "git pish")
    pub script: String,

    /// The error output from the shell (stdout + stderr combined)
    pub output: Option<String>,

    /// Lazily computed split of script into parts (shell-aware)
    script_parts: OnceLock<Vec<String>>,
}

impl Command {
    pub fn new(script: impl Into<String>, output: Option<String>) -> Self {
        Self {
            script: script.into(),
            output,
            script_parts: OnceLock::new(),
        }
    }

    /// Returns the command script split into parts (words/tokens).
    /// Splits by whitespace, respecting quotes (single and double).
    pub fn script_parts(&self) -> &[String] {
        self.script_parts.get_or_init(|| split_script(&self.script))
    }

    /// Create from raw script parts (e.g., from history or CLI args).
    pub fn from_raw_script(raw_script: &[String]) -> Self {
        let script = raw_script.join(" ");
        Self::new(script, None)
    }

    /// Update this command with new fields, returning a new Command.
    pub fn update(&self, script: Option<String>, output: Option<String>) -> Self {
        Self {
            script: script.unwrap_or_else(|| self.script.clone()),
            output: output.or_else(|| self.output.clone()),
            script_parts: OnceLock::new(),
        }
    }
}

/// Split a command string into shell-like tokens, handling quotes.
pub fn split_script(script: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut chars = script.chars().peekable();
    let mut in_single_quote = false;
    let mut in_double_quote = false;

    while let Some(&ch) = chars.peek() {
        match ch {
            '\'' if !in_double_quote => {
                chars.next();
                in_single_quote = !in_single_quote;
            }
            '"' if !in_single_quote => {
                chars.next();
                in_double_quote = !in_double_quote;
            }
            ' ' | '\t' if !in_single_quote && !in_double_quote => {
                chars.next();
                if !current.is_empty() {
                    parts.push(std::mem::take(&mut current));
                }
                // Skip additional whitespace
                while chars.peek().is_some_and(|c| *c == ' ' || *c == '\t') {
                    chars.next();
                }
            }
            _ => {
                current.push(ch);
                chars.next();
            }
        }
    }

    if !current.is_empty() {
        parts.push(current);
    }

    parts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_simple() {
        let parts = split_script("git push origin main");
        assert_eq!(parts, vec!["git", "push", "origin", "main"]);
    }

    #[test]
    fn test_split_with_quotes() {
        let parts = split_script(r#"git commit -m "fix bug""#);
        assert_eq!(parts, vec!["git", "commit", "-m", "fix bug"]);
    }

    #[test]
    fn test_split_with_single_quotes() {
        let parts = split_script("echo 'hello world'");
        assert_eq!(parts, vec!["echo", "hello world"]);
    }

    #[test]
    fn test_script_parts_cached() {
        let cmd = Command::new("git push", None);
        let parts1 = cmd.script_parts().to_vec();
        let parts2 = cmd.script_parts().to_vec();
        assert_eq!(parts1, parts2);
        assert_eq!(parts1, vec!["git", "push"]);
    }

    #[test]
    fn test_split_script() {
        // Re-test to ensure the function is accessible
        let parts = split_script("git push");
        assert_eq!(parts, vec!["git", "push"]);
    }
}
