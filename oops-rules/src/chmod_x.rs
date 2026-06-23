use oops_core::command::Command;
use oops_core::corrected_command::CorrectedCommand;
use oops_core::rule::Rule;

/// Suggest `chmod +x` when a script cannot be executed.
pub struct ChmodX;

impl Rule for ChmodX {
    fn name(&self) -> &'static str {
        "chmod_x"
    }

    fn match_command(&self, command: &Command) -> bool {
        command.output.as_ref().is_some_and(|out| {
            let lower = out.to_lowercase();
            (lower.contains("permission denied") || lower.contains("cannot execute"))
                && !command.script.starts_with("sudo ")
                && !command.script.starts_with("chmod ")
        })
    }

    fn get_new_command(&self, command: &Command) -> Vec<CorrectedCommand> {
        let script_file = command.script_parts().first()
            .cloned()
            .unwrap_or_default();

        vec![
            CorrectedCommand::new(
                format!("chmod +x {} && {}", script_file, command.script),
                self.name(),
                900,
                Some("Make the file executable before running it".into()),
            ),
            CorrectedCommand::new(
                format!("sudo {}", command.script),
                self.name(),
                901,
                Some("Run with sudo".into()),
            ),
        ]
    }

    fn priority(&self) -> i32 {
        900
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chmod_x_match() {
        let cmd = Command::new("./run.sh", Some("Permission denied".into()));
        assert!(ChmodX.match_command(&cmd));
        let results = ChmodX.get_new_command(&cmd);
        assert!(results.iter().any(|c| c.script.contains("chmod +x")));
    }
}
