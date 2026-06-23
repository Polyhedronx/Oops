use oops_core::command::Command;
use oops_core::corrected_command::CorrectedCommand;
use oops_core::rule::Rule;

/// Fix common `ls` flag typos like `lsa`, `lsl`, `lsla`.
pub struct LsAll;

impl Rule for LsAll {
    fn name(&self) -> &'static str {
        "ls_all"
    }

    fn match_command(&self, command: &Command) -> bool {
        // Match bare typos like "lsa", "lsl", "lsla", "lsal"
        let script = command.script.trim();
        matches!(script, "lsa" | "lsl" | "lsla" | "lsal" | "lsll")
    }

    fn get_new_command(&self, command: &Command) -> Vec<CorrectedCommand> {
        let suffix = &command.script[2..]; // everything after "ls"
        let mut results = Vec::new();

        // Map common typos
        let fixed = match suffix {
            "a" => "ls -a",
            "l" => "ls -l",
            "la" | "al" => "ls -la",
            "ll" => "ls -ll",
            "all" => "ls -a",
            _ => return vec![],
        };

        results.push(CorrectedCommand::new(
            fixed.into(),
            self.name(),
            self.priority(),
            Some(format!("Replace 'ls{}' with '{}'", suffix, fixed)),
        ));

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
    fn test_lsa() {
        let cmd = Command::new("lsa", None);
        assert!(LsAll.match_command(&cmd));
        assert_eq!(LsAll.get_new_command(&cmd)[0].script, "ls -a");
    }

    #[test]
    fn test_lsl() {
        let cmd = Command::new("lsl", None);
        assert!(LsAll.match_command(&cmd));
        assert_eq!(LsAll.get_new_command(&cmd)[0].script, "ls -l");
    }

    #[test]
    fn test_lsla() {
        let cmd = Command::new("lsla", None);
        assert!(LsAll.match_command(&cmd));
        assert_eq!(LsAll.get_new_command(&cmd)[0].script, "ls -la");
    }

    #[test]
    fn test_normal_ls_not_matched() {
        let cmd = Command::new("ls -la", None);
        assert!(!LsAll.match_command(&cmd));
    }
}
