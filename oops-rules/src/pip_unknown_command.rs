use oops_core::command::Command;
use oops_core::corrected_command::CorrectedCommand;
use oops_core::rule::Rule;

/// Fix `pip` commands when an unknown subcommand is used.
pub struct PipUnknownCommand;

impl Rule for PipUnknownCommand {
    fn name(&self) -> &'static str {
        "pip_unknown_command"
    }

    fn match_command(&self, command: &Command) -> bool {
        (command.script.starts_with("pip ")
            || command.script.starts_with("pip3 "))
            && command.output.as_ref().is_some_and(|out| {
                let lower = out.to_lowercase();
                lower.contains("unknown command")
                    || lower.contains("no such command")
            })
    }

    fn get_new_command(&self, command: &Command) -> Vec<CorrectedCommand> {
        let mut results = Vec::new();

        let fixes = [
            ("istall", "install"),
            ("insatll", "install"),
            ("unistall", "uninstall"),
            ("upgarde", "upgrade"),
            ("list ", "list "),
            ("intall", "install"),
        ];

        for (typo, correct) in &fixes {
            if typo != correct && command.script.contains(typo) {
                results.push(CorrectedCommand::new(
                    command.script.replace(typo, correct),
                    self.name(),
                    1000,
                    Some(format!("Fix typo: {} -> {}", typo, correct)),
                ));
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pip_typo() {
        let cmd = Command::new(
            "pip istall requests",
            Some("ERROR: unknown command \"istall\"".into()),
        );
        assert!(PipUnknownCommand.match_command(&cmd));
        let results = PipUnknownCommand.get_new_command(&cmd);
        assert_eq!(results[0].script, "pip install requests");
    }
}
