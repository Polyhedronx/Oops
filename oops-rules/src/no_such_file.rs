use oops_core::command::Command;
use oops_core::corrected_command::CorrectedCommand;
use oops_core::rule::Rule;

/// Suggests creating missing files referenced in the command output.
pub struct NoSuchFile;

impl Rule for NoSuchFile {
    fn name(&self) -> &'static str {
        "no_such_file"
    }

    fn match_command(&self, command: &Command) -> bool {
        command.output.as_ref().is_some_and(|out| {
            out.contains("No such file or directory")
                && !command.script.starts_with("mkdir ")
                && !command.script.starts_with("cd ")
                && !command.script.starts_with("touch ")
        })
    }

    fn get_new_command(&self, _command: &Command) -> Vec<CorrectedCommand> {
        vec![]
    }

    fn priority(&self) -> i32 {
        3000
    }
}
