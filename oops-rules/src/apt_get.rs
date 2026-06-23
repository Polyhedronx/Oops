use oops_core::command::Command;
use oops_core::corrected_command::CorrectedCommand;
use oops_core::rule::Rule;

/// Fix `apt-get` to `apt-get` with correct subcommand, or suggest `apt install`.
pub struct AptGet;

impl Rule for AptGet {
    fn name(&self) -> &'static str {
        "apt_get"
    }

    fn match_command(&self, command: &Command) -> bool {
        command.script.starts_with("apt-get ")
            || command.script.starts_with("apt ")
    }

    fn get_new_command(&self, command: &Command) -> Vec<CorrectedCommand> {
        let mut results = Vec::new();

        // Fix common apt-get typos
        if command.script.contains("udpate") {
            results.push(CorrectedCommand::new(
                command.script.replace("udpate", "update"),
                self.name(),
                1000,
                Some("Fix typo: udpate -> update".into()),
            ));
        }
        if command.script.contains("upgarde") || command.script.contains("upgard") {
            results.push(CorrectedCommand::new(
                command.script.replace("upgarde", "upgrade").replace("upgard", "upgrade"),
                self.name(),
                1000,
                Some("Fix typo: upgarde -> upgrade".into()),
            ));
        }
        if command.script.contains("istall") || command.script.contains("insatll") {
            results.push(CorrectedCommand::new(
                command.script.replace("istall", "install").replace("insatll", "install"),
                self.name(),
                1000,
                Some("Fix typo: istall -> install".into()),
            ));
        }

        results
    }

    fn requires_output(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apt_get_update_typo() {
        let cmd = Command::new("apt-get udpate", None);
        assert!(AptGet.match_command(&cmd));
        let results = AptGet.get_new_command(&cmd);
        assert_eq!(results[0].script, "apt-get update");
    }
}
