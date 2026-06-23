use oops_core::command::Command;
use oops_core::corrected_command::CorrectedCommand;
use oops_core::rule::Rule;

/// Matches when a command is not found in the system, but can be installed.
pub struct NoCommand;

impl Rule for NoCommand {
    fn name(&self) -> &'static str {
        "no_command"
    }

    fn match_command(&self, command: &Command) -> bool {
        command.output.as_ref().is_some_and(|out| {
            let lower = out.to_lowercase();
            lower.contains("command not found")
                || lower.contains("no such file")
                || lower.contains("is not recognized")
        })
    }

    fn get_new_command(&self, _command: &Command) -> Vec<CorrectedCommand> {
        // Just return empty — we can't suggest install commands generically.
        // Specific rules (apt_get, brew, etc.) handle installation suggestions.
        vec![]
    }

    fn priority(&self) -> i32 {
        3000
    }

    fn requires_output(&self) -> bool {
        true
    }
}
