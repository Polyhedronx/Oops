use oops_core::command::Command;
use oops_core::corrected_command::CorrectedCommand;
use oops_core::rule::Rule;

/// Suggest `rm -rf` when `rm` fails on a directory.
pub struct RmDir;

impl Rule for RmDir {
    fn name(&self) -> &'static str {
        "rm_dir"
    }

    fn match_command(&self, command: &Command) -> bool {
        command.script.starts_with("rm ")
            && !command.script.contains("-r")
            && !command.script.contains("-rf")
            && command.output.as_ref().is_some_and(|out| {
                let lower = out.to_lowercase();
                lower.contains("is a directory")
                    || lower.contains("cannot remove")
            })
    }

    fn get_new_command(&self, command: &Command) -> Vec<CorrectedCommand> {
        vec![CorrectedCommand::new(
            command.script.replacen("rm ", "rm -rf ", 1),
            self.name(),
            1000,
            Some("Add -rf to remove directory recursively".into()),
        )]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rm_dir_match() {
        let cmd = Command::new(
            "rm mydir",
            Some("rm: cannot remove 'mydir': Is a directory".into()),
        );
        assert!(RmDir.match_command(&cmd));
        assert_eq!(RmDir.get_new_command(&cmd)[0].script, "rm -rf mydir");
    }

    #[test]
    fn test_rm_dir_no_match() {
        let cmd = Command::new("rm -rf mydir", Some("".into()));
        assert!(!RmDir.match_command(&cmd));
    }
}
