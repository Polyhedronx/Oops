use oops_core::command::Command;
use oops_core::corrected_command::CorrectedCommand;
use oops_core::rule::Rule;

/// Matches commands where the output says "unknown command" but no more specific rule matches.
pub struct UnknownCommand;

impl Rule for UnknownCommand {
    fn name(&self) -> &'static str {
        "unknown_command"
    }

    fn match_command(&self, command: &Command) -> bool {
        command.output.as_ref().is_some_and(|out| {
            let lower = out.to_lowercase();
            lower.contains("unknown command")
                || lower.contains("no such command")
                || lower.contains("command not found")
        })
    }

    fn get_new_command(&self, _command: &Command) -> Vec<CorrectedCommand> {
        // This is a catch-all — specific tools should have their own rules.
        vec![]
    }

    fn priority(&self) -> i32 {
        5000
    }
}
