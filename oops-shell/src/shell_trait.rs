/// Trait representing a shell (bash, zsh, powershell, etc.).
pub trait Shell: Send + Sync {
    /// Human-readable name of this shell.
    fn name(&self) -> &'static str;

    /// Generate the shell function/alias that wraps oops.
    /// This is the code users `eval` in their shell config.
    fn app_alias(&self, alias_name: &str) -> String;

    /// Generate the instant mode alias (modifies PS1/PROMPT to capture output).
    fn instant_mode_alias(&self, alias_name: &str) -> String;

    /// Split a command string into parts using shell-aware tokenization.
    fn split_command(&self, script: &str) -> Vec<String>;

    /// Quote a string for safe shell usage.
    fn quote(&self, s: &str) -> String;

    /// Get the command to retrieve shell history.
    fn history_command(&self) -> &str;

    /// Print instructions for configuring this shell.
    fn how_to_configure(&self) -> ShellConfiguration {
        ShellConfiguration {
            content: "eval \"$(oops --alias)\"".to_string(),
            path: self.config_file().to_string(),
            reload: format!("source {}", self.config_file()),
        }
    }

    /// Get the default config file path for this shell.
    fn config_file(&self) -> &str;

    /// Generate the shell code to put a corrected command into history.
    fn history_put(&self, script: &str) -> String;
}

/// Instructions for configuring a shell to use oops.
#[derive(Debug, Clone)]
pub struct ShellConfiguration {
    /// Content to add to the shell config file.
    pub content: String,
    /// Path to the config file.
    pub path: String,
    /// Command to reload the config.
    pub reload: String,
}
