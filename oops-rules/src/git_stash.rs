use oops_core::command::Command;
use oops_core::corrected_command::CorrectedCommand;
use oops_core::rule::Rule;

/// When switching branches fails because of uncommitted local changes,
/// suggest stashing first or force-checkout.
pub struct GitStash;

impl Rule for GitStash {
    fn name(&self) -> &'static str {
        "git_stash"
    }

    fn match_command(&self, command: &Command) -> bool {
        if !command.script.starts_with("git checkout ")
            && !command.script.starts_with("git switch ")
            && !command.script.starts_with("git rebase ")
        {
            return false;
        }
        command.output.as_ref().is_some_and(|out| {
            let lower = out.to_lowercase();
            lower.contains("your local changes")
                || lower.contains("would be overwritten by checkout")
                || lower.contains("commit your changes or stash them")
        })
    }

    fn get_new_command(&self, command: &Command) -> Vec<CorrectedCommand> {
        let branch = command
            .script
            .split_whitespace()
            .nth(2)
            .unwrap_or("<branch>");

        let action: &str = if command.script.starts_with("git checkout ") {
            "checkout"
        } else if command.script.starts_with("git switch ") {
            "switch"
        } else {
            "rebase"
        };

        vec![CorrectedCommand::new(
            format!("git stash && git {} {} && git stash pop", action, branch),
            self.name(),
            self.priority(),
            Some("Stash changes, switch branch, then re-apply".into()),
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
    fn test_checkout_with_changes() {
        let cmd = Command::new(
            "git checkout main",
            Some("error: Your local changes to the following files would be overwritten".into()),
        );
        assert!(GitStash.match_command(&cmd));
        let r = GitStash.get_new_command(&cmd);
        assert_eq!(
            r[0].script,
            "git stash && git checkout main && git stash pop"
        );
    }

    #[test]
    fn test_stash_them_message() {
        let cmd = Command::new(
            "git switch feature",
            Some("Please commit your changes or stash them before you switch branches.".into()),
        );
        assert!(GitStash.match_command(&cmd));
    }

    #[test]
    fn test_clean_checkout() {
        let cmd = Command::new(
            "git checkout main",
            Some("Switched to branch 'main'".into()),
        );
        assert!(!GitStash.match_command(&cmd));
    }
}
