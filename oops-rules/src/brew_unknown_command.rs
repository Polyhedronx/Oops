use oops_core::command::Command;
use oops_core::corrected_command::CorrectedCommand;
use oops_core::rule::Rule;

/// Fix unknown `brew` commands by suggesting similar ones.
pub struct BrewUnknownCommand;

impl Rule for BrewUnknownCommand {
    fn name(&self) -> &'static str {
        "brew_unknown_command"
    }

    fn match_command(&self, command: &Command) -> bool {
        command.script.starts_with("brew ")
            && command.output.as_ref().is_some_and(|out| {
                let lower = out.to_lowercase();
                lower.contains("unknown command")
                    || lower.contains("error")
            })
    }

    fn get_new_command(&self, command: &Command) -> Vec<CorrectedCommand> {
        let mut results = Vec::new();

        // Common brew command typos
        let fixes = [
            ("istall", "install"),
            ("insatll", "install"),
            ("updtae", "update"),
            ("upgarde", "upgrade"),
            ("uniinstall", "uninstall"),
            ("unistall", "uninstall"),
        ];

        for (typo, correct) in &fixes {
            if command.script.contains(typo) {
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
    fn test_brew_typo() {
        let cmd = Command::new(
            "brew istall wget",
            Some("Error: Unknown command: istall".into()),
        );
        assert!(BrewUnknownCommand.match_command(&cmd));
        let results = BrewUnknownCommand.get_new_command(&cmd);
        assert_eq!(results[0].script, "brew install wget");
    }
}
