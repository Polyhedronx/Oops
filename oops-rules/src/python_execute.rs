use oops_core::command::Command;
use oops_core::corrected_command::CorrectedCommand;
use oops_core::rule::Rule;

/// Add `python` prefix when executing a .py file directly.
pub struct PythonExecute;

impl Rule for PythonExecute {
    fn name(&self) -> &'static str {
        "python_execute"
    }

    fn match_command(&self, command: &Command) -> bool {
        command
            .script_parts()
            .first()
            .is_some_and(|first| {
                first.ends_with(".py")
                    && command.output.as_ref().is_some_and(|out| {
                        let lower = out.to_lowercase();
                        lower.contains("permission denied")
                            || lower.contains("command not found")
                            || lower.contains("cannot execute")
                    })
            })
    }

    fn get_new_command(&self, command: &Command) -> Vec<CorrectedCommand> {
        vec![CorrectedCommand::new(
            format!("python {}", command.script),
            self.name(),
            900,
            Some("Add 'python' to execute the .py script".into()),
        )]
    }

    fn priority(&self) -> i32 {
        900
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_python_execute_match() {
        let cmd = Command::new("script.py", Some("Permission denied".into()));
        assert!(PythonExecute.match_command(&cmd));
        assert_eq!(PythonExecute.get_new_command(&cmd)[0].script, "python script.py");
    }

    #[test]
    fn test_python_execute_no_match() {
        let cmd = Command::new("script.py", None);
        assert!(!PythonExecute.match_command(&cmd));
    }
}
