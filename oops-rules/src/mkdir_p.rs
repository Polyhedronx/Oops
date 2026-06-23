use oops_core::command::Command;
use oops_core::corrected_command::CorrectedCommand;
use oops_core::rule::Rule;

pub struct MkdirP;

impl Rule for MkdirP {
    fn name(&self) -> &'static str {
        "mkdir_p"
    }

    fn match_command(&self, command: &Command) -> bool {
        command.script.starts_with("mkdir ")
            && command.output.as_ref().is_some_and(|out| {
                out.contains("No such file or directory")
            })
    }

    fn get_new_command(&self, command: &Command) -> Vec<CorrectedCommand> {
        vec![CorrectedCommand::new(
            command.script.replacen("mkdir ", "mkdir -p ", 1),
            self.name(),
            800,
            Some("Add -p to mkdir to create parent directories".into()),
        )]
    }

    fn priority(&self) -> i32 {
        800
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mkdir_p_match() {
        let cmd = Command::new(
            "mkdir /foo/bar/baz",
            Some("mkdir: cannot create directory '/foo/bar/baz': No such file or directory".into()),
        );
        let rule = MkdirP;
        assert!(rule.match_command(&cmd));
        assert_eq!(rule.get_new_command(&cmd)[0].script, "mkdir -p /foo/bar/baz");
    }

    #[test]
    fn test_mkdir_p_no_match() {
        let cmd = Command::new("mkdir test", Some("".into()));
        assert!(!MkdirP.match_command(&cmd));
    }
}
