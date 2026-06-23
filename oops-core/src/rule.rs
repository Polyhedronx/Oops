use crate::{Command, CorrectedCommand};

/// Trait that every correction rule must implement.
pub trait Rule: Send + Sync {
    /// Unique name for this rule (e.g., "git_not_command").
    fn name(&self) -> &'static str;

    /// Check whether this rule applies to the given command.
    fn match_command(&self, command: &Command) -> bool;

    /// Generate one or more corrected commands.
    fn get_new_command(&self, command: &Command) -> Vec<CorrectedCommand>;

    /// Whether this rule requires error output to function.
    /// If true and no output is available, `match_command` is skipped.
    fn requires_output(&self) -> bool {
        true
    }

    /// Priority: lower values are tried first and shown first.
    /// Default: 1000
    fn priority(&self) -> i32 {
        1000
    }

    /// Whether this rule is enabled by default when `rules = "all"`.
    fn enabled_by_default(&self) -> bool {
        true
    }
}

// ---------------------------------------------------------------------------
// Rule combinators — composable wrappers (replace Python's decorators)
// ---------------------------------------------------------------------------

/// Wrapper: only applies the inner rule when the command's first word
/// matches one of the given application names (e.g., "git", "hub").
pub struct AppOnly<T: Rule> {
    inner: T,
    apps: &'static [&'static str],
}

impl<T: Rule> AppOnly<T> {
    pub fn new(inner: T, apps: &'static [&'static str]) -> Self {
        Self { inner, apps }
    }
}

impl<T: Rule> Rule for AppOnly<T> {
    fn name(&self) -> &'static str {
        self.inner.name()
    }

    fn match_command(&self, command: &Command) -> bool {
        let is_app = command
            .script_parts()
            .first()
            .map(|first| self.apps.contains(&first.as_str()))
            .unwrap_or(false);
        is_app && self.inner.match_command(command)
    }

    fn get_new_command(&self, command: &Command) -> Vec<CorrectedCommand> {
        self.inner.get_new_command(command)
    }

    fn requires_output(&self) -> bool {
        self.inner.requires_output()
    }

    fn priority(&self) -> i32 {
        self.inner.priority()
    }

    fn enabled_by_default(&self) -> bool {
        self.inner.enabled_by_default()
    }
}

/// Wrapper: also generates sudo-prefixed versions of each correction.
/// Useful for rules that fix permission errors.
pub struct SudoCapable<T: Rule> {
    inner: T,
}

impl<T: Rule> SudoCapable<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }
}

impl<T: Rule> Rule for SudoCapable<T> {
    fn name(&self) -> &'static str {
        self.inner.name()
    }

    fn match_command(&self, command: &Command) -> bool {
        self.inner.match_command(command)
    }

    fn get_new_command(&self, command: &Command) -> Vec<CorrectedCommand> {
        let mut results = self.inner.get_new_command(command);
        let mut sudo_results: Vec<CorrectedCommand> = results
            .iter()
            .map(|cmd| {
                let mut sudo_cmd = cmd.clone();
                sudo_cmd.script = format!("sudo {}", cmd.script);
                sudo_cmd.description = Some(format!(
                    "sudo {}",
                    cmd.description.as_deref().unwrap_or(&cmd.script)
                ));
                // Sudo versions have slightly lower priority (higher number)
                sudo_cmd.priority += 1;
                sudo_cmd
            })
            .collect();
        results.append(&mut sudo_results);
        results
    }

    fn requires_output(&self) -> bool {
        self.inner.requires_output()
    }

    fn priority(&self) -> i32 {
        self.inner.priority()
    }

    fn enabled_by_default(&self) -> bool {
        self.inner.enabled_by_default()
    }
}

// ---------------------------------------------------------------------------
// Helper functions for rule authors
// ---------------------------------------------------------------------------

/// Check if the command's first word matches any of the given app names.
pub fn is_app(command: &Command, apps: &[&str]) -> bool {
    command
        .script_parts()
        .first()
        .map(|first| apps.contains(&first.as_str()))
        .unwrap_or(false)
}

/// Check if any of the given patterns appear in the command output.
/// Case-insensitive.
pub fn output_contains_any(command: &Command, patterns: &[&str]) -> bool {
    command.output.as_ref().is_some_and(|out| {
        let lower = out.to_lowercase();
        patterns.iter().any(|p| lower.contains(&p.to_lowercase()))
    })
}

/// Check if the command script contains the given word.
pub fn script_contains(command: &Command, word: &str) -> bool {
    command.script_parts().contains(&word.to_string())
}

/// Check if the command script starts with the given prefix.
pub fn script_starts_with(command: &Command, prefix: &str) -> bool {
    command.script.starts_with(prefix)
}
