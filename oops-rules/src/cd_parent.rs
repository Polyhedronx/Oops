use oops_core::command::Command;
use oops_core::corrected_command::CorrectedCommand;
use oops_core::rule::Rule;

/// Fix `cd..` → `cd ..` (forgot the space).
pub struct CdParent;

impl Rule for CdParent {
    fn name(&self) -> &'static str {
        "cd_parent"
    }

    fn match_command(&self, command: &Command) -> bool {
        command.script.trim() == "cd.."
    }

    fn get_new_command(&self, _command: &Command) -> Vec<CorrectedCommand> {
        vec![CorrectedCommand::new(
            "cd ..".into(),
            self.name(),
            self.priority(),
            Some("Insert space after cd".into()),
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
    fn test_cd_parent() {
        let cmd = Command::new("cd..", None);
        assert!(CdParent.match_command(&cmd));
        assert_eq!(CdParent.get_new_command(&cmd)[0].script, "cd ..");
    }

    #[test]
    fn test_cd_with_space_not_matched() {
        let cmd = Command::new("cd ..", None);
        assert!(!CdParent.match_command(&cmd));
    }
}
