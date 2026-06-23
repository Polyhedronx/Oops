use oops_core::command::Command;
use oops_core::corrected_command::CorrectedCommand;
use oops_core::rule::Rule;

/// Fix `cargo` commands when a subcommand is not found.
pub struct CargoNoCommand;

impl Rule for CargoNoCommand {
    fn name(&self) -> &'static str {
        "cargo_no_command"
    }

    fn match_command(&self, command: &Command) -> bool {
        command.script.starts_with("cargo ")
            && command.output.as_ref().is_some_and(|out| {
                let lower = out.to_lowercase();
                lower.contains("no such command")
                    || lower.contains("unknown command")
                    || lower.contains("error:")
            })
    }

    fn get_new_command(&self, command: &Command) -> Vec<CorrectedCommand> {
        let mut results = Vec::new();

        let fixes = [
            ("buil", "build"),
            ("buiild", "build"),
            ("run ", "run "), // keep as-is
            ("chcek", "check"),
            ("chekc", "check"),
            ("tets", "test"),
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
    fn test_cargo_typo() {
        let cmd = Command::new(
            "cargo buil",
            Some("error: no such command: `buil`".into()),
        );
        assert!(CargoNoCommand.match_command(&cmd));
        let results = CargoNoCommand.get_new_command(&cmd);
        assert_eq!(results[0].script, "cargo build");
    }
}
