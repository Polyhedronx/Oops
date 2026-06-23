use oops_core::command::Command;
use oops_core::corrected_command::CorrectedCommand;
use oops_core::rule::Rule;

/// Fix `sl` to `ls`.
pub struct SlLs;

impl Rule for SlLs {
    fn name(&self) -> &'static str {
        "sl_ls"
    }

    fn match_command(&self, command: &Command) -> bool {
        command.script_parts().first().is_some_and(|first| {
            first == "sl" && (command.script_parts().len() == 1
                || command.script_parts().iter().skip(1).all(|a| a.starts_with('-')))
        })
    }

    fn get_new_command(&self, command: &Command) -> Vec<CorrectedCommand> {
        let new_script = command.script.replacen("sl", "ls", 1);
        vec![CorrectedCommand::new(
            new_script,
            self.name(),
            1000,
            Some("Replace 'sl' with 'ls'".into()),
        )]
    }

    fn requires_output(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sl_ls_match() {
        let cmd = Command::new("sl -la", None);
        assert!(SlLs.match_command(&cmd));
        assert_eq!(SlLs.get_new_command(&cmd)[0].script, "ls -la");
    }

    #[test]
    fn test_sl_ls_no_match() {
        let cmd = Command::new("ls -la", None);
        assert!(!SlLs.match_command(&cmd));
    }
}
