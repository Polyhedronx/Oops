use oops_core::command::Command;
use oops_core::corrected_command::CorrectedCommand;
use oops_core::rule::Rule;

pub struct Sudo;

impl Rule for Sudo {
    fn name(&self) -> &'static str {
        "sudo"
    }

    fn match_command(&self, command: &Command) -> bool {
        command.output.as_ref().is_some_and(|out| {
            let lower = out.to_lowercase();
            lower.contains("permission denied")
                || lower.contains("not permitted")
                || lower.contains("eacces")
                || lower.contains("are you root")
        })
    }

    fn get_new_command(&self, command: &Command) -> Vec<CorrectedCommand> {
        vec![CorrectedCommand::new(
            format!("sudo {}", command.script),
            self.name(),
            500,
            Some("Prepend sudo to run with elevated permissions".into()),
        )]
    }

    fn priority(&self) -> i32 {
        500
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sudo_permission_denied() {
        let cmd = Command::new("apt install nginx", Some("Permission denied".into()));
        let rule = Sudo;
        assert!(rule.match_command(&cmd));
        let results = rule.get_new_command(&cmd);
        assert_eq!(results[0].script, "sudo apt install nginx");
    }

    #[test]
    fn test_sudo_no_match() {
        let cmd = Command::new("ls", Some("file not found".into()));
        let rule = Sudo;
        assert!(!rule.match_command(&cmd));
    }
}
